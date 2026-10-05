//! `yaks preflight`: a read-only landing-readiness check for a team farm.
//!
//! Run before landing (committing a shorn yak, merging a lane). It reports the
//! slips that earlier landings hit: a new artifacts directory that was never
//! `git add`ed, yak edits left unstaged, a shorn yak whose `verify:` command did
//! not last PASS, and a yak sitting in two status dirs. With `--push-main` it
//! also checks that the `local` remote is the human's checkout, before a
//! `git push local <branch>:main`. One line per failure; it never mutates the
//! farm or the repo (every git call here only reads: no push, fetch or write).

use crate::farm::{Farm, IssueKind};
use crate::model::Status;
use crate::store;
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The checks preflight runs; `code()` is the stable name used in `--json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// Something under the farm is untracked or has unstaged changes in git.
    GitState,
    /// A shorn yak's verify command (own, else the config default for its
    /// labels) did not last PASS.
    Verify,
    /// One id lives in two status dirs.
    DuplicateStatus,
    /// `--push-main`: the `local` remote is not a checkout that a push to
    /// `main` would reach and update.
    LocalCheckout,
}

impl Check {
    pub fn code(self) -> &'static str {
        match self {
            Check::GitState => "git-state",
            Check::Verify => "verify",
            Check::DuplicateStatus => "duplicate-status",
            Check::LocalCheckout => "local-checkout",
        }
    }
}

/// One failed check: a single actionable line, plus what it concerns (yak ids
/// or repo paths) for `--json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub check: Check,
    pub message: String,
    pub subjects: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Report {
    pub failures: Vec<Failure>,
    /// Checks that could not apply here (e.g. git state in a private farm),
    /// each a clear line saying why. Not failures.
    pub skipped: Vec<String>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.failures.is_empty()
    }
}

/// Run every check. Only the verify check is scoped (the shorn yaks being
/// landed); git state and duplicate status are farm-wide, since a stray file
/// or a clash anywhere would ride along into the landing commit or merge.
///
/// The verify scope is: the `ids` given; else, with `all`, every shorn yak; else
/// the shorn yaks that are part of the change in git (staged, modified or new
/// under the farm, including a new `artifacts/<id>/`), so an old shorn yak that
/// never ran `verify` is not blamed for a landing it is not in. A farm git
/// cannot describe (private, or outside a repository) checks every shorn yak.
///
/// `push_main` adds the `local`-remote check. It is opt-in: it depends on the
/// state of another checkout (the human's), which a worker committing its yak
/// can neither fix nor should be failed by; only the one landing on `main` asks.
pub fn run(farm: &Farm, ids: &[String], all: bool, push_main: bool) -> Result<Report> {
    let mut report = Report::default();
    let changed = git_state(farm.root(), &mut report)?;
    let scope = Scope { ids, all, changed };
    verify(farm, &scope, &mut report)?;
    duplicate_status(farm, &mut report)?;
    if push_main {
        local_checkout(farm.root(), &mut report)?;
    }
    Ok(report)
}

fn git_out(dir: &Path, args: &[&str]) -> Result<std::process::Output> {
    std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .context("running git")
}

fn stdout_of(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Check 1: nothing under the farm is untracked or has unstaged changes.
/// Staged changes pass: staging is the step right before the landing commit.
fn git_state(root: &Path, report: &mut Report) -> Result<Option<BTreeSet<String>>> {
    let git = |args: &[&str]| git_out(root, args);
    let tracked = git(&["ls-files", "--", "."])?;
    if !tracked.status.success() {
        report
            .skipped
            .push("git state: skipped (the farm is not inside a git repository)".into());
        return Ok(None);
    }
    if tracked.stdout.is_empty() {
        report.skipped.push(
            "git state: skipped (private farm: nothing under the farm is tracked by git)".into(),
        );
        return Ok(None);
    }
    let status = git(&["status", "--porcelain", "-z", "--", "."])?;
    if !status.status.success() {
        anyhow::bail!(
            "git status failed: {}",
            String::from_utf8_lossy(&status.stderr).trim()
        );
    }
    let out = String::from_utf8_lossy(&status.stdout);
    let mut changed = BTreeSet::new();
    let mut entries = out.split('\0').filter(|e| !e.is_empty());
    while let Some(entry) = entries.next() {
        let (xy, path) = entry.split_at(entry.len().min(2));
        let path = path.trim_start();
        let (x, y) = (
            xy.chars().next().unwrap_or(' '),
            xy.chars().nth(1).unwrap_or(' '),
        );
        // A rename/copy entry is followed by its source path; skip it.
        if matches!(x, 'R' | 'C') {
            entries.next();
        }
        // Every entry, staged or not, puts its yak in the change being landed.
        if let Some(id) = yak_id_of(path) {
            changed.insert(id);
        }
        let message = if xy == "??" {
            format!("{path} is untracked (git add it)")
        } else if x == 'U' || y == 'U' || xy == "AA" || xy == "DD" {
            format!("{path} has unresolved merge conflicts")
        } else if y != ' ' {
            format!("{path} has unstaged changes (git add it)")
        } else {
            continue; // staged only: ready to commit
        };
        report.failures.push(Failure {
            check: Check::GitState,
            message,
            subjects: vec![path.to_string()],
        });
    }
    Ok(Some(changed))
}

/// The yak a farm path belongs to: `.yaks/<status>/<id>.md` or
/// `.yaks/artifacts/<id>/...` (the farm may sit below the repo root).
pub(crate) fn yak_id_of(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if let Some(i) = parts.iter().position(|p| *p == "artifacts") {
        return parts
            .get(i + 1)
            .filter(|id| !id.is_empty())
            .map(|id| id.to_string());
    }
    let (file, dir) = (parts.last()?, parts.get(parts.len().checked_sub(2)?)?);
    let stem = file.strip_suffix(".md")?;
    ["hairy", "shaving", "shorn", "dead"]
        .contains(dir)
        .then(|| stem.to_string())
}

/// Which shorn yaks the verify check looks at (see [`run`]).
struct Scope<'a> {
    ids: &'a [String],
    all: bool,
    changed: Option<BTreeSet<String>>,
}

impl Scope<'_> {
    fn includes(&self, id: &str) -> bool {
        if !self.ids.is_empty() {
            self.ids.iter().any(|i| i == id)
        } else if self.all {
            true
        } else {
            self.changed.as_ref().is_none_or(|c| c.contains(id))
        }
    }
}

/// Check 2: each shorn yak in scope that has a verify command (its own, else
/// the config default for its labels, as `yaks verify` resolves it) last
/// passed. Same "last verify note" rule as `doctor --strict`.
fn verify(farm: &Farm, scope: &Scope, report: &mut Report) -> Result<()> {
    let tasks = store::load(farm.root(), &ALL)?;
    for id in scope.ids {
        if !tasks.iter().any(|t| &t.id == id) {
            report.failures.push(Failure {
                check: Check::Verify,
                message: format!("{id}: no such yak"),
                subjects: vec![id.clone()],
            });
        }
    }
    let cfg = farm.config();
    for t in tasks.iter().filter(|t| t.status == Status::Shorn) {
        if !scope.includes(&t.id) {
            continue;
        }
        let has_verify = t.verify.is_some()
            || cfg
                .resolve_verify(&t.labels, t.id.split('-').next())
                .is_some();
        if has_verify && !crate::farm::last_verify_passed(&t.body) {
            report.failures.push(Failure {
                check: Check::Verify,
                message: format!(
                    "{} is shorn but its verify command did not last PASS (run `yaks verify {}`)",
                    t.id, t.id
                ),
                subjects: vec![t.id.clone()],
            });
        }
    }
    Ok(())
}

/// Check 3: no yak sits in two status dirs (doctor's duplicate-status finding).
fn duplicate_status(farm: &Farm, report: &mut Report) -> Result<()> {
    for issue in farm.doctor(false)? {
        if issue.kind == IssueKind::DuplicateStatus {
            report.failures.push(Failure {
                check: Check::DuplicateStatus,
                message: issue.message,
                subjects: issue.ids,
            });
        }
    }
    Ok(())
}

/// Check 4 (`--push-main`): a `git push local <branch>:main` reaches the
/// human's checkout. On a thread shared to another machine `local` is a
/// Delta-managed bare repo with no branches: the push is accepted and lands
/// where the human never looks. Reads only: `remote get-url`, `ls-remote`,
/// `rev-parse`, `status`.
///
/// Fails when `local` is missing, is not a path on this machine (https, ssh),
/// does not exist, is not a git repository, is bare, has no `refs/heads/main`,
/// or is a checkout on `main` with uncommitted changes to tracked files (git
/// refuses the push then). Untracked files do not block an update.
fn local_checkout(root: &Path, report: &mut Report) -> Result<()> {
    let Some(top) = git_out(root, &["rev-parse", "--show-toplevel"])
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(stdout_of(&o)))
    else {
        report
            .skipped
            .push("local checkout: skipped (the farm is not inside a git repository)".into());
        return Ok(());
    };
    let mut fail = |message: String, subject: &str| {
        report.failures.push(Failure {
            check: Check::LocalCheckout,
            message,
            subjects: vec![subject.to_string()],
        });
    };
    let url = git_out(&top, &["remote", "get-url", "local"])?;
    if !url.status.success() {
        fail(
            "there is no `local` remote, so there is no checkout of the human's to push `main` \
             to (add one with `git remote add local <path>`, or hand the work over another way)"
                .into(),
            "local",
        );
        return Ok(());
    }
    let url = stdout_of(&url);
    if is_remote_url(&url) {
        fail(
            format!(
                "`local` is {url}, not a path on this machine, so it is not the human's \
                 checkout: a push to its `main` lands on that server, not in their working tree. \
                 Push `pr/<name>` there and tell the human to fetch it, or repoint `local` at \
                 the checkout (`git remote set-url local <path>`)"
            ),
            &url,
        );
        return Ok(());
    }
    let path = {
        let raw = url.strip_prefix("file://").unwrap_or(&url);
        top.join(raw) // absolute `raw` replaces `top`
    };
    let shown = path.display().to_string();
    let fallback = format!(
        "never push `main` there: push `pr/<name>` to `local` and give the human `git fetch \
         {shown} pr/<name>`, or repoint `local` at their checkout (`git remote set-url local \
         <path>`)"
    );
    if !path.is_dir() {
        fail(
            format!(
                "`local` points at {shown}, which does not exist or is not a directory: {fallback}"
            ),
            &shown,
        );
        return Ok(());
    }
    let bare = git_out(&path, &["rev-parse", "--is-bare-repository"])?;
    if !bare.status.success() {
        fail(
            format!(
                "`local` points at {shown}, which is not a readable git repository: {fallback}"
            ),
            &shown,
        );
        return Ok(());
    }
    if stdout_of(&bare) == "true" {
        fail(
            format!(
                "`local` ({shown}) is a bare repository, not the human's checkout (a push to \
                 `main` is accepted but lands where they never look): {fallback}"
            ),
            &shown,
        );
        return Ok(());
    }
    let heads = git_out(&top, &["ls-remote", "--heads", "local", "main"])?;
    if !heads.status.success() || heads.stdout.is_empty() {
        fail(
            format!(
                "`local` ({shown}) has no `refs/heads/main`, so it is not the human's checkout: {fallback}"
            ),
            &shown,
        );
        return Ok(());
    }
    // A `.git` directory is the repository of the checkout beside it.
    let work = if path.file_name().is_some_and(|n| n == ".git") {
        path.parent().unwrap_or(&path)
    } else {
        &path
    };
    let on_main = git_out(work, &["symbolic-ref", "--short", "-q", "HEAD"])
        .is_ok_and(|o| stdout_of(&o) == "main");
    if on_main {
        let status = git_out(
            work,
            &[
                "--no-optional-locks",
                "status",
                "--porcelain",
                "--untracked-files=no",
            ],
        )?;
        let dirty = stdout_of(&status).lines().count();
        if dirty > 0 {
            fail(
                format!(
                    "`local` ({shown}) is a checkout on `main` with {dirty} uncommitted \
                     change(s) to tracked files, so a push to `main` would be refused. Ask the \
                     human to commit or stash them (yak edits count), or push `pr/<name>` to \
                     `local` and give them `git fetch {shown} pr/<name>`"
                ),
                &shown,
            );
        }
    }
    Ok(())
}

/// A remote URL that is not a path on this machine: `scheme://host/...`
/// (except `file://`) or scp-style `host:path`.
fn is_remote_url(url: &str) -> bool {
    if url.starts_with("file://") {
        return false;
    }
    if url.contains("://") {
        return true;
    }
    // scp-style: a colon before any slash (and not a one-letter Windows drive).
    url.split_once(':')
        .is_some_and(|(host, _)| host.len() > 1 && !host.contains('/'))
}

const ALL: [Status; 4] = [Status::Hairy, Status::Shaving, Status::Shorn, Status::Dead];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Task;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn git(repo: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {out:?}");
    }

    /// A temp git repo with a near-empty farm committed in it, so the farm counts
    /// as team-mode (something under `.yaks/` is tracked).
    fn team_repo() -> (PathBuf, Farm) {
        let repo = std::env::temp_dir().join(format!(
            "yaks-preflight-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let root = repo.join(".yaks");
        for st in ALL {
            std::fs::create_dir_all(root.join(st.dir())).unwrap();
        }
        std::fs::write(root.join("config.yaml"), "herd: yak\n").unwrap();
        // An already-committed artifact dir, as in a real farm (git collapses an
        // untracked dir to its topmost new directory).
        std::fs::create_dir_all(root.join("artifacts/yak-0000")).unwrap();
        std::fs::write(root.join("artifacts/yak-0000/old.txt"), "x").unwrap();
        git(&repo, &["init", "-q"]);
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "init"]);
        let farm = Farm::open(&repo).unwrap_or_else(|_| panic!("open farm at {repo:?}"));
        (repo, farm)
    }

    fn task(id: &str, status: Status) -> Task {
        Task {
            id: id.into(),
            title: format!("title {id}"),
            kind: "task".into(),
            priority: 3,
            status,
            created: Some("2026-01-01T00:00:00Z".into()),
            updated: Some("2026-01-01T00:00:00Z".into()),
            parent: None,
            labels: vec![],
            depends_on: vec![],
            source: None,
            needs: None,
            verify: None,
            extra: Vec::new(),
            body: String::new(),
        }
    }

    fn note(text: &str) -> String {
        store::append_note("", "2026-01-02T00:00:00Z", Some("t"), text)
    }

    fn failures(farm: &Farm, ids: &[&str], check: Check) -> Vec<Failure> {
        let ids: Vec<String> = ids.iter().map(|s| s.to_string()).collect();
        run(farm, &ids, false, false)
            .unwrap()
            .failures
            .into_iter()
            .filter(|f| f.check == check)
            .collect()
    }

    #[test]
    fn clean_team_repo_is_ok() {
        let (_repo, farm) = team_repo();
        let report = run(&farm, &[], false, false).unwrap();
        assert!(report.ok(), "{:?}", report.failures);
        assert!(report.skipped.is_empty());
    }

    #[test]
    fn untracked_artifacts_dir_fails_naming_it_and_passes_once_added() {
        let (repo, farm) = team_repo();
        let dir = repo.join(".yaks/artifacts/yak-0001");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("frame.txt"), "x").unwrap();
        let f = failures(&farm, &[], Check::GitState);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message.contains(".yaks/artifacts/yak-0001/"),
            "{}",
            f[0].message
        );
        assert!(f[0].message.contains("untracked"));
        git(&repo, &["add", ".yaks"]);
        assert!(run(&farm, &[], false, false).unwrap().ok(), "staged passes");
    }

    #[test]
    fn unstaged_yak_edit_fails_and_passes_once_staged() {
        let (repo, farm) = team_repo();
        store::write::save(&repo.join(".yaks"), &task("yak-0001", Status::Hairy)).unwrap();
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "add yak"]);
        assert!(run(&farm, &[], false, false).unwrap().ok());
        let file = repo.join(".yaks/hairy/yak-0001.md");
        let text = std::fs::read_to_string(&file).unwrap();
        std::fs::write(&file, format!("{text}\nedited\n")).unwrap();
        let f = failures(&farm, &[], Check::GitState);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("yak-0001.md") && f[0].message.contains("unstaged"));
        git(&repo, &["add", ".yaks"]);
        assert!(run(&farm, &[], false, false).unwrap().ok());
    }

    #[test]
    fn changes_outside_the_farm_are_not_flagged() {
        let (repo, farm) = team_repo();
        std::fs::write(repo.join("code.rs"), "fn main() {}").unwrap();
        assert!(run(&farm, &[], false, false).unwrap().ok());
    }

    #[test]
    fn private_farm_skips_git_state_but_still_runs_other_checks() {
        let (repo, farm) = team_repo();
        // Untrack the farm and ignore it: now `git ls-files .yaks` is empty.
        git(&repo, &["rm", "-rq", "--cached", ".yaks"]);
        std::fs::write(repo.join(".gitignore"), ".yaks/\n").unwrap();
        let mut bad = task("yak-0001", Status::Shorn);
        bad.verify = Some("false".into());
        bad.body = note("verify: `false` -> FAIL (exit 1)");
        store::write::save(&repo.join(".yaks"), &bad).unwrap();
        let report = run(&farm, &[], false, false).unwrap();
        assert_eq!(report.skipped.len(), 1);
        assert!(report.skipped[0].contains("private farm"));
        assert!(report.failures.iter().all(|f| f.check != Check::GitState));
        assert_eq!(
            report.failures.len(),
            1,
            "verify still runs: {:?}",
            report.failures
        );
        assert_eq!(report.failures[0].check, Check::Verify);
    }

    #[test]
    fn verify_check_passes_and_fails_on_the_last_verify_note() {
        let (repo, farm) = team_repo();
        let root = repo.join(".yaks");
        let mut ok = task("yak-0001", Status::Shorn);
        ok.verify = Some("true".into());
        ok.body = note("verify: `true` -> PASS (exit 0)");
        let mut bad = task("yak-0002", Status::Shorn);
        bad.verify = Some("false".into());
        bad.body = note("verify: `false` -> FAIL (exit 1)");
        let mut never = task("yak-0003", Status::Shorn);
        never.verify = Some("true".into());
        never.body = note("did the work");
        let plain = task("yak-0004", Status::Shorn);
        for t in [&ok, &bad, &never, &plain] {
            store::write::save(&root, t).unwrap();
        }
        git(&repo, &["add", "."]);
        let f = failures(&farm, &[], Check::Verify);
        let ids: Vec<&str> = f.iter().map(|f| f.subjects[0].as_str()).collect();
        assert_eq!(ids, ["yak-0002", "yak-0003"]);
        assert!(f[0].message.contains("yaks verify yak-0002"));
        // Scoping to ids: only those are checked.
        let f = failures(&farm, &["yak-0001", "yak-0003"], Check::Verify);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].subjects, ["yak-0003"]);
    }

    #[test]
    fn verify_check_uses_the_config_default_for_the_yaks_labels() {
        let (repo, farm) = team_repo();
        let root = repo.join(".yaks");
        std::fs::write(
            root.join("config.yaml"),
            "herd: yak\nverify:\n  cli: cargo test\n",
        )
        .unwrap();
        let mut labelled = task("yak-0001", Status::Shorn);
        labelled.labels = vec!["cli".into()];
        labelled.body = note("did the work");
        let mut unlabelled = task("yak-0002", Status::Shorn);
        unlabelled.body = note("did the work");
        store::write::save(&root, &labelled).unwrap();
        store::write::save(&root, &unlabelled).unwrap();
        git(&repo, &["add", "."]);
        let f = failures(&farm, &[], Check::Verify);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].subjects, ["yak-0001"]);
    }

    #[test]
    fn unknown_id_fails_and_unshorn_ids_are_ignored() {
        let (repo, farm) = team_repo();
        let mut t = task("yak-0001", Status::Shaving);
        t.verify = Some("false".into());
        store::write::save(&repo.join(".yaks"), &t).unwrap();
        git(&repo, &["add", "."]);
        assert!(failures(&farm, &["yak-0001"], Check::Verify).is_empty());
        let f = failures(&farm, &["yak-9999"], Check::Verify);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("no such yak"));
    }

    #[test]
    fn yak_in_two_status_dirs_fails() {
        let (repo, farm) = team_repo();
        let root = repo.join(".yaks");
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        store::write::save(&root, &task("yak-0001", Status::Shorn)).unwrap();
        git(&repo, &["add", "."]);
        let f = failures(&farm, &[], Check::DuplicateStatus);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("yak-0001"));
    }

    /// An old shorn yak that never ran `verify` must not be blamed for a landing
    /// it is not part of; `--all` is the farm-wide check.
    #[test]
    fn default_scope_is_the_yaks_in_the_change_and_all_is_farm_wide() {
        let (repo, farm) = team_repo();
        let root = repo.join(".yaks");
        let mut old = task("yak-0001", Status::Shorn);
        old.verify = Some("true".into());
        old.body = note("did the work, never ran verify");
        store::write::save(&root, &old).unwrap();
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "old shorn yak"]);
        // The change being landed: a different yak, verified.
        let mut fresh = task("yak-0002", Status::Shorn);
        fresh.verify = Some("true".into());
        fresh.body = note("verify: `true` -> PASS (exit 0)");
        store::write::save(&root, &fresh).unwrap();
        git(&repo, &["add", "."]);
        assert!(
            run(&farm, &[], false, false).unwrap().ok(),
            "old yak is out of scope"
        );
        let all = run(&farm, &[], true, false).unwrap();
        assert_eq!(all.failures.len(), 1, "{:?}", all.failures);
        assert_eq!(all.failures[0].subjects, ["yak-0001"]);
        // Naming it puts it in scope too.
        let named = failures(&farm, &["yak-0001"], Check::Verify);
        assert_eq!(named.len(), 1);
    }

    #[test]
    fn a_new_artifacts_dir_puts_its_yak_in_scope() {
        let (repo, farm) = team_repo();
        let root = repo.join(".yaks");
        let mut old = task("yak-0001", Status::Shorn);
        old.verify = Some("true".into());
        old.body = note("did the work, never ran verify");
        store::write::save(&root, &old).unwrap();
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "old shorn yak"]);
        assert!(run(&farm, &[], false, false).unwrap().ok());
        // New evidence for that yak arrives: it is now part of the change.
        let dir = root.join("artifacts/yak-0001");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("frame.txt"), "x").unwrap();
        git(&repo, &["add", ".yaks"]);
        let f = failures(&farm, &[], Check::Verify);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].subjects, ["yak-0001"]);
    }

    #[test]
    fn yak_ids_come_from_status_files_and_artifact_dirs_only() {
        assert_eq!(
            yak_id_of(".yaks/shorn/yaks-aa49.md").as_deref(),
            Some("yaks-aa49")
        );
        assert_eq!(yak_id_of("sub/.yaks/hairy/x-1.md").as_deref(), Some("x-1"));
        assert_eq!(
            yak_id_of(".yaks/artifacts/yaks-aa49/frame.txt").as_deref(),
            Some("yaks-aa49")
        );
        assert_eq!(yak_id_of(".yaks/config.yaml"), None);
        assert_eq!(yak_id_of(".yaks/.gitignore"), None);
        assert_eq!(yak_id_of("src/main.rs"), None);
        assert_eq!(yak_id_of("README.md"), None);
    }

    /// A fresh non-bare repo on `main` with one commit: the human's checkout.
    fn human_checkout() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "yaks-human-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q", "-b", "main"]);
        std::fs::write(dir.join("a.txt"), "a").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-q", "-m", "init"]);
        dir
    }

    /// Point `repo`'s `local` remote at `url`, then run `--push-main` and return
    /// the local-checkout failures.
    fn local_failures(repo: &Path, farm: &Farm, url: &str) -> Vec<Failure> {
        git(repo, &["remote", "add", "local", url]);
        run(farm, &[], false, true)
            .unwrap()
            .failures
            .into_iter()
            .filter(|f| f.check == Check::LocalCheckout)
            .collect()
    }

    fn only_message(f: Vec<Failure>) -> String {
        assert_eq!(f.len(), 1, "{f:?}");
        f[0].message.clone()
    }

    #[test]
    fn local_that_is_a_real_checkout_with_main_passes() {
        let (repo, farm) = team_repo();
        let human = human_checkout();
        let f = local_failures(&repo, &farm, human.to_str().unwrap());
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn local_pointing_at_the_git_dir_of_a_checkout_passes_and_is_checked_for_dirt() {
        let (repo, farm) = team_repo();
        let human = human_checkout();
        git(
            &repo,
            &[
                "remote",
                "add",
                "local",
                human.join(".git").to_str().unwrap(),
            ],
        );
        let check = || {
            run(&farm, &[], false, true)
                .unwrap()
                .failures
                .into_iter()
                .filter(|f| f.check == Check::LocalCheckout)
                .count()
        };
        assert_eq!(check(), 0);
        std::fs::write(human.join("a.txt"), "changed").unwrap();
        assert_eq!(check(), 1, "the checkout beside the .git dir is dirty");
    }

    #[test]
    fn bare_local_fails_and_names_the_pr_branch_fallback() {
        let (repo, farm) = team_repo();
        let human = human_checkout();
        let bare = std::env::temp_dir().join(format!("yaks-bare-{}.git", std::process::id()));
        let _ = std::fs::remove_dir_all(&bare);
        git(
            &human,
            &["clone", "-q", "--bare", ".", bare.to_str().unwrap()],
        );
        let msg = only_message(local_failures(&repo, &farm, bare.to_str().unwrap()));
        assert!(msg.contains("bare repository"), "{msg}");
        assert!(msg.contains("push `pr/<name>` to `local`"), "{msg}");
        assert!(
            msg.contains(&format!("git fetch {} pr/<name>", bare.display())),
            "{msg}"
        );
    }

    #[test]
    fn empty_managed_style_bare_local_fails() {
        let (repo, farm) = team_repo();
        let bare = std::env::temp_dir().join(format!("yaks-managed-{}.git", std::process::id()));
        let _ = std::fs::remove_dir_all(&bare);
        std::fs::create_dir_all(&bare).unwrap();
        git(&bare, &["init", "-q", "--bare"]);
        let msg = only_message(local_failures(&repo, &farm, bare.to_str().unwrap()));
        assert!(msg.contains("bare repository"), "{msg}");
    }

    #[test]
    fn non_bare_local_without_main_fails() {
        let (repo, farm) = team_repo();
        let human = human_checkout();
        git(&human, &["checkout", "-q", "-b", "work"]);
        git(&human, &["branch", "-q", "-D", "main"]);
        let msg = only_message(local_failures(&repo, &farm, human.to_str().unwrap()));
        assert!(msg.contains("no `refs/heads/main`"), "{msg}");
        assert!(msg.contains("pr/<name>"), "{msg}");
    }

    #[test]
    fn missing_local_remote_fails_only_when_asked() {
        let (_repo, farm) = team_repo();
        assert!(
            run(&farm, &[], false, false).unwrap().ok(),
            "off by default"
        );
        let report = run(&farm, &[], false, true).unwrap();
        assert_eq!(report.failures.len(), 1, "{:?}", report.failures);
        assert_eq!(report.failures[0].check, Check::LocalCheckout);
        assert!(report.failures[0].message.contains("no `local` remote"));
    }

    #[test]
    fn missing_local_path_fails() {
        let (repo, farm) = team_repo();
        let gone = std::env::temp_dir().join(format!("yaks-gone-{}", std::process::id()));
        let msg = only_message(local_failures(&repo, &farm, gone.to_str().unwrap()));
        assert!(msg.contains("does not exist"), "{msg}");
        // A directory that is not a repository is reported too.
        let (repo, farm) = team_repo();
        let plain = std::env::temp_dir().join(format!("yaks-plain-{}", std::process::id()));
        std::fs::create_dir_all(&plain).unwrap();
        let msg = only_message(local_failures(&repo, &farm, plain.to_str().unwrap()));
        assert!(msg.contains("not a readable git repository"), "{msg}");
    }

    #[test]
    fn dirty_checkout_on_main_fails_but_untracked_files_and_other_branches_do_not() {
        let (repo, farm) = team_repo();
        let human = human_checkout();
        let url = human.to_str().unwrap();
        git(&repo, &["remote", "add", "local", url]);
        let n = || {
            run(&farm, &[], false, true)
                .unwrap()
                .failures
                .into_iter()
                .filter(|f| f.check == Check::LocalCheckout)
                .collect::<Vec<_>>()
        };
        std::fs::write(human.join("new.txt"), "x").unwrap();
        assert!(n().is_empty(), "untracked files do not block an update");
        std::fs::write(human.join("a.txt"), "changed").unwrap();
        let msg = only_message(n());
        assert!(msg.contains("on `main` with 1 uncommitted"), "{msg}");
        assert!(msg.contains("would be refused"), "{msg}");
        git(&human, &["checkout", "-q", "-b", "work"]);
        assert!(
            n().is_empty(),
            "dirty on another branch: main can be pushed"
        );
    }

    #[test]
    fn non_path_local_urls_are_reported_not_resolved() {
        for url in [
            "https://example.com/me/repo.git",
            "ssh://git@example.com/me/repo.git",
            "git@example.com:me/repo.git",
        ] {
            let (repo, farm) = team_repo();
            let msg = only_message(local_failures(&repo, &farm, url));
            assert!(
                msg.contains(url) && msg.contains("not a path on this machine"),
                "{msg}"
            );
        }
        assert!(!is_remote_url("/tmp/x/.git"));
        assert!(!is_remote_url("../sibling"));
        assert!(!is_remote_url("file:///tmp/x"));
    }
}
