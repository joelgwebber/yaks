//! `App`'s detail-pane navigation: building the detail line model, cursor/scroll
//! movement, visual selection + yank, find, link-following, and browser-style
//! nav history. Split out of `tui.rs` (yaks-b1cc / b1cc/6) as an `impl App`
//! block. `use super::*` inherits `tui.rs`'s scope; methods are `pub(crate)`.

use super::*;

/// Columns a no-wrap block scrolls per `h`/`l`.
const HSCROLL_STEP: usize = 8;

impl App {
    /// Select an existing task wherever it lives: switch to its status view (if
    /// one is pinned) and place the cursor on it, then focus the list.
    pub(crate) fn select_task(&mut self, id: &str) {
        if let Some(st) = self.task(id).map(|t| t.status) {
            if let Some(i) = self.views.iter().position(|v| v.status == Some(st)) {
                self.set_view(i);
            }
        }
        // Expand any collapsed ancestors so the target row is actually visible;
        // otherwise the cursor can't land on it and the jump silently no-ops.
        self.expand_ancestors(id);
        if let Some(pos) = self.rows().iter().position(|r| r.task.id == id) {
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

    /// The sideways scroll of no-wrap block `key` of yak `yak` (the selected
    /// one): the scroll state if it is this block's, else 0; clamped so the
    /// block never scrolls past its widest row (`extent`) at the current pane
    /// width. The caller passes the id rather than this calling `selected()`:
    /// the renderer asks once per no-wrap row, and `selected()` rebuilds the
    /// whole view (yaks-48d6).
    pub(crate) fn block_hscroll(&self, yak: &str, key: (usize, usize), extent: usize) -> usize {
        match &self.detail_hscroll {
            Some((id, b, n, off)) if id == yak && (*b, *n) == key => {
                (*off).min(extent.saturating_sub(self.detail_width.get() as usize))
            }
            _ => 0,
        }
    }

    /// `h`/`l` on a no-wrap block under the line cursor: scroll that block
    /// `dir` steps sideways (clamped). Returns whether the offset changed, so
    /// `h` at offset 0 (or off a no-wrap row) can fall through to "back to
    /// list" (yaks-c733).
    pub(crate) fn scroll_block(&mut self, dir: i32) -> bool {
        let lines = self.detail_dlines();
        let Some(key) = lines.get(self.detail_line).and_then(|l| l.nowrap_key()) else {
            return false;
        };
        let Some(&extent) = detail::nowrap_extents(&lines).get(&key) else {
            return false;
        };
        let Some(id) = self.selected().map(|t| t.id.clone()) else {
            return false;
        };
        let max = extent.saturating_sub(self.detail_width.get() as usize);
        let cur = self.block_hscroll(&id, key, extent);
        let next = (cur as i32 + dir * HSCROLL_STEP as i32).clamp(0, max as i32) as usize;
        self.detail_hscroll = Some((id, key.0, key.1, next));
        next != cur
    }

    /// Forget the sideways scroll once the line cursor is no longer on the
    /// scrolled block (or the yak changed): a block's offset lives only while
    /// you are on it. Called after every key and click in the detail pane, so
    /// the many ways the cursor can move need no bookkeeping of their own.
    pub(crate) fn drop_stale_hscroll(&mut self) {
        let Some((id, block, n, _)) = &self.detail_hscroll else {
            return;
        };
        let same_yak = self.selected().is_some_and(|t| t.id == *id);
        let key = (*block, *n);
        let on_block = same_yak
            && self.focus == Focus::Detail
            && self
                .detail_dlines()
                .get(self.detail_line)
                .and_then(|l| l.nowrap_key())
                == Some(key);
        if !on_block {
            self.detail_hscroll = None;
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
        let lines = self.detail_dlines();
        let stops = detail::cursor_stops(&lines);
        if stops.is_empty() {
            return;
        }
        // The current stop: the last one at or before the cursor. A cursor
        // inside an image block (only a click or a reflow puts it there) is on
        // that image's stop, its link row; a step up from inside it arrives
        // there, so it costs one step less.
        let at = stops
            .partition_point(|&r| r <= self.detail_line)
            .saturating_sub(1);
        let inside = stops[at] != self.detail_line;
        let delta = if inside && delta < 0 {
            delta + 1
        } else {
            delta
        };
        let to = (at as i32 + delta).clamp(0, stops.len() as i32 - 1) as usize;
        self.detail_line = stops[to];
        self.scroll_line_into_view(self.detail_line as u16);
    }

    /// g / G — jump the line cursor to the top / bottom stop.
    pub(crate) fn detail_line_to(&mut self, end: bool) {
        let lines = self.detail_dlines();
        let stops = detail::cursor_stops(&lines);
        self.detail_line = if end {
            stops.last().copied().unwrap_or(0)
        } else {
            0
        };
        self.scroll_line_into_view(self.detail_line as u16);
    }

    /// Tab / Shift-Tab — snap the line cursor to the next / prev line holding a
    /// link (wrapping), so Enter can follow it.
    pub(crate) fn jump_link(&mut self, delta: i32) {
        let jumps = self.detail_jumps();
        if jumps.is_empty() {
            return;
        }
        let mut lines: Vec<usize> = jumps.iter().map(|j| j.line).collect();
        lines.dedup();
        let target = if delta >= 0 {
            lines
                .iter()
                .find(|&&l| l > self.detail_line)
                .copied()
                .unwrap_or(lines[0])
        } else {
            lines
                .iter()
                .rev()
                .find(|&&l| l < self.detail_line)
                .copied()
                .unwrap_or_else(|| *lines.last().unwrap())
        };
        self.detail_line = target;
        self.scroll_line_into_view(self.detail_line as u16);
    }

    /// v — toggle visual selection anchored at the current line.
    pub(crate) fn toggle_visual(&mut self) {
        self.detail_anchor = if self.detail_anchor.is_some() {
            None
        } else {
            Some(self.detail_line)
        };
    }

    /// Shift-↑↓ — start (if needed) and extend the visual selection.
    pub(crate) fn extend_selection(&mut self, delta: i32) {
        if self.detail_anchor.is_none() {
            self.detail_anchor = Some(self.detail_line);
        }
        self.move_detail_line(delta);
    }

    /// The inclusive [lo, hi] line range currently selected, if any.
    pub(crate) fn selection_range(&self) -> Option<(usize, usize)> {
        self.detail_anchor
            .map(|a| (a.min(self.detail_line), a.max(self.detail_line)))
    }

    /// y / Enter (in visual mode) — copy the selected line block (dedented) to
    /// the clipboard and clear the selection.
    /// The text a yank of detail rows `lo..=hi` copies: source markdown for
    /// rendered body rows, the displayed text (soft wraps rejoined) for the
    /// rest, dedented.
    pub(crate) fn selection_text(&self, lo: usize, hi: usize) -> Option<String> {
        let lines = self.detail_dlines();
        if lines.is_empty() {
            return None;
        }
        let hi = hi.min(lines.len() - 1);
        let task = self.selected()?;
        let blocks = super::content::parse(&task.body);
        // Rejoin soft-wrapped continuations into their logical line so a
        // yanked paragraph doesn't carry hard breaks at wrap points.
        // A rendered body row yanks its *source* markdown instead
        // (yaks-649d): each source line once, in order.
        let mut rejoined: Vec<String> = Vec::new();
        let mut last_src: Option<(usize, usize)> = None;
        for line in &lines[lo..=hi] {
            if let Some(src) = line.src {
                let Some((a, b)) = src.lines else {
                    continue; // a comment header row
                };
                let Some(block) = blocks.get(src.block) else {
                    continue;
                };
                let text: Vec<&str> = block.text.split('\n').collect();
                // Within one block, include source lines between the last
                // copied one and this row (the blank lines a paragraph gap
                // renders as one row, a fence line, ...).
                let from = match last_src {
                    Some((lb, ln)) if lb == src.block && ln < a => ln + 1,
                    _ => a,
                };
                for n in from..=b {
                    if last_src.is_some_and(|(lb, ln)| lb == src.block && ln >= n) {
                        continue;
                    }
                    if let Some(t) = text.get(n) {
                        rejoined.push((*t).to_string());
                    }
                    last_src = Some((src.block, n));
                }
                continue;
            }
            if line.cont {
                if let Some(last) = rejoined.last_mut() {
                    last.push(' ');
                    last.push_str(&line.text);
                    continue;
                }
            }
            rejoined.push(line.text.clone());
        }
        Some(dedent(&rejoined).join("\n"))
    }

    pub(crate) fn yank_selection(&mut self) {
        let Some((lo, hi)) = self.selection_range() else {
            return;
        };
        let Some(text) = self.selection_text(lo, hi) else {
            return;
        };
        let ok = crate::clipboard::copy_text(&text);
        self.detail_anchor = None;
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
        let page = self.detail_page.max(1) as usize;
        if scroll == 0 {
            page
        } else {
            page.saturating_sub(PINNED_ROWS as usize).max(1)
        }
    }

    /// Furthest the detail can scroll: far enough that a blank row sits below
    /// the last line (so it never abuts the viewport edge). Content that fits
    /// the pane doesn't scroll at all.
    pub(crate) fn detail_max_scroll(&self) -> u16 {
        let n = self.detail_line_count();
        if n <= self.detail_page.max(1) as usize {
            return 0;
        }
        (n + 1).saturating_sub(self.detail_rows_shown(1)) as u16
    }

    /// Move `detail_scroll` the minimum needed so `line` sits inside the detail
    /// viewport; leave it untouched when the line is already visible. The last
    /// line also keeps a blank row beneath it, and the viewport shrinks by one
    /// row once scrolled (sticky header), which this accounts for.
    pub(crate) fn scroll_line_into_view(&mut self, line: u16) {
        let n = self.detail_line_count();
        if n <= self.detail_page.max(1) as usize {
            self.detail_scroll = 0;
            return;
        }
        let line = line as usize;
        let top = self.detail_scroll as usize;
        // The last line wants the blank row under it on screen too.
        let bottom = if line + 1 >= n { n } else { line };
        if line < top {
            self.detail_scroll = line as u16;
        } else if bottom >= top + self.detail_rows_shown(self.detail_scroll) {
            let vp = self.detail_rows_shown(1);
            self.detail_scroll = (bottom + 1 - vp).min(self.detail_max_scroll() as usize) as u16;
        }
    }

    pub(crate) fn open_detail_find(&mut self) {
        let cur = self.detail_find.clone();
        let mut sb = SearchBox::new(self.editor_vim, cur);
        // Stash where the cursor/scroll sit now so Esc can restore them (vi-like).
        sb.detail_origin = Some((self.detail_scroll, self.detail_line));
        self.overlay = Overlay::DetailFind(sb);
    }

    /// (line, col, len) of every detail-find match for the current selection.
    pub(crate) fn detail_find_matches(&self) -> Vec<(usize, usize, usize)> {
        let Some(q) = self.detail_find.as_deref().filter(|s| !s.is_empty()) else {
            return vec![];
        };
        if self.selected().is_none() {
            return vec![];
        }
        detail_scan(&self.detail_dlines(), q)
    }

    pub(crate) fn detail_find_jump(&mut self, delta: i32) {
        let m = self.detail_find_matches();
        if m.is_empty() {
            return;
        }
        let n = m.len() as i32;
        self.detail_match = (self.detail_match as i32 + delta).rem_euclid(n) as usize;
        self.detail_scroll = m[self.detail_match].0 as u16;
    }

    /// Select `id` and show it in the detail pane, resetting the per-task detail
    /// view state (scroll/link/find/match). Shared by link-follow and i/o nav.
    pub(crate) fn open_task_in_detail(&mut self, id: &str) {
        self.select_task(id); // drops focus to the list; we re-raise it below
        self.focus = Focus::Detail;
        self.detail_scroll = 0;
        self.detail_line = 0;
        self.detail_anchor = None;
        self.detail_find = None;
        self.detail_match = 0;
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
        let Some(j) = jumps.into_iter().find(|j| j.line == self.detail_line) else {
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
        }
    }

    /// Move the line cursor to footnote `n`'s definition in the block the
    /// cursor is in, recording the spot so `o` returns to the reference.
    fn goto_footnote(&mut self, n: &str) {
        let lines = self.detail_dlines();
        let Some(to) = detail::footnote_line(&lines, self.detail_line, n) else {
            self.notification = Some(format!("no definition for [{n}]"));
            return;
        };
        if let Some(cur) = self.current_nav_entry() {
            self.nav_back.push(cur);
            self.nav_fwd.clear();
        }
        self.detail_line = to;
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

    /// J/K — move to the next/prev task in the list while staying in the detail
    /// pane, resetting the per-task detail view state.
    pub(crate) fn detail_next_task(&mut self, delta: i32) {
        let len = self.row_count();
        if len == 0 {
            return;
        }
        let next = (self.cursor as i32 + delta).clamp(0, len as i32 - 1) as usize;
        if next != self.cursor {
            self.cursor = next;
            self.detail_scroll = 0;
            self.detail_line = 0;
            self.detail_anchor = None;
            self.detail_find = None;
            self.detail_match = 0;
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
            line: self.detail_line,
            scroll: self.detail_scroll,
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
        self.detail_line = e.line.min(last);
        self.detail_scroll = e.scroll.min(self.detail_max_scroll());
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
        if let Some(pos) = self.rows().iter().position(|r| r.task.id == id) {
            self.cursor = pos;
        }
    }

    // -- overlay resolution ----------------------------------------------
}
