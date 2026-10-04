//! `yaks preflight`: a read-only landing-readiness check for a team farm.
//!
//! Run before landing (committing a shorn yak, merging a lane). It reports the
//! slips that earlier landings hit: a new artifacts directory that was never
//! `git add`ed, yak edits left unstaged, a shorn yak whose `verify:` command did
//! not last PASS, and a yak sitting in two status dirs. One line per failure;
//! it never mutates the farm or the repo.

use crate::farm::{Farm, IssueKind};
use crate::model::Status;
use crate::store;
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::path::Path;

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
}

impl Check {
    pub fn code(self) -> &'static str {
        match self {
            Check::GitState => "git-state",
            Check::Verify => "verify",
            Check::DuplicateStatus => "duplicate-status",
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
pub fn run(farm: &Farm, ids: &[String], all: bool) -> Result<Report> {
    let mut report = Report::default();
    let changed = git_state(farm.root(), &mut report)?;
    let scope = Scope { ids, all, changed };
    verify(farm, &scope, &mut report)?;
    duplicate_status(farm, &mut report)?;
    Ok(report)
}

/// Check 1: nothing under the farm is untracked or has unstaged changes.
/// Staged changes pass: staging is the step right before the landing commit.
fn git_state(root: &Path, report: &mut Report) -> Result<Option<BTreeSet<String>>> {
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .context("running git")
    };
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
        run(farm, &ids, false)
            .unwrap()
            .failures
            .into_iter()
            .filter(|f| f.check == check)
            .collect()
    }

    #[test]
    fn clean_team_repo_is_ok() {
        let (_repo, farm) = team_repo();
        let report = run(&farm, &[], false).unwrap();
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
        assert!(run(&farm, &[], false).unwrap().ok(), "staged passes");
    }

    #[test]
    fn unstaged_yak_edit_fails_and_passes_once_staged() {
        let (repo, farm) = team_repo();
        store::write::save(&repo.join(".yaks"), &task("yak-0001", Status::Hairy)).unwrap();
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "add yak"]);
        assert!(run(&farm, &[], false).unwrap().ok());
        let file = repo.join(".yaks/hairy/yak-0001.md");
        let text = std::fs::read_to_string(&file).unwrap();
        std::fs::write(&file, format!("{text}\nedited\n")).unwrap();
        let f = failures(&farm, &[], Check::GitState);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("yak-0001.md") && f[0].message.contains("unstaged"));
        git(&repo, &["add", ".yaks"]);
        assert!(run(&farm, &[], false).unwrap().ok());
    }

    #[test]
    fn changes_outside_the_farm_are_not_flagged() {
        let (repo, farm) = team_repo();
        std::fs::write(repo.join("code.rs"), "fn main() {}").unwrap();
        assert!(run(&farm, &[], false).unwrap().ok());
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
        let report = run(&farm, &[], false).unwrap();
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
            run(&farm, &[], false).unwrap().ok(),
            "old yak is out of scope"
        );
        let all = run(&farm, &[], true).unwrap();
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
        assert!(run(&farm, &[], false).unwrap().ok());
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
}
