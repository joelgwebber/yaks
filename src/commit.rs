//! `yaks commit`: commit the farm's own changes, and nothing else, in one
//! command.
//!
//! A team farm (`.yaks/` committed) drifts in the working tree whenever a yak
//! is moved, noted, answered or given an artifact, and a dirty tree blocks a
//! landing. This stages everything under the farm root and commits just those
//! paths (`git commit --only`), so code the repo has staged or modified is
//! never swept in: files staged elsewhere stay staged, exactly as they were.
//! It runs a normal `git commit` (the repo's hooks apply) and never pushes.

use crate::changes::{self, Change, Kind, check, classify, git as run_git, pending_operation};
use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Output};

/// What `run` did (or, with `dry_run`, would do).
pub enum Outcome {
    /// The farm had no changes; nothing was committed.
    Nothing,
    Done(Plan),
}

pub struct Plan {
    /// The commit message used.
    pub message: String,
    /// Changed farm paths, with git's one-letter state (`A`, `M`, `D`).
    pub files: Vec<(char, String)>,
    /// Paths staged outside the farm, left staged and out of the commit.
    pub left_staged: Vec<String>,
    /// The new commit's short hash; `None` on a dry run.
    pub commit: Option<String>,
}

/// Commit the farm at `root`. `message` overrides the generated one.
pub fn run(root: &Path, message: Option<&str>, dry_run: bool) -> Result<Outcome> {
    let git = |args: &[&str]| -> Result<Output> { run_git(root, args) };

    let Some(changes) = changes::read(root)? else {
        bail!(
            "this is a private farm (nothing under {} is tracked by git); there is nothing \
             for `yaks commit` to commit",
            root.display()
        );
    };
    if changes.is_empty() {
        return Ok(Outcome::Nothing);
    }
    let message = match message {
        Some(m) => m.to_string(),
        None => generate_message(&changes),
    };
    let left_staged = check(
        git(&[
            "diff",
            "--cached",
            "--name-only",
            "-z",
            "--",
            ":(top)",
            ":(exclude).",
        ])?,
        "git diff --cached",
    )?;
    let left_staged = String::from_utf8_lossy(&left_staged)
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    let mut plan = Plan {
        message,
        files: changes.iter().map(|c| (c.state, c.path.clone())).collect(),
        left_staged,
        commit: None,
    };
    if dry_run {
        return Ok(Outcome::Done(plan));
    }

    // Git refuses a partial commit (`--only`) while a merge or cherry-pick is
    // in progress, and that is exactly the state a Delta landing leaves behind.
    // Say so before staging anything, instead of reporting a bare git failure.
    if let Some((head, what)) = pending_operation(root)? {
        bail!(
            "a {what} is in progress ({head} is set), and git refuses a partial commit \
             during one, which `yaks commit` relies on to leave other files out. Nothing \
             was staged or committed. Finish it with a plain `git commit` (your farm \
             changes are part of it) or abandon it (`git {what} --abort`), then run \
             `yaks commit` again if farm changes remain."
        );
    }

    // Stage the farm (new files, deletions, moves), then commit exactly those
    // paths. Hooks run and print through our stdio, as for any `git commit`.
    check(git(&["add", "-A", "--", "."])?, "git add")?;
    let committed = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["commit", "--only", "-m", &plan.message, "--", "."])
        .status()
        .context("running git commit")?;
    if !committed.success() {
        bail!("git commit failed ({committed}); the farm changes are staged, not committed");
    }
    let head = check(git(&["rev-parse", "--short", "HEAD"])?, "git rev-parse")?;
    plan.commit = Some(String::from_utf8_lossy(&head).trim().to_string());
    Ok(Outcome::Done(plan))
}

/// How many ids to name per phrase before saying "and N more".
const MAX_IDS: usize = 5;

/// A subject line built from the changed files, for example
/// `yaks: created a; shorn b, c; updated d; artifacts for e`. Each yak gets one
/// verb (see [`classify`]): a move reads as its destination status, a new file
/// as `created`, an edit in place as `updated`, a deleted file as `removed`.
pub(crate) fn generate_message(changes: &[Change]) -> String {
    let classified = classify(changes);
    let mut created = BTreeSet::new();
    let mut moved: [(&str, BTreeSet<String>); 4] = [
        ("shaving", BTreeSet::new()),
        ("shorn", BTreeSet::new()),
        ("dead", BTreeSet::new()),
        ("regrown", BTreeSet::new()),
    ];
    let mut updated = BTreeSet::new();
    let mut removed = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    for (id, change) in classified.yaks {
        if change.artifacts {
            artifacts.insert(id.clone());
        }
        match change.kind {
            Some(Kind::Created) => created.insert(id),
            Some(Kind::Moved { to, .. }) => {
                let slot = match to.as_str() {
                    "shaving" => 0,
                    "shorn" => 1,
                    "dead" => 2,
                    _ => 3,
                };
                moved[slot].1.insert(id)
            }
            Some(Kind::Updated) => updated.insert(id),
            Some(Kind::Removed) => removed.insert(id),
            None => continue,
        };
    }

    let mut parts = Vec::new();
    let mut phrase = |verb: &str, ids: &BTreeSet<String>| {
        if !ids.is_empty() {
            parts.push(format!("{verb} {}", name_ids(ids)));
        }
    };
    phrase("created", &created);
    for (verb, ids) in &moved {
        phrase(verb, ids);
    }
    phrase("updated", &updated);
    phrase("removed", &removed);
    phrase("artifacts for", &artifacts);
    if !classified.other.is_empty() {
        parts.push("farm config".to_string());
    }
    format!("yaks: {}", parts.join("; "))
}

fn name_ids(ids: &BTreeSet<String>) -> String {
    let shown: Vec<&str> = ids.iter().take(MAX_IDS).map(String::as_str).collect();
    let mut s = shown.join(", ");
    if ids.len() > MAX_IDS {
        s.push_str(&format!(" and {} more", ids.len() - MAX_IDS));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn git(repo: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
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

    /// A temp repo with a committed farm (two yaks, an artifact, a source file).
    fn team_repo() -> PathBuf {
        let repo = std::env::temp_dir().join(format!(
            "yaks-commit-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        put(&repo, ".yaks/config.yaml", "herd: yak\n");
        put(&repo, ".yaks/hairy/yak-0001.md", "one\n");
        put(&repo, ".yaks/shaving/yak-0002.md", "two\n");
        put(&repo, ".yaks/artifacts/yak-0002/a.txt", "a\n");
        put(&repo, "src/code.rs", "fn main() {}\n");
        git(&repo, &["init", "-q"]);
        // `yaks commit` runs a plain `git commit`, which needs an identity; a CI
        // runner has no global one, so the repo carries its own.
        git(&repo, &["config", "user.name", "t"]);
        git(&repo, &["config", "user.email", "t@t"]);
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "init"]);
        repo
    }

    fn farm(repo: &Path) -> PathBuf {
        repo.join(".yaks")
    }

    fn done(o: Outcome) -> Plan {
        match o {
            Outcome::Done(p) => p,
            Outcome::Nothing => panic!("expected a commit"),
        }
    }

    fn last_files(repo: &Path) -> Vec<String> {
        let mut files: Vec<String> = git(
            repo,
            &["show", "--no-renames", "--name-only", "--format=", "HEAD"],
        )
        .lines()
        .map(str::to_string)
        .collect();
        files.sort();
        files
    }

    #[test]
    fn commits_farm_changes_and_leaves_code_alone() {
        let repo = team_repo();
        // A move, an edit, a new yak, a new artifact, plus dirty code.
        mv(
            &repo,
            ".yaks/shaving/yak-0002.md",
            ".yaks/shorn/yak-0002.md",
        );
        put(&repo, ".yaks/hairy/yak-0001.md", "one, edited\n");
        put(&repo, ".yaks/hairy/yak-0003.md", "three\n");
        put(&repo, ".yaks/artifacts/yak-0002/b.txt", "b\n");
        put(&repo, "src/code.rs", "fn main() { dirty() }\n");

        let plan = done(run(&farm(&repo), None, false).unwrap());
        assert_eq!(
            plan.message,
            "yaks: created yak-0003; shorn yak-0002; updated yak-0001; artifacts for yak-0002"
        );
        assert!(plan.commit.is_some());
        assert_eq!(
            last_files(&repo),
            [
                ".yaks/artifacts/yak-0002/b.txt",
                ".yaks/hairy/yak-0001.md",
                ".yaks/hairy/yak-0003.md",
                ".yaks/shaving/yak-0002.md",
                ".yaks/shorn/yak-0002.md",
            ]
        );
        // Code is untouched: still modified, still unstaged.
        assert_eq!(git(&repo, &["status", "--porcelain"]), " M src/code.rs\n");
        assert_eq!(
            git(&repo, &["log", "-1", "--format=%s"]).trim(),
            plan.message
        );
    }

    #[test]
    fn other_staged_files_stay_staged_and_out_of_the_commit() {
        let repo = team_repo();
        put(&repo, "src/code.rs", "fn main() { staged() }\n");
        put(&repo, "src/new.rs", "\n");
        git(&repo, &["add", "src"]);
        put(&repo, ".yaks/hairy/yak-0001.md", "one, edited\n");

        let plan = done(run(&farm(&repo), Some("yaks: mine"), false).unwrap());
        assert_eq!(plan.message, "yaks: mine");
        let mut left = plan.left_staged.clone();
        left.sort();
        assert_eq!(left, ["src/code.rs", "src/new.rs"]);
        assert_eq!(last_files(&repo), [".yaks/hairy/yak-0001.md"]);
        assert_eq!(
            git(&repo, &["status", "--porcelain"]),
            "M  src/code.rs\nA  src/new.rs\n"
        );
    }

    #[test]
    fn nothing_to_commit_when_the_farm_is_clean() {
        let repo = team_repo();
        put(&repo, "src/code.rs", "fn main() { dirty() }\n");
        let before = git(&repo, &["rev-parse", "HEAD"]);
        assert!(matches!(
            run(&farm(&repo), None, false).unwrap(),
            Outcome::Nothing
        ));
        assert_eq!(git(&repo, &["rev-parse", "HEAD"]), before);
    }

    #[test]
    fn private_farm_fails_clearly() {
        let repo = team_repo();
        git(&repo, &["rm", "-rq", "--cached", ".yaks"]);
        put(&repo, ".gitignore", ".yaks/\n");
        put(&repo, ".yaks/hairy/yak-0009.md", "nine\n");
        let err = run(&farm(&repo), None, false).err().unwrap().to_string();
        assert!(err.contains("private farm"), "{err}");
    }

    #[test]
    fn dry_run_reports_without_staging_or_committing() {
        let repo = team_repo();
        put(&repo, ".yaks/hairy/yak-0001.md", "one, edited\n");
        put(&repo, ".yaks/hairy/yak-0003.md", "three\n");
        let before = git(&repo, &["rev-parse", "HEAD"]);
        let plan = done(run(&farm(&repo), None, true).unwrap());
        assert_eq!(plan.message, "yaks: created yak-0003; updated yak-0001");
        assert!(plan.commit.is_none());
        assert_eq!(plan.files.len(), 2);
        assert_eq!(git(&repo, &["rev-parse", "HEAD"]), before);
        assert_eq!(
            git(&repo, &["status", "--porcelain"]),
            " M .yaks/hairy/yak-0001.md\n?? .yaks/hairy/yak-0003.md\n"
        );
    }

    #[test]
    fn long_id_lists_are_capped_in_the_message() {
        let changes: Vec<Change> = (1..=8)
            .map(|n| Change {
                state: 'A',
                path: format!(".yaks/hairy/yak-{n:04}.md"),
                staged: false,
            })
            .collect();
        assert_eq!(
            generate_message(&changes),
            "yaks: created yak-0001, yak-0002, yak-0003, yak-0004, yak-0005 and 3 more"
        );
    }

    #[test]
    fn a_staged_rename_between_status_dirs_reads_as_a_move() {
        let repo = team_repo();
        std::fs::create_dir_all(repo.join(".yaks/dead")).unwrap();
        git(
            &repo,
            &["mv", ".yaks/hairy/yak-0001.md", ".yaks/dead/yak-0001.md"],
        );
        let status = git(&repo, &["status", "--porcelain", "-z", "-uall"]);
        assert_eq!(
            generate_message(&changes::parse_status(&status)),
            "yaks: dead yak-0001"
        );
    }

    /// A Delta landing leaves a merge pending; `git commit --only` would fail there
    /// with a bare error, so the command must explain and stage nothing.
    #[test]
    fn refuses_during_a_merge_and_stages_nothing() {
        let repo = team_repo();
        git(&repo, &["checkout", "-q", "-b", "side"]);
        put(&repo, "side.txt", "s\n");
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "side"]);
        git(&repo, &["checkout", "-q", "-"]);
        git(&repo, &["merge", "--no-commit", "--no-ff", "-q", "side"]);
        assert!(repo.join(".git/MERGE_HEAD").exists(), "merge is pending");
        put(&repo, ".yaks/hairy/yak-0003.md", "three\n");

        let err = match run(&farm(&repo), None, false) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected an error during a merge"),
        };
        assert!(err.contains("merge is in progress"), "{err}");
        assert!(
            err.contains("git commit") && err.contains("--abort"),
            "{err}"
        );
        // Nothing was staged: the new yak is still untracked.
        let status = git(&repo, &["status", "--porcelain", "--", ".yaks"]);
        assert!(status.contains("?? .yaks/hairy/yak-0003.md"), "{status}");
        // A dry run still previews.
        assert!(matches!(
            run(&farm(&repo), None, true).unwrap(),
            Outcome::Done(_)
        ));
    }
}
