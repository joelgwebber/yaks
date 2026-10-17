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

use ratatui::style::Style;

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
/// order via [`Highlighter::line`]; it carries fenced-code-block state so a
/// ```` ```lang ```` block colors every line until its closing fence.
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
    pub fn line(&mut self, text: &str) -> Vec<MdSpan> {
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
                style: marker(),
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
                style: palette::quote_bar(),
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
            spans.extend(inline(&chars, content_start));
            return spans;
        }

        // Plain paragraph line.
        inline(&chars, 0)
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

/// Inline scan of `chars[from..]` for code spans and emphasis, emitting spans
/// for the recognized (and properly closed) runs. Unmatched delimiters are left
/// unstyled. Delimiters are dimmed; the enclosed text carries the emphasis.
fn inline(chars: &[char], from: usize) -> Vec<MdSpan> {
    let n = chars.len();
    let mut spans = Vec::new();
    let mut i = from;
    while i < n {
        let c = chars[i];
        if c == '`' {
            if let Some(close) = find_char(chars, i + 1, '`') {
                spans.push(MdSpan {
                    start: i,
                    len: close - i + 1,
                    style: code(),
                });
                i = close + 1;
                continue;
            }
        } else if c == '*' || c == '_' {
            let double = chars.get(i + 1) == Some(&c);
            let dl = if double { 2 } else { 1 };
            if let Some(close) = find_emphasis(chars, i, c, dl) {
                spans.push(MdSpan {
                    start: i,
                    len: dl,
                    style: marker(),
                });
                let content_start = i + dl;
                if close > content_start {
                    spans.push(MdSpan {
                        start: content_start,
                        len: close - content_start,
                        style: if double { bold() } else { italic() },
                    });
                }
                spans.push(MdSpan {
                    start: close,
                    len: dl,
                    style: marker(),
                });
                i = close + dl;
                continue;
            }
        }
        i += 1;
    }
    spans
}

fn find_char(chars: &[char], from: usize, target: char) -> Option<usize> {
    (from..chars.len()).find(|&i| chars[i] == target)
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
        assert_eq!(line("run `cargo test`"), vec![(4, 12, 'c')]);
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
