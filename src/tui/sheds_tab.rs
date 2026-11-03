//! The Sheds tab (yaks-98f1, yaks-da23): `yaks sheds` on screen, live. A tree
//! view of the list (yaks-c354): one collapsible root per actor (a worker in
//! the farm), attention first, and below it the yaks it touched as ordinary
//! yak rows: what it is working on, what it is asking, and what else it
//! wrote, newest first. A shed row's detail pane is the shed (where its code
//! is, its yaks); a yak row's is the yak's.
//!
//! The data is exactly the CLI's: a scan calls `sheds::collect`, and every
//! summary text the CLI also prints comes from the same `sheds` function
//! (`status_text`, `yak_line`, `farm_line`).
//!
//! A scan reads the farm (and asks git once for the farm branch's upstream),
//! so it runs on a background thread and comes back as one [`Snapshot`] over
//! a channel; [`App::settle_sheds`] picks it up from the event loop (and from
//! toque's `settle`), so key handling and the farm reload never wait on it.
//! Scans run at startup, every [`RESCAN_EVERY`] while the tab is shown, and
//! on `r`; one at a time.
//!
//! The tab is a built-in view (`view::sheds_view`) whose rows `build_rows`
//! takes from [`App::shed_rows`]; a yak row there takes every yak key, and a
//! shed row the shed-kind bindings of `keymap`. A shed's yaks are yaks of this
//! farm: one the farm no longer has is not listed.

use super::*;
use crate::sheds::{self, Kind, Liveness, Shed, Touched};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{ListItem, Paragraph};
use std::time::Instant;

/// How often the sheds are rescanned while the tab is shown.
pub(crate) const RESCAN_EVERY: Duration = Duration::from_secs(10);

/// One finished scan: the farm's sheds in display order, as of `report.now`.
pub(crate) struct Snapshot {
    pub report: sheds::Report,
    /// The scan itself failed (the farm could not be read).
    pub error: Option<String>,
}

impl Snapshot {
    /// Run one scan, as `yaks sheds` would.
    pub(crate) fn scan(farm_root: &Path) -> Snapshot {
        match sheds::collect(farm_root) {
            Ok(report) => Snapshot {
                report,
                error: None,
            },
            Err(e) => Snapshot {
                report: sheds::Report {
                    now: chrono::Utc::now(),
                    farm: None,
                    sheds: Vec::new(),
                    fresh: Default::default(),
                },
                error: Some(format!("{e:#}")),
            },
        }
    }

    /// The sheds in display order (attention first, see `sheds::build`).
    pub(crate) fn sheds(&self) -> &[Shed] {
        &self.report.sheds
    }

    /// Sheds that want a look now (an open ask, `active` or holding a claim):
    /// the tab's count.
    pub(crate) fn busy(&self) -> usize {
        self.sheds().iter().filter(|s| !s.quiet()).count()
    }
}

/// The tab's state: the latest snapshot and the scan plumbing (the cursor is
/// the list's).
#[derive(Default)]
pub(crate) struct ShedsTab {
    pub snapshot: Option<Snapshot>,
    /// Seconds since the snapshot arrived, refreshed by `settle_sheds` (so
    /// `render` stays pure and tests can freeze it).
    pub age: Option<u64>,
    /// Whether to scan at all (the TUI has a farm to read); `false` =
    /// never scan (read-only and test constructors).
    pub scan: bool,
    /// Where the TUI was started; only source links use it (`source_base`).
    pub cwd: Option<PathBuf>,
    scanned_at: Option<Instant>,
    in_flight: Option<Receiver<Snapshot>>,
    queued: bool,
}

impl ShedsTab {
    pub(crate) fn scanning(&self) -> bool {
        self.in_flight.is_some()
    }
}

impl App {
    pub(crate) fn on_sheds_tab(&self) -> bool {
        self.active_view().key == view::SHEDS_KEY
    }

    /// Start a scan in the background, or queue one if a scan is running.
    pub(crate) fn request_sheds_scan(&mut self) {
        let (true, Some(farm)) = (self.sheds.scan, self.farm.as_ref()) else {
            return;
        };
        if self.sheds.in_flight.is_some() {
            self.sheds.queued = true;
            return;
        }
        let root = farm.root().to_path_buf();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(Snapshot::scan(&root));
        });
        self.sheds.in_flight = Some(rx);
    }

    /// Pick up a finished scan, start a due one (only while the tab is
    /// shown), refresh the age. Cheap; the event loop calls it every tick.
    /// `block` waits for a running scan (toque's `settle`, so a headless
    /// frame shows the settled state).
    pub(crate) fn settle_sheds(&mut self, block: bool) {
        if let Some(rx) = &self.sheds.in_flight {
            let got = if block {
                rx.recv().ok()
            } else {
                match rx.try_recv() {
                    Ok(s) => Some(s),
                    Err(mpsc::TryRecvError::Empty) => None,
                    // The scan thread died (a panic): drop it, rescan later.
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.sheds.in_flight = None;
                        None
                    }
                }
            };
            if let Some(snap) = got {
                self.sheds.in_flight = None;
                self.sheds.scanned_at = Some(Instant::now());
                self.apply_sheds_snapshot(snap);
                if std::mem::take(&mut self.sheds.queued) {
                    self.request_sheds_scan();
                }
            }
        }
        let due = self
            .sheds
            .scanned_at
            .is_none_or(|t| t.elapsed() >= RESCAN_EVERY);
        if due && !self.sheds.scanning() && self.on_sheds_tab() {
            self.request_sheds_scan();
        }
        self.sheds.age = self.sheds.scanned_at.map(|t| t.elapsed().as_secs());
    }

    /// The Sheds view's rows: a root per shed in display order, and under it
    /// the yaks it touched that this farm still has, as `Shed::rows` orders
    /// them. A live content filter (`/`, the drawer) keeps the yaks it matches
    /// and a shed with any of them or a name the search matches.
    pub(crate) fn shed_rows(&self) -> Vec<tree::Row<'_>> {
        let Some(snap) = &self.sheds.snapshot else {
            return Vec::new();
        };
        let by_id: std::collections::HashMap<&str, &Task> =
            self.all.iter().map(|t| (t.id.as_str(), t)).collect();
        let resolved = filter::resolved_ids(&self.all);
        let active = self.filter.content_active();
        let search = self
            .filter
            .search
            .as_deref()
            .filter(|q| !q.is_empty())
            .map(str::to_lowercase);
        let mut out = Vec::new();
        for shed in snap.sheds() {
            let kids: Vec<&Task> = shed
                .rows()
                .into_iter()
                .filter_map(|(_, y)| by_id.get(y.id.as_str()).copied())
                .filter(|t| !active || self.filter.matches(t, &resolved))
                .collect();
            let named = search
                .as_deref()
                .is_some_and(|q| shed.name.to_lowercase().contains(q));
            if active && kids.is_empty() && !named {
                continue;
            }
            out.push(tree::Row::shed(shed, !kids.is_empty()));
            out.extend(kids.into_iter().map(tree::Row::shed_yak));
        }
        out
    }

    /// Swap in a new snapshot. On the Sheds tab the cursor stays on the same
    /// yak under the same shed (by id and name), else on the shed.
    pub(crate) fn apply_sheds_snapshot(&mut self, snap: Snapshot) {
        let was = self.on_sheds_tab().then(|| {
            (
                self.cursor_shed().map(|s| s.name.clone()),
                self.selected_id(),
            )
        });
        self.sheds.snapshot = Some(snap);
        self.touch();
        let Some((Some(shed), yak)) = was else {
            return self.clamp_cursor();
        };
        let pos = {
            let rows = self.rows();
            let mut under = None;
            let mut shed_at = None;
            let mut yak_at = None;
            for (i, r) in rows.iter().enumerate() {
                match r.item {
                    tree::Item::Shed(s) => {
                        under = Some(&s.name);
                        if s.name == shed {
                            shed_at = Some(i);
                        }
                    }
                    tree::Item::Yak(t) if yak.as_deref() == Some(t.id.as_str()) => {
                        if under == Some(&shed) {
                            yak_at = Some(i);
                        }
                    }
                    _ => {}
                }
            }
            yak_at.or(shed_at)
        };
        match pos {
            Some(i) => self.cursor = i,
            None => self.clamp_cursor(),
        }
    }

    /// The selected shed, when the cursor is on a shed row.
    pub(crate) fn selected_shed(&self) -> Option<&Shed> {
        match self.selected_row()? {
            tree::Item::Shed(s) => Some(s),
            _ => None,
        }
    }

    /// The shed the cursor is in: a shed row, or a yak row below one.
    pub(crate) fn cursor_shed(&self) -> Option<&Shed> {
        let rows = self.cached_rows();
        let upto = self.cursor.min(rows.len().checked_sub(1)?);
        rows[..=upto]
            .iter()
            .rev()
            .find_map(|r| match self.resolve_item(r.loc) {
                tree::Item::Shed(s) => Some(s),
                _ => None,
            })
    }

    /// toque's state-header digest on the Sheds tab: the shed the cursor is in.
    pub(crate) fn sheds_state_header(&self) -> String {
        let shed = self
            .cursor_shed()
            .map_or_else(|| "-".into(), |s| s.name.clone());
        let n = self.sheds.snapshot.as_ref().map_or(0, |s| s.sheds().len());
        format!("sheds={n} \u{b7} shed={shed}")
    }

    /// The tab's count: sheds wanting a look right now, `…` before the
    /// first scan.
    pub(crate) fn sheds_count_label(&self) -> String {
        match &self.sheds.snapshot {
            Some(s) => format_count(s.busy()),
            None => "\u{2026}".into(),
        }
    }

    /// The tab row's right-hand slot on the Sheds tab.
    pub(crate) fn sheds_status_label(&self) -> String {
        if self.sheds.scanning() {
            "scanning\u{2026}".into()
        } else {
            match self.sheds.age {
                Some(a) => format!("scanned {a}s ago"),
                None => String::new(),
            }
        }
    }

    /// `r`: scan the sheds again now.
    pub(crate) fn rescan_sheds(&mut self) {
        self.request_sheds_scan();
        self.notification = Some("rescanning sheds".into());
    }

    /// `y` on a shed row: copy its name (the actor to pass as `YAKS_ACTOR`).
    pub(crate) fn copy_shed_name(&mut self) {
        let Some(text) = self.selected_shed().map(|s| s.name.clone()) else {
            return;
        };
        self.notification = Some(if crate::clipboard::copy_text(&text) {
            format!("copied {text}")
        } else {
            "clipboard unavailable".into()
        });
    }
}

// -- rendering ---------------------------------------------------------------

fn dim() -> Style {
    Style::new().add_modifier(Modifier::DIM)
}

fn liveness_span(s: &Shed) -> Span<'static> {
    match s.liveness {
        Liveness::Active => Span::styled(format!("{:<7}", "active"), Style::new().fg(Color::Green)),
        Liveness::Stale => Span::styled(format!("{:<7}", "stale"), Style::new().fg(Color::Yellow)),
        Liveness::Idle => Span::styled(format!("{:<7}", "idle"), dim()),
    }
}

fn pad(s: &str, w: usize) -> String {
    let t = truncate_disp(s, w);
    let fill = w.saturating_sub(disp_width(&t));
    format!("{t}{}", " ".repeat(fill))
}

/// The width of the name column: the widest shed name among `rows`.
pub(crate) fn name_width(rows: &[tree::Row<'_>]) -> usize {
    rows.iter()
        .filter_map(|r| match r.item {
            tree::Item::Shed(s) => Some(disp_width(&s.name)),
            _ => None,
        })
        .max()
        .unwrap_or(4)
        .clamp(4, 24)
}

/// One list row for a shed: mark, name, liveness, when it last wrote, then
/// what it is working on (else its newest yak), and the count of the yaks
/// hidden below it when it is collapsed.
pub(crate) fn shed_item(
    row: &tree::Row<'_>,
    s: &Shed,
    now: chrono::DateTime<chrono::Utc>,
    name_w: usize,
    width: u16,
) -> ListItem<'static> {
    let mark = if s.asks().next().is_some() {
        "\u{23f3}"
    } else {
        "  "
    };
    let quiet = s.quiet();
    let base = if quiet { dim() } else { Style::new() };
    let mut spans = vec![
        Span::styled(format!("{mark} "), Style::new().fg(Color::Yellow)),
        Span::styled(
            pad(&s.name, name_w),
            if quiet {
                dim()
            } else {
                Style::new().add_modifier(Modifier::BOLD)
            },
        ),
        Span::raw("  "),
        liveness_span(s),
        Span::styled(format!("{:<13}", sheds::ago(&s.last_write, now)), base),
    ];
    let working: Vec<&str> = s.working().map(|t| t.id.as_str()).collect();
    let tail = if !working.is_empty() {
        format!("shaving: {}", working.join(", "))
    } else {
        s.touched
            .first()
            .map(|t| format!("last: {} {}", t.id, t.title))
            .unwrap_or_default()
    };
    let badge = if row.collapsed && row.hidden > 0 {
        format!(" \u{25b6} {} ", row.hidden)
    } else {
        String::new()
    };
    let used: usize = spans.iter().map(|sp| disp_width(&sp.content)).sum();
    let room = (width as usize).saturating_sub(used + disp_width(&badge));
    // Padded to the full width so the cursor highlight spans the row.
    spans.push(Span::styled(pad(&tail, room), base));
    spans.push(Span::styled(badge, dim().fg(Color::DarkGray)));
    ListItem::new(Line::from(spans))
}

/// What the Sheds view's list says instead of rows: that no scan has come
/// back, why it failed, or that nothing is there or nothing matches. `None`
/// when there are rows.
pub(crate) fn sheds_notice(app: &App, no_rows: bool) -> Option<Paragraph<'static>> {
    let Some(snap) = &app.sheds.snapshot else {
        let msg = if app.sheds.scan {
            "scanning sheds\u{2026}"
        } else {
            "No shed scan in this view."
        };
        return Some(Paragraph::new(Span::styled(msg, dim())));
    };
    if let Some(e) = &snap.error {
        return Some(Paragraph::new(Span::styled(
            format!("shed scan failed: {e}"),
            Style::new().fg(Color::Red),
        )));
    }
    no_rows.then(|| {
        let msg = if snap.sheds().is_empty() {
            "No actor has written to this farm yet."
        } else {
            "No shed or yak matches the filter."
        };
        Paragraph::new(Span::styled(msg, dim()))
    })
}

/// Draw the farm's freshness (when it has an upstream) on the last line of
/// `area`, and return the part above it for the rows.
pub(crate) fn render_farm_footer(app: &App, frame: &mut Frame, area: Rect) -> Rect {
    let Some(farm) = app
        .sheds
        .snapshot
        .as_ref()
        .and_then(|s| sheds::farm_header(&s.report))
    else {
        return area;
    };
    let footer_h = area.height.min(2);
    let footer = Rect {
        y: area.y + area.height - footer_h,
        height: footer_h,
        ..area
    };
    let mut lines = vec![Line::default(); footer_h as usize];
    if let Some(last) = lines.last_mut() {
        *last = Line::from(Span::styled(format!("   {farm}"), dim()));
    }
    frame.render_widget(Paragraph::new(lines), footer);
    Rect {
        height: area.height - footer_h,
        ..area
    }
}

/// One yak's line in a shed's detail: the shared text of `yaks sheds`, the
/// label and id coloured, finished work dimmed.
fn yak_line(kind: Kind, y: &Touched, now: chrono::DateTime<chrono::Utc>) -> Line<'static> {
    let text = sheds::yak_line(kind, y, now);
    let (label, rest) = text.split_at(8);
    let rest = rest.trim_start();
    let (id, rest) = rest.split_once("  ").unwrap_or((rest, ""));
    let done = y.status.is_resolved();
    let mut spans = vec![
        Span::styled(label.to_string(), Style::new().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(format!("{id}  "), Style::new().fg(Color::Blue)),
        Span::styled(rest.to_string(), if done { dim() } else { Style::new() }),
    ];
    if y.asking() {
        spans.insert(0, Span::raw("\u{23f3}"));
    }
    Line::from(spans)
}

/// The detail pane's lines for shed `s`: its status, where its code is, and
/// its yaks.
fn detail_lines(s: &Shed, now: chrono::DateTime<chrono::Utc>) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = vec![Line::from(vec![
        Span::styled(
            s.name.clone(),
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  {}", sheds::status_text(s, now))),
    ])];
    if let Some(c) = &s.checkpoint {
        lines.push(Line::from(vec![Span::styled(c.line(now), dim())]));
    }
    lines.push(Line::default());
    lines.extend(s.rows().into_iter().map(|(kind, y)| yak_line(kind, y, now)));
    lines
}

/// The pane of a shed row. (A yak row's is the yak's.)
pub(crate) fn render_shed_detail(app: &App, frame: &mut Frame, area: Rect) {
    let (Some(snap), Some(s)) = (&app.sheds.snapshot, app.selected_shed()) else {
        return;
    };
    let visible: Vec<Line> = detail_lines(s, snap.report.now)
        .into_iter()
        .take(area.height as usize)
        .map(|l| {
            let text: String = l.spans.iter().map(|s| s.content.as_ref()).collect();
            if disp_width(&text) <= area.width as usize {
                l
            } else {
                truncate_line(l, area.width as usize)
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(visible), area);
}

/// Cut a styled line to `w` display columns, keeping span styles.
fn truncate_line(l: Line<'static>, w: usize) -> Line<'static> {
    let style = l.style;
    let mut used = 0;
    let mut spans = Vec::new();
    for s in l.spans {
        if used >= w {
            break;
        }
        let t = truncate_disp(&s.content, w - used);
        used += disp_width(&t);
        spans.push(Span::styled(t, s.style));
    }
    Line::from(spans).style(style)
}

/// A hand-built snapshot for tests and doc shots: a worker with an open ask,
/// two active workers (one holding a yak a coordinator claimed for it), a
/// stale claim, and idle actors.
#[cfg(test)]
pub(crate) mod fixture {
    use super::*;
    use crate::model::Status;
    use crate::sheds::{Report, Touched};
    use crate::status::SyncState;

    /// Fixed "now" of the fixture, so ages never depend on today.
    pub(crate) const NOW: &str = "2026-10-31T16:00:00Z";

    fn touched(id: &str, title: &str, status: Status, last: &str, working: bool) -> Touched {
        Touched {
            id: id.into(),
            title: title.into(),
            status,
            needs: None,
            last: last.into(),
            working,
        }
    }

    fn shed(name: &str, liveness: Liveness, touched: Vec<Touched>) -> Shed {
        Shed {
            name: name.into(),
            last_write: touched[0].last.clone(),
            liveness,
            touched,
            checkpoint: None,
        }
    }

    pub(crate) fn snapshot() -> Snapshot {
        let mut ask = touched(
            "mem-7aba",
            "Add open-source textures",
            Status::Shaving,
            "2026-10-31T15:46:00Z",
            true,
        );
        ask.needs = Some("human".into());
        let tex = shed("tex-1", Liveness::Active, vec![ask]);
        let walk = shed(
            "walk-1",
            Liveness::Active,
            vec![
                touched(
                    "mem-f13a",
                    "Add walk mode",
                    Status::Shaving,
                    "2026-10-31T15:51:00Z",
                    true,
                ),
                touched(
                    "mem-99d0",
                    "Left-button mouse lock",
                    Status::Shorn,
                    "2026-10-28T10:00:00Z",
                    false,
                ),
            ],
        );
        let mut walk = walk;
        walk.checkpoint = Some(crate::sheds::Checkpoint {
            branch: "work/walk-1".into(),
            commit: Some("47076b0e1c2d3f4a5b6c7d8e9f0a1b2c3d4e5f60".into()),
            at: "2026-10-31T15:51:00Z".into(),
        });
        let delta = shed(
            "delta:Set up remote yak workflow",
            Liveness::Active,
            vec![touched(
                "mem-f434",
                "Host memcraft-viewer web build on GitHub Pages",
                Status::Shaving,
                "2026-10-31T15:48:00Z",
                true,
            )],
        );
        let stale = shed(
            "perf-2",
            Liveness::Stale,
            vec![touched(
                "mem-0a01",
                "Profile the chunk mesher",
                Status::Shaving,
                "2026-10-31T11:00:00Z",
                true,
            )],
        );
        let idle = shed(
            "mig-2",
            Liveness::Idle,
            vec![touched(
                "mem-0001",
                "Migrate the save format",
                Status::Shorn,
                "2026-10-30T09:00:00Z",
                false,
            )],
        );
        let gone = shed(
            "gone-1",
            Liveness::Idle,
            vec![touched(
                "mem-0002",
                "Old spike",
                Status::Dead,
                "2026-10-01T09:00:00Z",
                false,
            )],
        );
        Snapshot {
            report: Report {
                now: crate::store::parse_ts(NOW).unwrap(),
                farm: Some(SyncState {
                    branch: "yaks".into(),
                    upstream: Some("origin/yaks".into()),
                    ahead: 0,
                    behind: 0,
                }),
                // Display order, as `sheds::build` returns it.
                sheds: vec![tex, walk, delta, stale, idle, gone],
                fresh: Default::default(),
            },
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_support::*;

    fn on_sheds(mut app: App) -> App {
        let i = app
            .views
            .iter()
            .position(|v| v.key == view::SHEDS_KEY)
            .unwrap();
        app.set_view(i);
        app.apply_sheds_snapshot(fixture::snapshot());
        app.sheds.age = Some(4);
        app
    }

    #[test]
    fn tab_count_is_sheds_wanting_a_look() {
        let app = on_sheds(sample());
        // tex-1 (asking), walk-1, delta, perf-2 (stale claim); the two idle
        // actors are not counted.
        assert_eq!(app.sheds_count_label(), "4");
        let v = app.active_view().clone();
        assert_eq!(app.view_count(&v), 4);
    }

    /// A farm that has every yak the fixture's sheds touched, on the Sheds tab.
    fn shed_farm() -> App {
        use crate::model::Status::*;
        let tasks = [
            ("mem-7aba", "Add open-source textures", Shaving),
            ("mem-f13a", "Add walk mode", Shaving),
            ("mem-99d0", "Left-button mouse lock", Shorn),
            ("mem-f434", "Host the web build", Shaving),
            ("mem-0a01", "Profile the chunk mesher", Shaving),
            ("mem-0001", "Migrate the save format", Shorn),
            ("mem-0002", "Old spike", Dead),
        ];
        on_sheds(App::new(
            tasks
                .iter()
                .map(|(id, title, st)| task(id, title, *st, 2, None))
                .collect(),
        ))
    }

    /// The rows as `shed:<name>` / `<yak id>`, in order.
    fn row_names(app: &App) -> Vec<String> {
        app.rows()
            .iter()
            .map(|r| match r.item {
                tree::Item::Shed(s) => format!("shed:{}", s.name),
                tree::Item::Yak(t) => t.id.clone(),
                tree::Item::Doc(e) => e.path.clone(),
            })
            .collect()
    }

    #[test]
    fn rows_are_sheds_with_the_yaks_they_touched_below() {
        let app = shed_farm();
        assert_eq!(
            row_names(&app),
            [
                "shed:tex-1",
                "mem-7aba",
                "shed:walk-1",
                "mem-f13a", // shaving, then recent
                "mem-99d0",
                "shed:delta:Set up remote yak workflow",
                "mem-f434",
                "shed:perf-2",
                "mem-0a01",
                "shed:mig-2",
                "mem-0001",
                "shed:gone-1",
                "mem-0002",
            ]
        );
        let rows = app.rows();
        assert!(rows.iter().all(|r| r.depth == (r.yak().is_some() as u16)));
        assert!(rows[0].has_children && !rows[1].has_children);
    }

    #[test]
    fn a_yak_the_farm_lacks_is_not_listed() {
        // Only mem-f13a is in this farm: tex-1 is a root with nothing below.
        let app = on_sheds(App::new(vec![task(
            "mem-f13a",
            "walk mode",
            crate::model::Status::Shaving,
            2,
            None,
        )]));
        let names = row_names(&app);
        assert_eq!(names[0], "shed:tex-1");
        assert_eq!(names[1], "shed:walk-1");
        assert!(!names.contains(&"mem-7aba".to_string()));
        assert!(names.contains(&"mem-f13a".to_string()));
        assert!(!app.rows()[0].has_children);
    }

    #[test]
    fn j_from_a_shed_root_lands_on_its_first_yak() {
        let mut app = shed_farm();
        assert!(app.selected_shed().is_some());
        press(&mut app, "j");
        assert_eq!(app.selected_id().as_deref(), Some("mem-7aba"));
        assert!(app.selected_shed().is_none());
        assert_eq!(app.cursor_shed().unwrap().name, "tex-1");
        press(&mut app, "j");
        assert_eq!(app.selected_shed().unwrap().name, "walk-1");
    }

    #[test]
    fn space_collapses_a_shed_and_again_expands_it() {
        let mut app = shed_farm();
        press(&mut app, "jj"); // walk-1
        assert_eq!(app.selected_shed().unwrap().name, "walk-1");
        press(&mut app, " ");
        assert!(app.collapsed.contains("shed:walk-1"));
        let names = row_names(&app);
        assert!(!names.contains(&"mem-f13a".to_string()));
        assert!(!names.contains(&"mem-99d0".to_string()));
        let walk = &app.rows()[2];
        assert!(walk.collapsed && walk.hidden == 2);
        assert!(draw(&app, 120, 14).contains("\u{25b6} 2"), "the badge");
        press(&mut app, " ");
        assert!(app.collapsed.is_empty());
        assert!(row_names(&app).contains(&"mem-f13a".to_string()));
        // A yak row below a shed has no children to collapse.
        press(&mut app, "j ");
        assert!(app.collapsed.is_empty());
    }

    #[test]
    fn a_shed_row_shows_the_shed_and_a_yak_row_the_yak() {
        let mut app = shed_farm();
        press(&mut app, "jjl"); // walk-1
        assert_eq!(app.focus, Focus::Detail);
        let frame = draw(&app, 120, 14);
        assert!(frame.contains("work/walk-1"), "{frame}");
        // The pane's own keys: rescan, and copy the name.
        press(&mut app, "r");
        assert_eq!(app.notification.as_deref(), Some("rescanning sheds"));
        press(&mut app, "y");
        let note = app.notification.clone().unwrap();
        assert!(
            note == "copied walk-1" || note == "clipboard unavailable",
            "{note}"
        );
        press(&mut app, "J"); // its first yak, still in the pane
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_id().as_deref(), Some("mem-f13a"));
        let frame = draw(&app, 120, 14);
        assert!(frame.contains("Add walk mode"), "{frame}");
        assert!(!frame.contains("work/walk-1"), "{frame}");
        press(&mut app, "J");
        assert_eq!(app.selected_id().as_deref(), Some("mem-99d0"));
        press(&mut app, "K");
        press(&mut app, "K");
        assert_eq!(app.selected_shed().unwrap().name, "walk-1");
        press(&mut app, "h");
        assert_eq!(app.focus, Focus::List);
    }

    #[test]
    fn enter_on_a_yak_child_opens_the_yak_and_j_moves_on() {
        let mut app = shed_farm();
        press(&mut app, "j");
        enter_key(&mut app);
        assert!(app.on_sheds_tab());
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_id().as_deref(), Some("mem-7aba"));
        press(&mut app, "J");
        assert_eq!(app.selected_shed().unwrap().name, "walk-1");
        press(&mut app, "J");
        assert_eq!(app.selected_id().as_deref(), Some("mem-f13a"));
    }

    #[test]
    fn yak_keys_work_on_a_yak_child_and_not_on_a_shed() {
        let mut app = shed_farm();
        press(&mut app, "S");
        assert!(matches!(app.overlay, Overlay::None), "S on a shed row");
        press(&mut app, "j");
        press(&mut app, "S");
        assert!(
            matches!(app.overlay, Overlay::Pick { .. }),
            "S on a yak row"
        );
    }

    #[test]
    fn u_on_the_sheds_view_requests_a_sync() {
        let (proj, farm) = temp_farm(&[task(
            "mem-f13a",
            "walk mode",
            crate::model::Status::Shaving,
            2,
            None,
        )]);
        let mut app = on_sheds(App::with_farm(farm).unwrap());
        assert!(app.on_sheds_tab());
        press(&mut app, "U");
        assert_eq!(app.notification.as_deref(), Some("syncing\u{2026}"));
        assert!(app.pending_farm_op.is_some());
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn search_keeps_only_the_matching_yaks_and_their_sheds() {
        let mut app = shed_farm();
        app.filter.search = Some("walk".into());
        // walk-1 (its name and a yak match), and no other shed.
        assert_eq!(
            row_names(&app),
            ["shed:walk-1", "mem-f13a"],
            "only the matching child, under its shed"
        );
        // A name alone keeps the shed, with no yaks below.
        app.filter.search = Some("perf".into());
        assert_eq!(row_names(&app), ["shed:perf-2"]);
        app.filter.search = Some("no such thing".into());
        assert!(row_names(&app).is_empty());
        assert!(draw(&app, 120, 14).contains("No shed or yak matches"));
        app.filter.search = None;
        assert_eq!(row_names(&app).len(), 13);
    }

    #[test]
    fn typing_a_search_filters_the_sheds_view() {
        let mut app = shed_farm();
        press(&mut app, "/");
        press(&mut app, "mouse");
        enter_key(&mut app);
        assert_eq!(row_names(&app), ["shed:walk-1", "mem-99d0"]);
    }

    #[test]
    fn y_copies_the_shed_name_and_r_rescans() {
        let mut app = shed_farm();
        press(&mut app, "y");
        let note = app.notification.clone().unwrap();
        assert!(
            note == "copied tex-1" || note == "clipboard unavailable",
            "{note}"
        );
        press(&mut app, "r");
        assert_eq!(app.notification.as_deref(), Some("rescanning sheds"));
        // An empty tab (no actor yet) still rescans.
        let mut empty = on_sheds(App::new(vec![]));
        let mut snap = fixture::snapshot();
        snap.report.sheds.clear();
        empty.apply_sheds_snapshot(snap);
        press(&mut empty, "r");
        assert_eq!(empty.notification.as_deref(), Some("rescanning sheds"));
    }

    #[test]
    fn a_new_snapshot_keeps_the_cursor_on_the_same_yak_under_its_shed() {
        let mut app = shed_farm();
        press(&mut app, "jjjj"); // walk-1's recent yak
        assert_eq!(app.selected_id().as_deref(), Some("mem-99d0"));
        // The next scan finds a new actor first: the cursor stays put.
        let mut snap = fixture::snapshot();
        let mut newcomer = snap.report.sheds[1].clone();
        newcomer.name = "aaa-new".into();
        snap.report.sheds.insert(0, newcomer);
        app.apply_sheds_snapshot(snap);
        assert_eq!(app.selected_id().as_deref(), Some("mem-99d0"));
        assert_eq!(app.cursor_shed().unwrap().name, "walk-1");
        // The same yak under another shed is not the one the cursor was on.
        press(&mut app, "k");
        press(&mut app, "k");
        assert_eq!(app.selected_shed().unwrap().name, "walk-1");
        let mut snap = fixture::snapshot();
        snap.report.sheds.retain(|s| s.name != "tex-1");
        app.apply_sheds_snapshot(snap);
        assert_eq!(app.selected_shed().unwrap().name, "walk-1");
    }

    #[test]
    fn a_background_scan_arrives_without_blocking() {
        // A farm with one yak by one actor: a real thread and channel round
        // trip, then the snapshot lists that actor.
        let mut t = task("yak-aaaa", "slow", crate::model::Status::Shaving, 2, None);
        t.body = crate::store::append_note("", "2026-10-31T10:00:00Z", Some("w1"), "hi");
        let (proj, farm) = temp_farm(&[t]);
        let mut app = on_sheds(App::new(vec![]));
        app.sheds.snapshot = None;
        app.touch();
        app.farm = Some(farm);
        app.sheds.scan = true;
        app.settle_sheds(false); // on the tab, never scanned: starts one
        assert!(app.sheds.scanning());
        assert_eq!(app.sheds_status_label(), "scanning\u{2026}");
        let start = Instant::now();
        while app.sheds.scanning() && start.elapsed() < Duration::from_secs(20) {
            std::thread::sleep(Duration::from_millis(10));
            app.settle_sheds(false);
        }
        let snap = app.sheds.snapshot.as_ref().expect("the scan arrived");
        assert!(snap.error.is_none(), "{:?}", snap.error);
        assert_eq!(snap.sheds().len(), 1);
        assert_eq!(snap.sheds()[0].name, "w1");
        assert_eq!(app.sheds.age, Some(0));
        // Not due again for RESCAN_EVERY.
        app.settle_sheds(false);
        assert!(!app.sheds.scanning());
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn no_scan_runs_off_the_tab() {
        let (proj, farm) = temp_farm(&[]);
        let mut app = App::new(vec![]);
        app.farm = Some(farm);
        app.sheds.scan = true;
        assert!(!app.on_sheds_tab());
        app.settle_sheds(false);
        assert!(!app.sheds.scanning());
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn sheds_tab_frames() {
        let mut app = shed_farm();
        insta::assert_snapshot!("sheds_tab_list", draw(&app, 120, 22));
        press(&mut app, "jjl"); // walk-1's detail
        insta::assert_snapshot!("sheds_tab_detail", draw(&app, 120, 14));
        press(&mut app, "J"); // its first yak, in the same pane
        insta::assert_snapshot!("sheds_tab_yak_detail", draw(&app, 120, 14));
    }
}
