//! `yaks sync` (yaks-e3b2): commit the farm, rebase it onto its remote farm
//! branch, push it. The one command that moves a farm branch between machines
//! and teammates.
//!
//! Runs in the farm's own worktree whatever the cwd, like `yaks commit`. Each
//! step can be skipped (`--no-commit`, `--no-pull`, `--no-push`), so the
//! half-syncs are the same command. The rebase relies on the yak-file merge
//! driver ([`crate::mergedriver`]), which this registers first, so concurrent
//! edits to one yak merge into a valid yak instead of conflicting. A farm
//! marked private ([`crate::privatefarm::is_private`]) is never pushed.
//!
//! Nothing is automatic: yaks commits and syncs only when asked (yaks-472e).

use crate::{commit, mergedriver, privatefarm, status, store};
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// What to do: every step unless skipped.
#[derive(Debug, Default, Clone)]
pub struct Opts {
    pub no_commit: bool,
    pub no_pull: bool,
    pub no_push: bool,
    /// `-m`: the commit message (else the generated one).
    pub message: Option<String>,
}

/// One line per step, in order, and where the branch ended up.
#[derive(Debug, Default)]
pub struct Report {
    pub lines: Vec<String>,
    pub state: Option<status::SyncState>,
}

impl Report {
    fn say(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    pub fn render(&self) -> String {
        let mut s = String::new();
        for l in &self.lines {
            s.push_str("sync: ");
            s.push_str(l);
            s.push('\n');
        }
        if let Some(st) = &self.state {
            s.push_str(&st.line());
            s.push('\n');
        }
        s
    }
}

fn git(dir: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))
}

fn ok(dir: &Path, args: &[&str]) -> Result<String> {
    let out = git(dir, args)?;
    if !out.status.success() {
        bail!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn line(dir: &Path, args: &[&str]) -> Option<String> {
    ok(dir, args).ok().filter(|s| !s.is_empty())
}

/// The farm's worktree (the git top-level holding `root`) and its branch.
/// Refuses a farm that is not on its own branch.
fn farm_worktree(root: &Path) -> Result<(PathBuf, String)> {
    let Some(branch) = store::farm_branch(root) else {
        bail!(
            "{} is not on its own farm branch, so there is nothing for `yaks sync` to do: \
             commit it with `yaks commit` and push it with the code (or move it onto a branch \
             with `yaks migrate --to-branch`)",
            root.display()
        );
    };
    let top = root
        .parent()
        .context("the farm has no parent directory")?
        .to_path_buf();
    Ok((top, branch))
}

/// The remote the farm branch syncs with: its configured `branch.<b>.remote`,
/// else `origin`; `None` when that remote does not exist.
fn remote_of(top: &Path, branch: &str) -> Option<String> {
    let name = line(
        top,
        &["config", "--get", &format!("branch.{branch}.remote")],
    )
    .unwrap_or_else(|| "origin".to_string());
    line(top, &["remote", "get-url", &name]).map(|_| name)
}

/// Fetch `branch` from `remote`. `Ok(false)` when the remote has no such
/// branch yet (a farm never published).
fn fetch(top: &Path, remote: &str, branch: &str) -> Result<bool> {
    let heads = ok(top, &["ls-remote", "--heads", remote, branch])?;
    if heads.is_empty() {
        return Ok(false);
    }
    ok(
        top,
        &[
            "fetch",
            "-q",
            remote,
            &format!("+refs/heads/{branch}:refs/remotes/{remote}/{branch}"),
        ],
    )?;
    Ok(true)
}

/// Rebase the farm branch onto `<remote>/<branch>`. A rebase that stops is
/// aborted (the branch is left exactly as it was) and reported with the
/// conflicting paths. Returns how many remote commits were brought in.
fn rebase(top: &Path, remote: &str, branch: &str) -> Result<usize> {
    let upstream = format!("{remote}/{branch}");
    let behind: usize = ok(top, &["rev-list", "--count", &format!("HEAD..{upstream}")])?
        .parse()
        .unwrap_or(0);
    if behind == 0 {
        return Ok(0);
    }
    let mut args = identity_fallback(top);
    args.extend(["rebase".to_string(), "-q".to_string(), upstream.clone()]);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = git(top, &args)?;
    if !out.status.success() {
        let conflicts = line(top, &["diff", "--name-only", "--diff-filter=U"]).unwrap_or_default();
        let _ = git(top, &["rebase", "--abort"]);
        bail!(
            "rebasing the farm branch onto {upstream} stopped on a conflict the yak merge driver \
             cannot settle{}; nothing changed (the rebase was aborted). Resolve it by hand in {}: \
             `git rebase {upstream}`, fix the files, `git rebase --continue`, then `yaks sync`",
            if conflicts.is_empty() {
                String::new()
            } else {
                format!(" ({})", conflicts.replace('\n', ", "))
            },
            top.display()
        );
    }
    Ok(behind)
}

/// `-c user.name=… -c user.email=…` for a repository with no committer
/// identity configured (a fresh machine, CI), so the rebase can rewrite
/// commits; nothing when one is set, which then stays the committer.
fn identity_fallback(top: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for (key, value) in [("user.name", "yaks"), ("user.email", "yaks@localhost")] {
        if line(top, &["config", "--get", key]).is_none() {
            out.push("-c".to_string());
            out.push(format!("{key}={value}"));
        }
    }
    out
}

/// Push the farm branch, creating the remote branch (same name) and setting
/// the upstream on the first publish. `Ok(false)` when the remote rejected it
/// as not a fast-forward (it moved since the fetch).
fn push(top: &Path, remote: &str, branch: &str) -> Result<bool> {
    let out = git(
        top,
        &["push", "-q", "-u", remote, &format!("{branch}:{branch}")],
    )?;
    if out.status.success() {
        return Ok(true);
    }
    let err = String::from_utf8_lossy(&out.stderr);
    if err.contains("non-fast-forward") || err.contains("fetch first") || err.contains("rejected") {
        return Ok(false);
    }
    bail!("git push {remote} {branch}: {}", err.trim());
}

/// Commits on the farm branch its remote branch lacks (all of them when the
/// remote has none).
fn unpublished(top: &Path, remote: &str, branch: &str) -> usize {
    let range = if line(
        top,
        &["rev-parse", "--verify", "-q", &format!("{remote}/{branch}")],
    )
    .is_some()
    {
        format!("{remote}/{branch}..HEAD")
    } else {
        "HEAD".to_string()
    };
    line(top, &["rev-list", "--count", &range])
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

/// Sync the farm at `root`: register the merge driver, commit, pull
/// (fetch + rebase), push; each step as `opts` allows.
pub fn run(root: &Path, opts: &Opts) -> Result<Report> {
    let (top, branch) = farm_worktree(root)?;
    let mut r = Report::default();

    // The driver must be in place before anything merges, and its
    // `.gitattributes` rule is committed on the branch below so every clone
    // merges yak files the same way.
    mergedriver::ensure_registered(&top)?;
    commit_gitattributes(&top, &mut r)?;

    if opts.no_commit {
        r.say("commit skipped (--no-commit)");
    } else {
        match commit::run(root, opts.message.as_deref(), false)? {
            commit::Outcome::Nothing => r.say("nothing to commit"),
            commit::Outcome::Done(plan) => r.say(format!(
                "committed {} {}",
                plan.commit.as_deref().unwrap_or("?"),
                plan.message
            )),
        }
    }

    let Some(remote) = remote_of(&top, &branch) else {
        r.say(format!(
            "no remote to sync with (the branch has no `branch.{branch}.remote` and there is no \
             `origin`); add one with `git -C {} remote add origin <url>`",
            top.display()
        ));
        r.state = status::sync_state(root);
        return Ok(r);
    };
    let private = privatefarm::is_private(&top);

    // A dirty farm cannot be rebased; with --no-commit the uncommitted edits
    // stay put and the pull is skipped rather than fail half-way.
    let dirty = !ok(
        &top,
        &["status", "--porcelain", "--", ".yaks", ".gitattributes"],
    )?
    .is_empty();
    let mut pulled = false;
    if opts.no_pull {
        r.say("pull skipped (--no-pull)");
    } else if dirty {
        r.say("pull skipped: the farm has uncommitted changes (run without --no-commit, or `yaks commit` first)");
    } else {
        pull(&top, &remote, &branch, root, &mut r)?;
        pulled = true;
    }

    if opts.no_push {
        r.say("push skipped (--no-push)");
    } else if private {
        r.say("push skipped: this farm is private (`yaks.private`; `yaks private --off` to publish it)");
    } else if unpublished(&top, &remote, &branch) == 0
        && line(
            top.as_path(),
            &["rev-parse", "--verify", "-q", &format!("{remote}/{branch}")],
        )
        .is_some()
    {
        r.say(format!("nothing to push: {remote}/{branch} has everything"));
    } else {
        let n = unpublished(&top, &remote, &branch);
        let mut pushed = push(&top, &remote, &branch)?;
        if !pushed && pulled {
            // The remote moved between our fetch and our push: one more round.
            r.say(format!("{remote}/{branch} moved meanwhile; pulling again"));
            pull(&top, &remote, &branch, root, &mut r)?;
            pushed = push(&top, &remote, &branch)?;
        }
        if !pushed {
            bail!(
                "{remote} rejected the push of {branch} (it has commits this farm lacks); run \
                 `yaks sync` again{}",
                if opts.no_pull {
                    " without --no-pull"
                } else {
                    ""
                }
            );
        }
        r.say(format!("pushed {n} commit(s) to {remote}/{branch}"));
    }
    r.state = status::sync_state(root);
    Ok(r)
}

/// Commit the farm branch's `.gitattributes` (the merge-driver rule
/// [`mergedriver::ensure_registered`] writes) when it is new or changed, so
/// every clone of the branch merges yak files the same way. Only that file.
fn commit_gitattributes(top: &Path, r: &mut Report) -> Result<()> {
    if ok(top, &["status", "--porcelain", "--", ".gitattributes"])?.is_empty() {
        return Ok(());
    }
    ok(top, &["add", "--", ".gitattributes"])?;
    let mut args = identity_fallback(top);
    args.extend(
        [
            "commit",
            "-q",
            "--only",
            "-m",
            "yaks: merge yak files with the yaks merge driver",
            "--",
            ".gitattributes",
        ]
        .map(String::from),
    );
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    ok(top, &args)?;
    r.say("committed .gitattributes (the yak merge-driver rule)");
    Ok(())
}

fn pull(top: &Path, remote: &str, branch: &str, root: &Path, r: &mut Report) -> Result<()> {
    if !fetch(top, remote, branch)? {
        r.say(format!(
            "{remote} has no {branch} branch yet: nothing to pull"
        ));
        return Ok(());
    }
    // Writers wait while the rebase rewrites the files under them.
    let _lock = store::lock(root)?;
    match rebase(top, remote, branch)? {
        0 => r.say(format!("up to date with {remote}/{branch}")),
        n => r.say(format!("pulled {n} commit(s) from {remote}/{branch}")),
    }
    Ok(())
}
