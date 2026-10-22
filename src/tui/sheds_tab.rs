//! The Sheds tab (yaks-98f1, yaks-9eaa): `yaks sheds` on screen, live. One
//! row per checkout of the repository (attention first), and a detail pane
//! with the selected shed's git state and the yaks it wrote, newest first.
//!
//! The data is exactly the CLI's: a scan calls `sheds::discover_all` (the
//! sheds, each joined to the farm's activity log, and the orphaned pins), and
//! every summary text the CLI also prints comes from the same `sheds`
//! function (`label_line`, `state_line`, `touched_line`).
//!
//! A scan runs git in every checkout, so it runs on a background thread and
//! comes back as one [`Snapshot`] over a channel; [`App::settle_sheds`]
//! picks it up from the event loop (and from toque's `settle`), so key
//! handling and the farm reload never wait on it. Scans run at startup,
//! every [`RESCAN_EVERY`] while the tab is shown, and on `r`; one at a time.
//!
//! The tab is a built-in view (`view::sheds_view`) whose yak rows are empty,
//! so every yak key is inert there; [`App::handle_sheds_key`] owns its keys.
//! Every shed shares the farm on screen, so a yak a shed wrote is a yak of
//! this farm: Enter opens it, and `a` on the yak views answers it.

use super::*;
use crate::sheds::{self, Liveness, Shed, Touched};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::time::Instant;

/// How often the sheds are rescanned while the tab is shown.
pub(crate) const RESCAN_EVERY: Duration = Duration::from_secs(10);

/// One finished scan: every shed (with its activity) and how many pinned
/// clones are orphaned (no checkout left).
pub(crate) struct Snapshot {
    pub sheds: Vec<Shed>,
    pub orphaned: usize,
    /// The scan itself failed (e.g. not a git repository).
    pub error: Option<String>,
}

impl Snapshot {
    /// Run one scan, as `yaks sheds` would.
    pub(crate) fn scan(cwd: &Path, farm_root: &Path) -> Snapshot {
        match sheds::discover_all(cwd, farm_root) {
            Ok((sheds, orphaned)) => Snapshot {
                sheds,
                orphaned: orphaned.len(),
                error: None,
            },
            Err(e) => Snapshot {
                sheds: Vec::new(),
                orphaned: 0,
                error: Some(format!("{e:#}")),
            },
        }
    }

    /// The yaks shed `i` wrote, newest first (empty when none).
    pub(crate) fn yaks(&self, i: usize) -> &[Touched] {
        self.sheds.get(i).map_or(&[][..], |s| &s.touched)
    }

    /// Indices into `sheds` in display order: attention first (see [`rank`]),
    /// then the most recent writer, then by name.
    pub(crate) fn order(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.sheds.len()).collect();
        idx.sort_by(|&a, &b| {
            let (sa, sb) = (&self.sheds[a], &self.sheds[b]);
            rank(sa)
                .cmp(&rank(sb))
                .then_with(|| sb.last_write.cmp(&sa.last_write))
                .then_with(|| sa.name().cmp(&sb.name()))
        });
        idx
    }

    /// Sheds working on something now (`active` or holding a claim): the
    /// tab's count.
    pub(crate) fn busy(&self) -> usize {
        self.sheds
            .iter()
            .filter(|s| rank(s) <= 3 && !s.primary)
            .count()
    }
}

/// Attention order: the primary, then a shed with an open ask, active,
/// stale (a claim gone quiet), then the rest.
fn rank(s: &Shed) -> u8 {
    if s.primary {
        0
    } else if s.open_asks().next().is_some() {
        1
    } else if s.liveness == Liveness::Active {
        2
    } else if s.liveness == Liveness::Stale {
        3
    } else {
        4
    }
}

/// The tab's state: the latest snapshot, the cursors, and the scan plumbing.
#[derive(Default)]
pub(crate) struct ShedsTab {
    pub snapshot: Option<Snapshot>,
    /// Index into `Snapshot::order()`.
    pub cursor: usize,
    /// Detail pane: index into the selected shed's touched yaks.
    pub line: usize,
    /// List: first visible row, kept across frames (yaks-9009).
    pub list_offset: Cell<usize>,
    /// Detail pane: first visible line, kept across frames.
    pub offset: Cell<usize>,
    /// Seconds since the snapshot arrived, refreshed by `settle_sheds` (so
    /// `render` stays pure and tests can freeze it).
    pub age: Option<u64>,
    /// Where to scan from (the TUI's cwd); `None` = never scan (read-only
    /// and test constructors).
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
        let (Some(cwd), Some(farm)) = (self.sheds.cwd.clone(), self.farm.as_ref()) else {
            return;
        };
        if self.sheds.in_flight.is_some() {
            self.sheds.queued = true;
            return;
        }
        let root = farm.root().to_path_buf();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(Snapshot::scan(&cwd, &root));
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

    /// Swap in a new snapshot, keeping the cursor on the same shed (by path)
    /// and the detail cursor on the same yak (by id).
    pub(crate) fn apply_sheds_snapshot(&mut self, snap: Snapshot) {
        let shed = self.selected_shed().map(|s| s.path.clone());
        let yak = self.selected_shed_yak().map(|y| y.id.clone());
        self.sheds.snapshot = Some(snap);
        self.touch();
        let s = self.sheds.snapshot.as_ref().unwrap();
        let order = s.order();
        self.sheds.cursor = shed
            .and_then(|p| order.iter().position(|&i| s.sheds[i].path == p))
            .unwrap_or(0)
            .min(order.len().saturating_sub(1));
        let yaks = order.get(self.sheds.cursor).map_or(&[][..], |&i| s.yaks(i));
        self.sheds.line = yak
            .and_then(|id| yaks.iter().position(|y| y.id == id))
            .unwrap_or(0)
            .min(yaks.len().saturating_sub(1));
    }

    /// The shed under the cursor, with its index into `Snapshot::sheds`.
    fn selected_shed_index(&self) -> Option<usize> {
        let s = self.sheds.snapshot.as_ref()?;
        s.order().get(self.sheds.cursor).copied()
    }

    pub(crate) fn selected_shed(&self) -> Option<&Shed> {
        let i = self.selected_shed_index()?;
        self.sheds.snapshot.as_ref()?.sheds.get(i)
    }

    /// The yak under the detail cursor.
    pub(crate) fn selected_shed_yak(&self) -> Option<&Touched> {
        self.selected_shed()?.touched.get(self.sheds.line)
    }

    fn move_shed(&mut self, delta: i32) {
        let n = self.sheds.snapshot.as_ref().map_or(0, |s| s.sheds.len());
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

    fn move_shed_yak(&mut self, delta: i32) {
        let n = self.selected_shed().map_or(0, |s| s.touched.len());
        if n == 0 {
            return;
        }
        let l = (self.sheds.line as i64 + delta as i64).clamp(0, n as i64 - 1);
        self.sheds.line = l as usize;
    }

    /// toque's state-header digest on the Sheds tab: the selected shed and yak.
    pub(crate) fn sheds_state_header(&self) -> String {
        let shed = self.selected_shed().map_or_else(|| "-".into(), Shed::name);
        let yak = self
            .selected_shed_yak()
            .map_or_else(|| "-".into(), |y| y.id.clone());
        let n = self.sheds.snapshot.as_ref().map_or(0, |s| s.sheds.len());
        format!("sheds={n} · shed={shed} · shed_yak={yak}")
    }

    /// The tab's count: sheds busy right now, `…` before the first scan.
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

    /// The Sheds tab's keys. Returns after handling every key, so yak keys
    /// never reach a yak from here.
    pub(crate) fn handle_sheds_key(&mut self, k: KeyEvent) {
        let half = (self.page / 2).max(1) as i32;
        let detail = self.focus == Focus::Detail;
        match k.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Tab | KeyCode::Char(']') => self.switch_tab(1),
            KeyCode::BackTab | KeyCode::Char('[') => self.switch_tab(-1),
            KeyCode::Char('v') => self.open_view_picker(),
            KeyCode::Char('?') => self.open_help(),
            KeyCode::Char('#') => self.open_goto_picker(),
            KeyCode::Char('r') => {
                self.request_sheds_scan();
                self.notification = Some("rescanning sheds".into());
            }
            KeyCode::Char('y') => self.copy_shed_thing(),
            _ if detail => self.handle_shed_detail_key(k, half),
            KeyCode::Char('j') | KeyCode::Down => self.move_shed(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_shed(-1),
            KeyCode::Char('g') => self.move_shed(i32::MIN / 4),
            KeyCode::Char('G') => self.move_shed(i32::MAX / 4),
            KeyCode::Char('d') | KeyCode::PageDown => self.move_shed(half),
            KeyCode::Char('u') | KeyCode::PageUp => self.move_shed(-half),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if self.selected_shed().is_some() {
                    self.focus = Focus::Detail;
                    self.sheds.line = 0;
                    self.sheds.offset.set(0);
                }
            }
            _ => {}
        }
    }

    fn handle_shed_detail_key(&mut self, k: KeyEvent, half: i32) {
        match k.code {
            KeyCode::Char('h') | KeyCode::Left | KeyCode::Esc => self.focus = Focus::List,
            KeyCode::Char('j') | KeyCode::Down => self.move_shed_yak(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_shed_yak(-1),
            KeyCode::Char('g') => self.move_shed_yak(i32::MIN / 4),
            KeyCode::Char('G') => self.move_shed_yak(i32::MAX / 4),
            KeyCode::Char('d') | KeyCode::PageDown => self.move_shed_yak(half),
            KeyCode::Char('u') | KeyCode::PageUp => self.move_shed_yak(-half),
            // Next / previous shed without leaving the detail (as J/K do
            // for yaks).
            KeyCode::Char('J') => self.move_shed(1),
            KeyCode::Char('K') => self.move_shed(-1),
            KeyCode::Enter => self.open_shed_yak(),
            _ => {}
        }
    }

    /// `y`: the yak id on a yak line of the detail, else the shed's path.
    fn copy_shed_thing(&mut self) {
        let text = if self.focus == Focus::Detail {
            self.selected_shed_yak().map(|y| y.id.clone())
        } else {
            None
        }
        .or_else(|| self.selected_shed().map(|s| s.path.display().to_string()));
        let Some(text) = text else { return };
        self.notification = Some(if crate::clipboard::copy_text(&text) {
            format!("copied {text}")
        } else {
            "clipboard unavailable".into()
        });
    }

    /// Enter on a yak in a shed's detail: open it (every shed shares this
    /// farm). A yak since removed from the farm says so.
    fn open_shed_yak(&mut self) {
        let Some(id) = self.selected_shed_yak().map(|y| y.id.clone()) else {
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

fn ahead_behind(s: &Shed) -> String {
    if s.head.is_none() {
        return "-".into();
    }
    let n = |c: Option<usize>| c.map_or_else(|| "?".into(), |c| c.to_string());
    format!("+{} -{}", n(s.ahead), n(s.behind))
}

fn pad(s: &str, w: usize) -> String {
    let t = truncate_disp(s, w);
    let fill = w.saturating_sub(disp_width(&t));
    format!("{t}{}", " ".repeat(fill))
}

/// One list row for a shed: mark, name, liveness, ahead/behind, dirty, then
/// what it is working on (else its newest yak) and its state.
fn shed_row(s: &Shed, name_w: usize, width: u16) -> Line<'static> {
    let mark = if s.open_asks().next().is_some() {
        "\u{23f3}"
    } else if s.current {
        "* "
    } else {
        "  "
    };
    let quiet = rank(s) >= 4;
    let base = if quiet { dim() } else { Style::new() };
    let mut spans = vec![
        Span::styled(format!("{mark} "), Style::new().fg(Color::Yellow)),
        Span::styled(
            pad(&s.name(), name_w),
            if quiet {
                dim()
            } else {
                Style::new().add_modifier(Modifier::BOLD)
            },
        ),
        Span::raw("  "),
        liveness_span(s),
        Span::styled(format!("{:<9}", ahead_behind(s)), base),
        Span::styled(
            match s.dirty {
                Some(d) if d > 0 => format!("{:<5}", format!("\u{270e}{d}")),
                _ => "     ".into(),
            },
            Style::new().fg(Color::Magenta),
        ),
    ];
    let mut tail = if !s.in_progress.is_empty() {
        format!("shaving: {}", s.in_progress.join(", "))
    } else {
        s.touched
            .first()
            .map(|t| format!("last: {} {}", t.id, t.title))
            .unwrap_or_default()
    };
    if let Some(st) = sheds::state_line(s) {
        if !tail.is_empty() {
            tail.push_str("  ");
        }
        tail.push_str(&st);
    }
    let used: usize = spans.iter().map(|sp| disp_width(&sp.content)).sum();
    let room = (width as usize).saturating_sub(used);
    // Padded to the full width so the cursor highlight spans the row.
    spans.push(Span::styled(pad(&tail, room), base));
    Line::from(spans)
}

pub(crate) fn render_sheds_list(app: &App, frame: &mut Frame, area: Rect) {
    let Some(snap) = &app.sheds.snapshot else {
        let msg = if app.sheds.cwd.is_some() {
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
    let order = snap.order();
    let name_w = order
        .iter()
        .map(|&i| disp_width(&snap.sheds[i].name()))
        .max()
        .unwrap_or(4)
        .clamp(4, 24);
    let focused = app.focus == Focus::List;
    let hl = if focused {
        Style::new().bg(Color::Indexed(237))
    } else {
        Style::new().bg(Color::Indexed(236))
    };
    // A footer line for the orphaned pins, when there are any.
    let footer = snap.orphaned > 0;
    let rows_h = (area.height as usize).saturating_sub(footer as usize * 2);
    let mut offset = app.sheds.list_offset.get();
    if app.sheds.cursor < offset {
        offset = app.sheds.cursor;
    } else if rows_h > 0 && app.sheds.cursor >= offset + rows_h {
        offset = app.sheds.cursor + 1 - rows_h;
    }
    app.sheds.list_offset.set(offset);
    let mut lines: Vec<Line> = order
        .iter()
        .enumerate()
        .skip(offset)
        .take(rows_h)
        .map(|(pos, &i)| {
            let row = shed_row(&snap.sheds[i], name_w, area.width);
            if pos == app.sheds.cursor {
                row.style(hl)
            } else {
                row
            }
        })
        .collect();
    if footer {
        while lines.len() < rows_h {
            lines.push(Line::default());
        }
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            format!(
                "   {} orphaned pins (finished threads; `yaks discover` lists them)",
                snap.orphaned
            ),
            dim(),
        )));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

/// Split `text` into lines no wider than `w` at spaces (a word longer than
/// `w` gets a line of its own and is truncated by the renderer).
fn wrap(text: &str, w: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in text.split(' ') {
        if !cur.is_empty() && disp_width(&cur) + 1 + disp_width(word) > w {
            out.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    out.push(cur);
    out
}

/// One touched yak's line in a shed's detail: `⏳` when it awaits a human,
/// id, status now, title, and when the shed last wrote it.
fn yak_line(y: &Touched) -> Line<'static> {
    let mut spans = Vec::new();
    if y.needs.as_deref() == Some("human") {
        spans.push(Span::raw("\u{23f3}"));
    }
    let status = y.status.map_or("gone", crate::model::Status::as_str);
    let done = matches!(
        y.status,
        None | Some(crate::model::Status::Shorn | crate::model::Status::Dead)
    );
    spans.push(Span::styled(
        format!("{}  ", y.id),
        Style::new().fg(Color::Blue),
    ));
    spans.push(Span::styled(
        format!("{status:<8}"),
        Style::new().fg(Color::Yellow),
    ));
    spans.push(Span::styled(
        y.title.clone(),
        if done { dim() } else { Style::new() },
    ));
    spans.push(Span::styled(format!("  {}", y.last), dim()));
    Line::from(spans)
}

/// The detail pane's lines for shed `i`, and the line each touched yak sits
/// on, indexed like `ShedsTab::line`.
fn detail_lines(snap: &Snapshot, i: usize, width: usize) -> (Vec<Line<'static>>, Vec<usize>) {
    let s = &snap.sheds[i];
    let label = |k: &str| Span::styled(format!("{k:<9}"), Style::new().fg(Color::Cyan));
    let mut lines: Vec<Line<'static>> = Vec::new();
    let here = if s.current { " (here)" } else { "" };
    let mut head = vec![Span::styled(
        format!("{}{here}", s.name()),
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )];
    if let Some(t) = &s.title {
        head.push(Span::raw(format!("  {t}")));
    }
    lines.push(Line::from(head));
    let mut field = |k: &str, v: String, style: Style| {
        for (n, part) in wrap(&v, width.saturating_sub(9).max(10))
            .into_iter()
            .enumerate()
        {
            let key = if n == 0 {
                label(k)
            } else {
                Span::raw(" ".repeat(9))
            };
            lines.push(Line::from(vec![key, Span::styled(part, style)]));
        }
    };
    field("path", s.path.display().to_string(), Style::new());
    let mut git = match (&s.branch, &s.head) {
        (Some(b), Some(h)) => format!("{b}@{h}"),
        (None, Some(h)) => format!("(detached)@{h}"),
        _ => "-".into(),
    };
    git.push_str(&format!(" \u{b7} {} vs here", ahead_behind(s)));
    if let Some(d) = s.dirty {
        git.push_str(&format!(" \u{b7} {d} dirty"));
    }
    field("git", git, Style::new());
    if let Some(e) = &s.error {
        field("error", e.clone(), Style::new().fg(Color::Red));
    }
    if let Some(st) = sheds::state_line(s) {
        field("state", st, Style::new());
    }
    if let Some(l) = sheds::label_line(s) {
        field("activity", l, Style::new());
    }
    lines.push(Line::default());
    let mut starts = Vec::new();
    if s.touched.is_empty() {
        lines.push(Line::from(Span::styled(
            "No yaks written from this checkout.",
            dim(),
        )));
        return (lines, starts);
    }
    lines.push(Line::from(Span::styled(
        format!("Yaks written here ({}), newest first", s.touched.len()),
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    for y in &s.touched {
        starts.push(lines.len());
        lines.push(yak_line(y));
    }
    (lines, starts)
}

pub(crate) fn render_shed_detail(app: &App, frame: &mut Frame, area: Rect) {
    let (Some(snap), Some(i)) = (&app.sheds.snapshot, app.selected_shed_index()) else {
        return;
    };
    let (mut lines, starts) = detail_lines(snap, i, area.width as usize);
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
            let hl = Style::new().bg(Color::Indexed(237));
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

/// A hand-built snapshot for tests and doc shots: the primary, a worker with
/// an open ask, an active worker, a finished (merged) one and an idle clone.
#[cfg(test)]
pub(crate) mod fixture {
    use super::*;
    use crate::model::Status;
    use crate::sheds::ShedKind;

    fn shed(name: &str, dir: &str) -> Shed {
        Shed {
            path: PathBuf::from(format!("/src/yaks/.delta/worktrees/{dir}/yaks")),
            kind: ShedKind::Delta,
            dir: Some(dir.into()),
            title: Some(format!("{name}: a worker")),
            slug: Some(name.into()),
            primary: false,
            current: false,
            branch: Some("main".into()),
            head: Some("1a2b3c4".into()),
            ahead: Some(1),
            behind: Some(0),
            shallow_here: false,
            dirty: Some(0),
            untracked: Some(0),
            merged: None,
            touched: vec![],
            in_progress: vec![],
            last_write: None,
            liveness: Liveness::Idle,
            error: None,
        }
    }

    fn touched(id: &str, title: &str, status: Status, last: &str) -> Touched {
        Touched {
            id: id.into(),
            title: title.into(),
            status: Some(status),
            needs: None,
            last: last.into(),
        }
    }

    /// Set a shed's activity from its touched list (newest first), as
    /// `sheds::discover` derives it.
    fn active(mut s: Shed, touched: Vec<Touched>, liveness: Liveness) -> Shed {
        s.in_progress = touched
            .iter()
            .filter(|t| t.status == Some(Status::Shaving))
            .map(|t| t.id.clone())
            .collect();
        s.last_write = touched.first().map(|t| t.last.clone());
        s.touched = touched;
        s.liveness = liveness;
        s
    }

    pub(crate) fn snapshot() -> Snapshot {
        let mut main = shed("main", "-");
        main.path = PathBuf::from("/src/yaks");
        main.kind = ShedKind::Worktree;
        main.dir = None;
        main.title = None;
        main.slug = None;
        main.primary = true;
        main.current = true;
        main.ahead = Some(0);
        let main = active(
            main,
            vec![touched(
                "yaks-6a34",
                "One farm per machine (umbrella)",
                Status::Shaving,
                "2026-10-13T00:05:00Z",
            )],
            Liveness::Active,
        );

        let mut asker = shed("needs-1-superseded", "epe9");
        asker.dirty = Some(2);
        asker.untracked = Some(1);
        let mut ask = touched(
            "yaks-8814",
            "landed: a superseded needs is not outstanding",
            Status::Shaving,
            "2026-10-13T00:15:37Z",
        );
        ask.needs = Some("human".into());
        let asker = active(asker, vec![ask], Liveness::Active);

        let mut perf = shed("perf-1-profile", "53sm");
        perf.dirty = Some(1);
        perf.untracked = Some(0);
        let perf = active(
            perf,
            vec![
                touched(
                    "yaks-07fe",
                    "yaks sheds is slow: profile it",
                    Status::Shaving,
                    "2026-10-13T00:12:57Z",
                ),
                touched(
                    "yaks-bc30",
                    "Perf: cache rows()",
                    Status::Shorn,
                    "2026-10-12T22:00:00Z",
                ),
            ],
            Liveness::Active,
        );

        let mut land = shed("land-1-landed", "h4n2");
        land.ahead = Some(0);
        land.behind = Some(5);
        land.merged = Some(true);
        let land = active(
            land,
            vec![touched(
                "yaks-ed5e",
                "yaks sheds: show a shed as landed",
                Status::Shorn,
                "2026-10-12T20:00:00Z",
            )],
            Liveness::Idle,
        );

        let mut quiet = shed("mig-2-yaks-migrate", "n5p2");
        quiet.behind = Some(23);

        Snapshot {
            // Path order, as `discover` returns them; the tab reorders.
            sheds: vec![main, quiet, land, asker, perf],
            orphaned: 53,
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

    fn names(app: &App) -> Vec<String> {
        let s = app.sheds.snapshot.as_ref().unwrap();
        s.order().iter().map(|&i| s.sheds[i].name()).collect()
    }

    #[test]
    fn order_is_attention_first_then_most_recent_writer() {
        let app = on_sheds(sample());
        assert_eq!(
            names(&app),
            [
                "main",
                "needs-1-superseded", // open ask
                "perf-1-profile",     // active
                "land-1-landed",      // idle, wrote most recently of the rest
                "mig-2-yaks-migrate", // idle, never wrote
            ]
        );
    }

    #[test]
    fn tab_count_is_busy_sheds() {
        let app = on_sheds(sample());
        // The asker and perf-1; main is not counted, the rest are idle.
        assert_eq!(app.sheds_count_label(), "2");
        let v = app.active_view().clone();
        assert_eq!(app.view_count(&v), 2);
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
        press(&mut app, "jjl"); // perf-1's detail
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_shed().unwrap().name(), "perf-1-profile");
        assert_eq!(app.selected_shed_yak().unwrap().id, "yaks-07fe");
        press(&mut app, "j");
        assert_eq!(app.selected_shed_yak().unwrap().id, "yaks-bc30");
        press(&mut app, "j"); // clamps
        assert_eq!(app.selected_shed_yak().unwrap().id, "yaks-bc30");
        press(&mut app, "K"); // previous shed, still in the detail
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_shed().unwrap().name(), "needs-1-superseded");
        assert_eq!(app.selected_shed_yak().unwrap().id, "yaks-8814");
        press(&mut app, "h");
        assert_eq!(app.focus, Focus::List);
    }

    #[test]
    fn a_new_snapshot_keeps_the_selected_shed_and_yak() {
        let mut app = on_sheds(sample());
        press(&mut app, "jjlj");
        assert_eq!(app.selected_shed_yak().unwrap().id, "yaks-bc30");
        // The next scan finds the asker's ask answered: it drops below perf-1
        // in the order, but the cursor stays on perf-1 and its yak.
        let mut snap = fixture::snapshot();
        snap.sheds[3].touched[0].needs = None;
        app.apply_sheds_snapshot(snap);
        assert_eq!(app.selected_shed().unwrap().name(), "perf-1-profile");
        assert_eq!(app.selected_shed_yak().unwrap().id, "yaks-bc30");
    }

    #[test]
    fn enter_opens_the_yak_or_says_it_is_gone() {
        // Only yaks-07fe is in this farm.
        let mut app = on_sheds(App::new(vec![task(
            "yaks-07fe",
            "slow",
            crate::model::Status::Shaving,
            2,
            None,
        )]));
        press(&mut app, "jl"); // the asker: yaks-8814 is not in this farm
        enter_key(&mut app);
        assert_eq!(
            app.notification.as_deref(),
            Some("yaks-8814 is no longer in this farm")
        );
        assert!(app.on_sheds_tab());
        press(&mut app, "J"); // perf-1: yaks-07fe
        enter_key(&mut app);
        assert!(!app.on_sheds_tab());
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_id().as_deref(), Some("yaks-07fe"));
    }

    #[test]
    fn a_background_scan_arrives_without_blocking() {
        // A farm outside any git repository: the scan fails fast, which is
        // still a real thread and channel round trip.
        let (proj, farm) = temp_farm(&[]);
        let mut app = on_sheds(App::new(vec![]));
        app.sheds.snapshot = None;
        app.touch();
        app.farm = Some(farm);
        app.sheds.cwd = Some(proj.clone());
        app.settle_sheds(false); // on the tab, never scanned: starts one
        assert!(app.sheds.scanning());
        assert_eq!(app.sheds_status_label(), "scanning\u{2026}");
        let start = Instant::now();
        while app.sheds.scanning() && start.elapsed() < Duration::from_secs(20) {
            std::thread::sleep(Duration::from_millis(10));
            app.settle_sheds(false);
        }
        let snap = app.sheds.snapshot.as_ref().expect("the scan arrived");
        assert!(snap.error.is_some(), "not a git repository is an error");
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
        app.sheds.cwd = Some(proj.clone());
        assert!(!app.on_sheds_tab());
        app.settle_sheds(false);
        assert!(!app.sheds.scanning());
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn sheds_tab_frames() {
        let mut app = on_sheds(sample());
        insta::assert_snapshot!("sheds_tab_list", draw(&app, 120, 14));
        press(&mut app, "jjl");
        insta::assert_snapshot!("sheds_tab_detail", draw(&app, 120, 18));
    }
}
