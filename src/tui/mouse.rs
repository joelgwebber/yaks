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
    /// The open document's page area (the text viewer, below its header and
    /// above any peek pane); row `y - doc.y` shows row `nav.scroll + ..`.
    pub(crate) doc: Option<Rect>,
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
                } else if hit(hits.doc, col, row).is_some() {
                    self.scroll_doc(dir * WHEEL_LINES);
                } else if hit(hits.list, col, row).is_some() {
                    // The wheel scrolls the list's viewport, as it scrolls the
                    // detail; the cursor only follows to stay on screen
                    // (yaks-000b). With the detail open it still steps yaks.
                    if self.focus == Focus::Detail && self.doc.is_none() {
                        self.detail_next_task(dir);
                    } else {
                        self.scroll_list(dir * WHEEL_LINES, hits.list.map_or(1, |r| r.height));
                    }
                }
            }
            // A horizontal wheel (a trackpad swipe, a tilt wheel) scrolls the
            // code block or table under the pointer sideways (yaks-000b): the
            // line cursor goes to that row first, as a click would, since a
            // block's offset belongs to the row the cursor is on.
            MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => {
                let dir = if m.kind == MouseEventKind::ScrollRight {
                    1
                } else {
                    -1
                };
                if let Some(r) = hit(hits.detail, col, row) {
                    let i = self.detail.scroll as usize + (row - r.y) as usize;
                    if i < self.detail_line_count() {
                        self.focus = Focus::Detail;
                        self.detail.line = i;
                        self.scroll_block(dir);
                    }
                } else if let Some(r) = hit(hits.doc, col, row) {
                    let Some((lines, _)) = self.attachment_rows() else {
                        return;
                    };
                    if let Some(v) = self.attach_mut() {
                        let i = v.nav.scroll as usize + (row - r.y) as usize;
                        if i < lines.len() {
                            let width = v.width.get() as usize;
                            v.nav.line = i;
                            v.nav.scroll_block(&lines, width, dir);
                        }
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
                        self.jump_view(i);
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
        // With a dok open the row replaces it: select the row, close the
        // dok and land where it was opened from.
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
            self.open_doks_entry();
        } else if i == self.cursor {
            self.open_detail_pane();
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

    /// Wheel over the list: scroll its viewport by `delta` rows (clamped so
    /// the last row stays at the bottom of the `height`-row area), then drag
    /// the cursor along so it stays on screen.
    fn scroll_list(&mut self, delta: i32, height: u16) {
        let n = self.row_count();
        if n == 0 {
            return;
        }
        let h = (height as usize).max(1);
        let max = n.saturating_sub(h) as i32;
        let top = (self.list_offset.get() as i32 + delta).clamp(0, max) as usize;
        self.list_offset.set(top);
        self.cursor = self.cursor.clamp(top, (top + h - 1).min(n - 1));
    }

    /// Wheel over an open document: scroll its viewport by `delta` rows and
    /// drag its line cursor along, as [`App::scroll_detail`] does.
    fn scroll_doc(&mut self, delta: i32) {
        let Some((lines, vp)) = self.attachment_rows() else {
            return;
        };
        let n = lines.len();
        let Some(v) = self.attach_mut() else { return };
        if n == 0 {
            return;
        }
        let max = vp.max_scroll(n) as i32;
        v.nav.scroll = (v.nav.scroll as i32 + delta).clamp(0, max) as u16;
        let top = v.nav.scroll as usize;
        let shown = vp.rows_shown(top);
        v.nav.line = v.nav.line.clamp(top, (top + shown - 1).min(n - 1));
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

    /// yaks-000b: the wheel scrolls the list's viewport (3 rows a notch), like
    /// the detail pane; the cursor stays put until it would leave the screen,
    /// then is dragged along at the edge.
    #[test]
    fn wheel_scrolls_the_list_and_drags_the_cursor() {
        let mut app = many(30);
        app.cursor = 4;
        let frame = draw(&app, 60, 10);
        let top = |app: &App| app.hits.borrow().list_offset;
        assert_eq!(top(&app), 0);
        mouse(&mut app, MouseEventKind::ScrollDown, 5, 4);
        draw(&app, 60, 10);
        assert_eq!(top(&app), 3, "the view moved");
        assert_eq!(
            app.selected_id().as_deref(),
            Some("t04"),
            "the cursor did not"
        );
        for _ in 0..3 {
            mouse(&mut app, MouseEventKind::ScrollDown, 5, 4);
            draw(&app, 60, 10);
        }
        assert_eq!(top(&app), 12);
        assert_eq!(
            app.selected_id().as_deref(),
            Some("t12"),
            "dragged to the top row"
        );
        println!(
            "--- before ---\n{frame}--- after 4 notches ---\n{}",
            draw(&app, 60, 10)
        );
        // Far past the end: the last row stays at the bottom.
        for _ in 0..20 {
            mouse(&mut app, MouseEventKind::ScrollDown, 5, 4);
            draw(&app, 60, 10);
        }
        let h = app.hits.borrow().list.unwrap().height as usize;
        assert_eq!(top(&app), 30 - h);
        // Wheel outside any pane (the help bar) does nothing.
        let at = app.cursor;
        mouse(&mut app, MouseEventKind::ScrollUp, 5, 9);
        assert_eq!(app.cursor, at);
    }

    /// yaks-000b: a horizontal wheel over a table scrolls that block sideways,
    /// with the line cursor moved to the row under the pointer.
    #[test]
    fn horizontal_wheel_scrolls_the_block_under_the_pointer() {
        let mut t = task("w0", "Wide", Status::Hairy, 3, None);
        let wide = "x".repeat(200);
        t.body = format!("```\n{wide}\n```");
        let mut app = App::new(vec![t]);
        enter_key(&mut app);
        let frame = draw(&app, 80, 20);
        let y = row_of(&frame, "xxxxxxxx");
        let rect = app.hits.borrow().detail.unwrap();
        mouse(&mut app, MouseEventKind::ScrollRight, rect.x + 4, y);
        let key = app.detail_dlines()[app.detail.line].nowrap_key();
        assert!(key.is_some(), "the cursor is on the code row");
        assert!(matches!(app.detail.hscroll, Some((k, off)) if Some(k) == key && off > 0));
        mouse(&mut app, MouseEventKind::ScrollLeft, rect.x + 4, y);
        assert!(matches!(app.detail.hscroll, Some((_, 0))));
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
    fn a_click_in_the_list_closes_an_open_dok_and_selects_the_row() {
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
        assert!(app.doc.is_none(), "the dok closed");
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
