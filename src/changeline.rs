//! The change line (yaks-bcb8, design `/doks/design/changelog.md` §1): a note
//! on a yak with `kind: change`, whose text is one line of markdown saying
//! what changed for a reader. `yaks shorn --change` writes it with the shear.
//!
//! Nothing here stores a derived value: a note carries `section:` only when
//! someone chose one (`--section`, or `none` for `--no-change`); [`latest`]
//! derives the rest from the yak's type each time it is read.

// `latest` and `Change` are read by the release changelog (yaks-045c); until
// it lands only the tests do.
#![allow(dead_code)]

use crate::model::Task;
use crate::store;

/// The note's `kind:`.
pub const KIND: &str = "change";
/// The `section:` of a `--no-change` note: internal work, not in the changelog.
pub const NONE: &str = "none";
/// The text of a `--no-change` note.
pub const NO_CHANGE_TEXT: &str = "No user-facing change.";
/// Keep a Changelog's sections, the values of `--section`.
pub const SECTIONS: [&str; 6] = [
    "added",
    "changed",
    "deprecated",
    "removed",
    "fixed",
    "security",
];

/// A change note to write: its text, and the `section:` to stamp (none: the
/// reader derives it from the yak's type).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeNote {
    pub text: String,
    pub section: Option<String>,
}

impl ChangeNote {
    /// A change line, optionally in a chosen section.
    pub fn line(text: &str, section: Option<&str>) -> Self {
        ChangeNote {
            text: text.trim().to_string(),
            section: section.map(str::to_string),
        }
    }

    /// The `--no-change` note.
    pub fn none() -> Self {
        ChangeNote {
            text: NO_CHANGE_TEXT.to_string(),
            section: Some(NONE.to_string()),
        }
    }

    /// The note's metadata, in the order it is written.
    pub fn meta(&self) -> Vec<(&str, &str)> {
        let mut meta = vec![("kind", KIND)];
        if let Some(s) = &self.section {
            meta.push(("section", s));
        }
        meta
    }
}

/// A yak's change line as read: the latest `kind: change` note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub ts: String,
    pub text: String,
    /// The note's `section:` when it has one, else the type's default;
    /// [`NONE`] for a `--no-change` note.
    pub section: String,
}

impl Change {
    /// An internal change, kept out of the changelog.
    pub fn is_none(&self) -> bool {
        self.section == NONE
    }
}

/// The section a yak's type puts a change in: `feature` is added, `bug` is
/// fixed, anything else is changed.
pub fn default_section(task_type: &str) -> &'static str {
    match task_type {
        "feature" => "added",
        "bug" => "fixed",
        _ => "changed",
    }
}

/// The latest change note of `task` (a correction is a newer note, so the
/// latest wins), with its effective section.
pub fn latest(task: &Task) -> Option<Change> {
    let note = store::parse_notes(&task.body)
        .into_iter()
        .rfind(|n| n.kind() == Some(KIND))?;
    let section = match note.get("section").map(str::trim) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => default_section(&task.kind).to_string(),
    };
    Some(Change {
        ts: note.ts,
        text: note.text.trim().to_string(),
        section,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yak(kind: &str, notes: &[(&str, &[(&str, &str)], &str)]) -> Task {
        let mut body = String::new();
        for (ts, meta, text) in notes {
            body = store::append_note_meta(&body, ts, None, meta, text);
        }
        Task {
            id: "y-1".into(),
            title: "t".into(),
            kind: kind.into(),
            priority: 3,
            status: crate::model::Status::Shorn,
            created: None,
            updated: None,
            parent: None,
            tags: vec![],
            depends_on: vec![],
            source: None,
            prs: vec![],
            needs: None,
            verify: None,
            extra: Default::default(),
            body,
        }
    }

    #[test]
    fn no_change_note_is_none() {
        assert_eq!(
            latest(&yak("task", &[("2026-01-01T00:00:00Z", &[], "hi")])),
            None
        );
    }

    #[test]
    fn section_comes_from_the_type_unless_the_note_says() {
        let meta: &[(&str, &str)] = &[("kind", "change")];
        for (ty, want) in [
            ("feature", "added"),
            ("bug", "fixed"),
            ("task", "changed"),
            ("idea", "changed"),
        ] {
            let c = latest(&yak(ty, &[("2026-01-01T00:00:00Z", meta, "line")])).unwrap();
            assert_eq!(c.section, want, "{ty}");
        }
        let meta: &[(&str, &str)] = &[("kind", "change"), ("section", "security")];
        let c = latest(&yak("bug", &[("2026-01-01T00:00:00Z", meta, "line")])).unwrap();
        assert_eq!(c.section, "security");
        assert!(!c.is_none());
    }

    #[test]
    fn the_latest_change_note_wins() {
        let kind: &[(&str, &str)] = &[("kind", "change")];
        let none: &[(&str, &str)] = &[("kind", "change"), ("section", "none")];
        let t = yak(
            "feature",
            &[
                ("2026-01-01T00:00:00Z", kind, "first"),
                ("2026-01-02T00:00:00Z", &[("kind", "checkpoint")], "other"),
                ("2026-01-03T00:00:00Z", kind, "corrected"),
            ],
        );
        let c = latest(&t).unwrap();
        assert_eq!(
            (c.text.as_str(), c.ts.as_str()),
            ("corrected", "2026-01-03T00:00:00Z")
        );
        // A newer no-change note retires an earlier line.
        let t = yak(
            "feature",
            &[
                ("2026-01-01T00:00:00Z", kind, "first"),
                ("2026-01-04T00:00:00Z", none, "x"),
            ],
        );
        assert!(latest(&t).unwrap().is_none());
    }

    #[test]
    fn change_note_meta_writes_section_only_when_given() {
        assert_eq!(ChangeNote::line("a", None).meta(), vec![("kind", "change")]);
        assert_eq!(
            ChangeNote::line(" a ", Some("fixed")).meta(),
            vec![("kind", "change"), ("section", "fixed")]
        );
        assert_eq!(ChangeNote::none().meta()[1], ("section", "none"));
    }
}
