//! The one style table for markdown, shared by the detail (read) pane and the
//! embedded editor (yaks-bc53).
//!
//! Plain functions returning [`Style`] (and, where a style comes with a glyph,
//! the glyph), so every consumer asks for the same thing by name and a palette
//! change is one edit. The detail pane's renderer (`mdrender`) and the editor's
//! highlighter (`markdown`) both read from here; nothing else defines a markdown
//! colour.
//!
//! Design rules:
//!
//! - **16 ANSI colours + modifiers only** (no `Rgb`, no `Indexed` greys), so the
//!   user's terminal theme decides the actual hues and both dark and light
//!   themes stay legible. Because of that there is deliberately **no code-block
//!   background**: any fixed grey that is subtle on a dark terminal is a black
//!   slab on a light one, and `Reset`-relative tints do not exist. Code is set
//!   apart by colour and the renderer's gutter instead.
//! - **Body headings never look like pane chrome.** The pane's own labels
//!   (`Children:`, `Depends on:`) are [`pane_label`] (cyan, bold). Headings use
//!   magenta / blue / plain-bold-italic, which no label uses, and differ from one
//!   another by colour *and* weight, so the levels still read on a monochrome
//!   terminal (underline vs bold vs italic).
//! - **Quiet by default:** markers, rules, URLs and quote bars are dark grey;
//!   quoted *text* carries italic only, so a quote that contains a list or a code
//!   block keeps their colours instead of turning into an unending sea of grey.
//! - **Semantic hues:** yellow = pending/caution, green = done/code/tip,
//!   blue = link/note, magenta = heading/important, red = danger.

// Consumed by the markdown renderer (yaks-649d) as it lands; until then not every
// constructor has a caller.
#![allow(dead_code)]

use ratatui::style::{Color, Modifier, Style};

// -- chrome ---------------------------------------------------------------

/// The pane's own field/section labels (`Children:`, `Depends on:`). Owned by
/// `render_dline`; listed here so headings can be checked against it.
pub fn pane_label() -> Style {
    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

/// Dimmed syntax: `#`, `**`, backticks, fences, link brackets (editor), and the
/// pane's quiet text.
pub fn marker() -> Style {
    Style::new().fg(Color::DarkGray)
}

// -- headings -------------------------------------------------------------

/// ATX/setext heading text for `level` 1..=6 (clamped).
///
/// - H1 magenta, bold, underlined
/// - H2 blue, bold
/// - H3 default colour, bold + italic
/// - H4..H6 dark grey, bold
pub fn heading(level: u8) -> Style {
    let s = Style::new().add_modifier(Modifier::BOLD);
    match level {
        0 | 1 => s.fg(Color::Magenta).add_modifier(Modifier::UNDERLINED),
        2 => s.fg(Color::Blue),
        3 => s.add_modifier(Modifier::ITALIC),
        _ => s.fg(Color::DarkGray),
    }
}

// -- inline ---------------------------------------------------------------

pub fn emphasis() -> Style {
    Style::new().add_modifier(Modifier::ITALIC)
}

pub fn strong() -> Style {
    Style::new().add_modifier(Modifier::BOLD)
}

pub fn strikethrough() -> Style {
    Style::new()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::CROSSED_OUT)
}

pub fn inline_code() -> Style {
    Style::new().fg(Color::Green)
}

/// A link's visible text.
pub fn link_text() -> Style {
    Style::new()
        .fg(Color::Blue)
        .add_modifier(Modifier::UNDERLINED)
}

/// A link's URL when shown beside its text (dimmed; the text is the link).
pub fn link_url() -> Style {
    marker()
}

// -- blocks ---------------------------------------------------------------

/// Fenced / indented code. Foreground only (see the module docs on why no
/// background); syntax colouring (yaks-3020) layers on top of this base.
pub fn code_block() -> Style {
    inline_code()
}

/// The fence line / language label of a code block.
pub fn code_fence() -> Style {
    marker()
}

/// Block quote text: italic only, so nested lists/code keep their own colours.
pub fn quote_text() -> Style {
    emphasis()
}

/// The `▎` bar drawn down the left of each quote row (one per nesting level).
pub fn quote_bar() -> Style {
    marker()
}

pub const QUOTE_BAR: &str = "▎";

/// A horizontal rule (`---`), drawn as a run of [`RULE_CHAR`].
pub fn rule() -> Style {
    marker()
}

pub const RULE_CHAR: char = '─';

/// A list bullet's style (the glyph is [`bullet_glyph`]; ordered markers
/// `1.` use the same style).
pub fn bullet() -> Style {
    Style::new().fg(Color::Cyan)
}

/// Bullet glyph by nesting depth (0 = top level), cycling every three levels.
pub fn bullet_glyph(depth: usize) -> &'static str {
    ["•", "◦", "▪"][depth % 3]
}

/// A task-list box: green when checked, yellow while open.
pub fn task(checked: bool) -> Style {
    Style::new().fg(if checked { Color::Green } else { Color::Yellow })
}

pub fn task_glyph(checked: bool) -> &'static str {
    if checked { "☑" } else { "☐" }
}

// -- alerts ---------------------------------------------------------------

/// A GitHub-style alert (`> [!NOTE]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl AlertKind {
    pub const ALL: [AlertKind; 5] = [
        AlertKind::Note,
        AlertKind::Tip,
        AlertKind::Important,
        AlertKind::Warning,
        AlertKind::Caution,
    ];

    /// The title shown on the alert's first row.
    pub fn label(self) -> &'static str {
        match self {
            AlertKind::Note => "NOTE",
            AlertKind::Tip => "TIP",
            AlertKind::Important => "IMPORTANT",
            AlertKind::Warning => "WARNING",
            AlertKind::Caution => "CAUTION",
        }
    }

    /// A single-cell, non-emoji glyph before the title (emoji are two cells wide
    /// and render inconsistently across terminals).
    pub fn glyph(self) -> &'static str {
        match self {
            AlertKind::Note => "●",
            AlertKind::Tip => "✦",
            AlertKind::Important => "◆",
            AlertKind::Warning => "▲",
            AlertKind::Caution => "✗",
        }
    }
}

/// The alert's bar and title colour (the title adds bold; body text stays
/// default so the colour marks the kind without drowning the content).
pub fn alert(kind: AlertKind) -> Style {
    let fg = match kind {
        AlertKind::Note => Color::Blue,
        AlertKind::Tip => Color::Green,
        AlertKind::Important => Color::Magenta,
        AlertKind::Warning => Color::Yellow,
        AlertKind::Caution => Color::Red,
    };
    Style::new().fg(fg)
}

pub fn alert_title(kind: AlertKind) -> Style {
    alert(kind).add_modifier(Modifier::BOLD)
}

// -- comments -------------------------------------------------------------

/// A comment header's label (`▸ comment · 2026-09-06 · joel`): cyan when it is
/// the focused block (edit form), dark grey otherwise. The trailing `─` run is
/// always [`rule`]. Same design as the edit form's `render_block_separator`.
pub fn comment_header(focused: bool) -> Style {
    Style::new().fg(if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_are_distinct_levels_and_not_pane_labels() {
        let hs: Vec<Style> = (1..=4).map(heading).collect();
        for (i, a) in hs.iter().enumerate() {
            assert_ne!(*a, pane_label(), "H{} reads like a pane label", i + 1);
            for b in &hs[i + 1..] {
                assert_ne!(a, b);
            }
        }
        // Out-of-range levels clamp rather than panic.
        assert_eq!(heading(0), heading(1));
        assert_eq!(heading(9), heading(4));
    }

    #[test]
    fn alert_kinds_have_distinct_colours() {
        let fgs: Vec<_> = AlertKind::ALL.iter().map(|k| alert(*k).fg).collect();
        for (i, a) in fgs.iter().enumerate() {
            assert!(!fgs[i + 1..].contains(a));
        }
    }

    #[test]
    fn only_the_16_ansi_colours_are_used() {
        // Rgb/Indexed would pin a hue that is unreadable on one of dark/light.
        let all = [
            marker(),
            heading(1),
            heading(2),
            heading(3),
            heading(4),
            strikethrough(),
            inline_code(),
            link_text(),
            bullet(),
            task(true),
            task(false),
            comment_header(true),
            comment_header(false),
        ]
        .into_iter()
        .chain(AlertKind::ALL.iter().map(|k| alert(*k)));
        for s in all {
            for c in [s.fg, s.bg].into_iter().flatten() {
                assert!(!matches!(c, Color::Rgb(..) | Color::Indexed(_)), "{c:?}");
            }
            assert_eq!(s.bg, None, "no background in the palette");
        }
    }

    #[test]
    fn bullets_cycle_by_depth() {
        assert_eq!(bullet_glyph(0), "•");
        assert_eq!(bullet_glyph(1), "◦");
        assert_eq!(bullet_glyph(2), "▪");
        assert_eq!(bullet_glyph(3), "•");
    }
}
