//! The Sheds tab (yaks-98f1): `yaks sheds` and `yaks changes <shed>` on
//! screen, live. One row per checkout of the repository (attention first),
//! and a detail pane with the selected shed's state and its per-yak changes.
//!
//! The data is exactly the CLI's: a scan calls `sheds::discover_all` (the
//! sheds and the orphaned pins, as `yaks sheds` does) and `sheds::changes`
//! (per shed with a farm of its own), and every summary text the CLI also prints comes
//! from the same `sheds` function (`farm_line`, `label_line`).
//!
//! A scan takes seconds (git subprocesses, several farms read), so it runs on
//! a background thread and comes back as one [`Snapshot`] over a channel;
//! [`App::settle_sheds`] picks it up from the event loop (and from toque's
//! `settle`), so key handling and the farm reload never wait on it. Scans run
//! at startup, every [`RESCAN_EVERY`] whatever view is shown (the yak views
//! annotate rows from the same snapshot, yaks-7204), and on `r`; one at a
//! time.
//!
//! The overlay on the yak views reads the snapshot through three queries only:
//! [`Snapshot::copies`] (what other sheds did to one yak),
//! [`Snapshot::open_asks`] (every other shed's outstanding ask) and
//! [`Snapshot::shed_only`] (the yaks that exist only in another shed, whose
//! full `Task` the scan read from that shed's farm; yaks-9631).
//!
//! The tab is a built-in view (`view::sheds_view`) whose yak rows are empty,
//! so every yak key is inert there; [`App::handle_sheds_key`] owns its keys.

use super::*;
use crate::sheds::{self, Changes, Liveness, Shed, ShedFarm, YakChange};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::time::Instant;

/// How often the sheds are rescanned (a scan is ~0.3 s since yaks-07fe).
pub(crate) const RESCAN_EVERY: Duration = Duration::from_secs(10);

/// One finished scan: every shed, its per-yak changes (parallel to `sheds`;
/// `None` for a shed with no farm of its own or a failed `changes`), and how
/// many pinned clones are orphaned (no checkout left).
pub(crate) struct Snapshot {
    pub sheds: Vec<Shed>,
    pub changes: Vec<Option<Changes>>,
    /// Parallel to `sheds`: the full `Task` of each yak that exists only in
    /// that shed (its `YakChange.added`, not landed), read once per scan from
    /// the shed's farm. Empty for the shed this TUI runs in and for sheds
    /// with nothing new.
    pub only_tasks: Vec<Vec<Task>>,
    pub orphaned: usize,
    /// The scan itself failed (e.g. not a git repository).
    pub error: Option<String>,
}

impl Snapshot {
    /// Run one scan, as `yaks sheds` + `yaks changes <each>` would.
    pub(crate) fn scan(cwd: &Path, farm_root: &Path) -> Snapshot {
        let (sheds, orphaned) = match sheds::discover_all(cwd, farm_root) {
            Ok(found) => found,
            Err(e) => {
                return Snapshot {
                    sheds: Vec::new(),
                    changes: Vec::new(),
                    only_tasks: Vec::new(),
                    orphaned: 0,
                    error: Some(format!("{e:#}")),
                };
            }
        };
        let changes = sheds
            .iter()
            .map(|s| match s.farm {
                ShedFarm::Own(_) => sheds::changes(s, cwd, farm_root, &sheds).ok(),
                _ => None,
            })
            .collect::<Vec<_>>();
        let only_tasks = sheds
            .iter()
            .zip(&changes)
            .map(|(s, c)| read_only_tasks(s, c.as_ref()))
            .collect();
        Snapshot {
            orphaned: orphaned.len(),
            sheds,
            changes,
            only_tasks,
            error: None,
        }
    }

    /// The yaks a shed's detail lists (its `changes`), empty when none.
    pub(crate) fn yaks(&self, i: usize) -> &[YakChange] {
        match self.changes.get(i) {
            Some(Some(Changes { yaks: Ok(y), .. })) => y,
            _ => &[],
        }
    }

    /// Indices into `sheds` in display order: attention first (see [`rank`]),
    /// then by name.
    pub(crate) fn order(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.sheds.len()).collect();
        idx.sort_by_key(|&i| (rank(&self.sheds[i]), self.sheds[i].name()));
        idx
    }

    /// What the OTHER sheds did to yak `id`: one `(shed, change)` per shed
    /// whose farm changed it and has not landed, in the tab's attention order.
    /// The shed this TUI runs in is left out (its farm is the one on screen),
    /// and so is a landed change (nothing outstanding about it). The one rule
    /// the overlay's row badges and detail section share.
    pub(crate) fn copies(&self, id: &str) -> Vec<(&Shed, &YakChange)> {
        self.order()
            .into_iter()
            .filter(|&i| !self.sheds[i].current)
            .filter_map(|i| {
                let yaks = self.yaks(i);
                let at = yaks.binary_search_by(|y| y.id.as_str().cmp(id)).ok()?;
                Some((&self.sheds[i], &yaks[at])).filter(|(_, y)| !y.landed)
            })
            .collect()
    }

    /// The yaks that exist only in another shed: `(shed, task)` with the
    /// shed's own copy of the yak, the shed this TUI runs in left out, in
    /// attention order. A yak added in several sheds is listed once, under the
    /// first of them (`copies(id).len() >= 2` is then the conflict). The App
    /// further drops any yak this farm also has.
    pub(crate) fn shed_only(&self) -> Vec<(&Shed, &Task)> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for i in self.order() {
            if self.sheds[i].current {
                continue;
            }
            for t in self.only_tasks.get(i).into_iter().flatten() {
                if seen.insert(t.id.as_str()) {
                    out.push((&self.sheds[i], t));
                }
            }
        }
        out
    }

    /// Every other shed's outstanding ask: `(shed, change)` for each yak
    /// whose shed copy is `needs: human` and not landed, the shed this TUI
    /// runs in left out, in attention order then by id.
    pub(crate) fn open_asks(&self) -> Vec<(&Shed, &YakChange)> {
        self.order()
            .into_iter()
            .filter(|&i| !self.sheds[i].current)
            .flat_map(|i| {
                self.yaks(i)
                    .iter()
                    .filter(|y| y.needs.as_deref() == Some("human") && !y.landed)
                    .map(move |y| (&self.sheds[i], y))
            })
            .collect()
    }

    /// Ids of yaks some other shed is asking a human about (the ids of
    /// [`Snapshot::open_asks`]): what the Inbox adds to the home `needs:` rows.
    pub(crate) fn asked_ids(&self) -> HashSet<&str> {
        self.open_asks()
            .iter()
            .map(|(_, y)| y.id.as_str())
            .collect()
    }

    /// Identity of each open ask, for noticing a new one between scans:
    /// shed path + yak id + the timestamp of the ask note (so asking again
    /// is a new ask).
    fn ask_keys(&self) -> HashSet<(PathBuf, String, String)> {
        self.open_asks()
            .into_iter()
            .map(|(s, y)| (s.path.clone(), y.id.clone(), ask_ts(y)))
            .collect()
    }

    /// Sheds with something outstanding: the tab's count.
    pub(crate) fn outstanding(&self) -> usize {
        self.sheds
            .iter()
            .filter(|s| sheds::is_outstanding(s))
            .count()
    }
}

/// The full tasks of the yaks only `shed` has (see [`Snapshot::only_tasks`]):
/// one read of its farm, only when `changes` lists one that has not landed.
/// An unreadable farm yields none (the badges and the Sheds tab still show
/// the change).
fn read_only_tasks(shed: &Shed, changes: Option<&Changes>) -> Vec<Task> {
    let Some(Changes { yaks: Ok(yaks), .. }) = changes else {
        return Vec::new();
    };
    let new: HashSet<&str> = yaks
        .iter()
        .filter(|y| y.added && !y.landed)
        .map(|y| y.id.as_str())
        .collect();
    if shed.current || new.is_empty() {
        return Vec::new();
    }
    let all = [Status::Hairy, Status::Shaving, Status::Shorn, Status::Dead];
    let Ok(root) = crate::store::discover_with(&shed.path, None) else {
        return Vec::new();
    };
    let mut tasks: Vec<Task> = crate::store::load(&root.root, &all)
        .unwrap_or_default()
        .into_iter()
        .filter(|t| new.contains(t.id.as_str()))
        .collect();
    tasks.sort_by(|a, b| a.id.cmp(&b.id));
    tasks
}

/// When the shed asked: the timestamp of the newest ask note it appended
/// (empty if it carries none, e.g. a hand-edited `needs:`).
fn ask_ts(y: &YakChange) -> String {
    y.notes
        .iter()
        .rev()
        .find(|n| crate::store::is_ask_text(&n.text))
        .map(|n| n.ts.clone())
        .unwrap_or_default()
}

/// A shed with an open ask that is still outstanding (not landed).
fn open_ask(s: &Shed) -> bool {
    match &s.farm {
        ShedFarm::Own(d) => d
            .needs
            .iter()
            .any(|(id, v)| v == "human" && !d.landed.contains(id)),
        _ => false,
    }
}

/// A shed whose farm cannot be compared (unreadable, absent, shared).
fn unreadable(s: &Shed) -> bool {
    !matches!(s.farm, ShedFarm::Own(_))
}

/// Attention order (Joel, yaks-98f1): the primary, then open asks, active,
/// outstanding, stale, quiet (nothing outstanding), unreadable last.
fn rank(s: &Shed) -> u8 {
    if s.primary {
        0
    } else if open_ask(s) {
        1
    } else if s.liveness == Liveness::Active {
        2
    } else if sheds::is_outstanding(s) {
        3
    } else if s.liveness == Liveness::Stale {
        4
    } else if !unreadable(s) {
        5
    } else {
        6
    }
}

/// The tab's state: the latest snapshot, the cursors, and the scan plumbing.
#[derive(Default)]
pub(crate) struct ShedsTab {
    pub snapshot: Option<Snapshot>,
    /// Index into `Snapshot::order()`.
    pub cursor: usize,
    /// Detail pane: index into the selected shed's yaks.
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
    /// The open asks of the last good scan, to flash a new one; `None` until
    /// the first, which sets the baseline without a flash.
    seen_asks: Option<HashSet<(PathBuf, String, String)>>,
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

    /// Pick up a finished scan, start a due one, refresh the age. Cheap; the
    /// event loop calls it every tick. `block` waits for a running scan
    /// (toque's `settle`, so a headless frame shows the settled state).
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
        if due && !self.sheds.scanning() {
            self.request_sheds_scan();
        }
        self.sheds.age = self.sheds.scanned_at.map(|t| t.elapsed().as_secs());
    }

    /// Swap in a new snapshot, keeping the cursor on the same shed (by path)
    /// and the detail cursor on the same yak (by id).
    pub(crate) fn apply_sheds_snapshot(&mut self, snap: Snapshot) {
        let shed = self.selected_shed().map(|s| s.path.clone());
        let yak = self.selected_shed_yak().map(|y| y.id.clone());
        if snap.error.is_none() {
            let keys = snap.ask_keys();
            if let Some(seen) = &self.sheds.seen_asks {
                let new: Vec<_> = snap
                    .open_asks()
                    .into_iter()
                    .filter(|(s, y)| !seen.contains(&(s.path.clone(), y.id.clone(), ask_ts(y))))
                    .collect();
                if let Some((s, y)) = new.first() {
                    let more = match new.len() - 1 {
                        0 => String::new(),
                        n => format!(" (+{n} more)"),
                    };
                    self.notification = Some(format!(
                        "\u{23f3} new ask in {}: {} {}{more}",
                        s.name(),
                        y.id,
                        y.title
                    ));
                }
            }
            self.sheds.seen_asks = Some(keys);
        }
        self.sheds.snapshot = Some(snap);
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

    pub(crate) fn selected_shed_yak(&self) -> Option<&YakChange> {
        let i = self.selected_shed_index()?;
        self.sheds.snapshot.as_ref()?.yaks(i).get(self.sheds.line)
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
        let n = self
            .selected_shed_index()
            .zip(self.sheds.snapshot.as_ref())
            .map_or(0, |(i, s)| s.yaks(i).len());
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

    /// The tab's count: outstanding sheds, `…` before the first scan.
    pub(crate) fn sheds_count_label(&self) -> String {
        match &self.sheds.snapshot {
            Some(s) => format_count(s.outstanding()),
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
            KeyCode::Char('a') => self.open_answer_in_shed(),
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

    /// Enter on a yak in a shed's detail: open this checkout's copy.
    fn open_shed_yak(&mut self) {
        let Some(id) = self.selected_shed_yak().map(|y| y.id.clone()) else {
            return;
        };
        if self.task(&id).is_some() {
            self.goto_task(&id);
        } else {
            let name = self.selected_shed().map(Shed::name).unwrap_or_default();
            self.notification = Some(format!("{id} exists only in shed {name}"));
        }
    }

    /// `a` on a yak awaiting a human in a shed's detail: answer it there
    /// (`yaks answer <id>@<shed>`).
    fn open_answer_in_shed(&mut self) {
        let (Some(shed), Some(y)) = (self.selected_shed(), self.selected_shed_yak()) else {
            return;
        };
        if y.needs.as_deref() != Some("human") {
            self.notification = Some(format!("{} is not awaiting a human", y.id));
            return;
        }
        let (id, shed) = (y.id.clone(), shed.clone());
        self.open_answer_in_shed_prompt(id, &shed);
    }

    /// The prompt for answering `id` in `shed` (`yaks answer <id>@<shed>`).
    pub(crate) fn open_answer_in_shed_prompt(&mut self, id: String, shed: &Shed) {
        self.overlay = Overlay::Edit(Editor::new(
            self.editor_vim,
            true,
            format!(
                "Answer {id} in shed {} (Enter save · Ctrl-C cancel): ",
                shed.name()
            ),
            "",
            EditAction::AnswerInShed {
                id,
                shed: shed.path.clone(),
            },
        ));
    }

    /// Commit an answer typed for a shed's ask: the same write as the CLI's
    /// `answer <id>@<shed>`, left uncommitted in the shed's working tree.
    pub(crate) fn commit_answer_in_shed(&mut self, id: &str, path: &Path, note: Option<&str>) {
        let Some(farm) = &self.farm else { return };
        let Some(shed) = self
            .sheds
            .snapshot
            .as_ref()
            .and_then(|s| s.sheds.iter().find(|s| s.path == path))
        else {
            self.notification = Some("that shed is gone; rescan with r".into());
            return;
        };
        let actor = crate::actor::resolve(None);
        self.notification = Some(
            match farm.answer_in_shed(shed, id, actor.as_deref(), note, false) {
                Ok(_) => format!(
                    "answered {id} in {}: uncommitted there, for its worker to commit",
                    shed.name()
                ),
                Err(e) => format!("answer failed: {e:#}"),
            },
        );
        self.request_sheds_scan();
    }
}

// -- rendering ---------------------------------------------------------------

fn dim() -> Style {
    Style::new().add_modifier(Modifier::DIM)
}

fn liveness_span(s: &Shed) -> Span<'static> {
    match s.liveness {
        _ if unreadable(s) => Span::styled(format!("{:<7}", "\u{b7}"), dim()),
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

/// One list row for a shed.
fn shed_row(s: &Shed, name_w: usize, width: u16) -> Line<'static> {
    let mark = if open_ask(s) {
        "\u{23f3}"
    } else if s.current {
        "* "
    } else {
        "  "
    };
    let quiet = rank(s) >= 5;
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
    let mut tail = sheds::farm_line(s);
    if !s.in_progress.is_empty() {
        tail.push_str(&format!("  shaving: {}", s.in_progress.join(", ")));
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
                "   {} orphaned pins (finished threads; `yaks sheds` lists them)",
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

/// How many lines of an ask's question the detail shows under its note.
const ASK_LINES: usize = 6;

/// The detail pane's lines for shed `i`, and the line each yak starts on.
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
    field("farm", sheds::farm_line(s), Style::new());
    if let Some(l) = sheds::label_line(s) {
        field("label", l, Style::new());
    }
    if let Some(a) = &s.farm_activity {
        field("active", a.clone(), dim());
    }
    lines.push(Line::default());
    let yaks = snap.yaks(i);
    let mut starts = Vec::new();
    if yaks.is_empty() {
        lines.push(Line::from(Span::styled(
            "No yak changes of its own.",
            dim(),
        )));
        return (lines, starts);
    }
    lines.push(Line::from(Span::styled(
        format!(
            "Changes ({} {})",
            yaks.len(),
            if yaks.len() == 1 { "yak" } else { "yaks" }
        ),
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    for y in yaks {
        starts.push(lines.len());
        // The same bracket words as `yaks changes`.
        let mut what = match (y.added, y.moved) {
            (true, _) => format!("new, {}", y.status),
            (false, Some((from, to))) => format!("{from} -> {to}"),
            (false, None) => y.status.to_string(),
        };
        if y.edited {
            what.push_str(", edited");
        }
        if y.landed {
            what.push_str(", landed");
        }
        let style = if y.landed { dim() } else { Style::new() };
        let mut spans = vec![
            Span::styled(format!("{}  ", y.id), Style::new().fg(Color::Blue)),
            Span::styled(format!("[{what}]  "), Style::new().fg(Color::Yellow)),
        ];
        if y.needs.as_deref() == Some("human") && !y.landed {
            spans.insert(0, Span::raw("\u{23f3}"));
        }
        spans.push(Span::styled(y.title.clone(), style));
        lines.push(Line::from(spans));
        for n in &y.notes {
            let first = n
                .text
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .unwrap_or("");
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {}  {}  ", n.ts, n.actor.as_deref().unwrap_or("-")),
                    dim(),
                ),
                Span::styled(first.to_string(), style),
            ]));
            // An ask shows its question too: it is what `a` answers.
            if crate::store::is_ask_text(&n.text) {
                let rest = n.text.lines().skip_while(|l| l.trim() != first).skip(1);
                for l in rest.filter(|l| !l.trim().is_empty()).take(ASK_LINES) {
                    for part in wrap(l.trim(), width.saturating_sub(6).max(10)) {
                        lines.push(Line::from(Span::styled(
                            format!("      {part}"),
                            Style::new().fg(Color::Yellow),
                        )));
                    }
                }
            }
        }
        if let Some(v) = &y.needs {
            lines.push(Line::from(Span::styled(
                format!("    needs: {v}"),
                Style::new().fg(Color::Yellow),
            )));
        }
    }
    (lines, starts)
}

pub(crate) fn render_shed_detail(app: &App, frame: &mut Frame, area: Rect) {
    let (Some(snap), Some(i)) = (&app.sheds.snapshot, app.selected_shed_index()) else {
        return;
    };
    let (mut lines, starts) = detail_lines(snap, i, area.width as usize);
    let h = area.height as usize;
    // Keep the selected yak's block in view (its first line at least).
    let mut offset = app.sheds.offset.get();
    if let Some(&start) = starts.get(app.sheds.line) {
        let end = starts
            .get(app.sheds.line + 1)
            .copied()
            .unwrap_or(lines.len());
        if start < offset {
            offset = start;
        } else if end > offset + h {
            offset = (end.saturating_sub(h)).min(start);
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
/// an open ask, an active worker, a landed one and an old-layout clone.
#[cfg(test)]
pub(crate) mod fixture {
    use super::*;
    use crate::sheds::{BaseKind, FarmBase, FarmDelta, Moved, ShedKind};
    use crate::store::NoteEntry;

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
            farm_activity: Some("2026-10-13T00:10:00Z".into()),
            farm: ShedFarm::Own(FarmDelta::default()),
            farm_base: Some(FarmBase {
                kind: BaseKind::Fork,
                sha: "0f0f0f0".into(),
            }),
            who: vec![],
            in_progress: vec![],
            liveness: Liveness::Idle,
            error: None,
        }
    }

    fn note(ts: &str, actor: &str, text: &str) -> NoteEntry {
        NoteEntry {
            ts: ts.into(),
            actor: Some(actor.into()),
            text: text.into(),
        }
    }

    fn change(id: &str, title: &str) -> YakChange {
        YakChange {
            id: id.into(),
            title: title.into(),
            status: "shaving",
            added: false,
            moved: None,
            notes: vec![],
            needs: None,
            edited: false,
            landed: false,
        }
    }

    fn changes_of(s: &Shed, yaks: Vec<YakChange>) -> Option<Changes> {
        Some(Changes {
            name: s.name(),
            path: s.path.clone(),
            primary: s.primary,
            base: s.farm_base.clone(),
            yaks: Ok(yaks),
        })
    }

    /// `snap` plus a yak that exists only in the shed named `shed`: a new
    /// change (`added`, `status`) in its `changes` and the shed's full copy in
    /// `only_tasks`, as the scan reads them.
    pub(crate) fn with_shed_only(mut snap: Snapshot, shed: &str, task: Task) -> Snapshot {
        let i = snap.sheds.iter().position(|s| s.name() == shed).unwrap();
        let mut c = change(&task.id, &task.title);
        c.added = true;
        c.status = match task.status {
            Status::Hairy => "hairy",
            Status::Shaving => "shaving",
            Status::Shorn => "shorn",
            Status::Dead => "dead",
        };
        c.needs = task.needs.clone();
        if let Some(Some(Changes { yaks: Ok(y), .. })) = snap.changes.get_mut(i) {
            y.push(c);
            y.sort_by(|a, b| a.id.cmp(&b.id));
        }
        snap.only_tasks[i].push(task);
        snap
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
        main.farm_base = Some(FarmBase {
            kind: BaseKind::Origin,
            sha: "9e9e9e9".into(),
        });

        let mut asker = shed("needs-1-superseded", "epe9");
        asker.dirty = Some(2);
        asker.who = vec!["needs-1".into()];
        asker.in_progress = vec!["yaks-8814".into()];
        asker.liveness = Liveness::Active;
        asker.farm = ShedFarm::Own(FarmDelta {
            notes: vec![("yaks-8814".into(), 2)],
            needs: vec![("yaks-8814".into(), "human".into())],
            ..Default::default()
        });
        let mut ask = change("yaks-8814", "landed: a superseded needs is not outstanding");
        ask.notes = vec![
            note(
                "2026-10-13T00:12:55Z",
                "needs-1",
                "Start: Tue Oct 13 00:12:52 UTC 2026",
            ),
            note(
                "2026-10-13T00:15:37Z",
                "needs-1",
                "asked: needs human\nMatch the ask by timestamp only, or the whole entry?",
            ),
        ];
        ask.needs = Some("human".into());

        let mut perf = shed("perf-1-profile", "53sm");
        perf.dirty = Some(1);
        perf.who = vec!["perf-1".into()];
        perf.in_progress = vec!["yaks-07fe".into()];
        perf.liveness = Liveness::Active;
        perf.farm = ShedFarm::Own(FarmDelta {
            notes: vec![("yaks-07fe".into(), 1)],
            ..Default::default()
        });
        let mut slow = change("yaks-07fe", "yaks sheds is slow: profile it");
        slow.notes = vec![note(
            "2026-10-13T00:12:57Z",
            "perf-1",
            "43 git calls are 0.95 s of 2.4 s",
        )];

        let mut land = shed("land-1-landed", "h4n2");
        land.ahead = Some(0);
        land.behind = Some(5);
        land.who = vec!["land-1".into()];
        land.farm = ShedFarm::Own(FarmDelta {
            moved: vec![Moved {
                id: "yaks-ed5e".into(),
                from: "hairy",
                to: "shorn",
            }],
            landed: vec!["yaks-ed5e".into()],
            ..Default::default()
        });
        let mut shorn = change("yaks-ed5e", "yaks sheds: show a shed as landed");
        shorn.moved = Some(("hairy", "shorn"));
        shorn.status = "shorn";
        shorn.landed = true;

        let mut old = shed("mig-2-yaks-migrate", "n5p2");
        old.behind = Some(23);
        old.farm = ShedFarm::Unreadable("old layout (pre-0.0.16 clone): not read".into());
        old.farm_base = None;

        let c_main = changes_of(&main, vec![]);
        let c_asker = changes_of(&asker, vec![ask]);
        let c_perf = changes_of(&perf, vec![slow]);
        let c_land = changes_of(&land, vec![shorn]);
        Snapshot {
            // Path order, as `discover` returns them; the tab reorders.
            sheds: vec![main, old, land, asker, perf],
            changes: vec![c_main, None, c_land, c_asker, c_perf],
            only_tasks: vec![Vec::new(); 5],
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
    fn order_is_attention_first() {
        let app = on_sheds(sample());
        assert_eq!(
            names(&app),
            [
                "main",
                "needs-1-superseded", // open ask
                "perf-1-profile",     // active
                "land-1-landed",      // quiet: all landed
                "mig-2-yaks-migrate", // unreadable
            ]
        );
    }

    #[test]
    fn tab_count_is_outstanding_sheds() {
        let app = on_sheds(sample());
        // The asker and perf-1; main has nothing, land-1 is all landed and
        // the old-layout clone is unreadable.
        assert_eq!(app.sheds_count_label(), "2");
        let v = app.active_view().clone();
        assert_eq!(app.view_count(&v), 2);
    }

    #[test]
    fn yak_keys_are_inert_on_the_sheds_tab() {
        let mut app = on_sheds(sample());
        for c in ['S', 'E', 'X', 'c', 'D', 'M', '/', 'f', '*', 'm'] {
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
        press(&mut app, "jl"); // the asker's detail
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_shed().unwrap().name(), "needs-1-superseded");
        assert_eq!(app.selected_shed_yak().unwrap().id, "yaks-8814");
        press(&mut app, "J"); // next shed, still in the detail
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_shed().unwrap().name(), "perf-1-profile");
        assert_eq!(app.selected_shed_yak().unwrap().id, "yaks-07fe");
        press(&mut app, "h");
        assert_eq!(app.focus, Focus::List);
    }

    #[test]
    fn a_new_snapshot_keeps_the_selected_shed_and_yak() {
        let mut app = on_sheds(sample());
        press(&mut app, "jjl");
        assert_eq!(app.selected_shed().unwrap().name(), "perf-1-profile");
        // The next scan finds the asker's ask answered: it drops below perf-1
        // in the order, but the cursor stays on perf-1.
        let mut snap = fixture::snapshot();
        if let ShedFarm::Own(d) = &mut snap.sheds[3].farm {
            d.needs.clear();
        }
        snap.sheds[3].liveness = Liveness::Idle;
        app.apply_sheds_snapshot(snap);
        assert_eq!(app.selected_shed().unwrap().name(), "perf-1-profile");
        assert_eq!(app.selected_shed_yak().unwrap().id, "yaks-07fe");
    }

    #[test]
    fn enter_opens_this_checkouts_copy_or_says_it_is_shed_only() {
        // Only yaks-07fe exists here.
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
            Some("yaks-8814 exists only in shed needs-1-superseded")
        );
        assert!(app.on_sheds_tab());
        press(&mut app, "J"); // perf-1: yaks-07fe
        enter_key(&mut app);
        assert!(!app.on_sheds_tab());
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_id().as_deref(), Some("yaks-07fe"));
    }

    #[test]
    fn a_answers_only_an_ask() {
        let mut app = on_sheds(sample());
        press(&mut app, "jl"); // the asker's yaks-8814 is needs: human
        press(&mut app, "a");
        match &app.overlay {
            Overlay::Edit(ed) => assert!(matches!(
                &ed.action,
                EditAction::AnswerInShed { id, shed }
                    if id == "yaks-8814" && shed.ends_with("epe9/yaks")
            )),
            _ => panic!("a did not open the answer prompt"),
        }
        app.overlay = Overlay::None;
        press(&mut app, "J"); // perf-1 has no ask
        press(&mut app, "a");
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(
            app.notification.as_deref(),
            Some("yaks-07fe is not awaiting a human")
        );
    }

    #[test]
    fn a_background_scan_arrives_without_blocking() {
        // A farm outside any git repository: the scan fails fast, which is
        // still a real thread and channel round trip.
        let (proj, farm) = temp_farm(&[]);
        let mut app = on_sheds(App::new(vec![]));
        app.sheds.snapshot = None;
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
    fn copies_and_open_asks_leave_out_this_shed_and_landed_changes() {
        let mut snap = fixture::snapshot();
        // Make the shed this TUI runs in (main, current) change yaks-8814
        // too: it is the farm on screen, so it is never a "copy".
        let main = snap.sheds.iter().position(|s| s.current).unwrap();
        let mut own = snap.yaks(3)[0].clone();
        own.needs = Some("human".into());
        if let Some(Some(Changes { yaks: Ok(y), .. })) = snap.changes.get_mut(main) {
            y.push(own);
        }
        let copies: Vec<String> = snap
            .copies("yaks-8814")
            .iter()
            .map(|(s, _)| s.name())
            .collect();
        assert_eq!(copies, ["needs-1-superseded"]);
        // land-1's change to yaks-ed5e has landed: not a copy.
        assert!(snap.copies("yaks-ed5e").is_empty());
        let asks: Vec<(String, String)> = snap
            .open_asks()
            .iter()
            .map(|(s, y)| (s.name(), y.id.clone()))
            .collect();
        assert_eq!(
            asks,
            [("needs-1-superseded".to_string(), "yaks-8814".to_string())]
        );
    }

    // -- the Inbox overlay (yaks-1ea3) ----------------------------------------

    /// An app on the Inbox with the fixture snapshot: yaks-8814 is in this farm
    /// (home copy not blocked) and asked about in needs-1-superseded;
    /// yaks-07fe is home-blocked; yaks-0001 is plain.
    fn on_inbox() -> App {
        let mut blocked = task(
            "yaks-07fe",
            "home blocked",
            crate::model::Status::Shaving,
            2,
            None,
        );
        blocked.needs = Some("human".into());
        let mut app = App::new(vec![
            task(
                "yaks-8814",
                "a superseded needs",
                crate::model::Status::Shaving,
                2,
                None,
            ),
            blocked,
            task("yaks-0001", "plain", crate::model::Status::Hairy, 2, None),
        ]);
        app.apply_sheds_snapshot(fixture::snapshot());
        let i = app.views.iter().position(|v| v.key == "inbox").unwrap();
        app.set_view(i);
        app
    }

    fn inbox_ids(app: &App) -> Vec<String> {
        app.rows().iter().map(|r| r.task.id.clone()).collect()
    }

    #[test]
    fn inbox_includes_a_home_yak_whose_shed_copy_asks() {
        let app = on_inbox();
        let mut ids = inbox_ids(&app);
        ids.sort();
        // yaks-8814 (shed asks) and yaks-07fe (home asks); not yaks-0001, and
        // the shed-only asks are no rows (yaks-8814 is in this farm, but a
        // yak only in a shed would not be).
        assert_eq!(ids, ["yaks-07fe", "yaks-8814"]);
        let v = app.active_view().clone();
        assert_eq!(app.view_count(&v), 2);
    }

    #[test]
    fn inbox_leaves_out_a_shed_only_ask() {
        let mut app = on_inbox();
        // Nothing of the asker's yak is in this farm any more.
        app.all.retain(|t| t.id != "yaks-8814");
        assert_eq!(inbox_ids(&app), ["yaks-07fe"]);
        let v = app.active_view().clone();
        assert_eq!(app.view_count(&v), 1);
    }

    #[test]
    fn inbox_without_a_snapshot_is_home_needs_only() {
        let mut app = on_inbox();
        app.sheds.snapshot = None;
        assert_eq!(inbox_ids(&app), ["yaks-07fe"]);
    }

    #[test]
    fn a_on_a_shed_ask_row_answers_it_in_that_shed() {
        let mut app = on_inbox();
        let pos = app
            .rows()
            .iter()
            .position(|r| r.task.id == "yaks-8814")
            .unwrap();
        app.cursor = pos;
        press(&mut app, "a");
        match &app.overlay {
            Overlay::Edit(ed) => {
                assert!(
                    ed.label
                        .contains("Answer yaks-8814 in shed needs-1-superseded"),
                    "{}",
                    ed.label
                );
                assert!(matches!(
                    &ed.action,
                    EditAction::AnswerInShed { id, shed }
                        if id == "yaks-8814" && shed.ends_with("epe9/yaks")
                ));
            }
            _ => panic!("a did not open the answer-in-shed prompt"),
        }
    }

    #[test]
    fn a_on_a_home_blocked_row_is_the_plain_answer() {
        let mut app = on_inbox();
        let pos = app
            .rows()
            .iter()
            .position(|r| r.task.id == "yaks-07fe")
            .unwrap();
        app.cursor = pos;
        press(&mut app, "a");
        match &app.overlay {
            Overlay::Edit(ed) => {
                assert!(matches!(&ed.action, EditAction::Answer(id) if id == "yaks-07fe"))
            }
            _ => panic!("a did not open the plain answer prompt"),
        }
    }

    #[test]
    fn a_outside_the_inbox_is_as_before() {
        // The same yak in the Hairy-style tree view: `a` asks, as ever.
        let mut app = on_inbox();
        app.set_view(0);
        app.cursor = app
            .rows()
            .iter()
            .position(|r| r.task.id == "yaks-8814")
            .unwrap_or(0);
        let id = app.selected_id().unwrap();
        press(&mut app, "a");
        match &app.overlay {
            Overlay::Edit(ed) => assert!(
                !matches!(&ed.action, EditAction::AnswerInShed { .. }),
                "{id}: a answered in a shed outside the Inbox"
            ),
            _ => panic!("a opened nothing"),
        }
    }

    #[test]
    fn inbox_frame_with_a_shed_ask_row() {
        let app = on_inbox();
        insta::assert_snapshot!("inbox_shed_ask_row", draw(&app, 100, 12));
    }

    // -- the arrival flash ------------------------------------------------------

    /// The fixture with one more ask in perf-1-profile (yaks-07fe).
    fn with_perf_ask(ts: &str) -> Snapshot {
        let mut snap = fixture::snapshot();
        let perf = snap
            .sheds
            .iter()
            .position(|s| s.name() == "perf-1-profile")
            .unwrap();
        if let Some(Some(Changes { yaks: Ok(y), .. })) = snap.changes.get_mut(perf) {
            y[0].needs = Some("human".into());
            y[0].notes.push(crate::store::NoteEntry {
                ts: ts.into(),
                actor: Some("perf-1".into()),
                text: "asked: needs human\nWhich?".into(),
            });
        }
        snap
    }

    #[test]
    fn the_first_snapshot_does_not_flash() {
        let mut app = App::new(vec![]);
        app.apply_sheds_snapshot(fixture::snapshot());
        assert_eq!(app.notification, None);
    }

    #[test]
    fn a_new_ask_flashes_once_and_a_re_ask_flashes_again() {
        let mut app = App::new(vec![]);
        app.apply_sheds_snapshot(fixture::snapshot());
        app.apply_sheds_snapshot(with_perf_ask("2026-10-13T00:20:00Z"));
        assert_eq!(
            app.notification.as_deref(),
            Some("\u{23f3} new ask in perf-1-profile: yaks-07fe yaks sheds is slow: profile it")
        );
        // The same ask again: nothing new.
        app.notification = None;
        app.apply_sheds_snapshot(with_perf_ask("2026-10-13T00:20:00Z"));
        assert_eq!(app.notification, None);
        // Asked again (a new timestamp): it counts again.
        app.apply_sheds_snapshot(with_perf_ask("2026-10-13T00:30:00Z"));
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .contains("new ask in perf-1-profile")
        );
    }

    #[test]
    fn several_new_asks_name_the_first_and_count_the_rest() {
        let mut app = App::new(vec![]);
        let mut first = fixture::snapshot();
        // The asker has not asked yet in the first snapshot.
        let asker = first
            .sheds
            .iter()
            .position(|s| s.name() == "needs-1-superseded")
            .unwrap();
        if let Some(Some(Changes { yaks: Ok(y), .. })) = first.changes.get_mut(asker) {
            y[0].needs = None;
        }
        app.apply_sheds_snapshot(first);
        app.apply_sheds_snapshot(with_perf_ask("2026-10-13T00:20:00Z"));
        let n = app.notification.clone().unwrap();
        // Attention order: the asker's shed ranks above perf-1.
        assert_eq!(
            n,
            "\u{23f3} new ask in needs-1-superseded: yaks-8814 landed: a superseded needs is not outstanding (+1 more)"
        );
    }

    #[test]
    fn this_sheds_own_asks_do_not_flash() {
        let mut app = App::new(vec![]);
        app.apply_sheds_snapshot(fixture::snapshot());
        let mut snap = with_perf_ask("2026-10-13T00:20:00Z");
        for s in &mut snap.sheds {
            s.current = s.name() == "perf-1-profile";
        }
        app.apply_sheds_snapshot(snap);
        assert_eq!(app.notification, None);
    }

    #[test]
    fn sheds_tab_frames() {
        let mut app = on_sheds(sample());
        insta::assert_snapshot!("sheds_tab_list", draw(&app, 120, 14));
        press(&mut app, "jl");
        insta::assert_snapshot!("sheds_tab_detail", draw(&app, 120, 18));
    }
}
