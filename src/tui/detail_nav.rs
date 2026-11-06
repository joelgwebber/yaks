//! `App`'s detail-pane navigation: building the detail line model, cursor/scroll
//! movement, visual selection + yank, find and link-following (a jump of the
//! one history, [`super::history`]). Split out of `tui.rs` (yaks-b1cc /
//! b1cc/6) as an `impl App` block. `use super::*` inherits `tui.rs`'s scope; methods are `pub(crate)`.

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
        // The pane's own yak (a followed link the list does not have), named
        // by its index in the farm's yaks like a row's; else the cursor's row.
        let loc = match self.pane_yak_id() {
            Some(id) => self
                .all
                .iter()
                .position(|t| t.id == id)
                .map(memo::Loc::Home),
            None => self.cached_rows().get(self.cursor).map(|r| r.loc),
        };
        let Some(loc) = loc else {
            return Rc::default();
        };
        let key = memo::DlinesKey {
            data_gen: self.data_gen,
            loc,
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
    /// [`App::show_yak_at`].) Called after every key and click, so
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

    /// The link the detail cursor targets (`LineNav::link_at`): what Enter,
    /// `O` and `r` act on and the status bar describes.
    pub(crate) fn detail_link(&self) -> Option<detail::Jump> {
        self.detail.link_at(&self.detail_dlines())
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

    /// `/` in the pane (a yak's detail or the viewer): the find box, editing
    /// the search register.
    pub(crate) fn open_find(&mut self) {
        let nav = match self.attach() {
            Some(v) => &v.nav,
            None => &self.detail,
        };
        self.overlay = Overlay::Find(nav.open_find(self.editor_vim, self.register.as_deref()));
    }

    /// (line, col, len) of every match of the register in the current yak's
    /// detail (tests ask; the renderer and keys go through [`LineNav`]).
    #[cfg(test)]
    pub(crate) fn detail_find_matches(&self) -> Vec<(usize, usize, usize)> {
        if self.selected().is_none() {
            return vec![];
        }
        linenav::find_matches(&self.detail_dlines(), self.register.as_deref())
    }

    /// n / N in a yak's detail: the next / previous match of the register.
    /// Past the last (first) one, to the next (previous) row whose yak
    /// matches it, landing on that yak's first (last) match.
    pub(crate) fn detail_find_jump(&mut self, delta: i32) {
        let Some(q) = self.register.clone() else {
            self.notification = Some("nothing to find: / sets the search".into());
            return;
        };
        let lines = self.detail_dlines();
        if self.detail.find_step(&lines, Some(&q), delta, false) {
            return;
        }
        // The next row, in the direction of travel, whose yak matches the
        // register by the list query's rule.
        let to = {
            let rows = self.rows();
            let ahead: Box<dyn Iterator<Item = usize>> = if delta >= 0 {
                Box::new(self.cursor + 1..rows.len())
            } else {
                Box::new((0..self.cursor).rev())
            };
            let mut ahead = ahead;
            ahead.find(|&i| {
                rows[i]
                    .yak()
                    .is_some_and(|t| tree::query_matches(tree::Item::Yak(t), &q))
            })
        };
        let Some(to) = to else {
            self.notification = Some(format!("no more matches for \"{q}\""));
            return;
        };
        self.show_row(to);
        let lines = self.detail_dlines();
        self.detail.find_step(&lines, Some(&q), delta, false);
    }

    /// Put the detail pane on row `to` of the list, from its top.
    fn show_row(&mut self, to: usize) {
        self.cursor = to;
        self.pane_yak = None;
        self.detail.to_top();
    }

    /// Show yak `id` in the pane (which has the focus), from its top: on its
    /// row when the cursor is there, else as the pane's own yak
    /// ([`App::pane_yak`]). A fresh detail state also drops the sideways
    /// scroll of the previous yak's blocks (the search register is the app's,
    /// and stays).
    pub(crate) fn show_yak_at(&mut self, id: &str) {
        let on_row = self.row_yak().is_some_and(|t| t.id == id);
        self.pane_yak = (!on_row).then(|| id.to_string());
        self.detail = LineNav::default();
    }

    /// Follow a link to yak `id`, a jump (`o` comes back; also the `#`
    /// go-to-yak picker, yaks-63f3). The list is left where it is: the cursor
    /// moves to the yak when the list has it (its ancestors expanded), else
    /// the pane shows it as its own yak, which it says is not in this list.
    pub(crate) fn goto_task(&mut self, id: &str) {
        self.goto_yak(id, None);
    }

    /// Follow a note link (`id#<timestamp>`, yaks-c5bd): [`App::goto_task`],
    /// then the detail cursor on that note's header row, as a `#heading` link
    /// lands on its heading in the viewer. A note the yak no longer has leaves
    /// the cursor at the top and says so.
    pub(crate) fn goto_note(&mut self, id: &str, anchor: &str) {
        self.goto_yak(id, Some(anchor));
    }

    fn goto_yak(&mut self, id: &str, note: Option<&str>) {
        let Some(task) = self.task(id) else {
            self.notification = Some(format!("{id} not found"));
            return;
        };
        // Comment `i` of the body is note `i`; block 0 is the description.
        let block = note.and_then(|a| {
            crate::links::find_note(&crate::store::parse_notes(&task.body), a).map(|i| i + 1)
        });
        let mut landed = false;
        self.jump(|app| {
            // The dok it was followed from, if any, is replaced.
            app.close_doc();
            let at = |app: &App| {
                app.rows()
                    .iter()
                    .position(|r| r.yak().is_some_and(|t| t.id == id))
            };
            let mut pos = at(app);
            if pos.is_none() {
                // A collapsed ancestor hides it: expand them so the cursor
                // can land on it.
                app.expand_ancestors(id);
                pos = at(app);
            }
            if let Some(pos) = pos {
                app.cursor = pos;
            }
            app.focus = Focus::Detail;
            app.show_yak_at(id);
            if let Some(block) = block {
                let lines = app.detail_dlines();
                let header = lines
                    .iter()
                    .position(|l| l.src.is_some_and(|s| s.block == block && s.lines.is_none()));
                if let Some(row) = header {
                    let vp = app.detail_viewport();
                    app.detail.goto(&lines, vp, row);
                    landed = true;
                }
            }
        });
        self.notification = Some(match note {
            Some(a) if landed => format!("→ {id}#{a}"),
            Some(a) => format!("→ {id} (no note {a})"),
            None => format!("→ {id}"),
        });
    }

    pub(crate) fn follow_link(&mut self) {
        let Some(j) = self.detail_link() else {
            return;
        };
        match j.target {
            detail::Target::Task(id) => self.goto_task(&id),
            detail::Target::Note { id, anchor } => self.goto_note(&id, &anchor),
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
        self.jump(|app| {
            app.detail.line = to;
            app.scroll_line_into_view(to as u16);
        });
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
            self.back_to_list();
        }
    }

    /// Esc in the detail pane peels back: selection → the search register →
    /// back to list (Python layering).
    pub(crate) fn detail_peel(&mut self) {
        if self.detail.anchor.is_some() {
            self.detail.anchor = None;
        } else if self.register.take().is_some() {
            self.detail.matched = None;
        } else {
            self.back_to_list();
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
        // On a document list (the Doks), the next row's dok replaces the
        // open one, as `J`/`K` show the next yak in a yak list (yaks-7c6e).
        if self.doc.is_some() && self.on_doks_tab() {
            return self.viewer_next_doc(delta);
        }
        // Another row's subject replaces an open dok.
        self.close_doc();
        let len = self.row_count();
        if len == 0 {
            return;
        }
        let next = (self.cursor as i32 + delta).clamp(0, len as i32 - 1) as usize;
        // Also from the pane's own yak, which is not on the cursor's row: the
        // row beside the cursor replaces it.
        if next != self.cursor || self.pane_yak.is_some() {
            self.show_row(next);
        }
    }

    /// `J`/`K` in the viewer on the Doks: move the list cursor and open that
    /// row's dok in place. Not a history jump (as yak `J`/`K` is not); the
    /// dok keeps where focus returns to when it closes.
    fn viewer_next_doc(&mut self, delta: i32) {
        let len = self.row_count();
        let next = (self.cursor as i32 + delta).clamp(0, len.saturating_sub(1) as i32) as usize;
        if next == self.cursor {
            return;
        }
        self.cursor = next;
        let Some(path) = self.selected_doc().map(|e| e.path.clone()) else {
            return;
        };
        match self.read_farm_doc(&path, None) {
            Ok(v) => {
                if let Some(d) = &mut self.doc {
                    d.view = DocView::Text(v);
                }
                self.notification = Some(format!("viewing {path}"));
            }
            Err(msg) => self.notification = Some(msg),
        }
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
