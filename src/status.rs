//! `yaks status`: what the farm has changed that git does not have yet, one
//! line per yak.
//!
//! Read-only. The classification is the one `yaks commit` builds its message
//! from ([`crate::changes::classify`]); this adds, per yak, what kind of edit
//! an in-place change was (notes appended, or a field or the description
//! changed) by comparing the file with its version at HEAD. A farm that is a
//! branch also says how that branch stands against its upstream
//! ([`sync_state`]), from the last fetch: nothing here touches the network.

use crate::changes::{self, Kind, YakChange, classify};
use crate::commit::generate_message;
use crate::model::{Status, Task};
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

/// Where a farm branch stands against its upstream, as of the last fetch.
/// What the TUI's sync indicator reuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncState {
    /// The farm branch (`yaks` by default).
    pub branch: String,
    /// Its upstream (`origin/yaks`), or `None` when the branch was never
    /// published (no upstream configured, or the one configured is gone).
    pub upstream: Option<String>,
    /// Commits on the branch the upstream does not have.
    pub ahead: usize,
    /// Commits on the upstream the branch does not have.
    pub behind: usize,
}

#[derive(Debug)]
pub struct Report {
    /// Nothing under the farm is tracked by git (a `YAKS_DIR` farm that is
    /// not a farm branch).
    pub untracked: bool,
    /// A merge or cherry-pick is in progress: `(the ref git sets, what it is)`.
    pub pending: Option<(&'static str, &'static str)>,
    pub yaks: Vec<YakLine>,
    pub other: Vec<OtherLine>,
    /// Changed files in the repository outside the farm (never listed).
    pub outside: usize,
    /// The message `yaks commit` would use, when there is anything to commit.
    pub message: Option<String>,
    /// The farm branch against its upstream; `None` unless the farm is a
    /// branch ([`sync_state`]).
    pub sync: Option<SyncState>,
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
            untracked: true,
            pending: None,
            yaks: Vec::new(),
            other: Vec::new(),
            outside: 0,
            message: None,
            sync: sync_state(root),
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
        untracked: false,
        pending: changes::pending_operation(root)?,
        yaks,
        other,
        outside: changes::outside(root)?.len(),
        message: (!changes.is_empty()).then(|| generate_message(&changes)),
        sync: sync_state(root),
    })
}

/// How the farm branch stands against its upstream, from the refs as of the
/// last fetch: never fetches, never writes. `None` unless the farm at `root`
/// is a dedicated farm-branch worktree ([`store::farm_branch`]) or git cannot
/// say.
pub fn sync_state(root: &Path) -> Option<SyncState> {
    let branch = store::farm_branch(root)?;
    let top = root.parent()?;
    let line = |args: &[&str]| -> Option<String> {
        let out = changes::git(top, args).ok()?;
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (out.status.success() && !text.is_empty()).then_some(text)
    };
    let Some(upstream) = line(&[
        "rev-parse",
        "--abbrev-ref",
        &format!("{branch}@{{upstream}}"),
    ]) else {
        return Some(SyncState {
            branch,
            upstream: None,
            ahead: 0,
            behind: 0,
        });
    };
    let counts = line(&[
        "rev-list",
        "--left-right",
        "--count",
        &format!("{branch}...{upstream}"),
    ])?;
    let mut n = counts.split_whitespace().map(|w| w.parse::<usize>());
    let (Some(Ok(ahead)), Some(Ok(behind))) = (n.next(), n.next()) else {
        return None;
    };
    Some(SyncState {
        branch,
        upstream: Some(upstream),
        ahead,
        behind,
    })
}

impl SyncState {
    /// The one status line for this state.
    pub fn line(&self) -> String {
        let branch = &self.branch;
        match &self.upstream {
            None => format!(
                "farm branch {branch}: not published (no upstream); `yaks sync` publishes it"
            ),
            Some(up) if self.ahead == 0 && self.behind == 0 => {
                format!("farm branch {branch}: up to date with {up}")
            }
            Some(up) => format!(
                "farm branch {branch}: {} ahead, {} behind {up} (as of the last fetch); \
                 `yaks sync` {}",
                self.ahead,
                self.behind,
                if self.behind == 0 {
                    "publishes"
                } else {
                    "reconciles"
                }
            ),
        }
    }
}

/// The change tokens for one yak's line, in the order `created`, `moved`,
/// `notes`, `edited`, `removed`, `artifacts`. `moved` is a changed `status:`.
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
            let old = change
                .old_path
                .as_deref()
                .and_then(|p| changes::head_text(root, p));
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

/// Compare a yak file's text at HEAD with now: how many notes were added
/// (`moved: a -> b` transition entries do not count), and whether the yak was
/// edited ([`store::edited`], the definition `yaks sheds` shares). A file that
/// does not parse counts as edited.
pub(crate) fn describe(old: &str, new: &str) -> (usize, bool) {
    let parse = |text: &str| store::parse_task(text, Status::Hairy);
    let (Some(old), Some(new)) = (parse(old), parse(new)) else {
        return (0, true);
    };
    let notes = |t: &Task| {
        store::parse_notes(&t.body)
            .iter()
            .filter(|n| !store::is_transition_text(&n.text))
            .count()
    };
    let added = notes(&new).saturating_sub(notes(&old));
    (added, store::edited(&old, &new))
}

/// The text output.
pub fn render(r: &Report) -> String {
    if r.untracked {
        return "farm not in git (not a farm branch): nothing to commit\n".to_string();
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
    if let Some(sync) = &r.sync {
        out.push_str(&sync.line());
        out.push('\n');
    }
    out
}

/// The `--json` output.
pub fn to_json(r: &Report) -> Value {
    json!({
        "untracked": r.untracked,
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
        "sync": r.sync.as_ref().map(|s| json!({
            "branch": s.branch,
            "upstream": s.upstream,
            "ahead": s.ahead,
            "behind": s.behind,
        })),
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

    /// A committed farm (the shape of a farm branch's worktree, as far as git
    /// is concerned): `yak-0001` hairy, `yak-0002` shaving (with an
    /// attachment), config and a source file.
    fn farm_repo() -> PathBuf {
        let repo = std::env::temp_dir().join(format!(
            "yaks-status-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        put(&repo, ".yaks/config.yaml", "herd: yak\n");
        put(
            &repo,
            ".yaks/yak/yaks/yak-0001.md",
            &flat("yak-0001", "hairy"),
        );
        put(
            &repo,
            ".yaks/yak/yaks/yak-0002.md",
            &flat("yak-0002", "shaving"),
        );
        put(&repo, ".yaks/yak/yaks/yak-0002/a.txt", "a\n");
        put(&repo, "src/code.rs", "fn main() {}\n");
        git(&repo, &["init", "-q"]);
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "init"]);
        repo
    }

    fn flat(id: &str, status: &str) -> String {
        format!(
            "---\nid: {id}\ntitle: t\nstatus: {status}\nupdated: '2026-01-01T00:00:00Z'\n---\n\nbody\n"
        )
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
        let repo = farm_repo();
        let r = report(&repo);
        assert!(r.clean() && !r.untracked && r.message.is_none());
        assert_eq!(render(&r), "farm clean: nothing to commit\n");
    }

    #[test]
    fn created_yak() {
        let repo = farm_repo();
        put(&repo, ".yaks/yak/yaks/yak-0003.md", "three\n");
        let r = report(&repo);
        assert_eq!(lines(&r), [("yak-0003".into(), "created".into())]);
        assert!(!r.yaks[0].staged);
    }

    #[test]
    fn moved_yak_with_its_transition_note_is_just_a_move() {
        let repo = farm_repo();
        put(
            &repo,
            ".yaks/yak/yaks/yak-0002.md",
            &flat("yak-0002", "shorn"),
        );
        append(
            &repo,
            ".yaks/yak/yaks/yak-0002.md",
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
        let repo = farm_repo();
        append(&repo, ".yaks/yak/yaks/yak-0001.md", &note("one"));
        append(&repo, ".yaks/yak/yaks/yak-0001.md", &note("two"));
        let r = report(&repo);
        assert_eq!(lines(&r), [("yak-0001".into(), "notes +2".into())]);
    }

    #[test]
    fn a_field_edit_is_an_edit_even_with_a_note() {
        let repo = farm_repo();
        put(
            &repo,
            ".yaks/yak/yaks/yak-0001.md",
            &flat("yak-0001", "hairy").replace("title: t", "title: new"),
        );
        assert_eq!(
            lines(&report(&repo)),
            [("yak-0001".into(), "edited".into())]
        );
        append(&repo, ".yaks/yak/yaks/yak-0001.md", &note("n"));
        assert_eq!(
            lines(&report(&repo)),
            [("yak-0001".into(), "notes +1, edited".into())]
        );
    }

    #[test]
    fn only_the_updated_stamp_changing_still_reads_as_edited() {
        let repo = farm_repo();
        put(
            &repo,
            ".yaks/yak/yaks/yak-0001.md",
            &flat("yak-0001", "hairy").replace("2026-01-01", "2026-03-03"),
        );
        assert_eq!(
            lines(&report(&repo)),
            [("yak-0001".into(), "edited".into())]
        );
    }

    #[test]
    fn removed_yak() {
        let repo = farm_repo();
        std::fs::remove_file(repo.join(".yaks/yak/yaks/yak-0001.md")).unwrap();
        assert_eq!(
            lines(&report(&repo)),
            [("yak-0001".into(), "removed".into())]
        );
    }

    #[test]
    fn new_artifact_and_config_change() {
        let repo = farm_repo();
        put(&repo, ".yaks/yak/yaks/yak-0002/b.txt", "b\n");
        put(&repo, ".yaks/config.yaml", "herd: other\n");
        let r = report(&repo);
        assert_eq!(lines(&r), [("yak-0002".into(), "artifacts".into())]);
        assert_eq!(r.other.len(), 1);
        assert_eq!(r.other[0].label, "config");
    }

    #[test]
    fn staged_versus_unstaged() {
        let repo = farm_repo();
        append(&repo, ".yaks/yak/yaks/yak-0001.md", &note("n"));
        put(&repo, ".yaks/yak/yaks/yak-0003.md", "three\n");
        git(&repo, &["add", ".yaks/yak/yaks/yak-0003.md"]);
        let r = report(&repo);
        let staged: Vec<(&str, bool)> = r.yaks.iter().map(|y| (&*y.id, y.staged)).collect();
        assert_eq!(staged, [("yak-0001", false), ("yak-0003", true)]);
        let text = render(&r);
        assert!(text.contains("  yak-0001  notes +1"), "{text}");
        assert!(text.contains("* yak-0003  created"), "{text}");
    }

    #[test]
    fn several_kinds_of_change_share_one_line() {
        let repo = farm_repo();
        put(
            &repo,
            ".yaks/yak/yaks/yak-0002.md",
            &flat("yak-0002", "shorn"),
        );
        append(
            &repo,
            ".yaks/yak/yaks/yak-0002.md",
            &(note("moved: shaving -> shorn") + &note("evidence")),
        );
        put(&repo, ".yaks/yak/yaks/yak-0002/b.txt", "b\n");
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
    fn flat_status_change_reads_as_a_move() {
        let repo = farm_repo();
        put(
            &repo,
            ".yaks/yak/yaks/yak-0001.md",
            &flat("yak-0001", "shaving"),
        );
        let r = report(&repo);
        assert_eq!(
            lines(&r),
            [("yak-0001".into(), "moved hairy -> shaving".into())]
        );
        assert_eq!(r.message.as_deref(), Some("yaks: shaving yak-0001"));
    }

    #[test]
    fn flat_status_change_plus_note_reports_both() {
        let repo = farm_repo();
        put(
            &repo,
            ".yaks/yak/yaks/yak-0002.md",
            &flat("yak-0002", "shorn"),
        );
        append(
            &repo,
            ".yaks/yak/yaks/yak-0002.md",
            &note("moved: shaving -> shorn"),
        );
        append(&repo, ".yaks/yak/yaks/yak-0002.md", &note("done, really"));
        let r = report(&repo);
        assert_eq!(
            lines(&r),
            [("yak-0002".into(), "moved shaving -> shorn, notes +1".into())]
        );
        assert_eq!(r.message.as_deref(), Some("yaks: shorn yak-0002"));
    }

    #[test]
    fn flat_edit_without_a_status_change_is_not_a_move() {
        let repo = farm_repo();
        append(&repo, ".yaks/yak/yaks/yak-0001.md", &note("just a note"));
        let r = report(&repo);
        assert_eq!(lines(&r), [("yak-0001".into(), "notes +1".into())]);
        assert_eq!(r.message.as_deref(), Some("yaks: updated yak-0001"));
    }

    #[test]
    fn flat_attachment_reports_artifacts() {
        let repo = farm_repo();
        put(&repo, ".yaks/yak/yaks/yak-0001/frame.svg", "<svg/>\n");
        let r = report(&repo);
        assert_eq!(lines(&r), [("yak-0001".into(), "artifacts".into())]);
        assert_eq!(r.message.as_deref(), Some("yaks: artifacts for yak-0001"));
    }

    #[test]
    fn a_staged_status_change_reads_as_a_staged_move() {
        let repo = farm_repo();
        put(
            &repo,
            ".yaks/yak/yaks/yak-0001.md",
            &flat("yak-0001", "dead"),
        );
        git(&repo, &["add", ".yaks/yak/yaks/yak-0001.md"]);
        let r = report(&repo);
        assert_eq!(
            lines(&r),
            [("yak-0001".into(), "moved hairy -> dead".into())]
        );
        assert!(r.yaks[0].staged);
    }

    #[test]
    fn an_untracked_farm() {
        let repo = farm_repo();
        git(&repo, &["rm", "-rq", "--cached", ".yaks"]);
        put(&repo, ".gitignore", ".yaks/\n");
        put(&repo, ".yaks/yak/yaks/yak-0009.md", "nine\n");
        let r = report(&repo);
        assert!(r.untracked && r.clean());
        assert_eq!(
            render(&r),
            "farm not in git (not a farm branch): nothing to commit\n"
        );
    }

    #[test]
    fn merge_in_progress_says_commit_will_refuse() {
        let repo = farm_repo();
        git(&repo, &["checkout", "-q", "-b", "side"]);
        put(&repo, "side.txt", "s\n");
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "side"]);
        git(&repo, &["checkout", "-q", "-"]);
        git(&repo, &["merge", "--no-commit", "--no-ff", "-q", "side"]);
        put(&repo, ".yaks/yak/yaks/yak-0003.md", "three\n");
        let r = report(&repo);
        assert!(r.pending.is_some());
        let text = render(&r);
        assert!(text.contains("`yaks commit` will refuse"), "{text}");
        assert!(!text.contains("run `yaks commit`"), "{text}");
        assert_eq!(to_json(&r)["merge_in_progress"], true);
    }

    #[test]
    fn code_changes_are_counted_not_listed() {
        let repo = farm_repo();
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
        let repo = farm_repo();
        append(&repo, ".yaks/yak/yaks/yak-0001.md", &note("n"));
        put(&repo, ".yaks/config.yaml", "herd: other\n");
        let v = to_json(&report(&repo));
        assert_eq!(v["clean"], false);
        assert_eq!(v["merge_in_progress"], false);
        assert_eq!(v["yaks"][0]["id"], "yak-0001");
        assert_eq!(v["yaks"][0]["changes"][0], "notes +1");
        assert_eq!(v["yaks"][0]["staged"], false);
        assert_eq!(v["other"][0]["path"], "config");
        let clean = to_json(&report(&farm_repo()));
        assert_eq!(clean["clean"], true);
        assert_eq!(clean["yaks"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn status_writes_nothing() {
        let repo = farm_repo();
        append(&repo, ".yaks/yak/yaks/yak-0001.md", &note("n"));
        put(&repo, ".yaks/yak/yaks/yak-0003.md", "three\n");
        let before = git(&repo, &["status", "--porcelain"]);
        let head = git(&repo, &["rev-parse", "HEAD"]);
        let _ = report(&repo);
        assert_eq!(git(&repo, &["status", "--porcelain"]), before);
        assert_eq!(git(&repo, &["rev-parse", "HEAD"]), head);
    }

    /// A farm that is a branch: `(code repo, farm worktree, bare origin)`. The
    /// farm branch `yaks` is an orphan branch holding only `.yaks/`, checked out
    /// in its own worktree; `origin` exists but the branch is not pushed yet.
    fn branch_farm() -> (PathBuf, PathBuf, PathBuf) {
        let base = std::env::temp_dir().join(format!(
            "yaks-status-branch-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let (code, wt, origin) = (
            base.join("code"),
            base.join("farm"),
            base.join("origin.git"),
        );
        std::fs::create_dir_all(&code).unwrap();
        git(&base, &["init", "-q", "--bare", origin.to_str().unwrap()]);
        git(&code, &["init", "-q", "-b", "main"]);
        put(&code, "src/code.rs", "fn main() {}\n");
        git(&code, &["add", "."]);
        git(&code, &["commit", "-q", "-m", "code"]);
        git(
            &code,
            &["remote", "add", "origin", origin.to_str().unwrap()],
        );
        git(
            &code,
            &["worktree", "add", "-q", "--detach", wt.to_str().unwrap()],
        );
        git(&wt, &["checkout", "-q", "--orphan", "yaks"]);
        git(&wt, &["rm", "-rfq", "."]);
        put(&wt, ".yaks/config.yaml", "herd: yak\n");
        put(
            &wt,
            ".yaks/yak/yaks/yak-0001.md",
            &flat("yak-0001", "hairy"),
        );
        git(&wt, &["add", "."]);
        git(&wt, &["commit", "-q", "-m", "farm"]);
        (code, wt, origin)
    }

    fn sync(wt: &Path) -> SyncState {
        sync_state(&wt.join(".yaks")).expect("a farm branch has a sync state")
    }

    fn state(upstream: Option<&str>, ahead: usize, behind: usize) -> SyncState {
        SyncState {
            branch: "yaks".into(),
            upstream: upstream.map(String::from),
            ahead,
            behind,
        }
    }

    /// A commit on the farm branch made from a second clone of `origin`, pushed.
    fn push_from_a_second_clone(origin: &Path) {
        let other = origin.parent().unwrap().join("clone2");
        git(
            origin.parent().unwrap(),
            &[
                "clone",
                "-q",
                origin.to_str().unwrap(),
                other.to_str().unwrap(),
            ],
        );
        git(&other, &["checkout", "-q", "yaks"]);
        put(
            &other,
            ".yaks/yak/yaks/yak-0002.md",
            &flat("yak-0002", "hairy"),
        );
        git(&other, &["add", "."]);
        git(&other, &["commit", "-q", "-m", "from elsewhere"]);
        git(&other, &["push", "-q", "origin", "yaks"]);
    }

    #[test]
    fn a_farm_branch_with_no_upstream_is_not_published() {
        let (_code, wt, _origin) = branch_farm();
        let s = sync(&wt);
        assert_eq!(s, state(None, 0, 0));
        let r = report(&wt);
        let text = render(&r);
        assert!(
            text.ends_with(
                "farm branch yaks: not published (no upstream); `yaks sync` publishes it\n"
            ),
            "{text}"
        );
        assert_eq!(
            to_json(&r)["sync"],
            serde_json::json!({"branch": "yaks", "upstream": null, "ahead": 0, "behind": 0})
        );
    }

    #[test]
    fn a_published_farm_branch_is_up_to_date_then_ahead_then_behind_and_ahead() {
        let (_code, wt, origin) = branch_farm();
        git(&wt, &["push", "-q", "-u", "origin", "yaks"]);
        assert_eq!(sync(&wt), state(Some("origin/yaks"), 0, 0));
        assert_eq!(
            render(&report(&wt)),
            "farm clean: nothing to commit\nfarm branch yaks: up to date with origin/yaks\n"
        );

        put(
            &wt,
            ".yaks/yak/yaks/yak-0003.md",
            &flat("yak-0003", "hairy"),
        );
        git(&wt, &["add", "."]);
        git(&wt, &["commit", "-q", "-m", "local"]);
        assert_eq!(sync(&wt), state(Some("origin/yaks"), 1, 0));
        let text = render(&report(&wt));
        assert!(
            text.contains(
                "farm branch yaks: 1 ahead, 0 behind origin/yaks (as of the last fetch); \
                 `yaks sync` publishes"
            ),
            "{text}"
        );

        push_from_a_second_clone(&origin);
        // Not fetched yet: status never fetches, so it still says 1 ahead.
        assert_eq!(sync(&wt), state(Some("origin/yaks"), 1, 0));
        git(&wt, &["fetch", "-q", "origin"]);
        assert_eq!(sync(&wt), state(Some("origin/yaks"), 1, 1));
        let v = to_json(&report(&wt));
        assert_eq!(v["sync"]["ahead"], 1);
        assert_eq!(v["sync"]["behind"], 1);
        assert_eq!(v["sync"]["upstream"], "origin/yaks");
        assert!(render(&report(&wt)).contains("1 ahead, 1 behind origin/yaks"));
    }

    #[test]
    fn a_farm_branch_only_behind_says_so() {
        let (_code, wt, origin) = branch_farm();
        git(&wt, &["push", "-q", "-u", "origin", "yaks"]);
        push_from_a_second_clone(&origin);
        git(&wt, &["fetch", "-q", "origin"]);
        assert_eq!(sync(&wt), state(Some("origin/yaks"), 0, 1));
    }

    #[test]
    fn an_in_tree_farm_has_no_sync_state() {
        let repo = farm_repo();
        assert_eq!(sync_state(&farm(&repo)), None);
        let r = report(&repo);
        assert!(r.sync.is_none() && !render(&r).contains("farm branch"));
        assert!(to_json(&r)["sync"].is_null());
    }

    #[test]
    fn sync_state_writes_nothing() {
        let (_code, wt, _origin) = branch_farm();
        git(&wt, &["push", "-q", "-u", "origin", "yaks"]);
        let before = git(&wt, &["for-each-ref"]);
        let _ = sync(&wt);
        assert_eq!(git(&wt, &["for-each-ref"]), before);
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
        let repo = farm_repo();
        put(&repo, ".yaks/yak/yaks/yak-0003.md", "three\n"); // created
        put(&repo, ".yaks/yak/yaks/yak-0004.md", "four\n");
        append(&repo, ".yaks/yak/yaks/yak-0001.md", &note("n")); // notes
        put(
            &repo,
            ".yaks/yak/yaks/yak-0002.md",
            &flat("yak-0002", "shorn"),
        ); // moved
        put(&repo, ".yaks/yak/yaks/yak-0002/b.txt", "b\n"); // artifacts
        put(&repo, ".yaks/yak/yaks/yak-0005/b.txt", "b\n"); // artifacts only
        put(&repo, ".yaks/config.yaml", "herd: other\n"); // config
        git(&repo, &["add", ".yaks/yak/yaks/yak-0004.md"]);

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
        let one = flat("yak-0001", "hairy");
        let new = one.replace("2026-01-01", "2026-05-05") + &note("moved: hairy -> shaving");
        assert_eq!(describe(&one, &new), (0, false));
    }
}
