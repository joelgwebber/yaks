//! Hand-rolled markdown highlighter shared by the detail (read) pane and the
//! embedded editor. Given logical lines it emits styled char-spans; fenced code
//! blocks are tracked across lines via [`Highlighter`]. Pure Rust — no syntect,
//! no onig, so it stays inside the C-free release matrix and we keep full
//! control over the palette (and, later, embedded code-block languages).
//!
//! Spans are in *char* coordinates within one logical line, so they compose
//! cleanly with the detail pane's link/find layers (which are also char-based)
//! and survive soft-wrapping the same way link spans do. The editor converts
//! them to edtui `Highlight`s (logical row/col) which edtui applies at render,
//! so they survive edtui's own wrapping too.

use ratatui::style::{Modifier, Style};

use super::palette;

/// A styled run of characters within one logical line (`start` is a char index,
/// `len` a char count). Runs never overlap and are emitted left-to-right; gaps
/// between them keep the caller's base style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MdSpan {
    pub start: usize,
    pub len: usize,
    pub style: Style,
}

// Every style below comes from the shared palette (`tui::palette`, yaks-bc53),
// which the read pane's renderer reads too; this file only decides *where* they
// apply on a line.
fn heading(level: u8) -> Style {
    palette::heading(level)
}
fn code() -> Style {
    palette::code_block()
}
fn marker() -> Style {
    palette::marker()
}
/// Syntax markers (`**`, `` ` ``, `~~`, `#`, `>`, link brackets and `(url)`): dim
/// on the cursor's line, additionally `Modifier::HIDDEN` (SGR 8: the cell keeps
/// its width but draws blank) on every other line. The dim colour stays under
/// HIDDEN so a terminal that ignores SGR 8 (the Linux console, ConPTY) still
/// shows a quiet marker rather than a loud one.
fn syntax(hide: bool) -> Style {
    if hide {
        marker().add_modifier(Modifier::HIDDEN)
    } else {
        marker()
    }
}
fn bold() -> Style {
    palette::strong()
}
fn italic() -> Style {
    palette::emphasis()
}
fn bullet() -> Style {
    palette::bullet()
}
fn quote() -> Style {
    palette::quote_text()
}

/// Stateful, line-at-a-time markdown highlighter. Feed it logical lines in
/// order via [`Highlighter::line`] (or [`Highlighter::line_with`] to hide
/// markers); it carries fenced-code-block state so a ```` ```lang ```` block
/// colors every line until its closing fence.
#[derive(Default)]
pub struct Highlighter {
    /// `Some((fence_char, run_len))` while inside a fenced code block.
    fence: Option<(char, usize)>,
}

impl Highlighter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Highlight one logical line, advancing block state. Returns the styled
    /// runs for that line (empty when there's nothing to color).
    #[cfg(test)]
    pub fn line(&mut self, text: &str) -> Vec<MdSpan> {
        self.line_with(text, false)
    }

    /// [`Highlighter::line`] with `hide_markers`: inline/heading/quote markers
    /// carry `Modifier::HIDDEN` too (the editor's rule for every line but the
    /// cursor's). Fence lines stay merely dim: they hold the info string, and a
    /// blank fence line would hide where a block starts.
    pub fn line_with(&mut self, text: &str, hide_markers: bool) -> Vec<MdSpan> {
        let hide = hide_markers;
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() {
            return Vec::new();
        }

        // Inside a fenced code block: everything is code until the closing fence.
        if let Some((fc, flen)) = self.fence {
            if is_closing_fence(&chars, fc, flen) {
                self.fence = None;
                return vec![whole(&chars, marker())];
            }
            return vec![whole(&chars, code())];
        }

        // Opening fence: mark the fence line and enter the block.
        if let Some((fc, flen)) = opening_fence(&chars) {
            self.fence = Some((fc, flen));
            return vec![whole(&chars, marker())];
        }

        // ATX heading: the `#` run and its space are a dimmed marker, the rest
        // takes the level's heading style. (Display width is unchanged: the editor never
        // collapses markers.)
        if let Some((level, hashes_end)) = atx_heading(&chars) {
            let text_start = (hashes_end + 1).min(chars.len());
            let mut spans = vec![MdSpan {
                start: indent(&chars),
                len: text_start - indent(&chars),
                style: syntax(hide),
            }];
            if text_start < chars.len() {
                spans.push(MdSpan {
                    start: text_start,
                    len: chars.len() - text_start,
                    style: heading(level),
                });
            }
            return spans;
        }

        // Blockquote: dimmed `>` marker, italic text.
        if let Some(gt) = blockquote_marker(&chars) {
            let mut spans = vec![MdSpan {
                start: gt,
                len: 1,
                style: syntax(hide),
            }];
            if gt + 1 < chars.len() {
                spans.push(MdSpan {
                    start: gt + 1,
                    len: chars.len() - gt - 1,
                    style: quote(),
                });
            }
            return spans;
        }

        // List item: color the bullet marker, inline-scan the rest.
        if let Some((marker_end, content_start)) = list_marker(&chars) {
            let ind = indent(&chars);
            let mut spans = vec![MdSpan {
                start: ind,
                len: marker_end - ind,
                style: bullet(),
            }];
            spans.extend(inline(&chars, content_start, hide));
            return spans;
        }

        // Plain paragraph line.
        inline(&chars, 0, hide)
    }
}

/// Highlight a whole multi-line string, returning per-line spans.
#[cfg(test)]
pub fn highlight(text: &str) -> Vec<Vec<MdSpan>> {
    let mut hl = Highlighter::new();
    text.split('\n').map(|l| hl.line(l)).collect()
}

fn whole(chars: &[char], style: Style) -> MdSpan {
    MdSpan {
        start: 0,
        len: chars.len(),
        style,
    }
}

fn indent(chars: &[char]) -> usize {
    chars.iter().take_while(|c| **c == ' ').count()
}

/// A code-fence opener: 3+ backticks or tildes after optional indent. The rest
/// of the line is an info string, which we don't parse here.
fn opening_fence(chars: &[char]) -> Option<(char, usize)> {
    let i = indent(chars);
    let fc = *chars.get(i)?;
    if fc != '`' && fc != '~' {
        return None;
    }
    let run = chars[i..].iter().take_while(|c| **c == fc).count();
    (run >= 3).then_some((fc, run))
}

/// A closing fence: at least as many of the opener's fence char as the opener,
/// followed only by whitespace.
fn is_closing_fence(chars: &[char], fc: char, flen: usize) -> bool {
    let i = indent(chars);
    let run = chars[i..].iter().take_while(|c| **c == fc).count();
    run >= flen && chars[i + run..].iter().all(|c| c.is_whitespace())
}

/// An ATX heading: 1–6 `#` after optional indent, then a space or end of line.
/// Returns `(level, hashes_end)` where `hashes_end` is one past the `#` run.
fn atx_heading(chars: &[char]) -> Option<(u8, usize)> {
    let i = indent(chars);
    let hashes = chars[i..].iter().take_while(|c| **c == '#').count();
    ((1..=6).contains(&hashes) && matches!(chars.get(i + hashes), None | Some(&' ')))
        .then_some((hashes as u8, i + hashes))
}

/// The column of a blockquote's `>` marker, if the line is one.
fn blockquote_marker(chars: &[char]) -> Option<usize> {
    let i = indent(chars);
    (chars.get(i) == Some(&'>')).then_some(i)
}

/// A list item marker. Returns `(marker_end, content_start)` where `marker_end`
/// is one past the bullet/number token and `content_start` skips the required
/// space after it. The marker itself starts at `indent(chars)`.
fn list_marker(chars: &[char]) -> Option<(usize, usize)> {
    let i = indent(chars);
    let c = *chars.get(i)?;
    if matches!(c, '-' | '*' | '+') && chars.get(i + 1) == Some(&' ') {
        return Some((i + 1, i + 2));
    }
    if c.is_ascii_digit() {
        let mut j = i;
        while j < chars.len() && chars[j].is_ascii_digit() {
            j += 1;
        }
        if matches!(chars.get(j), Some('.') | Some(')')) && chars.get(j + 1) == Some(&' ') {
            return Some((j + 1, j + 2));
        }
    }
    None
}

/// Inline scan of `chars[from..]` for code spans, emphasis, strikethrough and
/// links, emitting spans for the recognized (and properly closed) runs.
/// Unmatched delimiters are left unstyled. Delimiters are [`syntax`] markers;
/// the enclosed text carries the style. A link is `[text](url)`: the brackets
/// and the whole `(url)` are markers, the text takes [`palette::link_text`]
/// (with any emphasis inside it layered on top).
fn inline(chars: &[char], from: usize, hide: bool) -> Vec<MdSpan> {
    let n = chars.len();
    let mut spans = Vec::new();
    let mut i = from;
    let push = |spans: &mut Vec<MdSpan>, start: usize, len: usize, style: Style| {
        if len > 0 {
            spans.push(MdSpan { start, len, style });
        }
    };
    while i < n {
        let c = chars[i];
        if c == '`' {
            let run = chars[i..].iter().take_while(|x| **x == '`').count();
            if let Some(close) = find_backtick_run(chars, i + run, run) {
                push(&mut spans, i, run, syntax(hide));
                push(&mut spans, i + run, close - i - run, code());
                push(&mut spans, close, run, syntax(hide));
                i = close + run;
                continue;
            }
            i += run;
            continue;
        } else if c == '[' || (c == '!' && chars.get(i + 1) == Some(&'[')) {
            if let Some((text_start, text_end, url_end)) = link_at(chars, i) {
                push(&mut spans, i, text_start - i, syntax(hide));
                // Text: link style in the gaps, emphasis/code layered on it.
                let mut at = text_start;
                for sp in inline(&chars[..text_end], text_start, hide) {
                    push(&mut spans, at, sp.start - at, palette::link_text());
                    let style = if sp.style == marker() || sp.style == syntax(true) {
                        sp.style
                    } else {
                        palette::link_text().patch(sp.style)
                    };
                    push(&mut spans, sp.start, sp.len, style);
                    at = sp.start + sp.len;
                }
                push(&mut spans, at, text_end - at, palette::link_text());
                push(&mut spans, text_end, url_end + 1 - text_end, syntax(hide));
                i = url_end + 1;
                continue;
            }
        } else if c == '*' || c == '_' || (c == '~' && chars.get(i + 1) == Some(&'~')) {
            let double = chars.get(i + 1) == Some(&c);
            let dl = if double { 2 } else { 1 };
            // `~` only means anything doubled.
            if c != '~' || double {
                if let Some(close) = find_emphasis(chars, i, c, dl) {
                    let content_start = i + dl;
                    push(&mut spans, i, dl, syntax(hide));
                    let style = if c == '~' {
                        palette::strikethrough()
                    } else if double {
                        bold()
                    } else {
                        italic()
                    };
                    push(&mut spans, content_start, close - content_start, style);
                    push(&mut spans, close, dl, syntax(hide));
                    i = close + dl;
                    continue;
                }
            }
        }
        i += 1;
    }
    spans
}

/// The start of the closing backtick run of exactly `run` backticks at or after
/// `from` (CommonMark: a code span closes on a run of the same length).
fn find_backtick_run(chars: &[char], from: usize, run: usize) -> Option<usize> {
    let mut j = from;
    while j < chars.len() {
        if chars[j] == '`' {
            let r = chars[j..].iter().take_while(|x| **x == '`').count();
            if r == run {
                return Some(j);
            }
            j += r;
        } else {
            j += 1;
        }
    }
    None
}

/// An inline link/image `[text](url)` or `![alt](url)` opening at `at`.
/// Returns `(text_start, text_end, url_end)`: the text is `text_start..text_end`
/// and `url_end` is the index of the closing `)`.
fn link_at(chars: &[char], at: usize) -> Option<(usize, usize, usize)> {
    let open = if chars[at] == '!' { at + 1 } else { at };
    let text_end = matching(chars, open, '[', ']')?;
    if chars.get(text_end + 1) != Some(&'(') {
        return None;
    }
    let url_end = matching(chars, text_end + 1, '(', ')')?;
    Some((open + 1, text_end, url_end))
}

/// The index of the bracket closing the one at `open`, honouring nesting and
/// backslash escapes.
fn matching(chars: &[char], open: usize, l: char, r: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut j = open;
    while j < chars.len() {
        match chars[j] {
            '\\' => j += 1,
            c if c == l => depth += 1,
            c if c == r => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// Find the closing delimiter for an emphasis run of `dl` copies of `c` that
/// opens at `open`. Applies pragmatic flanking rules so arithmetic (`a * b`)
/// and `snake_case` aren't mistaken for emphasis. Returns the closing run's
/// start index.
fn find_emphasis(chars: &[char], open: usize, c: char, dl: usize) -> Option<usize> {
    let n = chars.len();
    // The char just inside the opener must not be whitespace.
    if chars.get(open + dl)?.is_whitespace() {
        return None;
    }
    // `_` only opens at a left word boundary (so `some_var` is safe).
    if c == '_' {
        if let Some(prev) = open.checked_sub(1).and_then(|k| chars.get(k)) {
            if prev.is_alphanumeric() {
                return None;
            }
        }
    }
    let mut i = open + dl;
    while i < n {
        if chars[i] == c {
            let run = chars[i..].iter().take_while(|x| **x == c).count();
            if run >= dl {
                let close = i;
                let inner_ok = !chars[close - 1].is_whitespace() && close > open + dl;
                let right_flank_ok = c != '_'
                    || chars
                        .get(close + dl)
                        .is_none_or(|next| !next.is_alphanumeric());
                if inner_ok && right_flank_ok {
                    return Some(close);
                }
            }
            i += run;
            continue;
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Collapse a line's spans into a compact `(start,len,tag)` form for asserts,
    /// where `tag` is a short mnemonic for the style.
    fn tags(spans: &[MdSpan]) -> Vec<(usize, usize, char)> {
        spans
            .iter()
            .map(|s| {
                let t = if s.style == heading(1) {
                    'h'
                } else if s.style == heading(2) {
                    '2'
                } else if s.style == heading(3) {
                    '3'
                } else if s.style == code() {
                    'c'
                } else if s.style == marker() {
                    'm'
                } else if s.style == syntax(true) {
                    'x'
                } else if s.style == palette::link_text() {
                    'l'
                } else if s.style == palette::strikethrough() {
                    's'
                } else if s.style == bold() {
                    'b'
                } else if s.style == italic() {
                    'i'
                } else if s.style == bullet() {
                    'u'
                } else if s.style == quote() {
                    'q'
                } else {
                    '?'
                };
                (s.start, s.len, t)
            })
            .collect()
    }

    fn line(text: &str) -> Vec<(usize, usize, char)> {
        tags(&Highlighter::new().line(text))
    }

    /// The same line as the editor shows it off the cursor row.
    fn hidden(text: &str) -> Vec<(usize, usize, char)> {
        tags(&Highlighter::new().line_with(text, true))
    }

    #[test]
    fn heading_dims_hashes_and_styles_text_by_level() {
        assert_eq!(line("# Title"), vec![(0, 2, 'm'), (2, 5, 'h')]);
        assert_eq!(line("## Sub"), vec![(0, 3, 'm'), (3, 3, '2')]);
        assert_eq!(line("### Deep"), vec![(0, 4, 'm'), (4, 4, '3')]);
        // A bare `#` is a heading with no text.
        assert_eq!(line("##"), vec![(0, 2, 'm')]);
        // Seven hashes is not a heading.
        assert_eq!(line("####### nope"), vec![]);
    }

    #[test]
    fn bold_and_italic_dim_their_markers() {
        // **bold**: open marker, content, close marker.
        assert_eq!(line("**hi**"), vec![(0, 2, 'm'), (2, 2, 'b'), (4, 2, 'm')]);
        // *italic*
        assert_eq!(line("*hi*"), vec![(0, 1, 'm'), (1, 2, 'i'), (3, 1, 'm')]);
    }

    #[test]
    fn inline_code_spans_the_backticks() {
        assert_eq!(
            line("run `cargo test`"),
            vec![(4, 1, 'm'), (5, 10, 'c'), (15, 1, 'm')]
        );
        // A double-backtick span closes on a double run and may hold a backtick.
        assert_eq!(line("``a`b``"), vec![(0, 2, 'm'), (2, 3, 'c'), (5, 2, 'm')]);
    }

    #[test]
    fn strikethrough_dims_its_tildes() {
        assert_eq!(
            line("~~gone~~"),
            vec![(0, 2, 'm'), (2, 4, 's'), (6, 2, 'm')]
        );
        // A single tilde is just text.
        assert_eq!(line("~a~"), vec![]);
    }

    #[test]
    fn link_text_is_styled_and_brackets_and_url_are_markers() {
        // "[ab](http://x)": `[` marker, text, `](http://x)` markers.
        assert_eq!(
            line("[ab](http://x)"),
            vec![(0, 1, 'm'), (1, 2, 'l'), (3, 11, 'm')]
        );
        // An image gets its `!` in the marker too.
        assert_eq!(
            line("![ab](u)"),
            vec![(0, 2, 'm'), (2, 2, 'l'), (4, 4, 'm')]
        );
        // Brackets without a (url) are plain text.
        assert_eq!(line("[ab] and [cd]"), vec![]);
        // Emphasis inside link text keeps the link colour and adds its weight.
        let spans = Highlighter::new().line("[**b**](u)");
        assert_eq!(spans.len(), 5);
        assert_eq!(spans[1].style, marker());
        assert_eq!(spans[2].style, palette::link_text().patch(bold()));
    }

    #[test]
    fn off_cursor_lines_hide_every_marker_but_keep_widths() {
        assert_eq!(
            hidden("**hi** and `c`"),
            vec![
                (0, 2, 'x'),
                (2, 2, 'b'),
                (4, 2, 'x'),
                (11, 1, 'x'),
                (12, 1, 'c'),
                (13, 1, 'x')
            ]
        );
        assert_eq!(hidden("# T"), vec![(0, 2, 'x'), (2, 1, 'h')]);
        assert_eq!(hidden("> q"), vec![(0, 1, 'x'), (1, 2, 'i')]);
        assert_eq!(
            hidden("[ab](http://x)"),
            vec![(0, 1, 'x'), (1, 2, 'l'), (3, 11, 'x')]
        );
        assert_eq!(hidden("~~s~~"), vec![(0, 2, 'x'), (2, 1, 's'), (3, 2, 'x')]);
        // Hidden or not, the spans cover exactly the same cells.
        for src in ["**hi** `c` [a](u) ~~s~~", "## H", "> *q*", "- _x_"] {
            let cells = |v: Vec<(usize, usize, char)>| -> Vec<(usize, usize)> {
                v.into_iter().map(|(a, b, _)| (a, b)).collect()
            };
            assert_eq!(cells(line(src)), cells(hidden(src)), "{src}");
        }
    }

    #[test]
    fn fence_lines_are_never_hidden() {
        let mut hl = Highlighter::new();
        assert_eq!(tags(&hl.line_with("```rust", true)), vec![(0, 7, 'm')]);
        assert_eq!(tags(&hl.line_with("x", true)), vec![(0, 1, 'c')]);
        assert_eq!(tags(&hl.line_with("```", true)), vec![(0, 3, 'm')]);
    }

    #[test]
    fn snake_case_is_not_emphasis() {
        assert_eq!(line("call some_var_name here"), vec![]);
    }

    #[test]
    fn spaced_asterisks_are_not_emphasis() {
        assert_eq!(line("2 * 3 * 4"), vec![]);
    }

    #[test]
    fn list_marker_is_colored_and_content_scanned() {
        // "- **x**": bullet at 0, then emphasis on the content.
        assert_eq!(
            line("- **x**"),
            vec![(0, 1, 'u'), (2, 2, 'm'), (4, 1, 'b'), (5, 2, 'm')]
        );
        // Ordered marker includes the dot.
        assert_eq!(line("12. item"), vec![(0, 3, 'u')]);
    }

    #[test]
    fn blockquote_dims_marker_and_italicises_text() {
        // Quote text is italic (same style as emphasis, so it tags as `i`).
        assert_eq!(line("> quoted"), vec![(0, 1, 'm'), (1, 7, 'i')]);
    }

    #[test]
    fn fenced_block_colors_until_close() {
        let out = highlight("```rust\nlet x = 1;\n```\nafter");
        assert_eq!(tags(&out[0]), vec![(0, 7, 'm')]); // opening fence
        assert_eq!(tags(&out[1]), vec![(0, 10, 'c')]); // code content
        assert_eq!(tags(&out[2]), vec![(0, 3, 'm')]); // closing fence
        assert_eq!(tags(&out[3]), vec![]); // back to prose
    }

    #[test]
    fn empty_line_has_no_spans() {
        assert_eq!(line(""), vec![]);
    }
}
