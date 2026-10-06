//! `yaks status`: what the farm has changed that git does not have yet, one
//! line per yak.
//!
//! Read-only. The classification is the one `yaks commit` builds its message
//! from ([`crate::changes::classify`]); this adds, per yak, what kind of edit
//! an in-place change was (notes appended, or a field or the description
//! changed) by comparing the file with its version at HEAD.

use crate::changes::{self, Kind, YakChange, classify};
use crate::commit::generate_message;
use crate::store;
use anyhow::Result;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// One yak's line: its id, what changed, and whether all of it is staged.
#[derive(Debug)]
pub struct YakLine {
    pub id: String,
    pub changes: Vec<String>,
    pub staged: bool,
}

/// A farm file that belongs to no yak (`config`, or its path).
#[derive(Debug)]
pub struct OtherLine {
    pub label: String,
    pub staged: bool,
}

#[derive(Debug)]
pub struct Report {
    /// Nothing under the farm is tracked by git.
    pub private: bool,
    /// A merge or cherry-pick is in progress: `(the ref git sets, what it is)`.
    pub pending: Option<(&'static str, &'static str)>,
    pub yaks: Vec<YakLine>,
    pub other: Vec<OtherLine>,
    /// Changed files in the repository outside the farm (never listed).
    pub outside: usize,
    /// The message `yaks commit` would use, when there is anything to commit.
    pub message: Option<String>,
}

impl Report {
    pub fn clean(&self) -> bool {
        self.yaks.is_empty() && self.other.is_empty()
    }
}

/// Read the farm at `root` (the `.yaks/` directory) and describe its pending
/// changes. Writes nothing.
pub fn collect(root: &Path) -> Result<Report> {
    let Some(changes) = changes::read(root)? else {
        return Ok(Report {
            private: true,
            pending: None,
            yaks: Vec::new(),
            other: Vec::new(),
            outside: 0,
            message: None,
        });
    };
    let top = PathBuf::from(
        String::from_utf8_lossy(&changes::check(
            changes::git(root, &["rev-parse", "--show-toplevel"])?,
            "git rev-parse",
        )?)
        .trim(),
    );
    let classified = classify(&changes);
    let yaks = classified
        .yaks
        .iter()
        .map(|(id, change)| YakLine {
            id: id.clone(),
            changes: describe_yak(root, &top, change),
            staged: change.staged,
        })
        .collect();
    let other = classified
        .other
        .iter()
        .map(|c| OtherLine {
            label: if c.path.rsplit('/').next() == Some("config.yaml") {
                "config".to_string()
            } else {
                c.path.clone()
            },
            staged: c.staged,
        })
        .collect();
    Ok(Report {
        private: false,
        pending: changes::pending_operation(root)?,
        yaks,
        other,
        outside: changes::outside(root)?.len(),
        message: (!changes.is_empty()).then(|| generate_message(&changes)),
    })
}

/// The change tokens for one yak's line, in the order `created`, `moved`,
/// `notes`, `edited`, `removed`, `artifacts`.
fn describe_yak(root: &Path, top: &Path, change: &YakChange) -> Vec<String> {
    let mut out = Vec::new();
    match &change.kind {
        Some(Kind::Created) => out.push("created".to_string()),
        Some(Kind::Removed) => out.push("removed".to_string()),
        Some(kind @ (Kind::Moved { .. } | Kind::Updated)) => {
            if let Kind::Moved { from, to } = kind {
                out.push(format!("moved {from} -> {to}"));
            }
            // Compare the file with HEAD's; if either side cannot be read the
            // change is reported as an edit rather than guessed at.
            let old = change.old_path.as_deref().and_then(|p| head_text(root, p));
            let new = change
                .path
                .as_deref()
                .and_then(|p| std::fs::read_to_string(top.join(p)).ok());
            let (notes, edited) = match (old, new) {
                (Some(old), Some(new)) => describe(&old, &new),
                _ => (0, true),
            };
            if notes > 0 {
                out.push(format!("notes +{notes}"));
            }
            // An in-place change that is neither a note nor a field edit (only
            // `updated:` moved) is still an edit; a move needs no further word.
            if edited || (notes == 0 && matches!(kind, Kind::Updated)) {
                out.push("edited".to_string());
            }
        }
        None => {}
    }
    if change.artifacts {
        out.push("artifacts".to_string());
    }
    out
}

fn head_text(root: &Path, path: &str) -> Option<String> {
    let out = changes::git(root, &["show", &format!("HEAD:{path}")]).ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Compare a yak file's text at HEAD with now: how many notes were added
/// (`moved: a -> b` transition entries do not count), and whether anything
/// besides appended notes and the `updated:` stamp changed.
fn describe(old: &str, new: &str) -> (usize, bool) {
    let (old, new) = (without_updated(old), without_updated(new));
    let notes = |text: &str| {
        store::parse_notes(text)
            .iter()
            .filter(|n| !store::is_transition_text(&n.text))
            .count()
    };
    let added = notes(&new).saturating_sub(notes(&old));
    let edited = match new.strip_prefix(old.trim_end()) {
        // Only appended notes: whatever precedes the first note block is blank.
        Some(rest) => {
            let rest = format!("\n{rest}");
            let head = rest
                .find("\n---\n\u{25b8} ")
                .map_or(&rest[..], |i| &rest[..i]);
            !head.trim().is_empty()
        }
        None => true,
    };
    (added, edited)
}

/// `text` without its `updated:` line, which every mutation re-stamps.
fn without_updated(text: &str) -> String {
    let mut dropped = false;
    text.split_inclusive('\n')
        .filter(|l| {
            let hit = !dropped && l.starts_with("updated:");
            dropped |= hit;
            !hit
        })
        .collect()
}

/// The text output.
pub fn render(r: &Report) -> String {
    if r.private {
        return "private farm: not tracked by git, nothing to commit\n".to_string();
    }
    let mut out = String::new();
    if r.clean() {
        out.push_str("farm clean: nothing to commit\n");
    } else {
        out.push_str("farm changes git does not have yet (* = staged):\n");
        let width = r
            .yaks
            .iter()
            .map(|y| y.id.len())
            .chain(r.other.iter().map(|o| o.label.len()))
            .max()
            .unwrap_or(0);
        let mark = |staged: bool| if staged { '*' } else { ' ' };
        for y in &r.yaks {
            out.push_str(&format!(
                "{} {:width$}  {}\n",
                mark(y.staged),
                y.id,
                y.changes.join(", ")
            ));
        }
        for o in &r.other {
            out.push_str(&format!("{} {}\n", mark(o.staged), o.label));
        }
    }
    if r.outside > 0 {
        let s = if r.outside == 1 { "" } else { "s" };
        out.push_str(&format!(
            "{} other changed file{s} outside the farm (`yaks commit` leaves {} alone)\n",
            r.outside,
            if r.outside == 1 { "it" } else { "them" }
        ));
    }
    if let Some(message) = &r.message {
        match r.pending {
            Some((head, what)) => out.push_str(&format!(
                "a {what} is in progress ({head} is set): `yaks commit` will refuse, because git \
                 forbids a partial commit during one. Finish it with a plain `git commit` (these \
                 farm changes are part of it) or `git {what} --abort`.\n"
            )),
            None => out.push_str(&format!(
                "run `yaks commit` to commit these as: {message}\n"
            )),
        }
    }
    out
}

/// The `--json` output.
pub fn to_json(r: &Report) -> Value {
    json!({
        "private": r.private,
        "clean": r.clean(),
        "merge_in_progress": r.pending.is_some(),
        "yaks": r.yaks.iter().map(|y| json!({
            "id": y.id,
            "changes": y.changes,
            "staged": y.staged,
        })).collect::<Vec<_>>(),
        "other": r.other.iter().map(|o| json!({
            "path": o.label,
            "staged": o.staged,
        })).collect::<Vec<_>>(),
        "outside_farm": r.outside,
        "message": r.message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn git(repo: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {out:?}");
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    fn put(repo: &Path, rel: &str, text: &str) {
        let p = repo.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn mv(repo: &Path, from: &str, to: &str) {
        std::fs::create_dir_all(repo.join(to).parent().unwrap()).unwrap();
        std::fs::rename(repo.join(from), repo.join(to)).unwrap();
    }

    const ONE: &str = "---\nid: yak-0001\nupdated: '2026-01-01T00:00:00Z'\n---\n\nbody\n";

    /// A committed team farm: two yaks, an artifact, config and a source file.
    fn team_repo() -> PathBuf {
        let repo = std::env::temp_dir().join(format!(
            "yaks-status-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        put(&repo, ".yaks/config.yaml", "herd: yak\n");
        put(&repo, ".yaks/hairy/yak-0001.md", ONE);
        put(
            &repo,
            ".yaks/shaving/yak-0002.md",
            &ONE.replace("0001", "0002"),
        );
        put(&repo, ".yaks/artifacts/yak-0002/a.txt", "a\n");
        put(&repo, "src/code.rs", "fn main() {}\n");
        git(&repo, &["init", "-q"]);
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "init"]);
        repo
    }

    fn farm(repo: &Path) -> PathBuf {
        repo.join(".yaks")
    }

    fn report(repo: &Path) -> Report {
        collect(&farm(repo)).unwrap()
    }

    /// `(id, changes)` for every yak line.
    fn lines(r: &Report) -> Vec<(String, String)> {
        r.yaks
            .iter()
            .map(|y| (y.id.clone(), y.changes.join(", ")))
            .collect()
    }

    fn note(text: &str) -> String {
        format!("\n---\n\u{25b8} 2026-02-02T00:00:00Z [t]\n{text}\n")
    }

    fn append(repo: &Path, rel: &str, extra: &str) {
        let p = repo.join(rel);
        let text = std::fs::read_to_string(&p).unwrap();
        std::fs::write(p, text + extra).unwrap();
    }

    #[test]
    fn clean_farm() {
        let repo = team_repo();
        let r = report(&repo);
        assert!(r.clean() && !r.private && r.message.is_none());
        assert_eq!(render(&r), "farm clean: nothing to commit\n");
    }

    #[test]
    fn created_yak() {
        let repo = team_repo();
        put(&repo, ".yaks/hairy/yak-0003.md", "three\n");
        let r = report(&repo);
        assert_eq!(lines(&r), [("yak-0003".into(), "created".into())]);
        assert!(!r.yaks[0].staged);
    }

    #[test]
    fn moved_yak_with_its_transition_note_is_just_a_move() {
        let repo = team_repo();
        mv(
            &repo,
            ".yaks/shaving/yak-0002.md",
            ".yaks/shorn/yak-0002.md",
        );
        append(
            &repo,
            ".yaks/shorn/yak-0002.md",
            &note("moved: shaving -> shorn"),
        );
        let r = report(&repo);
        assert_eq!(
            lines(&r),
            [("yak-0002".into(), "moved shaving -> shorn".into())]
        );
    }

    #[test]
    fn new_notes_are_counted() {
        let repo = team_repo();
        append(&repo, ".yaks/hairy/yak-0001.md", &note("one"));
        append(&repo, ".yaks/hairy/yak-0001.md", &note("two"));
        let r = report(&repo);
        assert_eq!(lines(&r), [("yak-0001".into(), "notes +2".into())]);
    }

    #[test]
    fn a_field_edit_is_an_edit_even_with_a_note() {
        let repo = team_repo();
        put(
            &repo,
            ".yaks/hairy/yak-0001.md",
            &ONE.replace("id: yak-0001", "id: yak-0001\ntitle: new"),
        );
        assert_eq!(
            lines(&report(&repo)),
            [("yak-0001".into(), "edited".into())]
        );
        append(&repo, ".yaks/hairy/yak-0001.md", &note("n"));
        assert_eq!(
            lines(&report(&repo)),
            [("yak-0001".into(), "notes +1, edited".into())]
        );
    }

    #[test]
    fn only_the_updated_stamp_changing_still_reads_as_edited() {
        let repo = team_repo();
        put(
            &repo,
            ".yaks/hairy/yak-0001.md",
            &ONE.replace("2026-01-01", "2026-03-03"),
        );
        assert_eq!(
            lines(&report(&repo)),
            [("yak-0001".into(), "edited".into())]
        );
    }

    #[test]
    fn removed_yak() {
        let repo = team_repo();
        std::fs::remove_file(repo.join(".yaks/hairy/yak-0001.md")).unwrap();
        assert_eq!(
            lines(&report(&repo)),
            [("yak-0001".into(), "removed".into())]
        );
    }

    #[test]
    fn new_artifact_and_config_change() {
        let repo = team_repo();
        put(&repo, ".yaks/artifacts/yak-0002/b.txt", "b\n");
        put(&repo, ".yaks/config.yaml", "herd: other\n");
        let r = report(&repo);
        assert_eq!(lines(&r), [("yak-0002".into(), "artifacts".into())]);
        assert_eq!(r.other.len(), 1);
        assert_eq!(r.other[0].label, "config");
    }

    #[test]
    fn staged_versus_unstaged() {
        let repo = team_repo();
        append(&repo, ".yaks/hairy/yak-0001.md", &note("n"));
        put(&repo, ".yaks/hairy/yak-0003.md", "three\n");
        git(&repo, &["add", ".yaks/hairy/yak-0003.md"]);
        let r = report(&repo);
        let staged: Vec<(&str, bool)> = r.yaks.iter().map(|y| (&*y.id, y.staged)).collect();
        assert_eq!(staged, [("yak-0001", false), ("yak-0003", true)]);
        let text = render(&r);
        assert!(text.contains("  yak-0001  notes +1"), "{text}");
        assert!(text.contains("* yak-0003  created"), "{text}");
    }

    #[test]
    fn several_kinds_of_change_share_one_line() {
        let repo = team_repo();
        mv(
            &repo,
            ".yaks/shaving/yak-0002.md",
            ".yaks/shorn/yak-0002.md",
        );
        append(
            &repo,
            ".yaks/shorn/yak-0002.md",
            &(note("moved: shaving -> shorn") + &note("evidence")),
        );
        put(&repo, ".yaks/artifacts/yak-0002/b.txt", "b\n");
        let r = report(&repo);
        assert_eq!(
            lines(&r),
            [(
                "yak-0002".into(),
                "moved shaving -> shorn, notes +1, artifacts".into()
            )]
        );
    }

    #[test]
    fn staged_rename_reads_as_a_move() {
        let repo = team_repo();
        git(
            &repo,
            &["mv", ".yaks/hairy/yak-0001.md", ".yaks/shaving/yak-0001.md"],
        );
        let r = report(&repo);
        assert_eq!(
            lines(&r),
            [("yak-0001".into(), "moved hairy -> shaving".into())]
        );
        assert!(r.yaks[0].staged);
    }

    #[test]
    fn private_farm() {
        let repo = team_repo();
        git(&repo, &["rm", "-rq", "--cached", ".yaks"]);
        put(&repo, ".gitignore", ".yaks/\n");
        put(&repo, ".yaks/hairy/yak-0009.md", "nine\n");
        let r = report(&repo);
        assert!(r.private && r.clean());
        assert_eq!(
            render(&r),
            "private farm: not tracked by git, nothing to commit\n"
        );
    }

    #[test]
    fn merge_in_progress_says_commit_will_refuse() {
        let repo = team_repo();
        git(&repo, &["checkout", "-q", "-b", "side"]);
        put(&repo, "side.txt", "s\n");
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "side"]);
        git(&repo, &["checkout", "-q", "-"]);
        git(&repo, &["merge", "--no-commit", "--no-ff", "-q", "side"]);
        put(&repo, ".yaks/hairy/yak-0003.md", "three\n");
        let r = report(&repo);
        assert!(r.pending.is_some());
        let text = render(&r);
        assert!(text.contains("`yaks commit` will refuse"), "{text}");
        assert!(!text.contains("run `yaks commit`"), "{text}");
        assert_eq!(to_json(&r)["merge_in_progress"], true);
    }

    #[test]
    fn code_changes_are_counted_not_listed() {
        let repo = team_repo();
        put(&repo, "src/code.rs", "dirty\n");
        put(&repo, "src/new.rs", "\n");
        let r = report(&repo);
        assert!(r.clean());
        assert_eq!(r.outside, 2);
        let text = render(&r);
        assert!(
            text.contains("2 other changed files outside the farm"),
            "{text}"
        );
        assert!(!text.contains("code.rs"), "{text}");
    }

    #[test]
    fn json_shape() {
        let repo = team_repo();
        append(&repo, ".yaks/hairy/yak-0001.md", &note("n"));
        put(&repo, ".yaks/config.yaml", "herd: other\n");
        let v = to_json(&report(&repo));
        assert_eq!(v["clean"], false);
        assert_eq!(v["merge_in_progress"], false);
        assert_eq!(v["yaks"][0]["id"], "yak-0001");
        assert_eq!(v["yaks"][0]["changes"][0], "notes +1");
        assert_eq!(v["yaks"][0]["staged"], false);
        assert_eq!(v["other"][0]["path"], "config");
        let clean = to_json(&report(&team_repo()));
        assert_eq!(clean["clean"], true);
        assert_eq!(clean["yaks"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn status_writes_nothing() {
        let repo = team_repo();
        append(&repo, ".yaks/hairy/yak-0001.md", &note("n"));
        put(&repo, ".yaks/hairy/yak-0003.md", "three\n");
        let before = git(&repo, &["status", "--porcelain"]);
        let head = git(&repo, &["rev-parse", "HEAD"]);
        let _ = report(&repo);
        assert_eq!(git(&repo, &["status", "--porcelain"]), before);
        assert_eq!(git(&repo, &["rev-parse", "HEAD"]), head);
    }

    /// The verb `yaks commit` puts a status change under.
    fn commit_verb(token: &str) -> Option<String> {
        let word = token.split_whitespace().next()?;
        match word {
            "created" | "removed" => Some(word.to_string()),
            "moved" => {
                let to = token.rsplit(" -> ").next()?;
                Some(if to == "hairy" { "regrown" } else { to }.to_string())
            }
            "notes" | "edited" => Some("updated".to_string()),
            "artifacts" => Some("artifacts for".to_string()),
            _ => None,
        }
    }

    /// The status lines and the message `yaks commit` generates name the same
    /// yaks under the same verbs, on a fixture with every kind of change.
    #[test]
    fn status_agrees_with_the_commit_message() {
        let repo = team_repo();
        put(&repo, ".yaks/hairy/yak-0003.md", "three\n"); // created
        put(&repo, ".yaks/hairy/yak-0004.md", "four\n");
        append(&repo, ".yaks/hairy/yak-0001.md", &note("n")); // notes
        mv(
            &repo,
            ".yaks/shaving/yak-0002.md",
            ".yaks/shorn/yak-0002.md",
        ); // moved
        put(&repo, ".yaks/artifacts/yak-0002/b.txt", "b\n"); // artifacts
        put(&repo, ".yaks/artifacts/yak-0005/b.txt", "b\n"); // artifacts only
        put(&repo, ".yaks/config.yaml", "herd: other\n"); // config
        git(&repo, &["add", ".yaks/hairy/yak-0004.md"]);

        let r = report(&repo);
        let plan = match commit::run(&farm(&repo), None, true).unwrap() {
            commit::Outcome::Done(p) => p,
            commit::Outcome::Nothing => panic!("expected changes"),
        };
        // The report carries exactly the message commit would use.
        assert_eq!(r.message.as_deref(), Some(plan.message.as_str()));

        // Verb -> ids from the status lines ...
        let mut from_status: Vec<(String, String)> = Vec::new();
        for y in &r.yaks {
            for token in &y.changes {
                let verb = commit_verb(token).unwrap();
                if !from_status.contains(&(verb.clone(), y.id.clone())) {
                    from_status.push((verb, y.id.clone()));
                }
            }
        }
        from_status.sort();
        // ... and from the message's `verb ids; verb ids` phrases.
        let mut from_message: Vec<(String, String)> = Vec::new();
        for phrase in plan.message.strip_prefix("yaks: ").unwrap().split("; ") {
            if phrase == "farm config" {
                continue;
            }
            let (verb, ids) = phrase.split_at(phrase.find(" yak-").unwrap());
            for id in ids.split(", ") {
                from_message.push((verb.to_string(), id.trim().to_string()));
            }
        }
        from_message.sort();
        assert_eq!(from_status, from_message, "{}", plan.message);
        assert!(plan.message.contains("farm config") && r.other[0].label == "config");
    }

    #[test]
    fn describe_ignores_the_updated_stamp_and_transition_notes() {
        let new = ONE.replace("2026-01-01", "2026-05-05") + &note("moved: hairy -> shaving");
        assert_eq!(describe(ONE, &new), (0, false));
    }
}
