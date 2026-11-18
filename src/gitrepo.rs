//! The git helpers farm discovery and the farm branch use to find a
//! repository's HOST and its checkouts (yaks-6a34, yaks-d3ad).
//!
//! Every checkout of one repository on a machine shares ONE farm (the farm
//! branch's worktree), so a checkout has to name the repository it belongs to
//! the same way from anywhere. A Delta clone borrows its object store from
//! another repository through `objects/info/alternates`; that repository is
//! the HOST: the human's `.git` on a machine where the project was added from
//! a folder, or Delta's managed BARE repo on a machine that only had the
//! thread shared to it. Anything else is its own host.
//!
//! Plain git plus the filesystem, read-only (`git --no-optional-locks`),
//! invoked from inside any checkout.

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository whose object store the checkout at `top` borrows.
#[derive(Debug, Clone)]
pub struct Host {
    /// The host's git dir: the parent of the first `objects/info/alternates`
    /// line, else this checkout's own common git dir (a primary checkout, a
    /// plain git worktree, a non-Delta clone).
    pub git_dir: PathBuf,
    /// `true` when found through `alternates`.
    pub via_alternates: bool,
    /// The host is a bare repository (Delta's managed repo on a machine with
    /// no linked checkout); `false` means a real checkout.
    pub bare: bool,
}

/// Run read-only git in `dir`, returning trimmed stdout.
fn git(dir: &Path, args: &[&str]) -> Result<String> {
    Ok(git_raw(dir, args)?.trim().to_string())
}

fn git_raw(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .context("running git")?;
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr);
        bail!("git {}: {}", args.join(" "), msg.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub(crate) fn canonical(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// The worktree of the repository containing `from` where `branch` is checked
/// out, if any: `git worktree list --porcelain` in the HOST ([`host`]: this
/// checkout's own repository, or the one its `alternates` names, so a Delta
/// clone finds the human's worktrees), matched on `branch refs/heads/<b>`.
/// Backs `live_branch:` discovery (`store::discover`). `None` outside git, or
/// when nothing has the branch checked out (a bare host lists only its own
/// worktrees). Read-only.
pub(crate) fn worktree_of_branch(from: &Path, branch: &str) -> Option<PathBuf> {
    let git_dir = host_git_dir(from)?;
    let text = git_raw(&git_dir, &["worktree", "list", "--porcelain"]).ok()?;
    let want = format!("branch refs/heads/{branch}");
    for block in text.split("\n\n") {
        let mut path: Option<&str> = None;
        let mut hit = false;
        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(p);
            } else if line == want {
                hit = true;
            }
        }
        if hit {
            return path.map(|p| canonical(Path::new(p)));
        }
    }
    None
}

/// Every entry of `git worktree list --porcelain` in `top`, each with why it
/// is not a checkout (`Err`) or `Ok` when it is one.
fn worktree_entries(top: &Path) -> Vec<(PathBuf, Result<(), String>)> {
    let Ok(text) = git_raw(top, &["worktree", "list", "--porcelain"]) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for block in text.split("\n\n") {
        let mut path: Option<&str> = None;
        let mut skip: Option<String> = None;
        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(p);
            } else if line == "bare" {
                skip = Some("bare".into());
            } else if line.starts_with("prunable") {
                skip = Some("prunable (directory gone)".into());
            }
        }
        let Some(p) = path else { continue };
        let p = PathBuf::from(p);
        let verdict = match skip {
            Some(why) => Err(why),
            None if !p.join(".git").exists() => {
                Err("no .git (a separate git dir, e.g. a Delta clone's)".into())
            }
            None => Ok(()),
        };
        out.push((p, verdict));
    }
    out
}

/// The host's git dir and whether it was found through `alternates`: a host
/// without the `bare` probe ([`host`]), for callers that only read its config.
pub(crate) fn host_dir(top: &Path) -> Option<(PathBuf, bool)> {
    let common = common_dir(top)?;
    Some(match alternates(&common) {
        // `<host>/objects`; a host with no `objects` dir is not one we read.
        Some(objects) if objects.ends_with("objects") && objects.is_dir() => {
            (canonical(objects.parent()?), true)
        }
        _ => (common, false),
    })
}

/// The git dir of the host of the checkout at `top` ([`Host::git_dir`]), one
/// git call cheaper than [`host`] (no `bare` probe).
pub(crate) fn host_git_dir(top: &Path) -> Option<PathBuf> {
    host_dir(top).map(|(d, _)| d)
}

/// Find the host of the checkout at `top`.
pub fn host(top: &Path) -> Option<Host> {
    let (git_dir, via_alternates) = host_dir(top)?;
    let bare = git(&git_dir, &["rev-parse", "--is-bare-repository"]).is_ok_and(|b| b == "true");
    Some(Host {
        git_dir,
        via_alternates,
        bare,
    })
}

/// The host's main checkout, when the host is not bare (the human's checkout
/// on a machine where the project was added from a folder): the first entry
/// of `git worktree list` run in the host's git dir, which git always lists
/// as the main working tree, when it is a real checkout.
pub(crate) fn host_checkout(host: &Host) -> Option<PathBuf> {
    if host.bare {
        return None;
    }
    let entries = worktree_entries(&host.git_dir);
    let (main, verdict) = entries.first()?;
    verdict.as_ref().ok()?;
    Some(canonical(main))
}

/// The remote a branch of the checkout at `top` syncs with: its configured
/// `branch.<b>.remote`, else `origin`; `None` when that remote does not
/// exist. The one "which remote" rule (`yaks sync` and `yaks url` use it).
pub(crate) fn remote_of(top: &Path, branch: &str) -> Option<String> {
    let name = git(
        top,
        &["config", "--get", &format!("branch.{branch}.remote")],
    )
    .ok()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| "origin".to_string());
    git(top, &["remote", "get-url", &name])
        .ok()
        .filter(|u| !u.is_empty())
        .map(|_| name)
}

/// The branch checked out in `dir`, `None` when HEAD is detached.
pub(crate) fn current_branch(dir: &Path) -> Option<String> {
    git(dir, &["symbolic-ref", "--short", "-q", "HEAD"])
        .ok()
        .filter(|b| !b.is_empty())
}

/// The commit `rev` names in `dir`, in full (`None` for an unknown rev).
pub(crate) fn commit_of(dir: &Path, rev: &str) -> Option<String> {
    git(
        dir,
        &["rev-parse", "--verify", "-q", &format!("{rev}^{{commit}}")],
    )
    .ok()
    .filter(|s| !s.is_empty())
}

/// One commit, as the commit link's peek and view show it (yaks-274c).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitInfo {
    /// The full hash.
    pub sha: String,
    pub author: String,
    /// The author date, RFC 3339.
    pub date: String,
    pub subject: String,
    /// The message after the subject (may be empty).
    pub body: String,
    /// `(added, deleted, path)` per file; a binary file's counts are `None`.
    pub files: Vec<(Option<u32>, Option<u32>, String)>,
}

/// The commit `rev` names in the repository at `dir`, or `None` when this
/// checkout does not have it. A merge's files are against its first parent.
pub(crate) fn commit_info(dir: &Path, rev: &str) -> Option<CommitInfo> {
    let sha = commit_of(dir, rev)?;
    // Unit and record separators cannot appear in these fields.
    let head = git_raw(
        dir,
        &["show", "-s", "--format=%H%x1f%an%x1f%aI%x1f%s%x1f%b", &sha],
    )
    .ok()?;
    let mut f = head.splitn(5, '\u{1f}');
    let (_, author, date, subject, body) = (
        f.next()?,
        f.next()?,
        f.next()?,
        f.next()?,
        f.next().unwrap_or(""),
    );
    let stat = git_raw(
        dir,
        &[
            "show",
            "--numstat",
            "--first-parent",
            "-m",
            "--format=",
            "--no-renames",
            &sha,
        ],
    )
    .unwrap_or_default();
    let files = stat
        .lines()
        .filter_map(|l| {
            let mut p = l.splitn(3, '\t');
            let (a, d, path) = (p.next()?, p.next()?, p.next()?);
            Some((a.parse().ok(), d.parse().ok(), path.to_string()))
        })
        .collect();
    Some(CommitInfo {
        sha,
        author: author.to_string(),
        date: date.to_string(),
        subject: subject.to_string(),
        body: body.trim_end().to_string(),
        files,
    })
}

/// Whether some branch of `remote` that this checkout knows contains `sha`:
/// the remote has the commit (as of the last fetch or push).
pub(crate) fn remote_has(dir: &Path, remote: &str, sha: &str) -> bool {
    git(
        dir,
        &[
            "branch",
            "-r",
            "--contains",
            sha,
            "--list",
            &format!("{remote}/*"),
        ],
    )
    .is_ok_and(|o| !o.is_empty())
}

/// Whether a commit is on the remote its checkout pushes to (yaks-ab81).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Pushed {
    /// Some branch of `remote` contains it, as of the last fetch or push.
    Yes(String),
    /// The remote exists and no branch of it has the commit.
    No(String),
    /// The checkout has no remote to have pushed to.
    NoRemote,
}

/// Whether `sha` is on the remote `branch` (or, detached, `origin`) syncs
/// with: [`remote_of`], then [`remote_has`]. Asks nothing of the network, so
/// "as last fetched or pushed".
pub(crate) fn pushed(dir: &Path, branch: Option<&str>, sha: &str) -> Pushed {
    match remote_of(dir, branch.unwrap_or("")) {
        None => Pushed::NoRemote,
        Some(r) if remote_has(dir, &r, sha) => Pushed::Yes(r),
        Some(r) => Pushed::No(r),
    }
}

/// Whether `path` (repo-relative) exists at `rev` (a commit or ref).
pub(crate) fn has_path(dir: &Path, rev: &str, path: &str) -> bool {
    git(dir, &["cat-file", "-e", &format!("{rev}:{path}")]).is_ok()
}

/// Whether `remote` has a branch `branch` as of the last fetch or push.
pub(crate) fn remote_branch_exists(dir: &Path, remote: &str, branch: &str) -> bool {
    git(
        dir,
        &[
            "rev-parse",
            "--verify",
            "-q",
            &format!("refs/remotes/{remote}/{branch}"),
        ],
    )
    .is_ok_and(|o| !o.is_empty())
}

/// The web host of the repository the checkout at `top` pushes `branch` to:
/// the remote's name ([`remote_of`]) and the recognised host, with
/// `git config yaks.host` ([`crate::hosting::HOST_KEY`]) declaring the kind
/// of a custom domain. `Err` says what is missing.
pub(crate) fn hosting_repo(
    top: &Path,
    branch: Option<&str>,
) -> Result<(String, crate::hosting::Repo)> {
    let Some(remote) = remote_of(top, branch.unwrap_or("")) else {
        bail!(
            "{} has no remote to link to (no `origin`, and the branch has no \
             `branch.<name>.remote`); add one with `git remote add origin <url>`",
            top.display()
        );
    };
    let url = git(top, &["remote", "get-url", &remote])?;
    let declared = crate::store::repo_config(top, crate::hosting::HOST_KEY);
    let repo = crate::hosting::Repo::from_remote(&url, declared.as_deref())?;
    Ok((remote, repo))
}

/// The shared (common) git dir of the checkout at `checkout`.
///
/// Asked once per checkout per process: discovery and the farm's git facts
/// ask it from several places, and a checkout's common dir does not move.
pub(crate) fn common_dir(checkout: &Path) -> Option<PathBuf> {
    static KNOWN: std::sync::Mutex<Option<std::collections::HashMap<PathBuf, PathBuf>>> =
        std::sync::Mutex::new(None);
    if let Some(d) = KNOWN
        .lock()
        .ok()
        .and_then(|k| k.as_ref().and_then(|m| m.get(checkout).cloned()))
    {
        return Some(d);
    }
    let d = PathBuf::from(git(checkout, &["rev-parse", "--git-common-dir"]).ok()?);
    let d = canonical(&if d.is_absolute() { d } else { checkout.join(d) });
    if let Ok(mut k) = KNOWN.lock() {
        k.get_or_insert_with(Default::default)
            .insert(checkout.to_path_buf(), d.clone());
    }
    Some(d)
}

/// The first line of `<common>/objects/info/alternates`: the object store a
/// Delta clone borrows (the human's checkout, or a managed bare repo).
pub(crate) fn alternates(common: &Path) -> Option<PathBuf> {
    let text = fs::read_to_string(common.join("objects/info/alternates")).ok()?;
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))?;
    let p = PathBuf::from(line);
    Some(if p.is_absolute() {
        p
    } else {
        common.join("objects").join(p)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn base(tag: &str) -> PathBuf {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "yaks-gitrepo-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        canonical(&p)
    }

    /// Run git (identity and signing pinned) and return trimmed stdout.
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

    /// A repo at `dir` with one commit.
    fn repo(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        sh(dir, &["init", "-q"]);
        fs::write(dir.join("f"), "x").unwrap();
        sh(dir, &["add", "-A"]);
        sh(dir, &["commit", "-q", "-m", "init"]);
    }

    /// A Delta-shaped clone of `host` (a git dir): the checkout at
    /// `<checkouts>/<dir>/<name>`, its git dir at `<store>/<dir>/<name>.git`
    /// with `core.worktree` set and `objects/info/alternates` naming the
    /// host's objects. Returns the checkout.
    fn delta_clone(host: &Path, store: &Path, checkouts: &Path, dir: &str, name: &str) -> PathBuf {
        let co = checkouts.join(dir).join(name);
        let gd = store.join(dir).join(format!("{name}.git"));
        fs::create_dir_all(co.parent().unwrap()).unwrap();
        fs::create_dir_all(gd.parent().unwrap()).unwrap();
        sh(
            checkouts,
            &[
                "clone",
                "-q",
                "--shared",
                "--separate-git-dir",
                gd.to_str().unwrap(),
                host.to_str().unwrap(),
                co.to_str().unwrap(),
            ],
        );
        sh(&co, &["config", "core.worktree", co.to_str().unwrap()]);
        co
    }

    #[test]
    fn a_primary_checkout_and_its_worktree_are_their_own_host() {
        let b = base("own");
        let t = b.join("repo");
        repo(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "w", wt.to_str().unwrap()],
        );
        for from in [&t, &wt] {
            let h = host(from).unwrap();
            assert_eq!(h.git_dir, canonical(&t.join(".git")));
            assert!(!h.via_alternates && !h.bare);
            assert_eq!(host_checkout(&h), Some(t.clone()));
        }
        assert!(host(&b).is_none(), "not a repository");
    }

    #[test]
    fn a_delta_clone_finds_the_human_checkout_through_its_alternates() {
        let b = base("linked");
        let t = b.join("repo");
        repo(&t);
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "d1",
            "repo",
        );
        let h = host(&c).unwrap();
        assert_eq!(h.git_dir, canonical(&t.join(".git")));
        assert!(h.via_alternates && !h.bare);
        assert_eq!(host_checkout(&h), Some(t.clone()));
        // The clone's own common dir is its separate git dir, not the host's.
        assert_ne!(common_dir(&c).unwrap(), h.git_dir);
    }

    #[test]
    fn a_managed_bare_host_has_no_checkout() {
        let b = base("managed");
        let src = b.join("src");
        repo(&src);
        let host_dir = b.join("managed/repository.git");
        sh(
            &b,
            &[
                "clone",
                "-q",
                "--bare",
                src.to_str().unwrap(),
                host_dir.to_str().unwrap(),
            ],
        );
        let root = b.join("worktrees");
        let c = delta_clone(&host_dir, &root, &root, "m1", "local_u1");
        let h = host(&c).unwrap();
        assert!(h.bare && h.via_alternates);
        assert_eq!(h.git_dir, canonical(&host_dir));
        assert_eq!(host_checkout(&h), None);
    }

    #[test]
    fn worktree_of_branch_finds_the_checkout_from_a_clone_too() {
        let b = base("branch");
        let t = b.join("repo");
        repo(&t);
        let wt = b.join("farm");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "yaks", wt.to_str().unwrap()],
        );
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "d1",
            "repo",
        );
        for from in [&t, &c] {
            assert_eq!(worktree_of_branch(from, "yaks"), Some(wt.clone()));
            assert_eq!(worktree_of_branch(from, "nope"), None);
        }
        assert_eq!(worktree_of_branch(&b, "yaks"), None);
    }

    #[test]
    fn alternates_resolve_a_relative_line_against_the_objects_dir() {
        let b = base("alt");
        let common = b.join("c.git");
        fs::create_dir_all(common.join("objects/info")).unwrap();
        fs::write(
            common.join("objects/info/alternates"),
            "# note\n\n../../host/objects\n",
        )
        .unwrap();
        assert_eq!(
            alternates(&common),
            Some(common.join("objects").join("../../host/objects"))
        );
        assert_eq!(alternates(&b), None);
    }
}
