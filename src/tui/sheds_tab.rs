//! The Sheds tab (yaks-98f1, yaks-da23): `yaks sheds` on screen, live. One row
//! per actor (a worker in the farm), attention first, and a detail pane with
//! the selected shed's yaks: what it is working on, what it is asking, and
//! what else it wrote, newest first.
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
//! The tab is a built-in view (`view::sheds_view`) whose yak rows are empty,
//! so every yak key is inert there; its keys are the `Sheds` and `ShedDetail` contexts of `keymap`.
//! A shed's yaks are yaks of this farm: Enter opens one, and `a` on the yak
//! views answers it.

use super::*;
use crate::sheds::{self, Kind, Liveness, Shed, Touched};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
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

/// The tab's state: the latest snapshot, the cursors, and the scan plumbing.
#[derive(Default)]
pub(crate) struct ShedsTab {
    pub snapshot: Option<Snapshot>,
    /// Index into the snapshot's sheds (already in display order).
    pub cursor: usize,
    /// Detail pane: index into the selected shed's rows.
    pub line: usize,
    /// List: first visible row, kept across frames (yaks-9009).
    pub list_offset: Cell<usize>,
    /// Detail pane: first visible line, kept across frames.
    pub offset: Cell<usize>,
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

    /// Swap in a new snapshot, keeping the cursor on the same shed (by name)
    /// and the detail cursor on the same yak (by id).
    pub(crate) fn apply_sheds_snapshot(&mut self, snap: Snapshot) {
        let shed = self.selected_shed().map(|s| s.name.clone());
        let yak = self.selected_shed_yak().map(|(_, y)| y.id.clone());
        self.sheds.snapshot = Some(snap);
        self.touch();
        let s = self.sheds.snapshot.as_ref().unwrap();
        self.sheds.cursor = shed
            .and_then(|n| s.sheds().iter().position(|x| x.name == n))
            .unwrap_or(0)
            .min(s.sheds().len().saturating_sub(1));
        let rows = s.sheds().get(self.sheds.cursor).map(Shed::rows);
        let rows = rows.as_deref().unwrap_or(&[]);
        self.sheds.line = yak
            .and_then(|id| rows.iter().position(|(_, y)| y.id == id))
            .unwrap_or(0)
            .min(rows.len().saturating_sub(1));
    }

    pub(crate) fn selected_shed(&self) -> Option<&Shed> {
        self.sheds.snapshot.as_ref()?.sheds().get(self.sheds.cursor)
    }

    /// The yak under the detail cursor, with the part of the shed it is in.
    pub(crate) fn selected_shed_yak(&self) -> Option<(Kind, &Touched)> {
        self.selected_shed()?.rows().get(self.sheds.line).copied()
    }

    pub(crate) fn move_shed(&mut self, delta: i32) {
        let n = self.sheds.snapshot.as_ref().map_or(0, |s| s.sheds().len());
        if n == 0 {
            return;
        }
        let c = (self.sheds.cursor as i64 + delta as i64).clamp(0, n as i64 - 1);
        if c as usize != self.sheds.cursor {
            self.sheds.cursor = c as usize;
            self.sheds.line = 0;
            self.sheds.offset.set(0);
        }
    }

    pub(crate) fn move_shed_yak(&mut self, delta: i32) {
        let n = self.selected_shed().map_or(0, |s| s.touched.len());
        if n == 0 {
            return;
        }
        let l = (self.sheds.line as i64 + delta as i64).clamp(0, n as i64 - 1);
        self.sheds.line = l as usize;
    }

    /// toque's state-header digest on the Sheds tab: the selected shed and yak.
    pub(crate) fn sheds_state_header(&self) -> String {
        let shed = self
            .selected_shed()
            .map_or_else(|| "-".into(), |s| s.name.clone());
        let yak = self
            .selected_shed_yak()
            .map_or_else(|| "-".into(), |(_, y)| y.id.clone());
        let n = self.sheds.snapshot.as_ref().map_or(0, |s| s.sheds().len());
        format!("sheds={n} \u{b7} shed={shed} \u{b7} shed_yak={yak}")
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

    /// `l` / Enter on a shed: show it in the detail pane, from the top.
    pub(crate) fn open_shed(&mut self) {
        if self.selected_shed().is_some() {
            self.focus = Focus::Detail;
            self.sheds.line = 0;
            self.sheds.offset.set(0);
        }
    }

    /// `y`: the yak id on a yak line of the detail, else the shed's name (the
    /// actor to pass as `YAKS_ACTOR`).
    pub(crate) fn copy_shed_thing(&mut self) {
        let text = if self.focus == Focus::Detail {
            self.selected_shed_yak().map(|(_, y)| y.id.clone())
        } else {
            None
        }
        .or_else(|| self.selected_shed().map(|s| s.name.clone()));
        let Some(text) = text else { return };
        self.notification = Some(if crate::clipboard::copy_text(&text) {
            format!("copied {text}")
        } else {
            "clipboard unavailable".into()
        });
    }

    /// Enter on a yak in a shed's detail: open it. A yak since removed from
    /// the farm says so.
    pub(crate) fn open_shed_yak(&mut self) {
        let Some(id) = self.selected_shed_yak().map(|(_, y)| y.id.clone()) else {
            return;
        };
        if self.task(&id).is_some() {
            self.goto_task(&id);
        } else {
            self.notification = Some(format!("{id} is no longer in this farm"));
        }
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

/// One list row for a shed: mark, name, liveness, when it last wrote, then
/// what it is working on (else its newest yak).
fn shed_row(
    s: &Shed,
    now: chrono::DateTime<chrono::Utc>,
    name_w: usize,
    width: u16,
) -> Line<'static> {
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
    let used: usize = spans.iter().map(|sp| disp_width(&sp.content)).sum();
    let room = (width as usize).saturating_sub(used);
    // Padded to the full width so the cursor highlight spans the row.
    spans.push(Span::styled(pad(&tail, room), base));
    Line::from(spans)
}

pub(crate) fn render_sheds_list(app: &App, frame: &mut Frame, area: Rect) {
    let Some(snap) = &app.sheds.snapshot else {
        let msg = if app.sheds.scan {
            "scanning sheds\u{2026}"
        } else {
            "No shed scan in this view."
        };
        frame.render_widget(Paragraph::new(Span::styled(msg, dim())), area);
        return;
    };
    if let Some(e) = &snap.error {
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("shed scan failed: {e}"),
                Style::new().fg(Color::Red),
            )),
            area,
        );
        return;
    }
    if snap.sheds().is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No actor has written to this farm yet.",
                dim(),
            )),
            area,
        );
        return;
    }
    let name_w = snap
        .sheds()
        .iter()
        .map(|s| disp_width(&s.name))
        .max()
        .unwrap_or(4)
        .clamp(4, 24);
    let focused = app.focus == Focus::List;
    let hl = if focused {
        Style::new().bg(super::surface::selected())
    } else {
        Style::new().bg(super::surface::band())
    };
    // A footer line for the farm's freshness, when it has an upstream.
    let farm = sheds::farm_header(&snap.report);
    let rows_h = (area.height as usize).saturating_sub(farm.is_some() as usize * 2);
    let mut offset = app.sheds.list_offset.get();
    if app.sheds.cursor < offset {
        offset = app.sheds.cursor;
    } else if rows_h > 0 && app.sheds.cursor >= offset + rows_h {
        offset = app.sheds.cursor + 1 - rows_h;
    }
    app.sheds.list_offset.set(offset);
    let mut lines: Vec<Line> = snap
        .sheds()
        .iter()
        .enumerate()
        .skip(offset)
        .take(rows_h)
        .map(|(pos, s)| {
            let row = shed_row(s, snap.report.now, name_w, area.width);
            if pos == app.sheds.cursor {
                row.style(hl)
            } else {
                row
            }
        })
        .collect();
    if let Some(farm) = farm {
        while lines.len() < rows_h {
            lines.push(Line::default());
        }
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(format!("   {farm}"), dim())));
    }
    frame.render_widget(Paragraph::new(lines), area);
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

/// The detail pane's lines for shed `s`, and the line each of its rows sits
/// on, indexed like `ShedsTab::line`.
fn detail_lines(s: &Shed, now: chrono::DateTime<chrono::Utc>) -> (Vec<Line<'static>>, Vec<usize>) {
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
    let mut starts = Vec::new();
    for (kind, y) in s.rows() {
        starts.push(lines.len());
        lines.push(yak_line(kind, y, now));
    }
    (lines, starts)
}

pub(crate) fn render_shed_detail(app: &App, frame: &mut Frame, area: Rect) {
    let (Some(snap), Some(s)) = (&app.sheds.snapshot, app.selected_shed()) else {
        return;
    };
    let (mut lines, starts) = detail_lines(s, snap.report.now);
    let h = area.height as usize;
    // Keep the selected yak's line in view.
    let mut offset = app.sheds.offset.get();
    if let Some(&start) = starts.get(app.sheds.line) {
        if start < offset {
            offset = start;
        } else if start + 1 > offset + h {
            offset = start + 1 - h;
        }
        if app.focus == Focus::Detail {
            let hl = Style::new().bg(super::surface::selected());
            let mut l = std::mem::take(&mut lines[start]);
            let w: usize = l.spans.iter().map(|s| disp_width(&s.content)).sum();
            l.spans.push(Span::raw(
                " ".repeat((area.width as usize).saturating_sub(w)),
            ));
            lines[start] = l.style(hl);
        }
    }
    app.sheds.offset.set(offset);
    let visible: Vec<Line> = lines
        .into_iter()
        .skip(offset)
        .take(h)
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

    #[test]
    fn yak_keys_are_inert_on_the_sheds_tab() {
        let mut app = on_sheds(sample());
        for c in ['S', 'E', 'X', 'c', 'D', 'M', '/', 'f', '*', 'm', 'a'] {
            press(&mut app, &c.to_string());
            assert!(
                matches!(app.overlay, Overlay::None),
                "{c} opened an overlay on the Sheds tab"
            );
        }
        assert!(app.selected.is_empty());
    }

    #[test]
    fn detail_moves_over_yaks_and_sheds() {
        let mut app = on_sheds(sample());
        press(&mut app, "jl"); // walk-1's detail
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_shed().unwrap().name, "walk-1");
        assert_eq!(app.selected_shed_yak().unwrap().1.id, "mem-f13a");
        press(&mut app, "j");
        assert_eq!(app.selected_shed_yak().unwrap().1.id, "mem-99d0");
        press(&mut app, "j"); // clamps
        assert_eq!(app.selected_shed_yak().unwrap().1.id, "mem-99d0");
        press(&mut app, "K"); // previous shed, still in the detail
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_shed().unwrap().name, "tex-1");
        assert_eq!(app.selected_shed_yak().unwrap().1.id, "mem-7aba");
        press(&mut app, "h");
        assert_eq!(app.focus, Focus::List);
    }

    #[test]
    fn a_new_snapshot_keeps_the_selected_shed_and_yak() {
        let mut app = on_sheds(sample());
        press(&mut app, "jlj");
        assert_eq!(app.selected_shed_yak().unwrap().1.id, "mem-99d0");
        // The next scan finds the asker's ask answered and a new actor first:
        // the cursor stays on walk-1 and its yak.
        let mut snap = fixture::snapshot();
        let mut newcomer = snap.report.sheds[1].clone();
        newcomer.name = "aaa-new".into();
        snap.report.sheds.insert(0, newcomer);
        app.apply_sheds_snapshot(snap);
        assert_eq!(app.selected_shed().unwrap().name, "walk-1");
        assert_eq!(app.selected_shed_yak().unwrap().1.id, "mem-99d0");
    }

    #[test]
    fn enter_opens_the_yak_or_says_it_is_gone() {
        // Only mem-f13a is in this farm.
        let mut app = on_sheds(App::new(vec![task(
            "mem-f13a",
            "walk mode",
            crate::model::Status::Shaving,
            2,
            None,
        )]));
        press(&mut app, "l"); // tex-1: mem-7aba is not in this farm
        enter_key(&mut app);
        assert_eq!(
            app.notification.as_deref(),
            Some("mem-7aba is no longer in this farm")
        );
        assert!(app.on_sheds_tab());
        press(&mut app, "J"); // walk-1: mem-f13a
        enter_key(&mut app);
        assert!(!app.on_sheds_tab());
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_id().as_deref(), Some("mem-f13a"));
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
        let mut app = on_sheds(sample());
        insta::assert_snapshot!("sheds_tab_list", draw(&app, 120, 14));
        press(&mut app, "jl");
        insta::assert_snapshot!("sheds_tab_detail", draw(&app, 120, 14));
    }
}
