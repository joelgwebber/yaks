//! Block markdown renderer for the detail (read) pane (yaks-649d).
//!
//! Parses one markdown document (the description, or a single comment: the
//! caller splits the body with `content::parse` first) with `pulldown-cmark`
//! and lays it out into screen rows already wrapped to a width. Unlike the
//! line-at-a-time [`super::markdown::Highlighter`] (still used by the editor,
//! where display width must equal text width), this renderer sees the block
//! structure, so it can hide markers, reflow paragraphs (CommonMark: a single
//! newline is a soft break, rendered as a space), and indent wrapped list
//! items and quotes.
//!
//! **Layout rule: a prefix stack.** Every open container (block quote, list
//! item, footnote definition) contributes a pair of prefixes, one for its first
//! row and one for every later row: `▎ `/`▎ ` for a quote, `• `/`  ` for a
//! bullet, `1. `/`   ` for an ordered item. A row's prefix is the stack's
//! prefixes joined, so nesting (a quote holding a list holding code) composes
//! with no per-combination code, and wrapped text hangs under its marker.
//!
//! **Source map.** Each row carries the 0-based range of source lines (within
//! the document) it came from, so yank can copy source and `E` can open the
//! editor at the right line.
//!

use std::collections::HashSet;
use std::ops::Range;

use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use ratatui::style::{Modifier, Style};
use unicode_width::UnicodeWidthChar;

use super::detail::Target;
use super::markdown::MdSpan;
use super::palette::{self, AlertKind};
use crate::refs;

/// One rendered screen row.
#[derive(Debug, Clone, PartialEq)]
pub struct MdRow {
    /// Displayed text, prefixes included.
    pub text: String,
    /// Styled runs (char coords), non-overlapping, left to right.
    pub spans: Vec<MdSpan>,
    /// Followable links (char col, char len, target).
    pub links: Vec<(usize, usize, Target)>,
    /// Inclusive 0-based source line range within the document.
    pub lines: (usize, usize),
    /// A no-wrap row (code block, table): may be wider than the render width;
    /// the pane clips it (horizontal scroll is yaks-c733).
    pub nowrap: bool,
}

// Styles all come from the shared palette (yaks-bc53).
fn heading(level: HeadingLevel) -> Style {
    palette::heading(level as u8)
}
fn code() -> Style {
    palette::inline_code()
}
fn dim() -> Style {
    palette::marker()
}
fn bullet() -> Style {
    palette::bullet()
}
fn link() -> Style {
    palette::link_text()
}
fn alert_kind(kind: BlockQuoteKind) -> AlertKind {
    match kind {
        BlockQuoteKind::Note => AlertKind::Note,
        BlockQuoteKind::Tip => AlertKind::Tip,
        BlockQuoteKind::Important => AlertKind::Important,
        BlockQuoteKind::Warning => AlertKind::Warning,
        BlockQuoteKind::Caution => AlertKind::Caution,
    }
}

/// One character of laid-out text with everything needed to emit it.
#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    style: Style,
    /// Index into `Renderer::targets`.
    link: Option<usize>,
    /// Source line this char came from.
    line: usize,
}

/// A forced row break inside inline content (a CommonMark hard break).
const HARD_BREAK: char = '\n';

struct Container {
    first: String,
    cont: String,
    style: Style,
    /// Whether the first row has been emitted (later rows use `cont`).
    used: bool,
}

struct List {
    next: Option<u64>,
    loose: bool,
}

struct Renderer<'k> {
    width: usize,
    known: &'k HashSet<&'k str>,
    line_starts: Vec<usize>,
    rows: Vec<MdRow>,
    containers: Vec<Container>,
    lists: Vec<List>,
    styles: Vec<Style>,
    targets: Vec<Target>,
    link: Option<usize>,
    buf: Vec<Cell>,
    pending_blank: bool,
    /// Inside a code or HTML block: text is emitted line by line, unwrapped.
    verbatim: Option<Style>,
    /// Table under construction: rows of cells of chars, plus alignments.
    table: Option<(Vec<Alignment>, Vec<Vec<Vec<Cell>>>)>,
}

/// Render one markdown document to rows wrapped at `width` display columns.
/// `known` is the farm's yak ids, so mentions become [`Target::Task`] links.
pub fn render(text: &str, width: usize, known: &HashSet<&str>) -> Vec<MdRow> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_GFM);
    let events: Vec<(Event, Range<usize>)> =
        Parser::new_ext(text, opts).into_offset_iter().collect();
    let mut line_starts = vec![0];
    line_starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
    let mut r = Renderer {
        width: width.max(1),
        known,
        line_starts,
        rows: Vec::new(),
        containers: Vec::new(),
        lists: Vec::new(),
        styles: Vec::new(),
        targets: Vec::new(),
        link: None,
        buf: Vec::new(),
        pending_blank: false,
        verbatim: None,
        table: None,
    };
    for (i, (ev, range)) in events.iter().enumerate() {
        let next = events.get(i + 1).map(|(e, _)| e);
        r.event(ev, range.clone(), next);
    }
    r.flush();
    r.rows
}

impl Renderer<'_> {
    fn line_of(&self, byte: usize) -> usize {
        self.line_starts
            .partition_point(|&s| s <= byte)
            .saturating_sub(1)
    }

    fn style(&self) -> Style {
        self.styles
            .iter()
            .fold(Style::new(), |acc, s| acc.patch(*s))
    }

    /// A block is about to start: flush inline text and emit the blank row
    /// separating it from the previous block, if one is owed.
    fn block_start(&mut self) {
        self.flush();
        if self.pending_blank && !self.rows.is_empty() {
            self.blank_row();
        }
        self.pending_blank = false;
    }

    fn block_end(&mut self) {
        self.flush();
        self.pending_blank = true;
    }

    fn event(&mut self, ev: &Event, range: Range<usize>, next: Option<&Event>) {
        let line = self.line_of(range.start);
        match ev {
            Event::Start(tag) => self.start(tag, line, next),
            Event::End(tag) => self.end(tag),
            Event::Text(t) => {
                if let Some(style) = self.verbatim {
                    self.verbatim_text(t, line, style);
                } else {
                    self.push_text(t, line);
                }
            }
            Event::Code(t) => {
                let st = self.style().patch(code());
                self.push_linked(t, line, st, true);
            }
            Event::Html(t) => self.verbatim_text(t, line, Style::new()),
            Event::InlineHtml(t) | Event::InlineMath(t) | Event::DisplayMath(t) => {
                let st = self.style();
                self.push_linked(t, line, st, false);
            }
            Event::FootnoteReference(label) => {
                let st = self.style().patch(dim());
                self.push_plain(&format!("[{label}]"), line, st);
            }
            Event::SoftBreak => {
                let st = self.style();
                self.push_plain(" ", line, st);
            }
            Event::HardBreak => self.push_plain(&HARD_BREAK.to_string(), line, Style::new()),
            Event::Rule => {
                self.block_start();
                let avail = self.avail();
                let cells = vec![
                    Cell {
                        ch: palette::RULE_CHAR,
                        style: palette::rule(),
                        link: None,
                        line,
                    };
                    avail
                ];
                self.emit(&cells, line, false);
                self.pending_blank = true;
            }
            Event::TaskListMarker(done) => {
                let glyph = format!("{} ", palette::task_glyph(*done));
                self.push_plain(&glyph, line, palette::task(*done));
            }
        }
    }

    fn start(&mut self, tag: &Tag, line: usize, next: Option<&Event>) {
        match tag {
            Tag::Paragraph => self.block_start(),
            Tag::Heading { level, .. } => {
                self.block_start();
                self.styles.push(heading(*level));
            }
            Tag::BlockQuote(kind) => {
                self.block_start();
                let (style, title) = match kind {
                    Some(k) => {
                        let k = alert_kind(*k);
                        (palette::alert(k), Some((k, palette::alert_title(k))))
                    }
                    None => (palette::quote_bar(), None),
                };
                if title.is_none() {
                    self.styles.push(palette::quote_text());
                } else {
                    self.styles.push(Style::new());
                }
                self.containers.push(Container {
                    first: format!("{} ", palette::QUOTE_BAR),
                    cont: format!("{} ", palette::QUOTE_BAR),
                    style,
                    used: false,
                });
                if let Some((k, tstyle)) = title {
                    let t = format!("{} {}", k.glyph(), k.label());
                    let cells: Vec<Cell> = t
                        .chars()
                        .map(|ch| Cell {
                            ch,
                            style: tstyle,
                            link: None,
                            line,
                        })
                        .collect();
                    self.emit(&cells, line, false);
                }
            }
            Tag::CodeBlock(kind) => {
                self.block_start();
                let _lang = match kind {
                    CodeBlockKind::Fenced(l) => l.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                self.verbatim = Some(palette::code_block());
            }
            Tag::HtmlBlock => {
                self.block_start();
                self.verbatim = Some(Style::new());
            }
            Tag::List(start) => {
                self.block_start();
                self.lists.push(List {
                    next: *start,
                    loose: false,
                });
            }
            Tag::Item => {
                // Items of a loose list are separated by a blank row (their
                // paragraphs owe one); a tight list's are not.
                let loose = matches!(next, Some(Event::Start(Tag::Paragraph)));
                if let Some(l) = self.lists.last_mut() {
                    l.loose |= loose;
                }
                let tight = self.lists.last().is_some_and(|l| !l.loose);
                if tight {
                    self.flush();
                    self.pending_blank = false;
                } else {
                    self.block_start();
                }
                let depth = self.lists.len().saturating_sub(1);
                let marker = match self.lists.last_mut().and_then(|l| l.next.as_mut()) {
                    Some(n) => {
                        let m = format!("{n}. ");
                        *n += 1;
                        m
                    }
                    None => format!("{} ", palette::bullet_glyph(depth)),
                };
                let w = disp(&marker);
                self.containers.push(Container {
                    first: marker,
                    cont: " ".repeat(w),
                    style: bullet(),
                    used: false,
                });
            }
            Tag::FootnoteDefinition(label) => {
                self.block_start();
                let first = format!("[{label}] ");
                let w = disp(&first);
                self.containers.push(Container {
                    first,
                    cont: " ".repeat(w),
                    style: dim(),
                    used: false,
                });
            }
            Tag::Table(align) => {
                self.block_start();
                self.table = Some((align.clone(), Vec::new()));
            }
            Tag::TableHead | Tag::TableRow => {
                if let Some((_, rows)) = self.table.as_mut() {
                    rows.push(Vec::new());
                }
            }
            Tag::TableCell => self.buf.clear(),
            Tag::Emphasis => self.styles.push(palette::emphasis()),
            Tag::Strong => self.styles.push(palette::strong()),
            Tag::Strikethrough => self.styles.push(palette::strikethrough()),
            Tag::Superscript | Tag::Subscript => self.styles.push(Style::new()),
            Tag::Link { dest_url, .. } => {
                self.targets.push(link_target(dest_url));
                self.link = Some(self.targets.len() - 1);
                self.styles.push(link());
            }
            Tag::Image { dest_url, .. } => {
                self.targets.push(Target::Artifact(dest_url.to_string()));
                self.link = Some(self.targets.len() - 1);
                self.styles.push(link());
                let st = self.style();
                self.push_plain("🖼 ", line, st);
            }
            Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::MetadataBlock(_) => {}
        }
    }

    fn end(&mut self, tag: &TagEnd) {
        match tag {
            TagEnd::Paragraph => self.block_end(),
            TagEnd::Heading(_) => {
                self.block_end();
                self.styles.pop();
            }
            TagEnd::BlockQuote(_) => {
                self.block_end();
                self.containers.pop();
                self.styles.pop();
            }
            TagEnd::FootnoteDefinition => {
                self.block_end();
                self.containers.pop();
            }
            TagEnd::CodeBlock | TagEnd::HtmlBlock => {
                self.verbatim = None;
                self.pending_blank = true;
            }
            TagEnd::List(_) => {
                self.block_end();
                self.lists.pop();
            }
            TagEnd::Item => {
                self.flush();
                self.containers.pop();
            }
            TagEnd::Table => {
                self.emit_table();
                self.pending_blank = true;
            }
            TagEnd::TableCell => {
                let cell = std::mem::take(&mut self.buf);
                if let Some(row) = self.table.as_mut().and_then(|(_, r)| r.last_mut()) {
                    row.push(cell);
                }
            }
            TagEnd::TableHead | TagEnd::TableRow => {}
            TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript => {
                self.styles.pop();
            }
            TagEnd::Link | TagEnd::Image => {
                self.styles.pop();
                self.link = None;
            }
            TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::MetadataBlock(_) => {}
        }
    }

    /// Push prose text, auto-linking bare URLs and yak ids (unless already
    /// inside an explicit link).
    fn push_text(&mut self, t: &str, line: usize) {
        let t = refs::strip_wikilinks(t);
        let st = self.style();
        self.push_linked(&t, line, st, true);
    }

    fn push_plain(&mut self, t: &str, line: usize, style: Style) {
        let link = self.link;
        self.buf.extend(t.chars().map(|ch| Cell {
            ch,
            style,
            link,
            line,
        }));
    }

    fn push_linked(&mut self, t: &str, line: usize, style: Style, autolink: bool) {
        if self.link.is_some() || !autolink {
            return self.push_plain(t, line, style);
        }
        let chars: Vec<char> = t.chars().collect();
        let mut found = autolinks(&chars, self.known).into_iter().peekable();
        let mut i = 0;
        while i < chars.len() {
            if let Some((s, len, target)) = found.next_if(|(s, _, _)| *s == i) {
                self.targets.push(target);
                let idx = self.targets.len() - 1;
                let ls = style.patch(link());
                self.buf.extend(chars[s..s + len].iter().map(|&ch| Cell {
                    ch,
                    style: ls,
                    link: Some(idx),
                    line,
                }));
                i = s + len;
                continue;
            }
            self.buf.push(Cell {
                ch: chars[i],
                style,
                link: None,
                line,
            });
            i += 1;
        }
    }

    /// Emit code/HTML block text one source line per row, unwrapped.
    fn verbatim_text(&mut self, t: &str, line: usize, style: Style) {
        let t = t.strip_suffix('\n').unwrap_or(t);
        for (k, l) in t.split('\n').enumerate() {
            let cells: Vec<Cell> = l
                .chars()
                .map(|ch| Cell {
                    ch,
                    style,
                    link: None,
                    line: line + k,
                })
                .collect();
            self.emit(&cells, line + k, true);
        }
    }

    /// Width left for text after the current prefix stack.
    fn avail(&self) -> usize {
        let w: usize = self
            .containers
            .iter()
            .map(|c| disp(if c.used { &c.cont } else { &c.first }))
            .sum();
        self.width.saturating_sub(w).max(8)
    }

    /// Word-wrap the buffered inline text into rows.
    fn flush(&mut self) {
        if self.buf.is_empty() || self.table.is_some() {
            return;
        }
        let buf = std::mem::take(&mut self.buf);
        let line = buf[0].line;
        for seg in buf.split(|c| c.ch == HARD_BREAK) {
            if seg.is_empty() {
                self.emit(&[], line, false);
                continue;
            }
            // The prefix width can shrink after the first row (an ordered
            // marker `10. ` vs its continuation is the same width, but a
            // container's first row may differ), so re-measure per row.
            let mut start = 0;
            while start < seg.len() {
                let width = self.avail();
                let end = wrap_end(&seg[start..], width) + start;
                let mut row_end = end;
                // Drop the break space.
                while row_end > start && seg[row_end - 1].ch == ' ' && end < seg.len() {
                    row_end -= 1;
                }
                self.emit(&seg[start..row_end], seg[start].line, false);
                start = end;
                while start < seg.len() && seg[start].ch == ' ' {
                    start += 1;
                }
            }
        }
    }

    fn blank_row(&mut self) {
        let mut text = String::new();
        let mut spans = Vec::new();
        for c in &self.containers {
            let p = c.cont.trim_end();
            if !p.is_empty() {
                spans.push(MdSpan {
                    start: text.chars().count(),
                    len: p.chars().count(),
                    style: c.style,
                });
            }
            text.push_str(&c.cont);
        }
        let line = self.rows.last().map_or(0, |r| r.lines.1);
        let trimmed = text.trim_end().to_string();
        self.rows.push(MdRow {
            text: trimmed,
            spans,
            links: Vec::new(),
            lines: (line, line),
            nowrap: false,
        });
    }

    /// Emit one row: the prefix stack, then `cells`.
    fn emit(&mut self, cells: &[Cell], line: usize, nowrap: bool) {
        let mut text = String::new();
        let mut spans: Vec<MdSpan> = Vec::new();
        let mut col = 0;
        for c in self.containers.iter_mut() {
            let p = if c.used { &c.cont } else { &c.first };
            let n = p.chars().count();
            if !p.trim().is_empty() {
                spans.push(MdSpan {
                    start: col,
                    len: n,
                    style: c.style,
                });
            }
            text.push_str(p);
            col += n;
            c.used = true;
        }
        let mut links: Vec<(usize, usize, Target)> = Vec::new();
        let (mut lo, mut hi) = (line, line);
        for (k, cell) in cells.iter().enumerate() {
            text.push(cell.ch);
            lo = lo.min(cell.line);
            hi = hi.max(cell.line);
            let at = col + k;
            if cell.style != Style::new() {
                match spans.last_mut() {
                    Some(s) if s.style == cell.style && s.start + s.len == at => s.len += 1,
                    _ => spans.push(MdSpan {
                        start: at,
                        len: 1,
                        style: cell.style,
                    }),
                }
            }
            if let Some(li) = cell.link {
                let target = &self.targets[li];
                let prev_same = k > 0 && cells[k - 1].link == Some(li);
                match links.last_mut() {
                    Some((_, len, _)) if prev_same => *len += 1,
                    _ => links.push((at, 1, target.clone())),
                }
            }
        }
        self.rows.push(MdRow {
            text,
            spans,
            links,
            lines: (lo, hi),
            nowrap,
        });
    }

    fn emit_table(&mut self) {
        let Some((align, rows)) = self.table.take() else {
            return;
        };
        let ncols = rows
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(align.len());
        let mut widths = vec![0usize; ncols];
        for row in &rows {
            for (c, cell) in row.iter().enumerate() {
                widths[c] = widths[c].max(cell.iter().map(|x| char_w(x.ch)).sum());
            }
        }
        let sep = palette::rule();
        for (ri, row) in rows.iter().enumerate() {
            let line = row
                .iter()
                .flat_map(|c| c.first())
                .map(|c| c.line)
                .next()
                .unwrap_or_else(|| self.rows.last().map_or(0, |r| r.lines.1));
            let mk = |ch: char, style: Style| Cell {
                ch,
                style,
                link: None,
                line,
            };
            let mut cells: Vec<Cell> = Vec::new();
            for c in 0..ncols {
                if c > 0 {
                    cells.extend(" │ ".chars().map(|ch| mk(ch, sep)));
                }
                let empty = Vec::new();
                let cell = row.get(c).unwrap_or(&empty);
                let w: usize = cell.iter().map(|x| char_w(x.ch)).sum();
                let pad = widths[c] - w;
                let (l, r) = match align.get(c) {
                    Some(Alignment::Right) => (pad, 0),
                    Some(Alignment::Center) => (pad / 2, pad - pad / 2),
                    _ => (0, pad),
                };
                cells.extend((0..l).map(|_| mk(' ', Style::new())));
                let head = ri == 0;
                cells.extend(cell.iter().map(|x| Cell {
                    style: if head {
                        x.style.add_modifier(Modifier::BOLD)
                    } else {
                        x.style
                    },
                    ..*x
                }));
                cells.extend((0..r).map(|_| mk(' ', Style::new())));
            }
            self.emit(&cells, line, true);
            if ri == 0 {
                let mut rule: Vec<Cell> = Vec::new();
                for (c, w) in widths.iter().enumerate() {
                    if c > 0 {
                        rule.extend("─┼─".chars().map(|ch| mk(ch, sep)));
                    }
                    rule.extend((0..*w).map(|_| mk('─', sep)));
                }
                self.emit(&rule, line, true);
            }
        }
    }
}

fn char_w(c: char) -> usize {
    c.width().unwrap_or(0)
}

fn disp(s: &str) -> usize {
    s.chars().map(char_w).sum()
}

/// Greedy wrap: the end (exclusive) of the first row of `cells` that fits in
/// `width` columns, breaking after the last space that fits, or mid-word when
/// a single word is wider than the row.
fn wrap_end(cells: &[Cell], width: usize) -> usize {
    let mut w = 0;
    let mut end = 0;
    while end < cells.len() {
        let cw = char_w(cells[end].ch);
        if w + cw > width && end > 0 {
            break;
        }
        w += cw;
        end += 1;
    }
    if end >= cells.len() {
        return cells.len();
    }
    match (1..=end)
        .rev()
        .find(|&k| cells[k - 1].ch == ' ' || cells.get(k).is_some_and(|c| c.ch == ' '))
    {
        Some(k) if k > 0 => k,
        _ => end,
    }
}

/// Where a markdown link points: a web URL, or a farm-relative attachment.
fn link_target(dest: &str) -> Target {
    if dest.contains("://") || dest.starts_with("mailto:") {
        Target::Url(dest.to_string())
    } else {
        Target::Artifact(dest.to_string())
    }
}

/// Where a bare URL in `chars[start..end]` (a whitespace-delimited run) really
/// ends: the GFM extended-autolink rule. Trailing `.,;:!?'"` punctuation and an
/// unbalanced trailing `)` are not part of the URL, while balanced parens
/// inside it (`.../Foo_(bar)`) stay.
fn trim_url_end(chars: &[char], start: usize, mut end: usize) -> usize {
    while end > start {
        match chars[end - 1] {
            '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '"' => end -= 1,
            ')' => {
                let span = &chars[start..end];
                let open = span.iter().filter(|&&c| c == '(').count();
                let close = span.iter().filter(|&&c| c == ')').count();
                if close > open {
                    end -= 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    end
}

/// Bare URLs and known yak ids in prose, as `(start, len, target)` char runs.
/// A URL ends at whitespace, then [`trim_url_end`] applies (yaks-f67e).
pub fn autolinks(chars: &[char], known: &HashSet<&str>) -> Vec<(usize, usize, Target)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let at_url = ["https://", "http://"].iter().any(|p| {
            let p: Vec<char> = p.chars().collect();
            chars[i..].starts_with(&p)
        });
        if at_url {
            let start = i;
            while i < chars.len() && !chars[i].is_whitespace() {
                i += 1;
            }
            let end = trim_url_end(chars, start, i);
            out.push((
                start,
                end - start,
                Target::Url(chars[start..end].iter().collect()),
            ));
            i = end.max(start + 1);
            continue;
        }
        if refs::is_ref_char(chars[i]) {
            let (tok, end, run_end) = refs::token_at(chars, i);
            if !tok.is_empty() {
                if let Some(id) = refs::resolve(&tok, |t| known.contains(t)) {
                    out.push((i, end - i, Target::Task(id)));
                }
            }
            i = run_end;
            continue;
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(md: &str, width: usize) -> Vec<String> {
        let known = HashSet::new();
        render(md, width, &known)
            .into_iter()
            .map(|r| r.text)
            .collect()
    }

    #[test]
    fn soft_breaks_join_into_one_paragraph() {
        assert_eq!(texts("one\ntwo\nthree", 80), vec!["one two three"]);
    }

    #[test]
    fn blank_line_separates_paragraphs_and_hard_break_breaks() {
        assert_eq!(texts("a\n\nb  \nc", 80), vec!["a", "", "b", "c"]);
    }

    #[test]
    fn emphasis_markers_are_hidden_and_styled_across_a_soft_break() {
        let known = HashSet::new();
        let rows = render("x _long styled\nstring_ y", 80, &known);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].text, "x long styled string y");
        let it: Vec<_> = rows[0]
            .spans
            .iter()
            .filter(|s| s.style.add_modifier.contains(Modifier::ITALIC))
            .collect();
        assert_eq!(it.len(), 1);
        assert_eq!((it[0].start, it[0].len), (2, 18));
    }

    #[test]
    fn wrapped_list_item_hangs_under_its_marker() {
        let out = texts("- alpha beta gamma delta\n  - nested one two three", 16);
        assert_eq!(
            out,
            vec![
                "• alpha beta",
                "  gamma delta",
                "  ◦ nested one",
                "    two three"
            ]
        );
    }

    #[test]
    fn ordered_list_lazy_continuation_hangs() {
        let out = texts("1. one two\nthree four\n2. five", 12);
        assert_eq!(out, vec!["1. one two", "   three", "   four", "2. five"]);
    }

    #[test]
    fn quote_keeps_its_bar_when_wrapped_and_holds_nested_blocks() {
        let out = texts(
            "> # Head\n> words words words\n> - item\n> ```\n> code\n> ```",
            14,
        );
        assert_eq!(
            out,
            vec![
                "▎ Head",
                "▎",
                "▎ words words",
                "▎ words",
                "▎",
                "▎ • item",
                "▎",
                "▎ code"
            ]
        );
    }

    #[test]
    fn code_blocks_do_not_wrap_and_rows_map_to_source_lines() {
        let known = HashSet::new();
        let rows = render(
            "para\n\n```java\nclass VeryLongNameThatWillNotFit {}\n}\n```",
            10,
            &known,
        );
        let code: Vec<&MdRow> = rows.iter().filter(|r| r.nowrap).collect();
        assert_eq!(code[0].text, "class VeryLongNameThatWillNotFit {}");
        assert_eq!(code[0].lines, (3, 3));
        assert_eq!(code[1].lines, (4, 4));
        assert_eq!(rows[0].lines, (0, 0));
    }

    #[test]
    fn rule_heading_and_table() {
        let out = texts("a\n\n---\n\n| x | yy |\n|---|---:|\n| 1 | 2 |\n| 3 |", 10);
        assert_eq!(
            out,
            vec![
                "a",
                "",
                "──────────",
                "",
                "x │ yy",
                "──┼───",
                "1 │  2",
                "3 │   "
            ]
        );
    }

    #[test]
    fn links_collapse_to_text_and_keep_targets() {
        let mut known = HashSet::new();
        known.insert("yak-0001");
        let rows = render(
            "[Links](https://example.com/) and yak-0001 or https://e.com/a).",
            80,
            &known,
        );
        assert_eq!(rows[0].text, "Links and yak-0001 or https://e.com/a).");
        let targets: Vec<_> = rows[0]
            .links
            .iter()
            .map(|l| (l.0, l.1, l.2.clone()))
            .collect();
        assert_eq!(
            targets,
            vec![
                (0, 5, Target::Url("https://example.com/".into())),
                (10, 8, Target::Task("yak-0001".into())),
                (22, 15, Target::Url("https://e.com/a".into())),
            ]
        );
    }

    #[test]
    fn task_list_and_alert() {
        let out = texts("- [ ] todo\n- [x] done\n\n> [!NOTE]\n> heads up", 40);
        assert_eq!(
            out,
            vec!["• ☐ todo", "• ☑ done", "", "▎ ● NOTE", "▎ heads up"]
        );
    }

    #[test]
    fn loose_list_items_get_blank_rows_between() {
        let out = texts("- a\n\n- b", 40);
        assert_eq!(out, vec!["• a", "", "• b"]);
    }

    /// The torture-test body of yaks-ea13, rendered at 60 columns.
    #[test]
    fn ea13_fixture_snapshot() {
        let md = include_str!("testdata/ea13.md");
        let mut known = HashSet::new();
        known.insert("yaks-e9de");
        let out = render(md, 60, &known)
            .into_iter()
            .map(|r| r.text)
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!(out);
    }

    // yaks-f67e: where a bare URL ends.
    fn urls(line: &str) -> Vec<(usize, usize, Target)> {
        let chars: Vec<char> = line.chars().collect();
        autolinks(&chars, &HashSet::new())
    }

    #[test]
    fn url_in_markdown_link_stops_before_the_closing_paren() {
        assert_eq!(
            urls("[x](https://e.com/)"),
            vec![(4, 14, Target::Url("https://e.com/".into()))]
        );
    }

    #[test]
    fn url_drops_trailing_paren_and_punctuation() {
        assert_eq!(
            urls("see https://e.com/a)."),
            vec![(4, 15, Target::Url("https://e.com/a".into()))]
        );
        assert_eq!(
            urls("(https://e.com/a, ok)"),
            vec![(1, 15, Target::Url("https://e.com/a".into()))]
        );
        assert_eq!(
            urls("really https://e.com/a?!"),
            vec![(7, 15, Target::Url("https://e.com/a".into()))]
        );
    }

    #[test]
    fn url_keeps_balanced_parens() {
        let u = "https://en.wikipedia.org/wiki/Foo_(bar)";
        assert_eq!(urls(u), vec![(0, u.len(), Target::Url(u.into()))]);
        let line = format!("see ({u}).");
        assert_eq!(urls(&line), vec![(5, u.len(), Target::Url(u.into()))]);
    }
}
