//! Per-change farm commits (yaks-a838, design yaks-63c3): every command that
//! writes the farm commits exactly what it wrote, on the farm branch, as one
//! commit.
//!
//! It hangs off the farm lock ([`crate::store::lock`]), which every mutation
//! already holds for its whole duration. Taking the lock records a
//! [`Snapshot`] of the farm's dirty files (path and blob); releasing it
//! commits the paths whose content changed since, while still holding the
//! lock, so no other writer can interleave.
//!
//! The commit is built in a temporary index (`read-tree HEAD`, `update-index`
//! for the command's paths only, `write-tree`, `commit-tree`, `update-ref`),
//! so anything else dirty or staged in the farm worktree (a person's edit in
//! progress) is left exactly as it was. When the command rewrote a file that
//! already held an uncommitted hand edit, that hand edit is committed first,
//! by itself, so the command's commit holds only the command's change and a
//! revert of it never takes the hand edit with it.
//!
//! Only a farm that is a farm branch commits (a `YAKS_DIR` farm outside git
//! is untouched). Nothing is done while a merge or cherry-pick is pending.
//! A failure never fails the command (its files are written): it is kept for
//! [`take_error`], and `yaks commit` picks the files up later.

use crate::changes;
use anyhow::{Context, Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The trailer naming who made a change.
pub const ACTOR_TRAILER: &str = "Yaks-Actor";

/// How close together (seconds) two writes to the same files by the same
/// actor must be to share one commit, when coalescing is on (the TUI).
const COALESCE_SECS: i64 = 5;

static COALESCE: AtomicBool = AtomicBool::new(false);
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);
/// The last commit this process made (only such a commit is ever folded
/// into, so a burst never swallows another command's commit).
static OURS: Mutex<Option<String>> = Mutex::new(None);

/// Fold a burst of writes to the same files into one commit (the TUI turns
/// this on: each saved field is a write).
pub fn set_coalesce(on: bool) {
    COALESCE.store(on, Ordering::Relaxed);
}

/// The last commit failure, cleared by reading it.
pub fn take_error() -> Option<String> {
    LAST_ERROR.lock().ok()?.take()
}

fn keep_error(e: String) {
    if let Ok(mut slot) = LAST_ERROR.lock() {
        *slot = Some(e);
    }
}

/// The farm's dirty files when the lock was taken: path (from the repository
/// top) to blob id, `None` for a deleted file.
pub struct Snapshot {
    root: PathBuf,
    top: PathBuf,
    before: BTreeMap<String, Option<String>>,
}

fn git_in(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Result<Output> {
    let mut c = Command::new("git");
    c.arg("-C").arg(dir).args(args).stdin(Stdio::null());
    c.env_remove("GIT_DIR").env_remove("GIT_WORK_TREE");
    for (k, v) in env {
        c.env(k, v);
    }
    c.output().context("running git")
}

fn ok(out: Output, what: &str) -> Result<String> {
    if !out.status.success() {
        bail!("{what}: {}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Blob ids of `paths` (from the repository top) as they are on disk now,
/// written to the object store; `None` for a path that is gone.
fn blobs(top: &Path, paths: &[String]) -> Result<BTreeMap<String, Option<String>>> {
    let (present, gone): (Vec<&String>, Vec<&String>) =
        paths.iter().partition(|p| top.join(p).is_file());
    let mut out: BTreeMap<String, Option<String>> =
        gone.into_iter().map(|p| (p.clone(), None)).collect();
    if !present.is_empty() {
        let mut args = vec!["hash-object", "-w", "--"];
        args.extend(present.iter().map(|p| p.as_str()));
        let ids = ok(git_in(top, &args, &[])?, "git hash-object")?;
        for (p, id) in present.into_iter().zip(ids.lines()) {
            out.insert(p.clone(), Some(id.to_string()));
        }
    }
    Ok(out)
}

/// Record the farm's dirty files, or `None` when this farm does not commit
/// per change (not a farm branch) or git can't say. `YAKS_NO_AUTOCOMMIT=1`
/// turns it off for one command (and for tests that inspect dirty state).
pub fn snapshot(root: &Path) -> Option<Snapshot> {
    if std::env::var("YAKS_NO_AUTOCOMMIT").is_ok_and(|v| !v.is_empty() && v != "0") {
        return None;
    }
    crate::store::farm_branch(root)?;
    let run = || -> Result<Snapshot> {
        let top = PathBuf::from(ok(
            git_in(root, &["rev-parse", "--show-toplevel"], &[])?,
            "git rev-parse",
        )?);
        let dirty: Vec<String> = changes::read(root)?
            .unwrap_or_default()
            .into_iter()
            .map(|c| c.path)
            .collect();
        Ok(Snapshot {
            root: root.to_path_buf(),
            before: blobs(&top, &dirty)?,
            top,
        })
    };
    match run() {
        Ok(s) => Some(s),
        Err(e) => {
            keep_error(format!("per-change commit skipped: {e:#}"));
            None
        }
    }
}

/// Commit what changed since `snap` (called as the lock is released).
pub fn finish(snap: Snapshot) {
    if let Err(e) = commit_changes(&snap) {
        keep_error(format!(
            "the change is written but not committed ({e:#}); `yaks commit` commits it"
        ));
    }
}

fn head(top: &Path) -> Option<String> {
    git_in(top, &["rev-parse", "-q", "--verify", "HEAD"], &[])
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Who commits: the repository's git identity, else the actor (an address
/// that never resolves), so a machine with no git identity still commits.
pub(crate) fn identity_env(top: &Path, actor: &str) -> Vec<(String, String)> {
    let has = git_in(top, &["var", "GIT_COMMITTER_IDENT"], &[])
        .map(|o| o.status.success())
        .unwrap_or(false);
    if has {
        return Vec::new();
    }
    let name = if actor.is_empty() { "yaks" } else { actor };
    let mail = format!(
        "{}@yaks.invalid",
        name.replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "-")
    );
    ["GIT_AUTHOR_NAME", "GIT_COMMITTER_NAME"]
        .into_iter()
        .map(|k| (k.to_string(), name.to_string()))
        .chain(
            ["GIT_AUTHOR_EMAIL", "GIT_COMMITTER_EMAIL"]
                .into_iter()
                .map(|k| (k.to_string(), mail.clone())),
        )
        .collect()
}

/// One commit on HEAD from a temporary index: start from `parent`'s tree,
/// apply `stage` (git update-index argument lists), commit with `message`.
/// Returns the new commit, or `None` when the tree did not change.
fn commit_tree(
    snap: &Snapshot,
    parent: Option<&str>,
    stage: &[Vec<String>],
    message: &str,
    ident: &[(String, String)],
) -> Result<Option<String>> {
    let tmp = std::env::temp_dir().join(format!(
        "yaks-index-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let idx = tmp.display().to_string();
    let mut env: Vec<(&str, &str)> = vec![("GIT_INDEX_FILE", idx.as_str())];
    env.extend(ident.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    let result = (|| -> Result<Option<String>> {
        match parent {
            Some(p) => ok(git_in(&snap.top, &["read-tree", p], &env)?, "git read-tree")?,
            None => ok(
                git_in(&snap.top, &["read-tree", "--empty"], &env)?,
                "git read-tree",
            )?,
        };
        for args in stage {
            let a: Vec<&str> = args.iter().map(String::as_str).collect();
            ok(git_in(&snap.top, &a, &env)?, "git update-index")?;
        }
        let tree = ok(git_in(&snap.top, &["write-tree"], &env)?, "git write-tree")?;
        if let Some(p) = parent {
            let ptree = ok(
                git_in(&snap.top, &["rev-parse", &format!("{p}^{{tree}}")], &env)?,
                "git rev-parse",
            )?;
            if ptree == tree {
                return Ok(None);
            }
        }
        let mut args = vec!["commit-tree", tree.as_str(), "-m", message];
        if let Some(p) = parent {
            args.extend(["-p", p]);
        }
        Ok(Some(ok(
            git_in(&snap.top, &args, &env)?,
            "git commit-tree",
        )?))
    })();
    let _ = std::fs::remove_file(&tmp);
    result
}

fn move_head(snap: &Snapshot, new: &str, old: Option<&str>) -> Result<()> {
    let mut args = vec!["update-ref", "-m", "yaks: per-change commit", "HEAD", new];
    if let Some(o) = old {
        args.push(o);
    }
    ok(git_in(&snap.top, &args, &[])?, "git update-ref")?;
    Ok(())
}

/// Whether HEAD is ours to fold into: made by `actor` within the coalescing
/// window, touching exactly `paths`, and not yet pushed to the upstream.
fn can_coalesce(snap: &Snapshot, head: &str, actor: &str, paths: &BTreeSet<String>) -> bool {
    if !COALESCE.load(Ordering::Relaxed) {
        return false;
    }
    if OURS.lock().ok().and_then(|o| o.clone()).as_deref() != Some(head) {
        return false;
    }
    let Ok(info) = git_in(&snap.top, &["log", "-1", "--format=%ct%x00%B", head], &[]) else {
        return false;
    };
    let info = String::from_utf8_lossy(&info.stdout).to_string();
    let Some((ct, body)) = info.split_once('\0') else {
        return false;
    };
    let fresh = ct
        .trim()
        .parse::<i64>()
        .is_ok_and(|t| chrono::Utc::now().timestamp() - t <= COALESCE_SECS);
    let same_actor = body
        .lines()
        .any(|l| l.trim() == format!("{ACTOR_TRAILER}: {actor}"));
    if !fresh || !same_actor {
        return false;
    }
    let touched: BTreeSet<String> = git_in(
        &snap.top,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            "--root",
            head,
        ],
        &[],
    )
    .map(|o| {
        String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(String::from)
            .collect()
    })
    .unwrap_or_default();
    if touched != *paths {
        return false;
    }
    // Never rewrite a commit others may have fetched.
    let pushed = git_in(
        &snap.top,
        &["merge-base", "--is-ancestor", head, "@{u}"],
        &[],
    )
    .map(|o| o.status.success())
    .unwrap_or(false);
    !pushed
}

/// One farm commit for `yaks undo`'s listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FarmCommit {
    pub sha: String,
    pub subject: String,
    pub actor: Option<String>,
    /// Unix seconds.
    pub time: i64,
}

/// The farm branch's commits, newest first: those whose `Yaks-Actor` is
/// `by` (any when `None`), newer than `since` (unix seconds).
pub fn farm_commits(root: &Path, by: Option<&str>, since: Option<i64>) -> Result<Vec<FarmCommit>> {
    let out = git_in(
        root,
        &[
            "log",
            "--format=%H%x00%ct%x00%s%x00%(trailers:key=Yaks-Actor,valueonly,separator=%x01)%x1e",
            "HEAD",
        ],
        &[],
    )?;
    if !out.status.success() {
        return Ok(Vec::new());
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut list = Vec::new();
    for rec in text.split('\x1e') {
        let f: Vec<&str> = rec.trim_start_matches('\n').split('\0').collect();
        let [sha, ct, subject, actor] = f.as_slice() else {
            continue;
        };
        let time: i64 = ct.parse().unwrap_or(0);
        if since.is_some_and(|s| time < s) {
            break;
        }
        let actor = actor.trim();
        let actor = (!actor.is_empty()).then(|| actor.to_string());
        if by.is_some_and(|b| actor.as_deref() != Some(b)) {
            continue;
        }
        list.push(FarmCommit {
            sha: sha.to_string(),
            subject: subject.to_string(),
            actor,
            time,
        });
    }
    Ok(list)
}

/// Revert `shas` (farm commits, applied newest first) as one new commit on
/// the farm branch, through the yaks merge driver so a yak stays valid.
/// Refuses while the farm has any uncommitted change. Returns the new commit.
pub fn revert(root: &Path, shas: &[String]) -> Result<String> {
    if shas.is_empty() {
        bail!("nothing to undo");
    }
    let dirty = changes::read(root)?.unwrap_or_default();
    if !dirty.is_empty() {
        bail!(
            "the farm has uncommitted changes ({}); commit them first (`yaks commit`)",
            dirty
                .iter()
                .map(|c| c.path.as_str())
                .take(3)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    // The yak merge driver merges a reverted yak field by field (git would
    // otherwise conflict on any neighbouring line). Register it, then commit
    // the `.gitattributes` it may add along with the undo.
    let top = PathBuf::from(ok(
        git_in(root, &["rev-parse", "--show-toplevel"], &[])?,
        "git rev-parse",
    )?);
    crate::mergedriver::ensure_registered(&top)?;
    let _ = git_in(&top, &["add", "--", ".gitattributes"], &[]);
    let mut args = vec!["revert", "--no-edit", "--no-commit"];
    args.extend(shas.iter().map(String::as_str));
    let out = git_in(root, &args, &[])?;
    if !out.status.success() {
        let _ = git_in(root, &["revert", "--abort"], &[]);
        let _ = git_in(&top, &["reset", "-q", "--", ".gitattributes"], &[]);
        bail!(
            "the undo does not apply cleanly (a later change touched the same lines): {}",
            String::from_utf8_lossy(&out.stderr)
                .lines()
                .next()
                .unwrap_or("")
        );
    }
    let short: Vec<&str> = shas.iter().map(|s| &s[..s.len().min(7)]).collect();
    let actor = crate::actor::resolve_detail(None)
        .map(|(n, _)| n)
        .unwrap_or_default();
    let mut msg = format!("yaks: undo {}", short.join(", "));
    if !actor.is_empty() {
        msg.push_str(&format!("\n\n{ACTOR_TRAILER}: {actor}"));
    }
    let ident = identity_env(&top, &actor);
    let env: Vec<(&str, &str)> = ident
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let c = git_in(root, &["commit", "-q", "--no-verify", "-m", &msg], &env)?;
    if !c.status.success() {
        bail!("git commit: {}", String::from_utf8_lossy(&c.stderr).trim());
    }
    ok(
        git_in(root, &["rev-parse", "--short", "HEAD"], &[])?,
        "git rev-parse",
    )
}

fn commit_changes(snap: &Snapshot) -> Result<()> {
    if changes::pending_operation(&snap.root)?.is_some() {
        return Ok(());
    }
    let now = changes::read(&snap.root)?.unwrap_or_default();
    let dirty_now: Vec<String> = now.iter().map(|c| c.path.clone()).collect();
    let current = blobs(&snap.top, &dirty_now)?;
    // A path the command changed: dirty now with other content than before,
    // or dirty before and clean (back to HEAD) now.
    let mut paths: BTreeSet<String> = current
        .iter()
        .filter(|(p, b)| snap.before.get(*p) != Some(b))
        .map(|(p, _)| p.clone())
        .collect();
    paths.extend(
        snap.before
            .keys()
            .filter(|p| !current.contains_key(*p))
            .cloned(),
    );
    if paths.is_empty() {
        return Ok(());
    }
    let actor = crate::actor::resolve_detail(None)
        .map(|(n, _)| n)
        .unwrap_or_default();
    let ident = identity_env(&snap.top, &actor);
    let mut parent = head(&snap.top);

    // A hand edit the command rewrote: commit it first, by itself.
    let hand: Vec<(&String, &Option<String>)> = snap
        .before
        .iter()
        .filter(|(p, _)| paths.contains(*p))
        .collect();
    if !hand.is_empty() {
        let stage: Vec<Vec<String>> = hand
            .iter()
            .map(|(p, blob)| match blob {
                Some(b) => vec![
                    "update-index".into(),
                    "--add".into(),
                    "--cacheinfo".into(),
                    format!("100644,{b},{p}"),
                ],
                None => vec![
                    "update-index".into(),
                    "--force-remove".into(),
                    "--".into(),
                    (*p).clone(),
                ],
            })
            .collect();
        let names: Vec<&str> = hand.iter().map(|(p, _)| p.as_str()).collect();
        let msg = format!(
            "yaks: hand edits to {} (committed before a yaks change)",
            names.join(", ")
        );
        if let Some(c) = commit_tree(snap, parent.as_deref(), &stage, &msg, &[])? {
            move_head(snap, &c, parent.as_deref())?;
            parent = Some(c);
        }
    }

    let scoped: Vec<changes::Change> = now
        .into_iter()
        .filter(|c| paths.contains(&c.path))
        .collect();
    let mut message = crate::commit::generate_message(&scoped);
    if message == "yaks: " {
        message = format!(
            "yaks: update {}",
            paths.iter().cloned().collect::<Vec<_>>().join(", ")
        );
    }
    if !actor.is_empty() {
        message.push_str(&format!("\n\n{ACTOR_TRAILER}: {actor}"));
    }
    let mut stage_args = vec![
        "update-index".to_string(),
        "--add".into(),
        "--remove".into(),
        "--".into(),
    ];
    stage_args.extend(paths.iter().cloned());
    let stage = vec![stage_args];

    // Fold into HEAD when it is this burst's earlier commit (TUI only): the
    // new commit replaces HEAD, on HEAD's parent, with HEAD's message.
    let (base, old, keep_msg) = match parent.as_deref() {
        Some(h) if hand.is_empty() && can_coalesce(snap, h, &actor, &paths) => {
            let msg = ok(
                git_in(&snap.top, &["log", "-1", "--format=%B", h], &[])?,
                "git log",
            )?;
            let grandparent = git_in(
                &snap.top,
                &["rev-parse", "-q", "--verify", &format!("{h}^")],
                &[],
            )?;
            let grandparent = grandparent.status.success().then(|| {
                String::from_utf8_lossy(&grandparent.stdout)
                    .trim()
                    .to_string()
            });
            (grandparent, Some(h.to_string()), Some(msg))
        }
        _ => (parent.clone(), parent.clone(), None),
    };
    let msg = keep_msg.unwrap_or(message);
    let made = commit_tree(snap, base.as_deref(), &stage, &msg, &ident)?;
    if let Some(c) = &made {
        move_head(snap, c, old.as_deref())?;
    } else if let (Some(b), Some(o)) = (base.as_deref(), old.as_deref()) {
        // Folding left nothing (the burst undid itself): drop the commit.
        if b != o {
            move_head(snap, b, Some(o))?;
        }
    }
    if let Ok(mut ours) = OURS.lock() {
        *ours = made;
    }
    // The real index: those paths now match HEAD (they are not staged).
    let mut reset = vec!["reset", "-q", "--"];
    reset.extend(paths.iter().map(String::as_str));
    let _ = git_in(&snap.top, &reset, &[]);
    Ok(())
}
