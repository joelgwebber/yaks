//! Line navigation over rendered rows (yaks-84e0): the line cursor, visual
//! selection, find, link cycling and sideways scrolling of no-wrap blocks,
//! shared by every pane that shows a page of [`detail::DLine`] rows.
//!
//! - [`LineNav`] is one page's navigation state (cursor, scroll, selection,
//!   find query and match, sideways scroll, in-page jump history) and its
//!   operations, each given the page's rows and [`Viewport`].
//! - [`Viewport`] is the scroll geometry: the rows a page shows, and how many
//!   of them a pinned header takes once scrolled.
//! - The free functions ([`yank_text`], [`link_dest`], [`hscroll_step`]) are
//!   the parts a host may use without the state.
//!
//! The text viewer ([`super::handlers::AttachView`]) owns a `LineNav`. The
//! detail pane keeps its state in `App`'s `detail_*` fields (tests and many
//! callers name them) and runs the same operations through
//! `App::with_detail_nav`, so a fix here applies to both. A new page host
//! (the wiki tab) owns a `LineNav` like the viewer, renders with
//! `render::render_nav_rows` and routes keys like `App::handle_attachment_key`.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::detail::{self, DLine, Jump, Target};
use super::fuzzy::SearchBox;
use super::render::{dedent, detail_scan};

/// Columns a no-wrap block scrolls per `h`/`l`.
pub(crate) const HSCROLL_STEP: usize = 8;

/// A page's scroll geometry.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Viewport {
    /// Rows the page area has.
    pub(crate) page: usize,
    /// Rows a pinned header takes once the page is scrolled (0 for none).
    pub(crate) pinned: usize,
}

impl Viewport {
    /// Content rows visible at `scroll`: the whole page at the top, less the
    /// pinned header once scrolled.
    pub(crate) fn rows_shown(&self, scroll: usize) -> usize {
        let page = self.page.max(1);
        if scroll == 0 {
            page
        } else {
            page.saturating_sub(self.pinned).max(1)
        }
    }

    /// Furthest `n` rows can scroll: far enough that a blank row sits below
    /// the last one. Content that fits the page doesn't scroll at all.
    pub(crate) fn max_scroll(&self, n: usize) -> usize {
        if n <= self.page.max(1) {
            return 0;
        }
        (n + 1).saturating_sub(self.rows_shown(1))
    }

    /// The scroll that puts `line` (of `n` rows) on screen, moving `scroll`
    /// as little as possible: unchanged when the line is already visible.
    /// The last line also keeps a blank row beneath it.
    pub(crate) fn scroll_into_view(&self, n: usize, scroll: usize, line: usize) -> usize {
        if n <= self.page.max(1) {
            return 0;
        }
        let bottom = if line + 1 >= n { n } else { line };
        if line < scroll {
            line
        } else if bottom >= scroll + self.rows_shown(scroll) {
            let vp = self.rows_shown(1);
            (bottom + 1 - vp).min(self.max_scroll(n))
        } else {
            scroll
        }
    }
}

/// One page's navigation state. See the module docs.
#[derive(Default, Debug)]
pub(crate) struct LineNav {
    /// The line cursor (a row index).
    pub(crate) line: usize,
    /// First visible row.
    pub(crate) scroll: u16,
    /// Visual-selection anchor, while a selection is active.
    pub(crate) anchor: Option<usize>,
    /// The find query, and which of its matches is current (`n`/`N`).
    pub(crate) find: Option<String>,
    pub(crate) matched: usize,
    /// Sideways scroll of the no-wrap block under the cursor:
    /// `(DLine::nowrap_key, columns)`. Dropped once the cursor leaves it.
    pub(crate) hscroll: Option<((usize, usize), usize)>,
    /// In-page jumps (a footnote): `(line, scroll)` to go back / forward to.
    pub(crate) back: Vec<(usize, u16)>,
    pub(crate) fwd: Vec<(usize, u16)>,
}

impl LineNav {
    fn reveal(&mut self, n: usize, vp: Viewport) {
        self.scroll = vp.scroll_into_view(n, self.scroll as usize, self.line) as u16;
    }

    /// Move the cursor `delta` stops, clamped, keeping it in view. The rows an
    /// inline image reserves are not stops ([`detail::cursor_stops`]): one
    /// step crosses a whole image.
    pub(crate) fn step(&mut self, lines: &[DLine], vp: Viewport, delta: i32) {
        let stops = detail::cursor_stops(lines);
        if stops.is_empty() {
            return;
        }
        // The current stop: the last one at or before the cursor. A cursor
        // inside an image block (only a click or a reflow puts it there) is on
        // that image's stop, its link row; a step up from inside it arrives
        // there, so it costs one step less.
        let at = stops.partition_point(|&r| r <= self.line).saturating_sub(1);
        let inside = stops[at] != self.line;
        let delta = if inside && delta < 0 {
            delta + 1
        } else {
            delta
        };
        let to = (at as i32 + delta).clamp(0, stops.len() as i32 - 1) as usize;
        self.line = stops[to];
        self.reveal(lines.len(), vp);
    }

    /// g / G: the cursor to the top / bottom stop.
    pub(crate) fn to_end(&mut self, lines: &[DLine], vp: Viewport, end: bool) {
        self.line = if end {
            detail::cursor_stops(lines).last().copied().unwrap_or(0)
        } else {
            0
        };
        self.reveal(lines.len(), vp);
    }

    /// Tab / Shift-Tab: the cursor to the next / previous row holding a link
    /// (wrapping), so Enter can follow it.
    pub(crate) fn jump_link(&mut self, lines: &[DLine], vp: Viewport, delta: i32) {
        let mut rows: Vec<usize> = detail::jumplist(lines).iter().map(|j| j.line).collect();
        rows.dedup();
        let (Some(&first), Some(&last)) = (rows.first(), rows.last()) else {
            return;
        };
        self.line = if delta >= 0 {
            rows.iter()
                .copied()
                .find(|&l| l > self.line)
                .unwrap_or(first)
        } else {
            rows.iter()
                .rev()
                .copied()
                .find(|&l| l < self.line)
                .unwrap_or(last)
        };
        self.reveal(lines.len(), vp);
    }

    /// The first link on the cursor's row: what Enter follows.
    pub(crate) fn link_at(&self, lines: &[DLine]) -> Option<Jump> {
        detail::jumplist(lines)
            .into_iter()
            .find(|j| j.line == self.line)
    }

    /// v: toggle a visual selection anchored at the cursor.
    pub(crate) fn toggle_visual(&mut self) {
        self.anchor = match self.anchor {
            Some(_) => None,
            None => Some(self.line),
        };
    }

    /// Shift-↑↓: start (if needed) and extend the visual selection.
    pub(crate) fn extend_selection(&mut self, lines: &[DLine], vp: Viewport, delta: i32) {
        self.anchor.get_or_insert(self.line);
        self.step(lines, vp, delta);
    }

    /// The inclusive `(lo, hi)` row range selected, if any.
    pub(crate) fn selection(&self) -> Option<(usize, usize)> {
        self.anchor.map(|a| (a.min(self.line), a.max(self.line)))
    }

    /// `(line, col, len)` of every match of the find query.
    pub(crate) fn find_matches(&self, lines: &[DLine]) -> Vec<(usize, usize, usize)> {
        match self.find.as_deref() {
            Some(q) if !q.is_empty() => detail_scan(lines, q),
            _ => vec![],
        }
    }

    /// n / N: scroll to the next / previous match (wrapping).
    pub(crate) fn find_jump(&mut self, lines: &[DLine], delta: i32) {
        let m = self.find_matches(lines);
        if m.is_empty() {
            return;
        }
        self.matched = (self.matched as i32 + delta).rem_euclid(m.len() as i32) as usize;
        self.scroll = m[self.matched].0 as u16;
    }

    /// /: a find box seeded with the current query, remembering where the
    /// cursor and scroll sit so cancelling returns there (vi-like).
    pub(crate) fn open_find(&self, vim: bool) -> SearchBox {
        let mut sb = SearchBox::new(vim, self.find.clone());
        sb.detail_origin = Some((self.scroll, self.line));
        sb
    }

    /// One key typed into the open find box `sb`: matches highlight live,
    /// Enter keeps the find and leaves the cursor on the current match,
    /// `cancel` (Esc / Ctrl-C) restores the query and position from before.
    /// Returns whether the box closed.
    pub(crate) fn find_key(
        &mut self,
        sb: &mut SearchBox,
        k: KeyEvent,
        cancel: bool,
        lines: &[DLine],
    ) -> bool {
        if cancel {
            self.find = sb.saved.clone();
            if let Some((scroll, line)) = sb.detail_origin {
                self.scroll = scroll;
                self.line = line;
            }
            return true;
        }
        if k.code == KeyCode::Enter {
            if let Some(&(line, ..)) = self.find_matches(lines).get(self.matched) {
                self.line = line;
            }
            return true;
        }
        sb.handler.on_key_event(k, &mut sb.query.borrow_mut());
        let q = sb.query_text();
        self.find = if q.is_empty() { None } else { Some(q) };
        self.matched = 0;
        if let Some(first) = self.find_matches(lines).first() {
            self.scroll = first.0 as u16;
        }
        false
    }

    /// Jump the cursor to row `to` within the page, remembering where it was
    /// so [`LineNav::go_back`] returns (a footnote reference to its text).
    pub(crate) fn jump_to(&mut self, lines: &[DLine], vp: Viewport, to: usize) {
        self.back.push((self.line, self.scroll));
        self.fwd.clear();
        self.line = to;
        self.reveal(lines.len(), vp);
    }

    /// o / i: back to (forward again from) an in-page jump. Returns whether
    /// there was one.
    pub(crate) fn go_back(&mut self, lines: &[DLine], forward: bool) -> bool {
        let (from, to) = if forward {
            (&mut self.fwd, &mut self.back)
        } else {
            (&mut self.back, &mut self.fwd)
        };
        let Some((line, scroll)) = from.pop() else {
            return false;
        };
        to.push((self.line, self.scroll));
        let last = lines.len().saturating_sub(1);
        self.line = line.min(last);
        self.scroll = scroll;
        true
    }

    /// The sideways scroll of no-wrap block `key`, whose widest row is
    /// `extent` columns, in a `width`-column pane.
    pub(crate) fn block_hscroll(&self, key: (usize, usize), extent: usize, width: usize) -> usize {
        match self.hscroll {
            Some((k, off)) if k == key => off.min(extent.saturating_sub(width)),
            _ => 0,
        }
    }

    /// h / l on a no-wrap row: scroll its block `dir` steps sideways. Returns
    /// whether the offset changed, so `h` at offset 0 can mean "back".
    pub(crate) fn scroll_block(&mut self, lines: &[DLine], width: usize, dir: i32) -> bool {
        let Some(key) = lines.get(self.line).and_then(|l| l.nowrap_key()) else {
            return false;
        };
        let Some(&extent) = detail::nowrap_extents(lines).get(&key) else {
            return false;
        };
        let cur = self.block_hscroll(key, extent, width);
        let next = hscroll_step(cur, extent, width, dir);
        self.hscroll = Some((key, next));
        next != cur
    }

    /// Forget the sideways scroll once the cursor has left its block.
    pub(crate) fn drop_stale_hscroll(&mut self, lines: &[DLine]) {
        if let Some((key, _)) = self.hscroll {
            if lines.get(self.line).and_then(|l| l.nowrap_key()) != Some(key) {
                self.hscroll = None;
            }
        }
    }

    /// Keep the cursor and scroll inside `n` rows (the page shrank).
    pub(crate) fn clamp(&mut self, n: usize, vp: Viewport) {
        self.line = self.line.min(n.saturating_sub(1));
        self.anchor = None;
        self.scroll = (self.scroll as usize).min(vp.max_scroll(n)) as u16;
    }
}

/// The offset after one sideways step `dir` from `cur`, for a block `extent`
/// columns wide in a `width`-column pane: clamped so the block never scrolls
/// past its widest row.
pub(crate) fn hscroll_step(cur: usize, extent: usize, width: usize, dir: i32) -> usize {
    let max = extent.saturating_sub(width);
    (cur as i32 + dir * HSCROLL_STEP as i32).clamp(0, max as i32) as usize
}

/// The text a yank of rows `lo..=hi` copies, dedented: a rendered markdown
/// row (`src` set) yanks its source lines from `blocks[src.block]`, each once
/// and in order (yaks-649d); any other row yanks its displayed text, soft
/// wraps rejoined. `None` when there are no rows.
pub(crate) fn yank_text(lines: &[DLine], lo: usize, hi: usize, blocks: &[&str]) -> Option<String> {
    if lines.is_empty() || lo >= lines.len() {
        return None;
    }
    let hi = hi.min(lines.len() - 1);
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
            let text: Vec<&str> = block.split('\n').collect();
            // Within one block, include source lines between the last copied
            // one and this row (the blank lines a paragraph gap renders as
            // one row, a fence line, ...).
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

/// Where a link points, for the status bar: a collapsed `[text](url)` shows
/// only its text, so its destination is shown there instead. URLs,
/// attachments, source files, farm documents and in-page anchors; yak ids and
/// footnotes speak for themselves.
pub(crate) fn link_dest(target: Target) -> Option<String> {
    use crate::links::{Link, label};
    match target {
        Target::Url(u) | Target::Artifact(u) => Some(u),
        Target::Source { path, lines, rev } => {
            Some(super::source::source_label(&path, lines, rev.as_deref()))
        }
        Target::Farm { path, rev, anchor } => Some(label(&Link::Farm { path, rev, anchor })),
        Target::Local(a) => Some(label(&Link::Local(a))),
        Target::Task(_) | Target::Footnote(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(text: &str, links: Vec<(usize, usize, Target)>) -> DLine {
        DLine {
            text: text.into(),
            kind: detail::Kind::Body,
            links,
            md: vec![],
            cont: false,
            src: None,
            nowrap: None,
            image: None,
        }
    }

    fn page() -> Vec<DLine> {
        let url = |c| vec![(c, 3, Target::Url("https://x".into()))];
        vec![
            row("alpha", vec![]),
            row("see foo", url(4)),
            row("beta alpha", vec![]),
            row("and bar", url(4)),
            row("gamma", vec![]),
        ]
    }

    const VP: Viewport = Viewport { page: 3, pinned: 0 };

    #[test]
    fn the_cursor_steps_and_clamps_and_scrolls_into_view() {
        let lines = page();
        let mut nav = LineNav::default();
        nav.step(&lines, VP, 1);
        assert_eq!((nav.line, nav.scroll), (1, 0));
        nav.step(&lines, VP, 10);
        assert_eq!(nav.line, 4);
        // Five rows on a three-row page: the last row keeps a blank under it.
        assert_eq!(nav.scroll, 3);
        nav.to_end(&lines, VP, false);
        assert_eq!((nav.line, nav.scroll), (0, 0));
    }

    #[test]
    fn tab_cycles_link_rows_and_wraps() {
        let lines = page();
        let mut nav = LineNav::default();
        nav.jump_link(&lines, VP, 1);
        assert_eq!(nav.line, 1);
        nav.jump_link(&lines, VP, 1);
        assert_eq!(nav.line, 3);
        nav.jump_link(&lines, VP, 1);
        assert_eq!(nav.line, 1, "wraps to the first link");
        nav.jump_link(&lines, VP, -1);
        assert_eq!(nav.line, 3, "and back to the last");
        assert!(nav.link_at(&lines).is_some());
    }

    #[test]
    fn find_matches_cycle_and_selection_yanks_rejoined_text() {
        let lines = page();
        let mut nav = LineNav {
            find: Some("ALPHA".into()),
            ..Default::default()
        };
        assert_eq!(nav.find_matches(&lines), vec![(0, 0, 5), (2, 5, 5)]);
        nav.find_jump(&lines, 1);
        assert_eq!((nav.matched, nav.scroll), (1, 2));
        nav.find_jump(&lines, 1);
        assert_eq!(nav.matched, 0, "wraps");
        nav.toggle_visual();
        nav.extend_selection(&lines, VP, 1);
        assert_eq!(nav.selection(), Some((0, 1)));
        let (lo, hi) = nav.selection().unwrap();
        assert_eq!(yank_text(&lines, lo, hi, &[]).unwrap(), "alpha\nsee foo");
    }

    #[test]
    fn an_in_page_jump_goes_back_and_forward() {
        let lines = page();
        let mut nav = LineNav::default();
        nav.jump_to(&lines, VP, 4);
        assert_eq!(nav.line, 4);
        assert!(nav.go_back(&lines, false));
        assert_eq!(nav.line, 0);
        assert!(nav.go_back(&lines, true));
        assert_eq!(nav.line, 4);
        assert!(!nav.go_back(&lines, true), "nothing further forward");
    }
}
