//! Core task model for yaks.
//!
//! Status and parentage are frontmatter fields (`status:`, `parent:`); every
//! yak is one file at a stable path, `.yaks/<id>.md` (flat, stable ids).

use crate::fields::Fields;

/// Lifecycle state, the frontmatter `status:` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Hairy,
    Shaving,
    Shorn,
    Dead,
}

impl Status {
    /// The status's name (`hairy`, `shaving`, `shorn`, `dead`): what the
    /// `status:` field holds.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Hairy => "hairy",
            Status::Shaving => "shaving",
            Status::Shorn => "shorn",
            Status::Dead => "dead",
        }
    }

    /// Not finished: hairy or shaving. Shorn and dead are both finished, and
    /// a finished yak never has an open child (the family invariant,
    /// yaks-9b41).
    pub fn is_open(self) -> bool {
        matches!(self, Status::Hairy | Status::Shaving)
    }

    /// Parse a `status:` value (exactly one of the names [`Status::as_str`]
    /// returns). `None` for anything else.
    pub fn parse(s: &str) -> Option<Status> {
        match s {
            "hairy" => Some(Status::Hairy),
            "shaving" => Some(Status::Shaving),
            "shorn" => Some(Status::Shorn),
            "dead" => Some(Status::Dead),
            _ => None,
        }
    }

    /// Compact single-character glyph for listings.
    pub fn glyph(self) -> char {
        match self {
            Status::Hairy => 'H',
            Status::Shaving => 'S',
            Status::Shorn => 'N',
            Status::Dead => 'X',
        }
    }

    /// Status emoji (bison/razor/sheep/skull), matching the TUI list + tab bar.
    pub fn emoji(self) -> &'static str {
        match self {
            Status::Hairy => "\u{1f9ac}",
            Status::Shaving => "\u{1fa92}",
            Status::Shorn => "\u{1f411}",
            Status::Dead => "\u{1f480}",
        }
    }

    /// A dependency counts as "resolved" once it is shorn or dead.
    pub fn is_resolved(self) -> bool {
        matches!(self, Status::Shorn | Status::Dead)
    }
}

/// A single task, parsed from a `.md` file with YAML frontmatter.
///
/// The task's field set. `created`/`updated` are
/// kept as opaque ISO-8601 strings so a read/write round-trip preserves them
/// byte-for-byte (we never reformat timestamps we did not author).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub priority: u8,
    pub status: Status,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub parent: Option<String>,
    pub tags: Vec<String>,
    pub depends_on: Vec<String>,
    pub source: Option<String>,
    /// Pull-request URLs (the active PR or PRs for this yak's code); a list,
    /// unlike `source`, because one yak routinely has several.
    pub prs: Vec<String>,
    /// A soft, external hand-off, not ownership and not a status. `human` (or
    /// any value but `agent`) is a block: the yak is not ready in `next`;
    /// raised by `ask`. `agent` is the answered counterpart: `answer` sets it,
    /// the yak is not blocked, and only `pickup` clears it. See
    /// [`Task::awaiting_human`] / [`Task::awaiting_agent`].
    pub needs: Option<String>,
    /// An optional rerunnable verification command (the scriptable form of a
    /// yak's evidence contract). `yaks verify <id>` runs it and records the
    /// PASS/FAIL as an attributed note. Never auto-run — invoked explicitly.
    pub verify: Option<String>,
    /// Every frontmatter field beyond the built-in ones above (`commit:`,
    /// `due:`, a hand-added key, a newer binary's field), in file order. Each
    /// is re-emitted exactly as read unless set, so a round-trip never drops or
    /// reformats it; values parse on demand (`extra.get("due")`). Keeps
    /// `.yaks/` authoritative across versions (yaks-1f34).
    pub extra: Fields,
    pub body: String,
}

/// The frontmatter keys [`Task`] models itself (each has its own field and
/// command-line flag). Any other key lives in [`Task::extra`]; `update --set`
/// refuses these.
pub const BUILT_IN_KEYS: &[&str] = &[
    "id",
    "title",
    "status",
    "type",
    "priority",
    "created",
    "updated",
    "parent",
    "depends_on",
    "tags",
    // The schema-6 name of `tags`, still read from an unmigrated file.
    "labels",
    "source",
    "prs",
    "needs",
    "verify",
];

/// The `needs` value meaning "awaiting an agent to pick the work up" (a human
/// answered a question, or requested work with `request`).
/// Every other value is an open question (awaiting a human).
pub const NEEDS_AGENT: &str = "agent";

impl Task {
    /// An open question: `needs` is set to anything but `agent`. Blocks `next`.
    pub fn awaiting_human(&self) -> bool {
        self.needs.as_deref().is_some_and(|n| n != NEEDS_AGENT)
    }

    /// An answered question nobody has picked up yet (`needs: agent`). Does not
    /// block `next`.
    pub fn awaiting_agent(&self) -> bool {
        self.needs.as_deref() == Some(NEEDS_AGENT)
    }
}

/// Split raw label input into canonical labels — the one parser every entry
/// point (CLI flags, TUI label fields, filters) shares. A label may not contain
/// a comma or whitespace, so each input string is split on both, trimmed,
/// emptied pieces dropped, and duplicates removed (first occurrence wins).
/// `foo, bar`, `foo bar`, and `foo,bar` all yield `[foo, bar]`. Idempotent.
pub fn normalize_tags<I, S>(raw: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut out: Vec<String> = Vec::new();
    for s in raw {
        for piece in s.as_ref().split(|c: char| c == ',' || c.is_whitespace()) {
            if !piece.is_empty() && !out.iter().any(|l| l == piece) {
                out.push(piece.to_string());
            }
        }
    }
    out
}

/// Whether `label` is already canonical: non-empty, no comma, no whitespace.
/// Legacy on-disk labels that fail this (e.g. `ui,docs`) are flagged by
/// `doctor` and re-split the next time the yak's labels are edited.
pub fn is_canonical_tag(label: &str) -> bool {
    !label.is_empty() && !label.contains(|c: char| c == ',' || c.is_whitespace())
}

/// Diff an edited label list against a yak's current labels, as the
/// (add, remove) pair a `TaskEdit` carries. `new` must already be normalized.
/// Compares against the *normalized* current labels, so a legacy `ui,docs`
/// label reads as `ui` + `docs`. Returns `None` when nothing changes; when the
/// only difference is legacy spelling, returns `new` as the add set so the
/// edit re-writes the yak's labels canonically (see `Farm::update`).
pub fn tag_edit(cur: &[String], new: &[String]) -> Option<(Vec<String>, Vec<String>)> {
    if cur == new {
        return None;
    }
    let cur_n = normalize_tags(cur);
    let add: Vec<String> = new.iter().filter(|l| !cur_n.contains(l)).cloned().collect();
    let remove: Vec<String> = cur_n.iter().filter(|l| !new.contains(l)).cloned().collect();
    if add.is_empty() && remove.is_empty() {
        if cur_n == new {
            // Only legacy spelling differs: re-assert the labels to canonicalize.
            return Some((new.to_vec(), Vec::new()));
        }
        // Same set, different order: not worth a rewrite.
        return None;
    }
    Some((add, remove))
}

#[cfg(test)]
mod label_tests {
    use super::*;

    fn v(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn normalize_splits_on_commas_and_whitespace() {
        for raw in [
            "foo, bar",
            "foo bar",
            "foo,bar",
            " foo ,, bar ,",
            "foo\tbar\n",
        ] {
            assert_eq!(normalize_tags([raw]), v(&["foo", "bar"]), "input {raw:?}");
        }
    }

    #[test]
    fn normalize_flattens_many_args_and_dedupes_in_order() {
        assert_eq!(normalize_tags(["b,a", "a", "c b", ""]), v(&["b", "a", "c"]));
        assert!(normalize_tags(["", " , "]).is_empty());
        // Idempotent.
        let once = normalize_tags(["x, y z"]);
        assert_eq!(normalize_tags(&once), once);
    }

    #[test]
    fn canonical_label_rejects_commas_and_spaces() {
        assert!(is_canonical_tag("ui"));
        assert!(is_canonical_tag("needs-review"));
        assert!(!is_canonical_tag("ui,docs"));
        assert!(!is_canonical_tag("bug ui"));
        assert!(!is_canonical_tag(""));
    }

    #[test]
    fn label_edit_diffs_against_normalized_current() {
        assert_eq!(tag_edit(&v(&["a", "b"]), &v(&["a", "b"])), None);
        assert_eq!(tag_edit(&v(&["a", "b"]), &v(&["b", "a"])), None);
        assert_eq!(
            tag_edit(&v(&["a", "b"]), &v(&["a", "c"])),
            Some((v(&["c"]), v(&["b"])))
        );
        // Legacy `ui,docs` with an unchanged set: re-assert to canonicalize.
        assert_eq!(
            tag_edit(&v(&["ui,docs"]), &v(&["ui", "docs"])),
            Some((v(&["ui", "docs"]), vec![]))
        );
        // Legacy label, dropping one half: remove the normalized piece.
        assert_eq!(
            tag_edit(&v(&["ui,docs"]), &v(&["ui"])),
            Some((vec![], v(&["docs"])))
        );
    }
}
