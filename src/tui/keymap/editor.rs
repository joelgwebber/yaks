//! The editors' own keys, for the `F1` popup (yaks-7c6e). edtui owns them, so
//! they are not in the command table: this is a hand table of what each key
//! family's editor does, as [`super::EXTRAS`] is for the rest of the modal
//! surfaces. It lists the keys that matter, not every binding edtui has;
//! emacs parity with vim is yaks-4736.
//!
//! A [`Row`] says how it reads (`dd`) and which single keys it stands for
//! (`d`): a key pressed in the popup that some row names closes the popup and
//! goes to the editor, as a listed key in the list's popup runs.

use edtui::EditorMode;

use super::{Family, Key, PopupGroups};
use ratatui::crossterm::event::KeyEvent;

/// One line of an editor's key table.
pub(crate) struct Row {
    /// How the keys read in the popup.
    pub(super) show: &'static str,
    /// The single keys it stands for (labels as [`Key::label`] writes them).
    pub(super) press: &'static [&'static str],
    what: &'static str,
}

const fn row(show: &'static str, press: &'static [&'static str], what: &'static str) -> Row {
    Row { show, press, what }
}

/// What kind of editor has the focus: its mode (vim only) and whether it is a
/// single line (a title, a prompt, the search box) or several.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EditorAt {
    pub(crate) mode: EditorMode,
    pub(crate) single_line: bool,
}

type Table = &'static [(&'static str, &'static [Row])];

pub(super) const VIM_NORMAL: Table = &[
    (
        "Move",
        &[
            row("h j k l", &["h", "j", "k", "l"], "Left, down, up, right"),
            row("w b e", &["w", "b", "e"], "Word forward, back, end"),
            row("0 $", &["0", "$"], "Start, end of the line"),
            row("gg G", &["g", "G"], "First, last line"),
            row(
                "Ctrl-D Ctrl-U",
                &["Ctrl-D", "Ctrl-U"],
                "Half a page down, up",
            ),
            row("{ }", &["{", "}"], "Paragraph back, forward"),
            row("f t", &["f", "t"], "To a character in line"),
        ],
    ),
    (
        "Change the text",
        &[
            row("i a", &["i", "a"], "Insert before, after"),
            row("o O", &["o", "O"], "Open a line below, above"),
            row("x", &["x"], "Delete the character"),
            row("dd dw", &["d"], "Delete the line, a word"),
            row("cw cc", &["c"], "Change a word, the line"),
            row("yy p", &["y", "p"], "Copy the line, paste"),
            row("u Ctrl-R", &["u", "Ctrl-R"], "Undo, redo"),
            row(".", &["."], "Repeat the last change"),
            row("v V", &["v", "V"], "Select, select lines"),
        ],
    ),
    (
        "Find",
        &[
            row("/", &["/"], "Search the text"),
            row("n N", &["n", "N"], "Next, previous match"),
        ],
    ),
];

pub(super) const VIM_INSERT: Table = &[
    (
        "Move",
        &[
            row(
                "← ↓ ↑ →",
                &["Left", "Down", "Up", "Right"],
                "Move the cursor",
            ),
            row(
                "Ctrl-← Ctrl-→",
                &["Ctrl-←", "Ctrl-→"],
                "A word back, forward",
            ),
            row("Home End", &["Home", "End"], "Start, end of the line"),
            row("PgUp PgDn", &["PgUp", "PgDn"], "A page up, down"),
        ],
    ),
    (
        "Change the text",
        &[
            row("Enter", &["Enter"], "New line (lists continue)"),
            row("Backspace", &["Backspace"], "Delete before"),
            row("Delete", &["Delete"], "Delete after"),
            row("Tab", &["Tab"], "Complete a yak id or dok link, else tab"),
            row("Esc", &["Esc"], "Back to Normal"),
        ],
    ),
];

pub(super) const STANDARD: Table = &[
    (
        "Move",
        &[
            row(
                "← ↓ ↑ →",
                &["Left", "Down", "Up", "Right"],
                "Move the cursor",
            ),
            row("Ctrl-F Ctrl-B", &["Ctrl-F", "Ctrl-B"], "Char forward, back"),
            row("Alt-F Alt-B", &["Alt-F", "Alt-B"], "Word forward, back"),
            row(
                "Ctrl-A Ctrl-E",
                &["Ctrl-A", "Ctrl-E"],
                "Start, end of the line",
            ),
            row("Home End", &["Home", "End"], "Start, end of the line"),
            row("Ctrl-V Alt-V", &["Ctrl-V", "Alt-V"], "Half a page down, up"),
        ],
    ),
    (
        "Change the text",
        &[
            row("Enter", &["Enter"], "New line (lists continue)"),
            row(
                "Backspace Ctrl-D",
                &["Backspace", "Ctrl-D"],
                "Delete before, after",
            ),
            row("Ctrl-K", &["Ctrl-K"], "Delete to end of line"),
            row("Alt-D", &["Alt-D"], "Delete the next word"),
            row("Ctrl-U Ctrl-R", &["Ctrl-U", "Ctrl-R"], "Undo, redo"),
            row("Ctrl-Y", &["Ctrl-Y"], "Paste"),
            row("Tab", &["Tab"], "Complete a yak id or dok link, else tab"),
        ],
    ),
];

/// A single-line field has no up or down, and Enter ends it.
pub(super) const LINE: Table = &[(
    "The field",
    &[
        row("Enter", &["Enter"], "Accept the field"),
        row("Tab", &["Tab"], "Next field, or complete id or dok link"),
    ],
)];

pub(super) const SAVE_VIM_MULTI: Table = &[(
    "Save and leave",
    &[
        row("Ctrl-S", &["Ctrl-S"], "Save"),
        row(":w :q :wq", &[":"], "Save, cancel, save+close"),
        row("Ctrl-C", &["Ctrl-C"], "Cancel (asks if changed)"),
        row("Esc Esc", &["Esc"], "Cancel (quick second Esc)"),
    ],
)];

pub(super) const SAVE_VIM_LINE: Table = &[(
    "Save and leave",
    &[
        row("Enter", &["Enter"], "Accept"),
        row("Ctrl-C", &["Ctrl-C"], "Cancel"),
        row("Esc Esc", &["Esc"], "Cancel (quick second Esc)"),
    ],
)];

pub(super) const SAVE_STANDARD_MULTI: Table = &[(
    "Save and leave",
    &[
        row("Ctrl-S", &["Ctrl-S"], "Save"),
        row("Ctrl-C", &["Ctrl-C"], "Cancel (asks if changed)"),
        row("Esc Esc", &["Esc"], "Cancel (quick second Esc)"),
    ],
)];

pub(super) const SAVE_STANDARD_LINE: Table = &[(
    "Save and leave",
    &[
        row("Enter", &["Enter"], "Accept"),
        row("Esc Ctrl-C", &["Esc", "Ctrl-C"], "Cancel"),
    ],
)];

/// The tables for the editor in `at` under `family`, in the popup's order.
fn tables(family: Family, at: EditorAt) -> Vec<(&'static str, &'static [Row])> {
    let mut out: Vec<(&'static str, &'static [Row])> = Vec::new();
    match family {
        Family::Vim => {
            let normal = at.mode != EditorMode::Insert;
            let base = if normal { VIM_NORMAL } else { VIM_INSERT };
            for t in base {
                // A single line has no vertical motion, paragraphs or lines to open.
                if at.single_line && t.0 == "Move" {
                    out.push((t.0, &t.1[..t.1.len().min(3)]));
                } else {
                    out.push(*t);
                }
            }
            if at.single_line {
                out.extend(LINE);
                out.extend(SAVE_VIM_LINE);
            } else {
                out.extend(SAVE_VIM_MULTI);
            }
        }
        Family::Standard => {
            out.extend(STANDARD);
            if at.single_line {
                out.extend(LINE);
                out.extend(SAVE_STANDARD_LINE);
            } else {
                out.extend(SAVE_STANDARD_MULTI);
            }
        }
    }
    out
}

/// The popup's groups for the editor in `at`: titled (keys, what) entries,
/// for [`super::popup_lines`].
pub(crate) fn editor_groups(family: Family, at: EditorAt) -> PopupGroups {
    tables(family, at)
        .into_iter()
        .map(|(title, rows)| {
            (
                title,
                rows.iter().map(|r| (r.show.to_string(), r.what)).collect(),
            )
        })
        .collect()
}

/// Whether the editor's popup lists `key`: a key it stands for in some row, so
/// that pressing it closes the popup and goes to the editor.
pub(crate) fn editor_lists(family: Family, at: EditorAt, key: &KeyEvent) -> bool {
    tables(family, at).iter().any(|(_, rows)| {
        rows.iter()
            .flat_map(|r| r.press.iter())
            .filter_map(|p| Key::parse(p))
            .any(|k| k.matches(key))
    })
}

/// What the popup's title says about the editor.
pub(crate) fn editor_title(family: Family, at: EditorAt) -> String {
    let line = if at.single_line { "field" } else { "editor" };
    match family {
        Family::Vim => format!(
            "{line}, vim {}",
            match at.mode {
                EditorMode::Insert => "Insert",
                EditorMode::Visual => "Visual",
                EditorMode::Search => "Search",
                EditorMode::Normal => "Normal",
            }
        ),
        Family::Standard => format!("{line}, standard"),
    }
}

/// The chip rows of a form or the filter drawer have no text under the
/// cursor, so the editor's keys do not apply: the row keys do.
pub(crate) fn rows_groups(family: Family) -> PopupGroups {
    let choose = match family {
        Family::Vim => ("← → / h l", "Choose a chip"),
        Family::Standard => ("← →", "Choose a chip"),
    };
    let rows = match family {
        Family::Vim => ("Tab ↓ j / S-Tab ↑ k", "Next, previous row"),
        Family::Standard => ("Tab ↓ / S-Tab ↑", "Next, previous row"),
    };
    vec![
        (
            "Rows",
            vec![
                (rows.0.to_string(), rows.1),
                (choose.0.to_string(), choose.1),
                ("Space".to_string(), "Toggle (filter drawer)"),
                ("Enter".to_string(), "Next row; apply (drawer)"),
            ],
        ),
        (
            "Save and leave",
            vec![
                ("Ctrl-S".to_string(), "Save the form"),
                ("Ctrl-C / Esc Esc".to_string(), "Cancel (asks if changed)"),
            ],
        ),
    ]
}
