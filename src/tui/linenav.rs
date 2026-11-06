//! Line navigation over rendered rows (yaks-84e0): the line cursor, visual
//! selection, find, link cycling and sideways scrolling of no-wrap blocks,
//! shared by every pane that shows a page of [`detail::DLine`] rows.
//!
//! - [`LineNav`] is one page's navigation state (cursor, scroll, selection,
//!   current find match, sideways scroll) and its operations, each given
//!   the page's rows and [`Viewport`]. Where it has been is not its own (the
//!   app's one history, [`super::history`], holds that), nor is what it finds: the search register (`App::register`, yaks-c215) is
//!   handed in, so every page searches the same text.
//! - [`Viewport`] is the scroll geometry: the rows a page shows, and how many
//!   of them a pinned header takes once scrolled.
//! - The free functions ([`yank_text`], [`link_dest`], [`hscroll_step`]) are
//!   the parts a host may use without the state.
//!
//! The text viewer ([`super::handlers::AttachView`]) owns a `LineNav`, and so
//! does the detail pane (`App::detail`), so a fix here applies to both. A new
//! page host (the doks tab) owns a `LineNav` like the viewer, renders with
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
    /// Which match of the search register is current (`n`/`N`); `None` until
    /// one has been landed on, so the first `n` goes to the first match.
    pub(crate) matched: Option<usize>,
    /// Sideways scroll of the no-wrap block under the cursor:
    /// `(DLine::nowrap_key, columns)`. Dropped once the cursor leaves it.
    pub(crate) hscroll: Option<((usize, usize), usize)>,
    /// The link Tab/`]`/`[` targeted, as `(row, column)` (yaks-fc9e). Any
    /// other cursor move (a step, `g`/`G`, a jump, a find) drops it, so back on
    /// a row its first link is the target again; it is ignored off its row.
    pub(crate) link: Option<(usize, usize)>,
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
        self.link = None;
        self.reveal(lines.len(), vp);
    }

    /// g / G: the cursor to the top / bottom stop.
    pub(crate) fn to_end(&mut self, lines: &[DLine], vp: Viewport, end: bool) {
        self.line = if end {
            detail::cursor_stops(lines).last().copied().unwrap_or(0)
        } else {
            0
        };
        self.link = None;
        self.reveal(lines.len(), vp);
    }

    /// Tab / `]` and Shift-Tab / `[`: target the next / previous link
    /// (wrapping), every link a stop, several on one row included (yaks-fc9e).
    /// The cursor goes to its row, so Enter, `O` and the peek act on it.
    pub(crate) fn jump_link(&mut self, lines: &[DLine], vp: Viewport, delta: i32) {
        let stops: Vec<(usize, usize)> = detail::jumplist(lines)
            .iter()
            .map(|j| (j.line, j.col))
            .collect();
        let (Some(&first), Some(&last)) = (stops.first(), stops.last()) else {
            return;
        };
        // Where we are: the targeted link, else just before (after) the
        // cursor's row, so the first Tab lands on its first link's successor
        // only if the row's first link was already the target.
        let here = self.link_at(lines).map(|j| (j.line, j.col));
        let to = if delta >= 0 {
            let after = |s: &&(usize, usize)| match here {
                Some(h) => **s > h,
                None => s.0 > self.line,
            };
            stops.iter().find(after).copied().unwrap_or(first)
        } else {
            let before = |s: &&(usize, usize)| match here {
                Some(h) => **s < h,
                None => s.0 < self.line,
            };
            stops.iter().rev().find(before).copied().unwrap_or(last)
        };
        self.line = to.0;
        self.link = Some(to);
        self.reveal(lines.len(), vp);
    }

    /// The targeted link on the cursor's row (else its first): what Enter
    /// follows, `O` opens, the peek reads and the row highlights.
    pub(crate) fn link_at(&self, lines: &[DLine]) -> Option<Jump> {
        let on_row: Vec<Jump> = detail::jumplist(lines)
            .into_iter()
            .filter(|j| j.line == self.line)
            .collect();
        let picked = self
            .link
            .filter(|(l, _)| *l == self.line)
            .and_then(|(_, c)| on_row.iter().position(|j| j.col == c));
        on_row.into_iter().nth(picked.unwrap_or(0))
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

    /// Back to the top of a page: the cursor, scroll, selection and current
    /// match (a different subject, or the same one opened afresh).
    pub(crate) fn to_top(&mut self) {
        self.line = 0;
        self.scroll = 0;
        self.anchor = None;
        self.matched = None;
        self.link = None;
    }

    /// n / N: land on the next / previous match of `find`, the cursor on its
    /// row and the row at the top of the page. Returns whether it did: not
    /// when there is no match, nor, without `wrap`, when the current one is
    /// the last (first). From no current match `n` goes to the first match,
    /// `N` to the last.
    pub(crate) fn find_step(
        &mut self,
        lines: &[DLine],
        find: Option<&str>,
        delta: i32,
        wrap: bool,
    ) -> bool {
        let m = find_matches(lines, find);
        let n = m.len() as i32;
        let to = match self.matched {
            Some(i) => i as i32 + delta,
            None if delta >= 0 => 0,
            None => n - 1,
        };
        let to = match (n, wrap) {
            (0, _) => return false,
            (n, true) => to.rem_euclid(n),
            (n, false) if (0..n).contains(&to) => to,
            _ => return false,
        } as usize;
        self.matched = Some(to);
        self.line = m[to].0;
        self.link = None;
        self.scroll = m[to].0 as u16;
        true
    }

    /// /: a find box seeded with the search register, remembering where the
    /// cursor, scroll and current match are so cancelling returns there
    /// (vi-like).
    pub(crate) fn open_find(&self, vim: bool, register: Option<&str>) -> SearchBox {
        let mut sb = SearchBox::new(vim, register.map(String::from));
        sb.find_origin = Some((self.scroll, self.line, self.matched));
        sb
    }

    /// One key typed into the open find box `sb`, which edits `register`:
    /// matches highlight live, Enter keeps the register and leaves the cursor
    /// on the current match, `cancel` (Esc / Ctrl-C) restores the register and
    /// position from before. Returns whether the box closed.
    pub(crate) fn find_key(
        &mut self,
        sb: &mut SearchBox,
        k: KeyEvent,
        cancel: bool,
        lines: &[DLine],
        register: &mut Option<String>,
    ) -> bool {
        if cancel {
            *register = sb.saved.clone();
            if let Some((scroll, line, matched)) = sb.find_origin {
                self.scroll = scroll;
                self.line = line;
                self.matched = matched;
            }
            return true;
        }
        if k.code == KeyCode::Enter {
            let m = find_matches(lines, register.as_deref());
            if let Some(&(line, ..)) = m.get(self.matched.unwrap_or(0)) {
                self.matched = Some(self.matched.unwrap_or(0));
                self.line = line;
                self.scroll = line as u16;
            }
            return true;
        }
        sb.handler.on_key_event(k, &mut sb.query.borrow_mut());
        let q = sb.query_text();
        *register = if q.is_empty() { None } else { Some(q) };
        let first = find_matches(lines, register.as_deref()).first().copied();
        self.matched = first.map(|_| 0);
        // The cursor follows the view to the match as you type, as `n` and
        // Enter do (yaks-fc9e); Esc puts both back.
        if let Some((line, ..)) = first {
            self.line = line;
            self.scroll = line as u16;
        }
        false
    }

    /// Put the cursor on row `to` within the page, keeping it in view. An
    /// in-page jump (a footnote, a `#section`) is a jump of the app's
    /// history ([`App::jump`](super::App::jump)), which remembers where the
    /// cursor was; the page keeps no history of its own.
    pub(crate) fn goto(&mut self, lines: &[DLine], vp: Viewport, to: usize) {
        self.line = to;
        self.link = None;
        self.reveal(lines.len(), vp);
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

/// `(line, col, len)` of every match of `find` (the search register) in the
/// rows.
pub(crate) fn find_matches(lines: &[DLine], find: Option<&str>) -> Vec<(usize, usize, usize)> {
    match find {
        Some(q) if !q.is_empty() => detail_scan(lines, q),
        _ => vec![],
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
/// attachments, source files, farm documents, notes and in-page anchors; yak
/// ids and footnotes speak for themselves.
pub(crate) fn link_dest(target: Target) -> Option<String> {
    use crate::links::{Link, label};
    match target {
        Target::Url(u) | Target::Artifact(u) => Some(u),
        Target::Source { path, lines, rev } => {
            Some(super::source::source_label(&path, lines, rev.as_deref()))
        }
        Target::Farm { path, rev, anchor } => Some(label(&Link::Farm { path, rev, anchor })),
        Target::Local(a) => Some(label(&Link::Local(a))),
        Target::Note { id, anchor } => Some(label(&Link::Yak {
            id,
            note: Some(anchor),
        })),
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

    /// yaks-fc9e: several links on one row are each a Tab stop, and the one
    /// targeted is what `link_at` (Enter, `O`, the peek, the highlight)
    /// names; moving the cursor off the row and back targets its first again.
    #[test]
    fn tab_visits_every_link_on_a_row_and_link_at_follows_it() {
        let two = vec![
            (0, 1, Target::Task("a".into())),
            (4, 1, Target::Task("b".into())),
        ];
        let lines = vec![
            row("a , b", two),
            row("text", vec![]),
            row("see c", vec![(4, 1, Target::Task("c".into()))]),
        ];
        let target = |nav: &LineNav| match nav.link_at(&lines).unwrap().target {
            Target::Task(id) => id,
            _ => unreachable!(),
        };
        let mut nav = LineNav::default();
        // The cursor starts on row 0: its first link is the target already,
        // so the first Tab goes to the second link of the same row.
        assert_eq!(target(&nav), "a");
        nav.jump_link(&lines, VP, 1);
        assert_eq!((nav.line, target(&nav).as_str()), (0, "b"));
        nav.jump_link(&lines, VP, 1);
        assert_eq!((nav.line, target(&nav).as_str()), (2, "c"));
        nav.jump_link(&lines, VP, 1);
        assert_eq!(target(&nav), "a", "wraps");
        nav.jump_link(&lines, VP, -1);
        assert_eq!(target(&nav), "c", "and back");
        nav.jump_link(&lines, VP, -1);
        assert_eq!(target(&nav), "b");
        // Off the row and back: the row's first link again.
        nav.step(&lines, VP, 1);
        nav.step(&lines, VP, -1);
        assert_eq!(target(&nav), "a");
    }

    #[test]
    fn find_matches_cycle_and_selection_yanks_rejoined_text() {
        let lines = page();
        let mut nav = LineNav::default();
        let find = Some("ALPHA");
        assert_eq!(find_matches(&lines, find), vec![(0, 0, 5), (2, 5, 5)]);
        assert!(nav.find_step(&lines, find, 1, true));
        assert_eq!(
            (nav.matched, nav.line),
            (Some(0), 0),
            "the first, from none"
        );
        assert!(nav.find_step(&lines, find, 1, true));
        assert_eq!((nav.matched, nav.line, nav.scroll), (Some(1), 2, 2));
        assert!(nav.find_step(&lines, find, 1, true));
        assert_eq!(nav.matched, Some(0), "wraps");
        assert!(nav.find_step(&lines, find, -1, true));
        assert_eq!(nav.matched, Some(1), "and back");
        assert!(
            !nav.find_step(&lines, find, 1, false),
            "no wrap past the last"
        );
        assert_eq!(nav.matched, Some(1));
        assert!(!nav.find_step(&lines, Some("zzz"), 1, true), "no match");
        nav.to_top();
        assert!(nav.find_step(&lines, find, -1, false));
        assert_eq!(nav.matched, Some(1), "N from none goes to the last");
        nav.to_top();
        assert_eq!((nav.line, nav.scroll, nav.matched), (0, 0, None));
        nav.toggle_visual();
        nav.extend_selection(&lines, VP, 1);
        assert_eq!(nav.selection(), Some((0, 1)));
        let (lo, hi) = nav.selection().unwrap();
        assert_eq!(yank_text(&lines, lo, hi, &[]).unwrap(), "alpha\nsee foo");
    }

    #[test]
    fn goto_puts_the_cursor_on_a_row_in_view() {
        let lines = page();
        let mut nav = LineNav::default();
        nav.goto(&lines, VP, 4);
        assert_eq!(nav.line, 4);
    }
}
