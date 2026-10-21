//! `yaks migrate --to-branch`: move a repository's committed in-tree `.yaks/`
//! farm onto an orphan-tree farm branch in its own worktree (yaks-bd7e), the
//! by-hand cutover of yaks-87a3 as one command.
//!
//! The branch starts at HEAD, so every past commit of the farm stays reachable
//! from it (`yaks commits <id>`, `git log --follow`); its first commit drops
//! everything but the top-level `.yaks/`. The worktree is created with no
//! checkout and then filled from that commit, so nothing else (a nested
//! fixture `.yaks/`, the code) ever lands on disk there. The current branch
//! then gets one commit removing `.yaks/`, and the ignored leftovers (`.lock`)
//! go too. Discovery ([`crate::store::discover`]) then finds the farm through
//! the branch's worktree from every checkout. Nothing is pushed.
//!
//! Every refusal is checked before the first change, so a refused run changes
//! nothing.

use crate::{changes, sheds, store};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What the user asked for.
pub struct Opts {
    /// `--branch`; else `git config yaks.branch`, else `yaks`.
    pub branch: Option<String>,
    /// `--worktree`; else [`store::farm_home`].
    pub worktree: Option<PathBuf>,
    pub dry_run: bool,
}

/// The result of a run that was not refused.
#[derive(Debug)]
pub enum Outcome {
    /// The farm is already on a branch: nothing to do.
    AlreadyBranch {
        branch: String,
        worktree: Option<PathBuf>,
    },
    /// The plan; `applied` says whether it ran.
    Moved(Plan),
}

#[derive(Debug)]
pub struct Plan {
    /// The code checkout (its git top-level).
    pub repo: PathBuf,
    /// The branch the code checkout is on, which gets the removal commit.
    pub code_branch: String,
    pub branch: String,
    pub worktree: PathBuf,
    /// Tracked files under `.yaks/`.
    pub files: usize,
    /// `git config yaks.branch <branch>` is (to be) set in the repository.
    pub set_config: bool,
    pub applied: bool,
    /// Short SHAs, after a real run.
    pub farm_commit: Option<String>,
    pub code_commit: Option<String>,
}

/// Run git in `dir` with a clean environment; trimmed stdout, or an error
/// carrying git's stderr.
pub(crate) fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub(crate) fn ref_exists(dir: &Path, name: &str) -> bool {
    git(dir, &["rev-parse", "--verify", "-q", name]).is_ok()
}

/// Move the farm of the checkout containing `cwd` onto a branch. `home` gives
/// the default worktree location for the repository's top-level when
/// `--worktree` is not set: [`store::farm_home`] (tests pass a data dir in).
pub fn run(cwd: &Path, opts: &Opts, home: impl FnOnce(&Path) -> PathBuf) -> Result<Outcome> {
    let top = git(cwd, &["rev-parse", "--show-toplevel"])
        .map(PathBuf::from)
        .map_err(|_| anyhow::anyhow!("not in a git repository: {}", cwd.display()))?;
    let top = sheds::canonical(&top);
    let farm = top.join(".yaks");

    // Already on a branch: this checkout IS a farm worktree, or it has no
    // in-tree farm and its repository has a farm branch.
    if let Some(b) = store::farm_branch(&farm) {
        return Ok(Outcome::AlreadyBranch {
            branch: b,
            worktree: Some(top),
        });
    }
    if farm.is_file() || farm.is_symlink() {
        bail!(
            "{} is a pointer or a link to a farm outside this checkout, not a committed farm; \
             nothing to move",
            farm.display()
        );
    }
    let in_tree = farm.is_dir() && !store::only_leftovers(&farm);
    let configured = store::configured_farm_branch(&top);
    if !in_tree {
        // The asked-for branch, else the one discovery resolves.
        for b in opts.branch.iter().chain([&configured]) {
            if ref_exists(&top, &format!("refs/heads/{b}"))
                || ref_exists(&top, &format!("refs/remotes/origin/{b}"))
            {
                return Ok(Outcome::AlreadyBranch {
                    worktree: sheds::worktree_of_branch(&top, b),
                    branch: b.clone(),
                });
            }
        }
        bail!("no committed .yaks/ farm in {} to move", top.display());
    }

    // Refusals, all before any change.
    let host = sheds::host(&top).context("reading the repository")?;
    if host.via_alternates {
        let main = sheds::host_checkout(&host).map_or_else(
            || host.git_dir.display().to_string(),
            |p| p.display().to_string(),
        );
        bail!(
            "this checkout borrows another repository's objects (a Delta clone); a farm worktree \
             made here would be invisible to that repository's other checkouts. Run \
             `yaks migrate --to-branch` in its main checkout: {main}"
        );
    }
    if git(&top, &["ls-files", "--", ":(top).yaks"])?.is_empty() {
        bail!(
            "{} is not tracked by git (a private farm); only a committed farm moves to a branch",
            farm.display()
        );
    }
    if let Some((head, what)) = changes::pending_operation(&top)? {
        bail!("a {what} is in progress ({head} is set); finish or abort it first");
    }
    let dirty = git(&top, &["status", "--porcelain", "--untracked-files=all"])?;
    if !dirty.is_empty() {
        let n = dirty.lines().count();
        bail!(
            "the checkout has {n} uncommitted change(s) (untracked files included); commit or \
             stash them first:\n{dirty}"
        );
    }
    let code_branch = git(&top, &["symbolic-ref", "--short", "-q", "HEAD"])
        .map_err(|_| anyhow::anyhow!("HEAD is detached; check out the branch whose farm moves"))?;
    let branch = opts.branch.clone().unwrap_or_else(|| configured.clone());
    if git(&top, &["check-ref-format", "--branch", &branch]).is_err() {
        bail!("`{branch}` is not a valid branch name");
    }
    if branch == code_branch {
        bail!("`{branch}` is the branch you are on; pick another with --branch");
    }
    if ref_exists(&top, &format!("refs/heads/{branch}")) {
        bail!("the branch `{branch}` already exists; pick another with --branch");
    }
    if ref_exists(&top, &format!("refs/remotes/origin/{branch}")) {
        bail!(
            "origin already has a branch `{branch}`; if it is the farm, add its worktree \
             (`git worktree add <path> {branch}`), else pick another name with --branch"
        );
    }
    let worktree = match &opts.worktree {
        Some(p) => store::expand_path(cwd, &p.to_string_lossy()),
        None => home(&top),
    };
    if fs::symlink_metadata(&worktree).is_ok() {
        bail!(
            "{} already exists; remove it or pick another path with --worktree",
            worktree.display()
        );
    }
    let files = git(&top, &["ls-files", "--", ":(top).yaks"])?
        .lines()
        .count();
    // Discovery must find the branch: record it unless it is already what
    // the repository resolves (the default `yaks`, or an earlier setting).
    let set_config = configured != branch;
    let mut plan = Plan {
        repo: top.clone(),
        code_branch,
        branch,
        worktree,
        files,
        set_config,
        applied: false,
        farm_commit: None,
        code_commit: None,
    };
    if opts.dry_run {
        return Ok(Outcome::Moved(plan));
    }
    apply(&mut plan)?;
    Ok(Outcome::Moved(plan))
}

fn apply(plan: &mut Plan) -> Result<()> {
    let (top, wt, b) = (&plan.repo, &plan.worktree, plan.branch.as_str());
    if let Some(parent) = wt.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let wt_s = wt.to_string_lossy().into_owned();
    git(
        top,
        &["worktree", "add", "--no-checkout", "-b", b, &wt_s, "HEAD"],
    )?;
    // The farm side. On failure, undo it (the code checkout is untouched).
    let farm_side = || -> Result<String> {
        git(wt, &["read-tree", "HEAD"])?;
        git(
            wt,
            &[
                "rm",
                "-r",
                "-q",
                "--cached",
                "--ignore-unmatch",
                "--",
                ":(top)",
                ":(top,exclude).yaks",
            ],
        )?;
        git(
            wt,
            &[
                "commit",
                "-q",
                "--no-verify",
                "--allow-empty",
                "-m",
                &format!("yaks: branch {b} keeps only the farm (.yaks/)"),
            ],
        )?;
        git(wt, &["checkout", "HEAD", "--", "."])?;
        git(wt, &["rev-parse", "--short", "HEAD"])
    };
    let farm_commit = match farm_side() {
        Ok(c) => c,
        Err(e) => {
            let _ = git(top, &["worktree", "remove", "--force", &wt_s]);
            let _ = git(top, &["branch", "-D", b]);
            return Err(e.context("preparing the farm branch (undone; nothing changed)"));
        }
    };
    plan.farm_commit = Some(farm_commit);
    if plan.set_config {
        git(top, &["config", "yaks.branch", b])?;
    }
    // The code side: one commit removing the farm, then its ignored leftovers.
    let code = || -> Result<String> {
        git(top, &["rm", "-r", "-q", "--", ":(top).yaks"])?;
        git(
            top,
            &[
                "commit",
                "-q",
                "--no-verify",
                "-m",
                &format!("yaks: the farm moves to branch {b}"),
            ],
        )?;
        git(top, &["rev-parse", "--short", "HEAD"])
    };
    let code_commit = code().with_context(|| {
        format!(
            "the farm branch `{b}` is ready at {}, but removing .yaks/ from `{}` failed; \
             finish by hand: `git rm -r .yaks && git commit`",
            wt.display(),
            plan.code_branch
        )
    })?;
    plan.code_commit = Some(code_commit);
    let leftover = top.join(".yaks");
    if leftover.exists() {
        fs::remove_dir_all(&leftover)
            .with_context(|| format!("removing the leftover {}", leftover.display()))?;
    }
    plan.applied = true;
    Ok(())
}

/// Give a repository whose farm branch has no worktree one (yaks-a260):
/// `git worktree add` of `branch` at `home(top)`, where `top` is the git
/// top-level of the checkout discovery started in. A branch that exists only
/// as `origin/<branch>` (a fresh clone) becomes a local branch tracking it,
/// so `git pull`/`push` in the farm worktree work with no setup. Never
/// fetches, and never touches the checkout's own branch or files. Returns the
/// worktree.
///
/// Refuses a checkout that borrows another repository's objects (a Delta
/// clone): a worktree made here would be invisible to that repository's other
/// checkouts, the same refusal as `migrate --to-branch`.
pub fn bootstrap(top: &Path, branch: &str, home: impl FnOnce(&Path) -> PathBuf) -> Result<PathBuf> {
    let host = sheds::host(top).context("reading the repository")?;
    if host.via_alternates {
        let main = sheds::host_checkout(&host).map_or_else(
            || {
                format!(
                    "{} (no checkout there: set YAKS_DIR to the farm)",
                    host.git_dir.display()
                )
            },
            |p| p.display().to_string(),
        );
        bail!(
            "this repository's farm is the branch `{branch}`, but no worktree has it checked out, \
             and this checkout borrows another repository's objects (a Delta clone), so it \
             cannot create one for the others. Run any yaks command in its main checkout to \
             create the worktree: {main}"
        );
    }
    let wt = home(top);
    if fs::symlink_metadata(&wt).is_ok() {
        bail!(
            "the farm branch `{branch}` has no worktree, and the place for one, {}, already \
             exists. Remove it, or add the worktree elsewhere: `git -C {} worktree add <path> \
             {branch}`",
            wt.display(),
            top.display()
        );
    }
    if let Some(parent) = wt.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let wt_s = wt.to_string_lossy().into_owned();
    // Forget a record of a worktree whose directory was deleted, or git
    // refuses to check the branch out a second time.
    git(top, &["worktree", "prune"])?;
    if ref_exists(top, &format!("refs/heads/{branch}")) {
        git(top, &["worktree", "add", "-q", &wt_s, branch])?;
    } else {
        git(
            top,
            &[
                "worktree",
                "add",
                "-q",
                "--track",
                "-b",
                branch,
                &wt_s,
                &format!("origin/{branch}"),
            ],
        )?;
    }
    Ok(sheds::canonical(&wt))
}

/// The report `yaks migrate --to-branch` prints.
pub fn render(o: &Outcome) -> String {
    let p = match o {
        Outcome::AlreadyBranch { branch, worktree } => {
            return match worktree {
                Some(w) => format!(
                    "the farm is already on the branch `{branch}`, at {}; nothing to do\n",
                    w.join(".yaks").display()
                ),
                None => format!(
                    "the farm is already on the branch `{branch}` (no worktree has it yet: \
                     `git worktree add <path> {branch}`); nothing to do\n"
                ),
            };
        }
        Outcome::Moved(p) => p,
    };
    let (repo, wt, b) = (p.repo.display(), p.worktree.display(), &p.branch);
    let mut s = String::new();
    if !p.applied {
        s.push_str(&format!(
            "would move the farm ({} tracked file(s) in {repo}/.yaks) to the branch `{b}`:\n\
             - create `{b}` from HEAD in a new worktree at {wt}, its first commit keeping only \
             .yaks/ (the farm's history stays reachable)\n",
            p.files
        ));
        if p.set_config {
            s.push_str(&format!("- set `git config yaks.branch {b}` in {repo}\n"));
        }
        s.push_str(&format!(
            "- commit the removal of .yaks/ on `{}` (\"yaks: the farm moves to branch {b}\"), \
             then delete the leftover .yaks/ directory\n\
             (dry run: nothing changed; re-run without --dry-run to move it)\n",
            p.code_branch
        ));
        return s;
    }
    s.push_str(&format!(
        "moved the farm to the branch `{b}`: it is now {wt}/.yaks\n\
         - `{b}` {}: keeps only .yaks/ ({} file(s)), with the farm's history\n\
         - `{}` {}: removes .yaks/\n",
        p.farm_commit.as_deref().unwrap_or("?"),
        p.files,
        p.code_branch,
        p.code_commit.as_deref().unwrap_or("?"),
    ));
    if p.set_config {
        s.push_str(&format!("- set `git config yaks.branch {b}`\n"));
    }
    s.push_str(&format!(
        "Nothing was pushed. To publish, push both:\n  \
         yaks sync    (publishes the farm branch `{b}` to origin)\n  \
         git -C {repo} push origin {}\n\
         On another machine or for a teammate, a clone needs no setup: the first yaks \
         command in it creates the farm worktree from `origin/{b}`.\n",
        p.code_branch
    ));
    if p.set_config {
        s.push_str(&format!("  git config yaks.branch {b}\n"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn base(tag: &str) -> PathBuf {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "yaks-branchfarm-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        sheds::canonical(&p)
    }

    fn sh(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args([
                "-c",
                "commit.gpgsign=false",
                "-c",
                "init.defaultBranch=main",
            ])
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} in {}: {}",
            dir.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn write(p: &Path, text: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }

    /// A repo `r` on `main` with a committed farm (two commits touching
    /// `yaks-aaaa`), code, and a nested `x/.yaks/` fixture farm; the ignored
    /// `.yaks/.lock` is on disk, as in a live farm.
    fn repo(b: &Path) -> PathBuf {
        let r = b.join("r");
        fs::create_dir_all(&r).unwrap();
        sh(&r, &["init", "-q"]);
        sh(&r, &["config", "user.name", "t"]);
        sh(&r, &["config", "user.email", "t@t"]);
        sh(&r, &["config", "commit.gpgsign", "false"]);
        write(&r.join(".yaks/schema"), &format!("{}\n", store::SCHEMA));
        write(&r.join(".yaks/.gitignore"), ".lock\n");
        write(&r.join(".yaks/config.yaml"), "prefix: t\n");
        write(
            &r.join(".yaks/yaks-aaaa.md"),
            "---\nid: yaks-aaaa\ntitle: a\nstatus: hairy\n---\n",
        );
        write(&r.join("src/main.rs"), "fn main() {}\n");
        write(&r.join("x/.yaks/yaks-nest.md"), "nested fixture\n");
        sh(&r, &["add", "-A"]);
        sh(&r, &["commit", "-q", "-m", "one"]);
        write(
            &r.join(".yaks/yaks-aaaa.md"),
            "---\nid: yaks-aaaa\ntitle: a\nstatus: shaving\n---\n",
        );
        sh(&r, &["commit", "-q", "-am", "two"]);
        write(&r.join(".yaks/.lock"), "");
        r
    }

    fn opts(wt: &Path, dry_run: bool) -> Opts {
        Opts {
            branch: None,
            worktree: Some(wt.to_path_buf()),
            dry_run,
        }
    }

    /// The default location, never the real data dir.
    fn no_env(r: &Path) -> PathBuf {
        store::farm_home_with(r, |_| None::<String>).with_file_name("unused-in-tests")
    }

    /// Every path under `dir` (relative), `.git` contents excluded.
    fn tree(dir: &Path) -> BTreeSet<String> {
        fn walk(root: &Path, d: &Path, out: &mut BTreeSet<String>) {
            for e in fs::read_dir(d).unwrap().flatten() {
                let p = e.path();
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                out.insert(rel.clone());
                if p.is_dir() && rel != ".git" {
                    walk(root, &p, out);
                }
            }
        }
        let mut out = BTreeSet::new();
        if dir.exists() {
            walk(dir, dir, &mut out);
        }
        out
    }

    /// Everything a refused or dry run must leave alone.
    fn snapshot(r: &Path) -> (String, String, String, BTreeSet<String>, String) {
        (
            sh(r, &["rev-parse", "HEAD"]),
            sh(r, &["for-each-ref", "--format=%(refname) %(objectname)"]),
            sh(r, &["worktree", "list", "--porcelain"]),
            tree(r),
            sh(r, &["config", "--list", "--local"]),
        )
    }

    #[test]
    fn moves_the_farm_keeping_history() {
        let b = base("move");
        let r = repo(&b);
        let wt = b.join("farm");
        let out = run(&r, &opts(&wt, false), no_env).unwrap();
        let Outcome::Moved(p) = &out else {
            panic!("{out:?}")
        };
        assert!(p.applied && !p.set_config, "{p:?}");
        let text = render(&out);
        assert!(
            text.contains("yaks sync    (publishes the farm branch `yaks` to origin)"),
            "{text}"
        );
        assert!(text.contains("a clone needs no setup"), "{text}");
        assert!(!text.contains("git worktree add <path>"), "{text}");

        // main: no .yaks/ tracked, and none on disk (the .lock leftover too).
        assert_eq!(sh(&r, &["ls-files", ".yaks"]), "");
        assert!(!r.join(".yaks").exists());
        assert_eq!(
            sh(&r, &["log", "-1", "--format=%s"]),
            "yaks: the farm moves to branch yaks"
        );
        assert_eq!(sh(&r, &["status", "--porcelain"]), "");
        // The nested fixture is code, and stays on main.
        assert!(r.join("x/.yaks/yaks-nest.md").is_file());

        // The branch tree is only .yaks/; the worktree holds only .git + .yaks.
        let files = sh(&r, &["ls-tree", "-r", "--name-only", "yaks"]);
        assert!(files.lines().all(|f| f.starts_with(".yaks/")), "{files}");
        assert!(files.contains(".yaks/yaks-aaaa.md"));
        let top: BTreeSet<String> = fs::read_dir(&wt)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(top, [".git", ".yaks"].map(String::from).into());
        assert_eq!(sh(&wt, &["status", "--porcelain", "--ignored"]), "");

        // History: both old commits of the yak are reachable on the branch.
        let log = sh(
            &wt,
            &["log", "--format=%s", "--follow", "--", ".yaks/yaks-aaaa.md"],
        );
        assert_eq!(log.lines().collect::<Vec<_>>(), ["two", "one"], "{log}");

        // Discovery from main, a subdir, and a --shared clone finds the worktree.
        let want = sheds::canonical(&wt.join(".yaks"));
        let found = |d: &Path| sheds::canonical(&store::discover_with(d, None).unwrap().root);
        assert_eq!(found(&r), want);
        assert_eq!(found(&r.join("src")), want);
        let clone = b.join("clone");
        sh(
            &b,
            &[
                "clone",
                "-q",
                "--shared",
                r.to_str().unwrap(),
                clone.to_str().unwrap(),
            ],
        );
        assert_eq!(found(&clone), want);

        // Idempotent: from main and from the worktree it says so.
        let again = run(&r, &opts(&b.join("other"), false), no_env).unwrap();
        assert!(
            matches!(&again, Outcome::AlreadyBranch { branch, .. } if branch == "yaks"),
            "{again:?}"
        );
        let inside = run(&wt, &opts(&b.join("other"), false), no_env).unwrap();
        assert!(matches!(inside, Outcome::AlreadyBranch { .. }));
        let named = Opts {
            branch: Some("other".into()),
            ..opts(&b.join("other"), false)
        };
        let named = run(&r, &named, no_env).unwrap();
        assert!(
            matches!(&named, Outcome::AlreadyBranch { branch, .. } if branch == "yaks"),
            "{named:?}"
        );
        assert!(!b.join("other").exists());
    }

    #[test]
    fn another_branch_name_is_recorded_for_discovery() {
        let b = base("named");
        let r = repo(&b);
        let wt = b.join("farm");
        let o = Opts {
            branch: Some("tracker".into()),
            ..opts(&wt, false)
        };
        let Outcome::Moved(p) = run(&r, &o, no_env).unwrap() else {
            panic!()
        };
        assert!(p.set_config);
        assert_eq!(sh(&r, &["config", "yaks.branch"]), "tracker");
        let found = store::discover_with(&r, None).unwrap().root;
        assert_eq!(
            sheds::canonical(&found),
            sheds::canonical(&wt.join(".yaks"))
        );
    }

    #[test]
    fn dry_run_changes_nothing() {
        let b = base("dry");
        let r = repo(&b);
        let wt = b.join("farm");
        let before = snapshot(&r);
        let out = run(&r, &opts(&wt, true), no_env).unwrap();
        let Outcome::Moved(p) = &out else { panic!() };
        assert!(!p.applied);
        assert_eq!(p.files, 4);
        assert!(render(&out).contains("dry run: nothing changed"));
        assert_eq!(snapshot(&r), before);
        assert!(!wt.exists());
    }

    #[test]
    fn default_worktree_is_the_farm_home() {
        let b = base("home");
        let r = repo(&b);
        let data = b.join("data");
        let d = data.to_string_lossy().into_owned();
        let env = move |k: &str| (k == "XDG_DATA_HOME").then(|| d.clone());
        let o = Opts {
            branch: None,
            worktree: None,
            dry_run: false,
        };
        let Outcome::Moved(p) = run(&r, &o, |t| store::farm_home_with(t, &env)).unwrap() else {
            panic!()
        };
        assert_eq!(p.worktree, store::farm_home_with(&r, &env));
        assert!(p.worktree.starts_with(data.join("yaks/farms")));
        assert!(p.worktree.join(".yaks/yaks-aaaa.md").is_file());
    }

    #[test]
    fn farm_home_honours_xdg_data_home_then_home() {
        let b = base("fh");
        let r = repo(&b);
        let xdg = store::farm_home_with(&r, |k| (k == "XDG_DATA_HOME").then(|| "/xdg".to_string()));
        let name = xdg.file_name().unwrap().to_string_lossy().into_owned();
        assert_eq!(xdg.parent().unwrap(), Path::new("/xdg/yaks/farms"));
        assert!(
            name.starts_with("r-") && name.len() == "r-".len() + 8,
            "{name}"
        );
        // Empty XDG_DATA_HOME counts as unset: HOME/.local/share.
        let home = store::farm_home_with(&r, |k| match k {
            "XDG_DATA_HOME" => Some(String::new()),
            "HOME" => Some("/h".into()),
            _ => None,
        });
        assert_eq!(home, Path::new("/h/.local/share/yaks/farms").join(&name));
        // Stable: every checkout of the repository agrees, and a subdir too.
        let sub = store::farm_home_with(&r.join("src"), |k| {
            (k == "XDG_DATA_HOME").then(|| "/xdg".to_string())
        });
        assert_eq!(sub, xdg);
    }

    /// Run, expect a refusal mentioning `why`, and that nothing changed.
    fn refuses(r: &Path, o: &Opts, why: &str) {
        let before = snapshot(r);
        let err = format!("{:#}", run(r, o, no_env).unwrap_err());
        assert!(err.contains(why), "want {why:?}, got: {err}");
        assert_eq!(snapshot(r), before);
        if let Some(wt) = &o.worktree {
            assert!(!wt.exists() || fs::read_dir(wt).unwrap().next().is_some());
        }
    }

    #[test]
    fn refusals_change_nothing() {
        let b = base("refuse");
        let wt = b.join("farm");

        // Uncommitted changes: a modified file, then an untracked one.
        let r = repo(&b);
        write(&r.join("src/main.rs"), "fn main() { dirty() }\n");
        refuses(&r, &opts(&wt, false), "uncommitted change");
        sh(&r, &["checkout", "--", "src/main.rs"]);
        write(&r.join("new.txt"), "x\n");
        refuses(&r, &opts(&wt, false), "uncommitted change");
        fs::remove_file(r.join("new.txt")).unwrap();

        // The branch already exists.
        sh(&r, &["branch", "yaks"]);
        refuses(&r, &opts(&wt, false), "already exists");
        sh(&r, &["branch", "-D", "yaks"]);

        // The worktree path already exists.
        fs::create_dir_all(wt.join("something")).unwrap();
        refuses(&r, &opts(&wt, false), "already exists");
        fs::remove_dir_all(&wt).unwrap();

        // A merge pending.
        sh(&r, &["checkout", "-q", "-b", "side"]);
        write(&r.join("src/main.rs"), "fn main() { side() }\n");
        sh(&r, &["commit", "-q", "-am", "side"]);
        sh(&r, &["checkout", "-q", "main"]);
        write(&r.join("src/main.rs"), "fn main() { main() }\n");
        sh(&r, &["commit", "-q", "-am", "main"]);
        let merged = Command::new("git")
            .arg("-C")
            .arg(&r)
            .args(["-c", "user.name=t", "-c", "user.email=t@t", "merge", "side"])
            .output()
            .unwrap();
        assert!(!merged.status.success());
        refuses(&r, &opts(&wt, false), "merge is in progress");

        // A private (untracked, ignored) farm.
        let p = b.join("p");
        fs::create_dir_all(&p).unwrap();
        sh(&p, &["init", "-q"]);
        write(&p.join(".gitignore"), ".yaks/\n");
        write(&p.join(".yaks/schema"), "5\n");
        sh(&p, &["add", "-A"]);
        sh(&p, &["commit", "-q", "-m", "private"]);
        refuses(&p, &opts(&wt, false), "not tracked");
        assert!(!wt.exists());
    }

    #[test]
    fn a_shared_clone_is_sent_to_its_host() {
        let b = base("clone");
        let r = repo(&b);
        let clone = b.join("clone");
        sh(
            &b,
            &[
                "clone",
                "-q",
                "--shared",
                r.to_str().unwrap(),
                clone.to_str().unwrap(),
            ],
        );
        refuses(&clone, &opts(&b.join("farm"), false), "main checkout");
    }
}
