//! Markdown style checks over a yak's body, shared by the CLI filters (no TUI
//! dependency).
//!
//! The one check so far is **run-on lines** (yaks-ef3b): adjacent source lines
//! that CommonMark joins into one paragraph, so what the author wrote as
//! separate lines renders as a single run of prose. The detector parses with
//! the same options as the TUI's detail renderer ([`options`], which
//! `tui::mdrender` uses too), so "flagged" means "renders joined in yaks".

use pulldown_cmark::{Event, Options, Parser};

use crate::store;

/// The `pulldown-cmark` options every yaks markdown render uses (GFM tables,
/// footnotes, strikethrough, task lists, alerts). The single source of truth
/// for the TUI renderer and the lint, so the two agree on what joins.
pub fn options() -> Options {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_GFM);
    opts
}

/// True if one markdown document has a soft line break: two source lines that
/// render joined. `pulldown-cmark` emits `SoftBreak` only for a newline inside
/// inline content (a paragraph, a tight list item, a multi-line heading), never
/// in code blocks, tables or HTML, and a hard break (two trailing spaces or a
/// trailing backslash) is a `HardBreak`, so any `SoftBreak` is a run-on.
fn doc_has_run_on(text: &str) -> bool {
    Parser::new_ext(text, options()).any(|e| matches!(e, Event::SoftBreak))
}

/// True if a yak body's description or any of its notes has run-on lines.
/// Each block is checked on its own (split by the note delimiters), so the
/// `---` / `▸ <timestamp>` note headers never count.
pub fn has_run_on(body: &str) -> bool {
    doc_has_run_on(&store::description(body))
        || store::parse_notes(body)
            .iter()
            .any(|n| doc_has_run_on(&n.text))
}

/// True if a yak body's description holds a `#### <RFC 3339 timestamp>`
/// heading outside a code fence: in the new note shape that line starts a note,
/// so the description cannot keep it (the one new ambiguity of the format, see
/// `/doks/design/note-format.md`). `convert_notes` leaves such a body in the
/// old shape; the fix is to rewrite the heading (or fence it).
#[allow(dead_code)] // for `doctor`, which check.rs owns (follow-up)
pub fn note_heading_in_description(body: &str) -> bool {
    let lines: Vec<&str> = body.lines().collect();
    let (shape, starts) = store::scan_notes(&lines);
    let end = match shape {
        // New shape: the first heading is the first note, so the description
        // before it holds none by construction.
        Some(store::NoteShape::New) => return false,
        _ => starts.first().copied().unwrap_or(lines.len()),
    };
    let fenced = store::fenced_lines(&lines);
    (0..end).any(|i| !fenced[i] && store::heading_ts(lines[i]).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A body with `desc` as its description and each of `notes` appended the
    /// way `yaks update --note` writes them.
    fn body(desc: &str, notes: &[&str]) -> String {
        let mut b = desc.to_string();
        for n in notes {
            b = store::append_note(&b, "2026-10-26T00:00:00Z", Some("t"), n);
        }
        b
    }

    #[test]
    fn flags_lines_that_join() {
        // A two-line paragraph.
        assert!(has_run_on(&body("first line\nsecond line", &[])));
        // A run-on paragraph in a note, the description clean.
        assert!(has_run_on(&body(
            "Clean description.",
            &["fine note", "one\ntwo"]
        )));
        // `a\nb` inside a list item (lazy continuation and indented).
        assert!(has_run_on(&body("- a\nb", &[])));
        assert!(has_run_on(&body("- a\n  b\n- c", &[])));
        // In a block quote.
        assert!(has_run_on(&body("> a\n> b", &[])));
    }

    #[test]
    fn leaves_well_formed_markdown_alone() {
        let clean = [
            // Blank-line paragraphs.
            "one\n\ntwo\n\nthree",
            // Lists, bullets and ordered, and a heading above one.
            "## Facts\n\n- a\n- b\n- c\n\n1. x\n2. y",
            // Fenced code with many lines.
            "Output:\n\n```text\nline 1\nline 2\nline 3\n```",
            // Hard breaks: two trailing spaces, and a trailing backslash.
            "line one  \nline two",
            "line one\\\nline two",
            // A table.
            "| a | b |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |",
            // Empty.
            "",
        ];
        for c in clean {
            assert!(!has_run_on(&body(c, &[])), "description flagged: {c:?}");
            assert!(!has_run_on(&body("Desc.", &[c])), "note flagged: {c:?}");
        }
    }

    #[test]
    fn note_headers_do_not_count() {
        // One-line description and notes: the `---` / `▸` header lines must
        // not join with the text around them.
        let b = body("Desc.", &["note one", "note two"]);
        assert!(b.contains("\n---\n"), "{b}");
        assert!(!has_run_on(&b), "{b}");
    }

    #[test]
    fn note_headers_do_not_count_in_the_new_shape_either() {
        let mut b = "Desc.".to_string();
        for n in ["note one", "note two"] {
            b = store::append_note_meta_in(
                store::NoteShape::New,
                &b,
                "2026-10-26T00:00:00Z",
                Some("t"),
                &[("kind", "move")],
                n,
            );
        }
        assert!(b.contains("\n#### "), "{b}");
        assert!(!has_run_on(&b), "{b}");
        // The run-on in a note is still found, and the italic line is not text.
        let b = store::append_note_meta_in(
            store::NoteShape::New,
            &b,
            "2026-10-27T00:00:00Z",
            Some("t"),
            &[],
            "one\ntwo",
        );
        assert!(has_run_on(&b), "{b}");
    }

    #[test]
    fn a_note_heading_in_an_old_description_is_flagged() {
        let old = body("Desc.", &["n"]);
        assert!(!note_heading_in_description(&old));
        let bad = format!(
            "Intro\n\n#### 2026-01-01T00:00:00Z\n\nmore{}",
            &old["Desc.".len()..]
        );
        assert!(note_heading_in_description(&bad));
        let fenced = "Intro\n\n```\n#### 2026-01-01T00:00:00Z\n```\n".to_string();
        assert!(!note_heading_in_description(&fenced));
        assert!(!note_heading_in_description("#### Not a timestamp"));
    }
}
