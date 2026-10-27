//! Farm-branch plumbing: git helpers and [`bootstrap`], which gives a
//! repository whose farm branch has no worktree one at its farm home
//! (yaks-a260). Starting a farm is `yaks init` (`crate::init`).

use crate::sheds;
#[cfg(test)]
use crate::store;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
/// checkouts, the same refusal as `yaks init`.
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

#[cfg(test)]
mod tests {
    use super::*;
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

    fn repo(b: &Path) -> PathBuf {
        let r = b.join("r");
        fs::create_dir_all(r.join("src")).unwrap();
        git(&r, &["init", "-q"]).unwrap();
        r
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
}
