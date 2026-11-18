//! A yak's code commits, read from its notes (yaks-3515, yaks-ccb9): the one
//! collection `yaks commits` and the TUI's `Code:` rows share. Derived on
//! read, never stored: a yak's commits are the `commit:` fields of its notes
//! (`yaks land`, `update --commit`) and the commit links in their text
//! (`commit:<sha>`, or the code repo's host commit URL read back through
//! [`crate::locate::Space::read_url`]). Branches are the checkpoint notes'
//! `branch:` fields, grouped by [`branches`].

use crate::links::{self, Link};
use crate::store::NoteEntry;

/// One commit a yak's notes name, once (the first note that names it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// The sha as written (7 to 40 hex digits).
    pub commit: String,
    /// The note's `kind:` for a `commit:` field (`landed`, `checkpoint`, ...;
    /// `commit` when it has none); `link` for a commit linked in the text.
    pub kind: String,
    /// The worker's sha a landing came from, when the note has one.
    pub source: Option<String>,
    pub branch: Option<String>,
    /// The note's timestamp and actor.
    pub ts: String,
    pub actor: Option<String>,
}

/// Whether two written shas name one commit (one is a prefix of the other).
fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.to_ascii_lowercase(), b.to_ascii_lowercase());
    a.starts_with(&b) || b.starts_with(&a)
}

/// The commits `notes` name, in note order. `read` turns a parsed link into
/// what it names (pass `Space::read_url` so host commit URLs count, or the
/// identity when there is no code checkout).
pub fn collect(notes: &[NoteEntry], read: &dyn Fn(Link) -> Link) -> Vec<Recorded> {
    let mut out: Vec<Recorded> = Vec::new();
    for n in notes {
        let mut push = |commit: String, kind: String| {
            if !out.iter().any(|r| same(&r.commit, &commit)) {
                out.push(Recorded {
                    commit,
                    kind,
                    source: n.get("source").map(str::to_string),
                    branch: n.get("branch").map(str::to_string),
                    ts: n.ts.clone(),
                    actor: n.actor.clone(),
                });
            }
        };
        if let Some(c) = n.get("commit").map(str::trim).filter(|c| !c.is_empty()) {
            push(c.to_string(), n.kind().unwrap_or("commit").to_string());
        }
        for f in links::extract(&n.text) {
            if let Link::Commit { sha } = read(links::parse(&f.dest, links::From::Yak, |_| false)) {
                push(sha, "link".to_string());
            }
        }
    }
    out
}

/// The branches the notes record, as `(remote, branch)` once each in note
/// order: a note with a `branch:` (its `remote:`, else `origin`), except the
/// `landed` notes `yaks land` writes, whose branch is where the work landed.
pub fn branches(notes: &[NoteEntry]) -> Vec<((String, String), Vec<&NoteEntry>)> {
    let mut groups: Vec<((String, String), Vec<&NoteEntry>)> = Vec::new();
    for n in notes {
        if n.kind() == Some("landed") {
            continue;
        }
        let Some(branch) = n.get("branch").map(str::trim).filter(|b| !b.is_empty()) else {
            continue;
        };
        let remote = n
            .get("remote")
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .unwrap_or("origin");
        let key = (remote.to_string(), branch.to_string());
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => v.push(n),
            None => groups.push((key, vec![n])),
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(ts: &str, meta: &[(&str, &str)], text: &str) -> NoteEntry {
        NoteEntry {
            ts: ts.into(),
            actor: Some("w".into()),
            meta: meta
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            text: text.into(),
        }
    }

    #[test]
    fn fields_and_links_once_each_and_branches_but_not_landed_ones() {
        let full = "3f2a9c1d00aa3f2a9c1d00aa3f2a9c1d00aa3f2a";
        let notes = vec![
            n(
                "t1",
                &[
                    ("kind", "checkpoint"),
                    ("commit", "abc1234"),
                    ("branch", "wip/x"),
                ],
                "pushed",
            ),
            n(
                "t2",
                &[
                    ("kind", "landed"),
                    ("commit", full),
                    ("source", "abc1234"),
                    ("branch", "main"),
                ],
                &format!("landed: [3f2a9c1](commit:{full})"),
            ),
            n(
                "t3",
                &[],
                "see [the fix](commit:ABC1234) and [x](commit:9999999)",
            ),
            n("t4", &[("branch", "wip/x"), ("remote", "fork")], "again"),
        ];
        let got = collect(&notes, &|l| l);
        let short: Vec<(&str, &str)> = got
            .iter()
            .map(|r| (r.commit.as_str(), r.kind.as_str()))
            .collect();
        assert_eq!(
            short,
            [
                ("abc1234", "checkpoint"),
                (full, "landed"),
                ("9999999", "link")
            ]
        );
        assert_eq!(got[1].source.as_deref(), Some("abc1234"));
        let b: Vec<(String, String)> = branches(&notes).into_iter().map(|(k, _)| k).collect();
        assert_eq!(
            b,
            [
                ("origin".to_string(), "wip/x".to_string()),
                ("fork".to_string(), "wip/x".to_string())
            ]
        );
    }

    #[test]
    fn a_reader_turns_host_urls_into_commits() {
        let notes = vec![n("t1", &[], "[c](https://h.test/o/r/commit/abc1234)")];
        assert!(collect(&notes, &|l| l).is_empty());
        let read = |l: Link| match l {
            Link::Url(u) if u.contains("/commit/") => Link::Commit {
                sha: "abc1234".into(),
            },
            other => other,
        };
        assert_eq!(collect(&notes, &read)[0].commit, "abc1234");
    }
}
