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
    // A live farm first catches its branch up with main when nothing would be
    // lost, so its commits stay a short line on main (see `sync_live`).
    if !dry_run {
        sync_live(root)?;
    }

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

// -- live farm: keep the farm branch a short line on main, land it linearly --

/// The branch a live farm is kept on top of: `main`, else `master`, in the
/// farm's repository. `None` when neither exists.
fn base_branch(root: &Path) -> Option<&'static str> {
    ["main", "master"].into_iter().find(|b| {
        run_git(
            root,
            &["rev-parse", "--verify", "-q", &format!("refs/heads/{b}")],
        )
        .is_ok_and(|o| o.status.success())
    })
}

fn rev(root: &Path, spec: &str) -> Option<String> {
    run_git(root, &["rev-parse", "--verify", "-q", spec])
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Move a live farm's branch onto its base branch (`main`) when that loses
/// nothing: the base's farm tree is IDENTICAL to the branch's (every farm
/// commit is already on main, by any route). A mixed reset (ref + index only),
/// so the worktree's files, uncommitted yak edits included, are untouched, and
/// with equal trees no committed yak content changes either. This keeps the
/// farm a short line on top of main, so landing it is a cherry-pick, never a
/// merge (yaks-2355). Returns the base it moved onto, or `None` when it did
/// nothing (already there, a farm commit not yet on main, an operation in
/// progress, not a live farm).
pub fn sync_live(root: &Path) -> Result<Option<&'static str>> {
    if crate::store::read_config(root).live_branch.is_none() {
        return Ok(None);
    }
    let Some(base) = base_branch(root) else {
        return Ok(None);
    };
    if pending_operation(root)?.is_some() {
        return Ok(None);
    }
    let (Some(head), Some(base_sha)) = (rev(root, "HEAD"), rev(root, base)) else {
        return Ok(None);
    };
    if head == base_sha {
        return Ok(None);
    }
    let prefix = check(
        run_git(root, &["rev-parse", "--show-prefix"])?,
        "git rev-parse --show-prefix",
    )?;
    let rel = String::from_utf8_lossy(&prefix)
        .trim()
        .trim_end_matches('/')
        .to_string();
    let farm_tree = |c: &str| rev(root, &format!("{c}:{rel}"));
    if farm_tree(&head).is_none() || farm_tree(&head) != farm_tree(&base_sha) {
        return Ok(None);
    }
    check(run_git(root, &["reset", "-q", base])?, "git reset")?;
    Ok(Some(base))
}

/// What `yaks commit --land` did.
pub struct Landed {
    /// The farm branch was first moved onto this base (see [`sync_live`]).
    pub synced: Option<&'static str>,
    /// The farm commit made first, when the farm had changes.
    pub committed: Option<Plan>,
    /// The farm commits cherry-picked onto the invoking checkout's HEAD, as
    /// `(short sha, subject)`, oldest first. Empty when HEAD already had them.
    pub picked: Vec<(String, String)>,
}

/// `yaks commit --land`: commit a LIVE farm's changes (on its branch, in its
/// worktree), then bring every farm commit the checkout at `cwd` lacks onto
/// its HEAD as a cherry-pick, oldest first, so landing the farm never makes a
/// merge commit (yaks-2355). "Lacks" is by patch (`rev-list --cherry-pick`),
/// so a second run picks nothing. `cwd` is the checkout about to be pushed
/// (the coordinator's), never the farm worktree itself. Pushing stays yours;
/// once `main` has them, the next `yaks commit` moves the farm branch onto it.
pub fn land(root: &Path, cwd: &Path, message: Option<&str>) -> Result<Landed> {
    let Some(branch) = crate::store::read_config(root).live_branch else {
        bail!(
            "`yaks commit --land` is for a live farm (`live_branch:` in config.yaml); this farm \
             lives in the checkout itself, so a plain `yaks commit` already lands it"
        );
    };
    let top = |dir: &Path| -> Result<String> {
        let out = check(
            run_git(dir, &["rev-parse", "--show-toplevel"])?,
            "git rev-parse --show-toplevel",
        )?;
        Ok(String::from_utf8_lossy(&out).trim().to_string())
    };
    let farm_wt = top(root)?;
    let here = top(cwd)?;
    let same = |a: &str, b: &str| match (Path::new(a).canonicalize(), Path::new(b).canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    };
    if same(&farm_wt, &here) {
        bail!(
            "run `yaks commit --land` from the code checkout you are about to push, not from the \
             farm worktree ({farm_wt})"
        );
    }
    if let Some((head, what)) = pending_operation(cwd)? {
        bail!(
            "a {what} is in progress in {here} ({head} is set); finish or abort it, then run \
             `yaks commit --land` again (nothing was changed)"
        );
    }
    let synced = sync_live(root)?;
    let committed = match run(root, message, false)? {
        Outcome::Done(plan) => Some(plan),
        Outcome::Nothing => None,
    };
    let tip = rev(root, "HEAD").context("the farm branch has no commit")?;
    // The farm's objects may live in another repository (a Delta clone borrows
    // them, a plain clone does not): fetch the branch tip from the worktree.
    check(
        run_git(
            cwd,
            &["fetch", "-q", &farm_wt, &format!("refs/heads/{branch}")],
        )?,
        "git fetch of the farm branch",
    )?;
    let list = check(
        run_git(
            cwd,
            &[
                "rev-list",
                "--reverse",
                "--no-merges",
                "--right-only",
                "--cherry-pick",
                &format!("HEAD...{tip}"),
            ],
        )?,
        "git rev-list",
    )?;
    let shas: Vec<String> = String::from_utf8_lossy(&list)
        .lines()
        .map(str::to_string)
        .collect();
    let mut picked = Vec::new();
    if !shas.is_empty() {
        let mut args = vec!["cherry-pick"];
        args.extend(shas.iter().map(String::as_str));
        let out = run_git(cwd, &args)?;
        if !out.status.success() {
            let _ = run_git(cwd, &["cherry-pick", "--abort"]);
            bail!(
                "cherry-picking the farm commits onto {here} failed and was aborted (nothing \
                 changed there): {}. A code checkout's .yaks/ should only ever change by these \
                 picks; look for a commit that touched it directly.",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        for sha in &shas {
            let s = check(
                run_git(cwd, &["log", "-1", "--format=%h %s", sha])?,
                "git log",
            )?;
            let s = String::from_utf8_lossy(&s).trim().to_string();
            let (h, subject) = s.split_once(' ').unwrap_or((&s, ""));
            picked.push((h.to_string(), subject.to_string()));
        }
    }
    Ok(Landed {
        synced,
        committed,
        picked,
    })
}

/// How many ids to name per phrase before saying "and N more".
const MAX_IDS: usize = 5;

/// A subject line built from the changed files, for example
/// `yaks: created a; shorn b, c; updated d; artifacts for e`. Each yak gets one
/// verb (see [`classify`]): a status change reads as its destination status, a
/// new file as `created`, any other edit as `updated`, a deleted file as
/// `removed`.
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

    /// A yak file with a `status:` field (what a status change edits).
    fn flat(id: &str, status: &str) -> String {
        format!("---\nid: {id}\ntitle: t\nstatus: {status}\n---\n\nbody\n")
    }

    /// A temp repo with a committed farm (two yaks, an artifact, a source file).
    fn team_repo() -> PathBuf {
        let repo = std::env::temp_dir().join(format!(
            "yaks-commit-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        put(&repo, ".yaks/config.yaml", "herd: yak\n");
        put(&repo, ".yaks/yak-0001.md", &flat("yak-0001", "hairy"));
        put(&repo, ".yaks/yak-0002.md", &flat("yak-0002", "shaving"));
        put(&repo, ".yaks/yak-0002/a.txt", "a\n");
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
        put(&repo, ".yaks/yak-0002.md", &flat("yak-0002", "shorn"));
        put(&repo, ".yaks/yak-0001.md", "one, edited\n");
        put(&repo, ".yaks/yak-0003.md", "three\n");
        put(&repo, ".yaks/yak-0002/b.txt", "b\n");
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
                ".yaks/yak-0001.md",
                ".yaks/yak-0002.md",
                ".yaks/yak-0002/b.txt",
                ".yaks/yak-0003.md",
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
        put(&repo, ".yaks/yak-0001.md", "one, edited\n");

        let plan = done(run(&farm(&repo), Some("yaks: mine"), false).unwrap());
        assert_eq!(plan.message, "yaks: mine");
        let mut left = plan.left_staged.clone();
        left.sort();
        assert_eq!(left, ["src/code.rs", "src/new.rs"]);
        assert_eq!(last_files(&repo), [".yaks/yak-0001.md"]);
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
        put(&repo, ".yaks/yak-0009.md", "nine\n");
        let err = run(&farm(&repo), None, false).err().unwrap().to_string();
        assert!(err.contains("private farm"), "{err}");
    }

    #[test]
    fn dry_run_reports_without_staging_or_committing() {
        let repo = team_repo();
        put(&repo, ".yaks/yak-0001.md", "one, edited\n");
        put(&repo, ".yaks/yak-0003.md", "three\n");
        let before = git(&repo, &["rev-parse", "HEAD"]);
        let plan = done(run(&farm(&repo), None, true).unwrap());
        assert_eq!(plan.message, "yaks: created yak-0003; updated yak-0001");
        assert!(plan.commit.is_none());
        assert_eq!(plan.files.len(), 2);
        assert_eq!(git(&repo, &["rev-parse", "HEAD"]), before);
        assert_eq!(
            git(&repo, &["status", "--porcelain"]),
            " M .yaks/yak-0001.md\n?? .yaks/yak-0003.md\n"
        );
    }

    #[test]
    fn long_id_lists_are_capped_in_the_message() {
        let changes: Vec<Change> = (1..=8)
            .map(|n| Change::new('A', format!(".yaks/yak-{n:04}.md"), false))
            .collect();
        assert_eq!(
            generate_message(&changes),
            "yaks: created yak-0001, yak-0002, yak-0003, yak-0004, yak-0005 and 3 more"
        );
    }

    // -- live farm (yaks-2355) ---------------------------------------------

    /// A repo on `main` whose committed farm is live on branch `farm`, plus
    /// the farm worktree (pattern-sparse to `.yaks/`): `(code, farm root)`.
    fn live_repo() -> (PathBuf, PathBuf) {
        let code = team_repo();
        put(&code, ".yaks/config.yaml", "herd: yak\nlive_branch: farm\n");
        git(&code, &["branch", "-q", "-M", "main"]);
        git(&code, &["commit", "-qam", "live"]);
        let wt = code.with_extension("farm");
        git(
            &code,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "farm",
                wt.to_str().unwrap(),
                "main",
            ],
        );
        git(&wt, &["sparse-checkout", "set", "--no-cone", "/.yaks/"]);
        (code, wt.join(".yaks"))
    }

    fn subjects(repo: &Path) -> Vec<String> {
        git(repo, &["log", "--format=%s"])
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn merges(repo: &Path) -> usize {
        git(repo, &["rev-list", "--merges", "HEAD"]).lines().count()
    }

    #[test]
    fn land_cherry_picks_farm_commits_linearly_across_rounds_and_keeps_drift() {
        let (code, root) = live_repo();
        let wt = root.parent().unwrap().to_path_buf();
        assert!(
            !wt.join("src").exists(),
            "the farm worktree holds only .yaks/"
        );
        // Round 1: a yak edit in the farm while code lands on main.
        put(&wt, ".yaks/yak-0001.md", &flat("yak-0001", "shorn"));
        put(&code, "src/code.rs", "fn main() { one() }\n");
        git(&code, &["commit", "-qam", "code one"]);
        let l = land(&root, &code, None).unwrap();
        assert!(l.committed.is_some());
        assert_eq!(l.picked.len(), 1, "the one farm commit");
        assert_eq!(merges(&code), 0, "no merge commit on main");
        assert_eq!(subjects(&code)[0], "yaks: shorn yak-0001");
        // A second land with nothing new picks nothing (idempotent).
        assert!(land(&root, &code, None).unwrap().picked.is_empty());
        // Round 2: uncommitted drift sits in the farm while the next commit
        // first moves the farm branch onto main (which has its tree now).
        put(&wt, ".yaks/yak-0003.md", "three\n");
        put(&code, "src/code.rs", "fn main() { two() }\n");
        git(&code, &["commit", "-qam", "code two"]);
        assert_eq!(sync_live(&root).unwrap(), Some("main"));
        assert_eq!(
            git(&wt, &["rev-parse", "HEAD"]),
            git(&code, &["rev-parse", "main"]),
            "farm branch == main"
        );
        assert_eq!(
            std::fs::read_to_string(wt.join(".yaks/yak-0003.md")).unwrap(),
            "three\n"
        );
        assert!(!wt.join("src").exists(), "the reset did not check out code");
        let l = land(&root, &code, None).unwrap();
        assert_eq!(l.picked.len(), 1);
        assert_eq!(merges(&code), 0);
        assert_eq!(
            subjects(&code)[..4],
            [
                "yaks: created yak-0003",
                "code two",
                "yaks: shorn yak-0001",
                "code one"
            ]
        );
        assert_eq!(git(&wt, &["status", "--porcelain"]), "", "farm clean");
    }

    #[test]
    fn sync_live_never_moves_a_farm_commit_main_lacks() {
        let (code, root) = live_repo();
        let wt = root.parent().unwrap().to_path_buf();
        put(&wt, ".yaks/yak-0001.md", &flat("yak-0001", "shorn"));
        git(&wt, &["commit", "-qam", "yaks: shorn yak-0001"]);
        put(&code, "src/code.rs", "fn main() { moved() }\n");
        git(&code, &["commit", "-qam", "code"]);
        let before = git(&wt, &["rev-parse", "HEAD"]);
        assert_eq!(
            sync_live(&root).unwrap(),
            None,
            "main lacks the farm commit"
        );
        assert_eq!(git(&wt, &["rev-parse", "HEAD"]), before);
    }

    #[test]
    fn land_refuses_outside_a_live_farm_and_from_the_farm_worktree() {
        let repo = team_repo();
        let err = land(&farm(&repo), &repo, None).err().unwrap().to_string();
        assert!(err.contains("live farm"), "{err}");
        let (_code, root) = live_repo();
        let err = land(&root, root.parent().unwrap(), None)
            .err()
            .unwrap()
            .to_string();
        assert!(err.contains("not from the farm worktree"), "{err}");
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
        put(&repo, ".yaks/yak-0003.md", "three\n");

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
        assert!(status.contains("?? .yaks/yak-0003.md"), "{status}");
        // A dry run still previews.
        assert!(matches!(
            run(&farm(&repo), None, true).unwrap(),
            Outcome::Done(_)
        ));
    }
}
