//! Mouse support (yaks-97a2): scroll wheel + left-click, as an `impl App` block
//! like the other submodules. `use super::*` inherits `tui.rs`'s scope.
//!
//! Hit-testing never recomputes layout: `render` records the rects it actually
//! laid out into [`HitMap`] (`App.hits`), and the handler reads them back. The
//! map is reset at the top of every frame, so a pane that isn't drawn (e.g. the
//! detail pane while the list has focus) can't be hit.
//!
//! Mouse back/forward (buttons 8/9) aren't reachable: crossterm 0.29's SGR
//! parser rejects any button beyond left/middle/right + the wheel ("We do not
//! support other buttons"), so they never arrive as events. `o`/`i` remain the
//! history keys.

use super::*;
use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};

/// Detail-pane lines scrolled per wheel notch.
const WHEEL_LINES: i32 = 3;

/// Screen regions from the last render, for mouse hit-testing.
#[derive(Default, Clone, Debug)]
pub(crate) struct HitMap {
    /// The list/tree area; row `y - list.y` shows `rows()[list_offset + ..]`.
    pub(crate) list: Option<Rect>,
    /// The list's scroll offset as ratatui resolved it this frame.
    pub(crate) list_offset: usize,
    /// The detail content area (below any sticky header); row `y - detail.y`
    /// shows detail line `detail.scroll + ..`.
    pub(crate) detail: Option<Rect>,
    /// Each drawn tab's rect and the view index it activates.
    pub(crate) tabs: Vec<(Rect, usize)>,
}

fn hit(r: Option<Rect>, col: u16, row: u16) -> Option<Rect> {
    r.filter(|r| r.contains(Position::new(col, row)))
}

impl App {
    /// Dispatch one mouse event. Overlays (editors, pickers, drawer, help) own
    /// their input, so the mouse is ignored while one is open.
    pub(crate) fn handle_mouse(&mut self, m: MouseEvent) {
        // A notification lasts until the next key, click or scroll. Not a
        // bare pointer move: mouse capture reports every motion, which would
        // wipe a toast before it could be read.
        if !matches!(m.kind, MouseEventKind::Moved) {
            self.notification = None;
        }
        self.handle_mouse_inner(m);
        self.drop_stale_hscroll();
    }

    fn handle_mouse_inner(&mut self, m: MouseEvent) {
        if !matches!(self.overlay, Overlay::None) {
            return;
        }
        let hits = self.hits.borrow().clone();
        let (col, row) = (m.column, m.row);
        match m.kind {
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let dir = if m.kind == MouseEventKind::ScrollDown {
                    1
                } else {
                    -1
                };
                if hit(hits.detail, col, row).is_some() {
                    self.scroll_detail(dir * WHEEL_LINES);
                } else if hit(hits.list, col, row).is_some() {
                    if self.focus == Focus::Detail {
                        // Keep the detail pane open and step through tasks (J/K).
                        self.detail_next_task(dir);
                    } else {
                        self.move_cursor(dir);
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(&(_, i)) = hits
                    .tabs
                    .iter()
                    .find(|(r, _)| r.contains(Position::new(col, row)))
                {
                    if i != self.view {
                        self.set_view(i);
                    }
                } else if let Some(r) = hit(hits.list, col, row) {
                    self.click_list_row(hits.list_offset + (row - r.y) as usize);
                } else if let Some(r) = hit(hits.detail, col, row) {
                    self.click_detail_line(self.detail.scroll as usize + (row - r.y) as usize);
                }
            }
            _ => {}
        }
    }

    /// Click on list row `i`: select it and focus the list; clicking the row
    /// that is already selected opens it in the detail pane (like Enter). With
    /// the detail already open, a click shows that row's yak in the detail and
    /// keeps it open (like `J`/`K`).
    fn click_list_row(&mut self, i: usize) {
        if i >= self.rows().len() {
            return;
        }
        // With a page open the row replaces it: select the row, close the
        // page and land where it was opened from.
        if self.doc.is_some() {
            self.close_doc();
            if self.focus == Focus::Detail {
                self.detail_next_task(i as i32 - self.cursor as i32);
            } else {
                self.cursor = i;
            }
            return;
        }
        if self.focus == Focus::Detail {
            self.detail_next_task(i as i32 - self.cursor as i32);
        } else if i == self.cursor && self.selected_doc().is_some() {
            // A document has no detail pane: the second click opens it.
            self.open_wiki_entry();
        } else if i == self.cursor {
            self.focus = Focus::Detail;
            self.detail.scroll = 0;
            self.detail.line = 0;
            self.detail.anchor = None;
            self.detail.find = None;
            self.detail.matched = 0;
        } else {
            self.cursor = i;
        }
    }

    /// Click on detail line `i`: move the line cursor there (focusing the
    /// detail); clicking the line already under the cursor follows its link.
    fn click_detail_line(&mut self, i: usize) {
        if i >= self.detail_line_count() {
            return;
        }
        self.focus = Focus::Detail;
        self.detail.anchor = None;
        // A click on an inline image's picture lands on its link row, the
        // image's one cursor stop; a second click there follows the link.
        let lines = self.detail_dlines();
        let stops = detail::cursor_stops(&lines);
        let i = stops[stops.partition_point(|&r| r <= i).saturating_sub(1)];
        if i == self.detail.line {
            self.follow_link();
        } else {
            self.detail.line = i;
        }
    }

    /// Wheel over the detail: scroll the viewport (not the cursor) by `delta`
    /// lines, then drag the line cursor along so it stays on screen.
    fn scroll_detail(&mut self, delta: i32) {
        let n = self.detail_line_count();
        if n == 0 {
            return;
        }
        let max = self.detail_max_scroll() as i32;
        self.detail.scroll = (self.detail.scroll as i32 + delta).clamp(0, max) as u16;
        let top = self.detail.scroll as usize;
        let shown = self.detail_rows_shown(self.detail.scroll);
        self.detail.line = self.detail.line.clamp(top, (top + shown - 1).min(n - 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_support::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn mouse(app: &mut App, kind: MouseEventKind, column: u16, row: u16) {
        app.handle_mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        });
    }

    fn click(app: &mut App, column: u16, row: u16) {
        mouse(app, MouseEventKind::Down(MouseButton::Left), column, row);
    }

    /// Screen row (y) whose text contains `needle`.
    fn row_of(frame: &str, needle: &str) -> u16 {
        frame
            .lines()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} not on screen:\n{frame}")) as u16
    }

    fn many(n: usize) -> App {
        App::new(
            (0..n)
                .map(|i| {
                    task(
                        &format!("t{i:02}"),
                        &format!("Task {i}"),
                        Status::Hairy,
                        3,
                        None,
                    )
                })
                .collect(),
        )
    }

    #[test]
    fn click_selects_list_row_then_opens_detail() {
        let mut app = sample();
        let before = draw(&app, 60, 10);
        let before_state = app.state_header();
        let y = row_of(&before, "Child A1");
        click(&mut app, 3, y);
        assert_eq!(app.selected_id().as_deref(), Some("a1"));
        assert_eq!(app.focus, Focus::List);
        let after = draw(&app, 60, 10);
        println!(
            "--- before click (x=3,y={y}) ---\n{before_state}\n{before}--- after ---\n{}\n{after}",
            app.state_header()
        );
        // A second click on the selected row opens it in the detail pane.
        click(&mut app, 3, y);
        assert_eq!(app.focus, Focus::Detail);
        // With the detail open (wide enough that the list shows beside it),
        // clicking another row shows that yak in the detail and keeps it open.
        let frame = draw(&app, 160, 10);
        let y0 = row_of(&frame, "Root A");
        click(&mut app, 3, y0);
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_id().as_deref(), Some("a0"));
        println!(
            "--- detail open, clicked another row ---\n{}",
            draw(&app, 160, 10)
        );
        // Clicking past the last row is a no-op.
        let mut app2 = sample();
        draw(&app2, 60, 10);
        click(&mut app2, 3, 8);
        assert_eq!(app2.cursor, 0);
    }

    #[test]
    fn click_list_row_respects_scroll_offset() {
        let mut app = many(30);
        app.cursor = 25; // forces the list to scroll
        let frame = draw(&app, 60, 10);
        let y = row_of(&frame, "t22");
        click(&mut app, 3, y);
        assert_eq!(app.selected_id().as_deref(), Some("t22"));
    }

    #[test]
    fn wheel_moves_list_selection() {
        let mut app = many(30);
        let before = draw(&app, 60, 10);
        for _ in 0..12 {
            mouse(&mut app, MouseEventKind::ScrollDown, 5, 4);
            draw(&app, 60, 10);
        }
        assert_eq!(app.selected_id().as_deref(), Some("t12"));
        let after = draw(&app, 60, 10);
        println!("--- before 12x wheel-down ---\n{before}--- after ---\n{after}");
        mouse(&mut app, MouseEventKind::ScrollUp, 5, 4);
        assert_eq!(app.selected_id().as_deref(), Some("t11"));
        // Wheel outside any pane (the help bar) does nothing.
        mouse(&mut app, MouseEventKind::ScrollDown, 5, 9);
        assert_eq!(app.selected_id().as_deref(), Some("t11"));
    }

    #[test]
    fn wheel_scrolls_detail_and_click_moves_line_cursor() {
        let mut t = task("d0", "Long", Status::Hairy, 3, None);
        t.body = (0..40)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut app = App::new(vec![t]);
        app.detail_page = 9;
        enter_key(&mut app);
        let before = draw(&app, 60, 12);
        // Wheel over the detail pane (right side) scrolls it.
        mouse(&mut app, MouseEventKind::ScrollDown, 40, 4);
        mouse(&mut app, MouseEventKind::ScrollDown, 40, 4);
        assert_eq!(app.detail.scroll, 6);
        assert!(app.detail.line >= 6, "cursor dragged into view");
        let after = draw(&app, 60, 12);
        println!("--- detail before 2x wheel-down ---\n{before}--- after ---\n{after}");
        mouse(&mut app, MouseEventKind::ScrollUp, 40, 4);
        assert_eq!(app.detail.scroll, 3);
        // Click a detail line: the line cursor lands on the clicked line.
        let frame = draw(&app, 60, 12);
        let y = row_of(&frame, "Tags:");
        click(&mut app, 40, y);
        assert_eq!(app.focus, Focus::Detail);
        let dl = app.detail_dlines();
        assert!(dl[app.detail.line].text.contains("Tags:"));
    }

    #[test]
    fn click_tab_switches_view() {
        let mut app = sample();
        let frame = draw(&app, 100, 10);
        let tabs = app.hits.borrow().tabs.clone();
        let (r, i) = tabs.iter().find(|(_, i)| *i != app.view).copied().unwrap();
        click(&mut app, r.x + 1, r.y);
        assert_eq!(app.view, i, "{frame}");
    }

    #[test]
    fn a_click_in_the_list_closes_an_open_page_and_selects_the_row() {
        let (proj, farm) = temp_farm(&[
            task("t0", "first", Status::Hairy, 3, None),
            task("t1", "second", Status::Hairy, 3, None),
        ]);
        farm.attach("t0", "notes.md", b"# Notes\n").unwrap();
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "l");
        draw(&app, 140, 20);
        tab_key(&mut app);
        enter_key(&mut app);
        assert!(app.attach().is_some());
        let frame = draw(&app, 140, 20);
        click(&mut app, 3, row_of(&frame, "second"));
        assert!(app.doc.is_none(), "the page closed");
        assert_eq!(app.selected_id().as_deref(), Some("t1"));
        // It was opened from the yak detail, so the detail shows the new row.
        assert_eq!(app.focus, Focus::Detail);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn mouse_ignored_while_overlay_open() {
        let mut app = sample();
        let frame = draw(&app, 60, 10);
        press(&mut app, "?");
        click(&mut app, 3, row_of(&frame, "Child A1"));
        assert_eq!(app.cursor, 0);
    }
}
