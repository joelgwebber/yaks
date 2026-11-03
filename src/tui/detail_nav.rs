//! `App`'s detail-pane navigation: building the detail line model, cursor/scroll
//! movement, visual selection + yank, find, link-following, and browser-style
//! nav history. Split out of `tui.rs` (yaks-b1cc / b1cc/6) as an `impl App`
//! block. `use super::*` inherits `tui.rs`'s scope; methods are `pub(crate)`.

use super::linenav::{self, LineNav, Viewport};
use super::*;

impl App {
    /// The detail pane's scroll geometry: its page, less the pinned header
    /// once scrolled.
    pub(crate) fn detail_viewport(&self) -> Viewport {
        Viewport {
            page: self.detail_page as usize,
            pinned: PINNED_ROWS as usize,
        }
    }

    /// Select an existing task wherever it lives: switch to its status view (if
    /// one is pinned) and place the cursor on it, then focus the list.
    pub(crate) fn select_task(&mut self, id: &str) {
        self.close_doc();
        if let Some(st) = self.task(id).map(|t| t.status) {
            if let Some(i) = self.views.iter().position(|v| v.status == Some(st)) {
                self.set_view(i);
            }
        }
        // Expand any collapsed ancestors so the target row is actually visible;
        // otherwise the cursor can't land on it and the jump silently no-ops.
        self.expand_ancestors(id);
        if let Some(pos) = self
            .rows()
            .iter()
            .position(|r| r.yak().is_some_and(|t| t.id == id))
        {
            self.cursor = pos;
        }
        self.focus = Focus::List;
    }

    /// Remove every ancestor of `id` from the collapsed set (walking the parent
    /// chain), persisting the change so a jumped-to task is never hidden.
    pub(crate) fn expand_ancestors(&mut self, id: &str) {
        let mut changed = false;
        let mut cur = self.task(id).and_then(|t| t.parent.clone());
        while let Some(pid) = cur {
            if self.collapsed.remove(&pid) {
                changed = true;
            }
            cur = self.task(&pid).and_then(|t| t.parent.clone());
        }
        if changed {
            self.save_ui_state();
        }
    }

    /// The detail lines for the current selection, soft-wrapped to the width
    /// captured at the last render. This is the single source of truth for the
    /// row-indexed detail model (cursor, jumplist, find, scroll, render).
    ///
    /// Memoized (`memo.rs`, yaks-bc30): built once per change of the selected
    /// yak, farm data, width, in-place editor or clock minute, and shared by
    /// every caller of a keypress or frame.
    pub(crate) fn detail_dlines(&self) -> Rc<Vec<detail::DLine>> {
        let Some(row) = self.cached_rows().get(self.cursor).copied() else {
            return Rc::default();
        };
        let key = memo::DlinesKey {
            data_gen: self.data_gen,
            loc: row.loc,
            width: self.detail_width.get() as usize,
            slot: self.edit_slot(),
            images: self.image_env().map(|e| (e.cell_px, e.max_rows)),
            minute: self.clock.minute(),
        };
        if let Some(c) = &*self.dlines_cache.borrow() {
            if c.key == key {
                #[cfg(test)]
                assert_eq!(*c.lines, self.build_dlines(), "stale detail-lines cache");
                return c.lines.clone();
            }
        }
        #[cfg(test)]
        memo::DLINES_CALLS.with(|c| c.set(c.get() + 1));
        let lines = Rc::new(self.build_dlines());
        *self.dlines_cache.borrow_mut() = Some(memo::DlinesCache {
            key,
            lines: lines.clone(),
        });
        lines
    }

    /// The uncached [`App::detail_dlines`]: the memo's one source of truth.
    pub(super) fn build_dlines(&self) -> Vec<detail::DLine> {
        let Some(t) = self.selected() else {
            return Vec::new();
        };
        let width = self.detail_width.get() as usize;
        let slot = self.edit_slot();
        let env = self.image_env();
        let lines = detail::build_at(
            t,
            &self.all,
            &self.clock,
            width,
            slot.as_ref(),
            env.as_ref(),
        );
        detail::wrap(lines, width)
    }

    /// Where and how big inline images are laid out, or `None` when they are
    /// off (`inline_images: false`) or there is no farm to resolve attachments
    /// in. No image is taller than two thirds of the detail pane unless the
    /// author asked for a size.
    pub(crate) fn image_env(&self) -> Option<detail::ImageEnv<'_>> {
        if !self.inline_images {
            return None;
        }
        let font = self.picker.font_size();
        Some(detail::ImageEnv {
            root: self.farm.as_ref()?.root(),
            cell_px: (font.width, font.height),
            max_rows: (self.detail_page * 2 / 3).max(4),
        })
    }

    /// `h`/`l` on a no-wrap block under the line cursor: scroll that block
    /// `dir` steps sideways (clamped). Returns whether the offset changed, so
    /// `h` at offset 0 (or off a no-wrap row) can fall through to "back to
    /// list" (yaks-c733).
    pub(crate) fn scroll_block(&mut self, dir: i32) -> bool {
        let lines = self.detail_dlines();
        let width = self.detail_width.get() as usize;
        self.detail.scroll_block(&lines, width, dir)
    }

    /// Forget the sideways scroll once the line cursor is no longer on the
    /// scrolled block, or focus left the pane: a block's offset lives only
    /// while you are on it. (A different yak resets the whole detail state,
    /// [`App::open_task_in_detail`].) Called after every key and click, so
    /// the many ways the cursor can move need no bookkeeping of their own.
    pub(crate) fn drop_stale_hscroll(&mut self) {
        if self.detail.hscroll.is_none() {
            return;
        }
        if self.focus == Focus::Detail {
            let lines = self.detail_dlines();
            self.detail.drop_stale_hscroll(&lines);
        } else {
            self.detail.hscroll = None;
        }
    }

    /// Detail jumplist for the current selection (empty when nothing selected).
    pub(crate) fn detail_jumps(&self) -> Vec<detail::Jump> {
        detail::jumplist(&self.detail_dlines())
    }

    pub(crate) fn detail_line_count(&self) -> usize {
        self.detail_dlines().len()
    }

    /// Move the detail line cursor by `delta` stops, clamped, keeping it in
    /// view. The rows an inline image reserves are not stops (see
    /// [`detail::cursor_stops`]): one step crosses a whole image, as in edit
    /// mode.
    pub(crate) fn move_detail_line(&mut self, delta: i32) {
        let (lines, vp) = (self.detail_dlines(), self.detail_viewport());
        self.detail.step(&lines, vp, delta);
    }

    /// g / G — jump the line cursor to the top / bottom stop.
    pub(crate) fn detail_line_to(&mut self, end: bool) {
        let (lines, vp) = (self.detail_dlines(), self.detail_viewport());
        self.detail.to_end(&lines, vp, end);
    }

    /// Tab / Shift-Tab — snap the line cursor to the next / prev line holding a
    /// link (wrapping), so Enter can follow it.
    pub(crate) fn jump_link(&mut self, delta: i32) {
        let (lines, vp) = (self.detail_dlines(), self.detail_viewport());
        self.detail.jump_link(&lines, vp, delta);
    }

    /// v — toggle visual selection anchored at the current line.
    pub(crate) fn toggle_visual(&mut self) {
        self.detail.toggle_visual();
    }

    /// Shift-↑↓ — start (if needed) and extend the visual selection.
    pub(crate) fn extend_selection(&mut self, delta: i32) {
        let (lines, vp) = (self.detail_dlines(), self.detail_viewport());
        self.detail.extend_selection(&lines, vp, delta);
    }

    /// The inclusive [lo, hi] line range currently selected, if any.
    pub(crate) fn selection_range(&self) -> Option<(usize, usize)> {
        self.detail.selection()
    }

    /// The text a yank of detail rows `lo..=hi` copies: source markdown for
    /// rendered body rows, the displayed text (soft wraps rejoined) for the
    /// rest, dedented ([`linenav::yank_text`]).
    pub(crate) fn selection_text(&self, lo: usize, hi: usize) -> Option<String> {
        let task = self.selected()?;
        let blocks = super::content::parse(&task.body);
        let texts: Vec<&str> = blocks.iter().map(|b| b.text.as_str()).collect();
        linenav::yank_text(&self.detail_dlines(), lo, hi, &texts)
    }

    pub(crate) fn yank_selection(&mut self) {
        let Some((lo, hi)) = self.selection_range() else {
            return;
        };
        let Some(text) = self.selection_text(lo, hi) else {
            return;
        };
        let ok = crate::clipboard::copy_text(&text);
        self.detail.anchor = None;
        let hi = hi.min(self.detail_line_count().saturating_sub(1));
        let n = hi - lo + 1;
        self.notification = Some(if ok {
            format!("copied {n} line(s)")
        } else {
            "clipboard unavailable".into()
        });
    }

    /// Detail content rows visible at `scroll`: the whole pane at the top, one
    /// fewer once scrolled (the sticky header and its rule take the top rows).
    pub(crate) fn detail_rows_shown(&self, scroll: u16) -> usize {
        self.detail_viewport().rows_shown(scroll as usize)
    }

    /// Furthest the detail can scroll: far enough that a blank row sits below
    /// the last line (so it never abuts the viewport edge). Content that fits
    /// the pane doesn't scroll at all.
    pub(crate) fn detail_max_scroll(&self) -> u16 {
        self.detail_viewport().max_scroll(self.detail_line_count()) as u16
    }

    /// Move `detail.scroll` the minimum needed so `line` sits inside the detail
    /// viewport; leave it untouched when the line is already visible. The last
    /// line also keeps a blank row beneath it, and the viewport shrinks by one
    /// row once scrolled (sticky header), which this accounts for.
    pub(crate) fn scroll_line_into_view(&mut self, line: u16) {
        let (n, scroll) = (self.detail_line_count(), self.detail.scroll as usize);
        self.detail.scroll = self
            .detail_viewport()
            .scroll_into_view(n, scroll, line as usize) as u16;
    }

    pub(crate) fn open_detail_find(&mut self) {
        self.overlay = Overlay::DetailFind(self.detail.open_find(self.editor_vim));
    }

    /// (line, col, len) of every detail-find match for the current selection
    /// (tests ask; the renderer and keys go through [`LineNav`]).
    #[cfg(test)]
    pub(crate) fn detail_find_matches(&self) -> Vec<(usize, usize, usize)> {
        if self.detail.find.is_none() || self.selected().is_none() {
            return vec![];
        }
        self.detail.find_matches(&self.detail_dlines())
    }

    pub(crate) fn detail_find_jump(&mut self, delta: i32) {
        let lines = self.detail_dlines();
        self.detail.find_jump(&lines, delta);
    }

    /// Select `id` and show it in the detail pane, resetting the per-task detail
    /// view state (scroll/link/find/match/sideways scroll). Shared by link-follow and i/o nav.
    pub(crate) fn open_task_in_detail(&mut self, id: &str) {
        self.select_task(id); // drops focus to the list; we re-raise it below
        self.focus = Focus::Detail;
        // A fresh state, also dropping the sideways scroll of the previous
        // yak's blocks.
        self.detail = LineNav::default();
    }

    /// Jump to `id`'s detail, recording the jump for o/i history
    /// (browser-style: a new jump clears the forward stack). Shared by
    /// link-follow and the `#` go-to-yak picker (yaks-63f3).
    pub(crate) fn goto_task(&mut self, id: &str) {
        // Snapshot where we are (id + detail cursor/scroll) before moving, so
        // o/i can restore it.
        let from = self.current_nav_entry();
        self.open_task_in_detail(id);
        if self.selected_id().as_deref() != Some(id) {
            // No view shows the target (e.g. no tab for its status), so the
            // cursor couldn't land on it; say so rather than silently no-op.
            self.notification = Some(format!("{id} isn't shown in any view"));
            return;
        }
        if let Some(cur) = from {
            if cur.id != id {
                self.nav_back.push(cur);
                self.nav_fwd.clear();
            }
        }
        self.notification = Some(format!("→ {id}"));
    }

    pub(crate) fn follow_link(&mut self) {
        let jumps = self.detail_jumps();
        let Some(j) = jumps.into_iter().find(|j| j.line == self.detail.line) else {
            return;
        };
        match j.target {
            detail::Target::Task(id) => self.goto_task(&id),
            detail::Target::Url(u) => self.open_external(&u),
            detail::Target::Artifact(p) => self.open_external(&p),
            detail::Target::Source { path, lines, rev } => {
                self.open_source(&path, lines, rev.as_deref())
            }
            detail::Target::Footnote(n) => self.goto_footnote(&n),
            // A farm document opens in the viewer (yaks-7395).
            detail::Target::Farm { path, rev, anchor } => {
                self.open_farm_doc(&path, rev.as_deref(), anchor.as_ref())
            }
            detail::Target::Local(_) => {
                self.notification = Some("in-page anchors are followed in the viewer".into())
            }
        }
    }

    /// Move the line cursor to footnote `n`'s definition in the block the
    /// cursor is in, recording the spot so `o` returns to the reference.
    fn goto_footnote(&mut self, n: &str) {
        let lines = self.detail_dlines();
        let Some(to) = detail::footnote_line(&lines, self.detail.line, n) else {
            self.notification = Some(format!("no definition for [{n}]"));
            return;
        };
        if let Some(cur) = self.current_nav_entry() {
            self.nav_back.push(cur);
            self.nav_fwd.clear();
        }
        self.detail.line = to;
        self.scroll_line_into_view(to as u16);
        self.notification = Some(format!("→ [{n}] (o to go back)"));
    }

    /// y — copy the selected task's id to the system clipboard (best-effort).
    pub(crate) fn copy_selected_id(&mut self) {
        let Some(id) = self.selected_id() else { return };
        self.notification = Some(if crate::clipboard::copy_text(&id) {
            format!("copied {id}")
        } else {
            "clipboard unavailable".into()
        });
    }

    /// h / ← in the detail pane. On a no-wrap row (code, table) h/← scroll that
    /// block sideways (yaks-c733); at offset 0 (or anywhere else) it is "back
    /// to list".
    pub(crate) fn detail_back(&mut self) {
        if !self.scroll_block(-1) {
            self.focus = Focus::List;
        }
    }

    /// Esc in the detail pane peels back: selection → find → back to list
    /// (Python layering).
    pub(crate) fn detail_peel(&mut self) {
        if self.detail.anchor.is_some() {
            self.detail.anchor = None;
        } else if self.detail.find.is_some() {
            self.detail.find = None;
        } else {
            self.focus = Focus::List;
        }
    }

    /// Enter in the detail pane yanks an active selection, else follows the
    /// link on the line.
    pub(crate) fn detail_enter(&mut self) {
        if self.detail.anchor.is_some() {
            self.yank_selection();
        } else {
            self.follow_link();
        }
    }

    /// y in the detail pane yanks an active selection, else copies the task id.
    pub(crate) fn yank_selection_or_copy_id(&mut self) {
        if self.detail.anchor.is_some() {
            self.yank_selection();
        } else {
            self.copy_selected_id();
        }
    }

    /// J/K — move to the next/prev task in the list while staying in the detail
    /// pane, resetting the per-task detail view state.
    pub(crate) fn detail_next_task(&mut self, delta: i32) {
        // Another row's subject replaces an open page.
        self.close_doc();
        let len = self.row_count();
        if len == 0 {
            return;
        }
        let next = (self.cursor as i32 + delta).clamp(0, len as i32 - 1) as usize;
        if next != self.cursor {
            self.cursor = next;
            self.detail.scroll = 0;
            self.detail.line = 0;
            self.detail.anchor = None;
            self.detail.find = None;
            self.detail.matched = 0;
        }
    }

    /// o — jump back to the previously visited task (browser back).
    pub(crate) fn nav_back(&mut self) {
        let Some(prev) = self.nav_back.pop() else {
            self.notification = Some("no earlier yak".into());
            return;
        };
        if let Some(cur) = self.current_nav_entry() {
            self.nav_fwd.push(cur);
        }
        self.restore_nav_entry(&prev);
        self.notification = Some(format!("← {}", prev.id));
    }

    /// i — jump forward again after going back (browser forward).
    pub(crate) fn nav_forward(&mut self) {
        let Some(next) = self.nav_fwd.pop() else {
            self.notification = Some("no later yak".into());
            return;
        };
        if let Some(cur) = self.current_nav_entry() {
            self.nav_back.push(cur);
        }
        self.restore_nav_entry(&next);
        self.notification = Some(format!("→ {}", next.id));
    }

    /// Snapshot the selected task + its detail cursor/scroll as a nav stop.
    fn current_nav_entry(&self) -> Option<NavEntry> {
        Some(NavEntry {
            id: self.selected_id()?,
            line: self.detail.line,
            scroll: self.detail.scroll,
        })
    }

    /// Reopen a nav stop and put its detail cursor/scroll back where it was
    /// left, clamped in case the task's content shrank meanwhile.
    fn restore_nav_entry(&mut self, e: &NavEntry) {
        self.open_task_in_detail(&e.id);
        if self.selected_id().as_deref() != Some(e.id.as_str()) {
            return; // target vanished/filtered out: stay at the top
        }
        let last = self.detail_dlines().len().saturating_sub(1);
        self.detail.line = e.line.min(last);
        self.detail.scroll = e.scroll.min(self.detail_max_scroll());
    }

    /// Jump to the Hairy view (where new tasks land) and select `id`.
    pub(crate) fn select_id(&mut self, id: &str) {
        if let Some(i) = self
            .views
            .iter()
            .position(|v| v.status == Some(Status::Hairy))
        {
            self.set_view(i);
        }
        if let Some(pos) = self
            .rows()
            .iter()
            .position(|r| r.yak().is_some_and(|t| t.id == id))
        {
            self.cursor = pos;
        }
    }

    // -- overlay resolution ----------------------------------------------
}
