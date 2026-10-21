//! A farm marked private never leaves the machine through yaks or a stray
//! `git push` (yaks-85bb).
//!
//! `yaks private` sets `git config yaks.private true` in the repository and
//! installs a `pre-push` hook that refuses any push of the farm branch while
//! that flag is set. `yaks sync` (yaks-e3b2) calls [`is_private`] and refuses
//! to push; the hook covers the pushes yaks does not make (`git push origin
//! yaks`, `git push --all`). The hook re-reads both settings each time it
//! runs, so `yaks private --off` (which unsets the flag and removes our hook)
//! and a hand-edited `yaks.branch` need no reinstall.

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// First comment line of the hook we write; a `pre-push` hook without it is
/// someone else's and is never overwritten or removed.
const MARKER: &str = "# yaks-private-guard";

/// What [`install_guard`] did with the `pre-push` hook.
#[derive(Debug, PartialEq, Eq)]
pub enum GuardOutcome {
    /// Our hook is in place (written, or rewritten over an older copy of ours).
    Installed(PathBuf),
    /// A `pre-push` hook that is not ours already exists; it was left alone.
    Foreign(PathBuf),
}

const HOOK: &str = r#"#!/bin/sh
# yaks-private-guard: installed by `yaks private`; `yaks private --off` removes it.
# Refuses a push of the yaks farm branch while `git config yaks.private` is true.
branch=$(git config --get yaks.branch 2>/dev/null)
[ -n "$branch" ] || branch=yaks
private=$(git config --bool yaks.private 2>/dev/null)
while read -r local_ref local_sha remote_ref remote_sha; do
    if [ "$private" = true ] && [ "$local_ref" = "refs/heads/$branch" ]; then
        echo "yaks: refusing to push the farm branch '$branch': this farm is private (yaks.private)." >&2
        echo "yaks: run 'yaks private --off' first if you really mean to publish it." >&2
        cat >/dev/null
        exit 1
    fi
done
exit 0
"#;

fn git(repo: &Path, args: &[&str]) -> Option<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .ok()
}

fn git_ok(repo: &Path, args: &[&str]) -> Option<String> {
    git(repo, args)
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Whether the repository containing `repo` (and, for a Delta clone, its host,
/// as [`crate::store::configured_farm_branch`] reads `yaks.branch`) has its
/// farm marked private: `git config --bool yaks.private` is true.
pub fn is_private(repo: &Path) -> bool {
    let flag =
        |dir: &Path| git_ok(dir, &["config", "--bool", "yaks.private"]).as_deref() == Some("true");
    flag(repo)
        || crate::sheds::host(repo)
            .map(|h| flag(&h.git_dir))
            .unwrap_or(false)
}

/// The repository's hooks directory (`git rev-parse --git-path hooks`, which
/// honors `core.hooksPath` and is shared by every worktree).
fn hooks_dir(repo: &Path) -> Result<PathBuf> {
    let p = git_ok(repo, &["rev-parse", "--git-path", "hooks"])
        .with_context(|| format!("{} is not inside a git repository", repo.display()))?;
    Ok(repo.join(p)) // absolute stays absolute
}

fn is_ours(hook: &Path) -> bool {
    fs::read_to_string(hook).is_ok_and(|t| t.lines().any(|l| l.starts_with(MARKER)))
}

/// Write our `pre-push` hook into the repository's hooks directory. A
/// `pre-push` hook that is not ours is left untouched
/// ([`GuardOutcome::Foreign`]); ours is rewritten (idempotent).
pub fn install_guard(repo: &Path) -> Result<GuardOutcome> {
    let hook = hooks_dir(repo)?.join("pre-push");
    if hook.exists() && !is_ours(&hook) {
        return Ok(GuardOutcome::Foreign(hook));
    }
    fs::create_dir_all(hook.parent().unwrap())?;
    fs::write(&hook, HOOK).with_context(|| format!("writing {}", hook.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755))?;
    }
    Ok(GuardOutcome::Installed(hook))
}

/// Remove our `pre-push` hook; a foreign one stays. `Some(path)` when removed.
pub fn remove_guard(repo: &Path) -> Result<Option<PathBuf>> {
    let hook = hooks_dir(repo)?.join("pre-push");
    if hook.exists() && is_ours(&hook) {
        fs::remove_file(&hook).with_context(|| format!("removing {}", hook.display()))?;
        return Ok(Some(hook));
    }
    Ok(None)
}

/// `yaks private [--off]` for the repository at `repo`; returns the report to
/// print.
pub fn run(repo: &Path, off: bool) -> Result<String> {
    if git_ok(repo, &["rev-parse", "--git-dir"]).is_none() {
        bail!("{} is not inside a git repository", repo.display());
    }
    let branch = crate::store::configured_farm_branch(repo);
    if off {
        // Exit 5 means the key was not set: already off.
        git(repo, &["config", "--unset-all", "yaks.private"]);
        let removed = match remove_guard(repo)? {
            Some(p) => format!("farm is no longer private; removed {}\n", p.display()),
            None => "farm is no longer private; no yaks pre-push hook to remove\n".to_string(),
        };
        // `yaks.private` can also be set in the host (Delta) repository or a
        // global config, which this command does not touch.
        return Ok(if is_private(repo) {
            format!(
                "{removed}warning: yaks.private is still true from another git config (`git config --show-origin yaks.private`)\n"
            )
        } else {
            removed
        });
    }
    let set = git(repo, &["config", "yaks.private", "true"]);
    if !set.is_some_and(|o| o.status.success()) {
        bail!("could not set yaks.private in {}", repo.display());
    }
    Ok(match install_guard(repo)? {
        GuardOutcome::Installed(p) => format!(
            "farm is private: `yaks.private` is set (for `yaks sync`), and {} refuses any `git push` of '{branch}'\n",
            p.display()
        ),
        GuardOutcome::Foreign(p) => format!(
            "farm is private: `yaks.private` is set (for `yaks sync`).\n\
             warning: {} already exists and is not yaks's, so it was left alone and a manual `git push` of '{branch}' is NOT blocked.\n\
             To block it, fold the check below into that hook (read git's stdin once, and keep your own logic):\n\
             \n\
             {hook}",
            p.display(),
            hook = HOOK
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn base() -> PathBuf {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "yaks-private-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        crate::sheds::canonical(&p)
    }

    fn try_git(dir: &Path, args: &[&str]) -> std::process::Output {
        Command::new("git")
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
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .unwrap()
    }

    fn sh(dir: &Path, args: &[&str]) -> String {
        let out = try_git(dir, args);
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// A code checkout `r` on `main` with a bare `origin`, plus the farm
    /// branch `yaks` (an orphan holding one file) checked out as a worktree
    /// `farm`. Nothing is pushed yet.
    fn scratch() -> (PathBuf, PathBuf, PathBuf) {
        let b = base();
        let (r, origin, farm) = (b.join("r"), b.join("origin.git"), b.join("farm"));
        fs::create_dir_all(&r).unwrap();
        sh(&b, &["init", "-q", "--bare", "origin.git"]);
        sh(&r, &["init", "-q"]);
        fs::write(r.join("code"), "c").unwrap();
        sh(&r, &["add", "code"]);
        sh(&r, &["commit", "-q", "-m", "code"]);
        sh(&r, &["remote", "add", "origin", origin.to_str().unwrap()]);
        sh(
            &r,
            &[
                "worktree",
                "add",
                "-q",
                "--orphan",
                "-b",
                "yaks",
                farm.to_str().unwrap(),
            ],
        );
        fs::create_dir_all(farm.join(".yaks")).unwrap();
        fs::write(farm.join(".yaks/schema"), "1\n").unwrap();
        sh(&farm, &["add", "-A"]);
        sh(&farm, &["commit", "-q", "-m", "farm"]);
        (r, origin, farm)
    }

    fn remote_refs(origin: &Path) -> String {
        sh(origin, &["for-each-ref", "--format=%(refname)"])
    }

    #[test]
    fn private_blocks_both_pushes_of_the_farm_and_off_restores() {
        let (r, origin, farm) = scratch();
        // Before: the farm pushes (the failing-before baseline).
        assert!(
            try_git(&farm, &["push", "-q", "origin", "yaks"])
                .status
                .success()
        );
        sh(&origin, &["update-ref", "-d", "refs/heads/yaks"]);

        assert!(!is_private(&r));
        let report = run(&r, false).unwrap();
        assert!(report.starts_with("farm is private"), "{report}");
        assert!(
            is_private(&r) && is_private(&farm),
            "config is shared by worktrees"
        );

        let p = try_git(&farm, &["push", "origin", "yaks"]);
        assert!(!p.status.success());
        assert!(
            String::from_utf8_lossy(&p.stderr).contains("refusing to push the farm branch 'yaks'")
        );
        let p = try_git(&r, &["push", "origin", "--all"]);
        assert!(!p.status.success(), "push --all must be refused");
        assert_eq!(remote_refs(&origin), "", "remote unchanged: no main either");

        // `main` alone still pushes.
        sh(&r, &["push", "-q", "origin", "main"]);
        assert_eq!(remote_refs(&origin), "refs/heads/main");

        // Idempotent.
        assert!(run(&r, false).unwrap().starts_with("farm is private"));

        let off = run(&r, true).unwrap();
        assert!(off.contains("removed"), "{off}");
        assert!(!is_private(&r));
        assert!(!hooks_dir(&r).unwrap().join("pre-push").exists());
        sh(&farm, &["push", "-q", "origin", "yaks"]);
        assert!(remote_refs(&origin).contains("refs/heads/yaks"));
        // Off again is fine.
        assert!(run(&r, true).unwrap().contains("no yaks pre-push hook"));
    }

    #[test]
    fn guard_follows_a_configured_farm_branch_and_turning_the_flag_off() {
        let (r, origin, farm) = scratch();
        sh(&farm, &["branch", "-m", "farm-data"]);
        sh(&r, &["config", "yaks.branch", "farm-data"]);
        run(&r, false).unwrap();
        assert!(
            !try_git(&farm, &["push", "origin", "farm-data"])
                .status
                .success()
        );
        // The hook stays but is inert once the flag is unset.
        sh(&r, &["config", "--unset", "yaks.private"]);
        sh(&farm, &["push", "-q", "origin", "farm-data"]);
        assert!(remote_refs(&origin).contains("refs/heads/farm-data"));
    }

    #[test]
    fn a_foreign_pre_push_hook_is_left_alone_and_reported() {
        let (r, _origin, farm) = scratch();
        let hook = hooks_dir(&r).unwrap().join("pre-push");
        fs::create_dir_all(hook.parent().unwrap()).unwrap();
        fs::write(&hook, "#!/bin/sh\n# mine\nexit 0\n").unwrap();

        assert_eq!(
            install_guard(&r).unwrap(),
            GuardOutcome::Foreign(hook.clone())
        );
        let report = run(&r, false).unwrap();
        assert!(
            report.contains("not yaks's") && report.contains("NOT blocked"),
            "{report}"
        );
        assert!(
            report.contains(MARKER),
            "the report carries the check to fold in"
        );
        assert!(is_private(&r), "sync is still refused");
        assert_eq!(
            fs::read_to_string(&hook).unwrap(),
            "#!/bin/sh\n# mine\nexit 0\n"
        );
        // `--off` does not remove a hook that is not ours either.
        run(&r, true).unwrap();
        assert!(hook.exists());
        let _ = farm;
    }

    #[test]
    fn hook_honors_core_hooks_path() {
        let (r, origin, farm) = scratch();
        let hp = r.join("my-hooks");
        sh(&r, &["config", "core.hooksPath", hp.to_str().unwrap()]);
        run(&r, false).unwrap();
        assert!(hp.join("pre-push").exists());
        assert!(!try_git(&farm, &["push", "origin", "yaks"]).status.success());
        assert_eq!(remote_refs(&origin), "");
    }

    #[test]
    fn outside_a_repository_is_an_error() {
        let b = base();
        assert!(run(&b, false).is_err());
        assert!(!is_private(&b));
    }
}
