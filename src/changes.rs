//! The farm's uncommitted changes, read from git and classified per yak.
//!
//! The one place that turns `git status` under the farm into yak terms. Both
//! `yaks commit` (which builds its message from it) and `yaks status` (which
//! lists it) use [`classify`], so the two cannot disagree about what happened
//! to a yak.

use crate::model::Status;
use crate::preflight::yak_id_of;
use crate::status::SyncState;
use crate::store;
use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Run `git -C <root> <args>`.
pub fn git(root: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .context("running git")
}

/// `out`'s stdout, or an error naming `what` and git's stderr.
pub fn check(out: Output, what: &str) -> Result<Vec<u8>> {
    if !out.status.success() {
        bail!(
            "{what} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out.stdout)
}

/// One changed path under the farm, as `git status` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// `A` (new or untracked), `M` or `D`.
    pub state: char,
    pub path: String,
    /// The change is in the index and nothing more is pending in the working
    /// tree for this path.
    pub staged: bool,
    /// For a yak's own flat file (`<id>.md`) that existed at HEAD (`M`, `D`):
    /// its `status:` there. `None` for anything else, or when HEAD's text does
    /// not parse. [`read`] fills it; [`parse_status`] leaves it `None`.
    pub status_before: Option<String>,
    /// For a yak's own flat file that exists in the working tree (`A`, `M`):
    /// its `status:` now (a missing field reads as `hairy`).
    pub status_after: Option<String>,
}

impl Change {
    pub fn new(state: char, path: String, staged: bool) -> Change {
        Change {
            state,
            path,
            staged,
            status_before: None,
            status_after: None,
        }
    }
}

/// The farm's changes at `root` (paths relative to the repository top), or
/// `None` when nothing under it is tracked by git (a `YAKS_DIR` farm that is
/// not a farm branch).
pub fn read(root: &Path) -> Result<Option<Vec<Change>>> {
    Ok(read_inner(root, false)?.map(|(changes, _)| changes))
}

/// [`read`], and the farm branch's standing against its upstream from the same
/// `git status` call (its `--branch` header), so a caller that wants both pays
/// one spawn, not six. `None` for the state when git cannot say (a detached
/// HEAD); the same facts as [`crate::status::sync_state`].
pub fn read_with_sync(root: &Path) -> Result<Option<(Vec<Change>, Option<SyncState>)>> {
    read_inner(root, true)
}

fn read_inner(root: &Path, with_branch: bool) -> Result<Option<(Vec<Change>, Option<SyncState>)>> {
    // A farm branch's worktree IS the farm, tracked or not yet (a fresh orphan
    // branch has no commit before the first `yaks commit`), so it needs no
    // look at what git tracks. Any other farm with nothing tracked is outside
    // git.
    let farm = crate::store::farm_git(root);
    if farm.is_none() {
        let tracked = git(root, &["ls-files", "--", "."])?;
        if !tracked.status.success() {
            bail!(
                "the farm at {} is not inside a git repository",
                root.display()
            );
        }
        if tracked.stdout.is_empty() {
            return Ok(None);
        }
    }
    // `--ahead-behind` because `status.aheadBehind=false` would otherwise read
    // as "in sync".
    let mut args = vec!["status", "--porcelain", "-z", "--untracked-files=all"];
    if with_branch {
        args.extend(["--branch", "--ahead-behind"]);
    }
    args.extend(["--", "."]);
    let status = check(git(root, &args)?, "git status")?;
    let status = String::from_utf8_lossy(&status);
    let (header, status) = split_branch_header(&status);
    // Only a farm branch has a standing ([`crate::status::sync_state`]).
    let sync = header
        .filter(|_| farm.is_some())
        .and_then(parse_branch_header);
    let mut changes = parse_status(status);
    if changes.iter().any(|c| is_flat_yak_file(&c.path)) {
        let top = match farm {
            Some(f) => f.top,
            None => {
                let top = check(
                    git(root, &["rev-parse", "--show-toplevel"])?,
                    "git rev-parse",
                )?;
                PathBuf::from(String::from_utf8_lossy(&top).trim())
            }
        };
        for c in changes.iter_mut().filter(|c| is_flat_yak_file(&c.path)) {
            read_statuses(root, &top, c);
        }
    }
    Ok(Some((changes, sync)))
}

/// The `## ...` line `git status --branch -z` puts first, and the entries
/// after it.
pub(crate) fn split_branch_header(out: &str) -> (Option<&str>, &str) {
    match out.strip_prefix("## ") {
        Some(rest) => match rest.split_once('\0') {
            Some((header, entries)) => (Some(header), entries),
            None => (Some(rest), ""),
        },
        None => (None, out),
    }
}

/// A branch header as a [`SyncState`]: `main`, `main...origin/main`,
/// `main...origin/main [ahead 1, behind 2]`, `main...origin/main [gone]`,
/// `No commits yet on main`. `None` for a detached HEAD, which has no branch
/// to stand anywhere. `...` cannot be in a branch name, so the split is safe.
pub(crate) fn parse_branch_header(header: &str) -> Option<SyncState> {
    let header = header
        .strip_prefix("No commits yet on ")
        .or_else(|| header.strip_prefix("Initial commit on "))
        .unwrap_or(header);
    if header.starts_with("HEAD (no branch)") {
        return None;
    }
    let (names, counts) = match header.split_once(" [") {
        Some((n, c)) => (n, c.trim_end_matches(']')),
        None => (header, ""),
    };
    let (branch, upstream) = match names.split_once("...") {
        Some((b, u)) => (b, Some(u)),
        None => (names, None),
    };
    // A deleted upstream is no upstream, as `rev-parse branch@{upstream}`
    // says.
    let upstream = upstream.filter(|_| counts != "gone");
    let count = |word: &str| {
        counts
            .split(", ")
            .find_map(|p| p.strip_prefix(word))
            .and_then(|n| n.trim().parse::<usize>().ok())
            .unwrap_or(0)
    };
    Some(SyncState {
        branch: branch.to_string(),
        upstream: upstream.map(String::from),
        ahead: count("ahead "),
        behind: count("behind "),
    })
}

/// Fill `c`'s `status:` fields from HEAD's text and the working-tree file (a
/// flat yak file is the only place a status change is visible to git).
fn read_statuses(root: &Path, top: &Path, c: &mut Change) {
    let status_of =
        |text: &str| store::parse_task(text, Status::Hairy).map(|t| t.status.as_str().to_string());
    if matches!(c.state, 'M' | 'D') {
        c.status_before = head_text(root, &c.path).and_then(|t| status_of(&t));
    }
    if matches!(c.state, 'A' | 'M') {
        c.status_after = std::fs::read_to_string(top.join(&c.path))
            .ok()
            .and_then(|t| status_of(&t));
    }
}

/// `path`'s text at HEAD (`path` relative to the repository top), if it was
/// there.
pub fn head_text(root: &Path, path: &str) -> Option<String> {
    let out = git(root, &["show", &format!("HEAD:{path}")]).ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Paths changed in the repository outside the farm at `root` (staged,
/// modified or untracked).
pub fn outside(root: &Path) -> Result<Vec<String>> {
    let status = check(
        git(
            root,
            &[
                "status",
                "--porcelain",
                "-z",
                "--untracked-files=all",
                "--",
                ":(top)",
                ":(exclude).",
            ],
        )?,
        "git status",
    )?;
    Ok(parse_status(&String::from_utf8_lossy(&status))
        .into_iter()
        .map(|c| c.path)
        .collect())
}

/// A merge or cherry-pick in progress: `(the ref git sets, what it is)`. Git
/// refuses a partial commit during one, which `yaks commit` relies on.
pub fn pending_operation(root: &Path) -> Result<Option<(&'static str, &'static str)>> {
    // The ref git sets is a file in the worktree's own git dir: look there
    // when the farm is a farm branch, and ask git for any other.
    let farm = crate::store::farm_git(root);
    for (head, what) in [("MERGE_HEAD", "merge"), ("CHERRY_PICK_HEAD", "cherry-pick")] {
        let set = match &farm {
            Some(f) => f.git_dir.join(head).is_file(),
            None => git(root, &["rev-parse", "-q", "--verify", head])?
                .status
                .success(),
        };
        if set {
            return Ok(Some((head, what)));
        }
    }
    Ok(None)
}

/// Parse `git status --porcelain -z` output into one `Change` per path. A
/// rename is a deletion of the old path plus an addition of the new one.
pub fn parse_status(out: &str) -> Vec<Change> {
    let mut changes = Vec::new();
    let mut entries = out.split('\0').filter(|e| !e.is_empty());
    while let Some(entry) = entries.next() {
        let (xy, path) = entry.split_at(entry.len().min(2));
        let path = path.trim_start().to_string();
        let mut chars = xy.chars();
        let (x, y) = (chars.next().unwrap_or(' '), chars.next().unwrap_or(' '));
        let staged = x != ' ' && x != '?' && y == ' ';
        if matches!(x, 'R' | 'C') {
            if let Some(from) = entries.next().filter(|_| x == 'R') {
                changes.push(Change::new('D', from.to_string(), staged));
            }
            changes.push(Change::new('A', path, staged));
            continue;
        }
        let state = if xy == "??" || x == 'A' {
            'A'
        } else if x == 'D' || y == 'D' {
            'D'
        } else {
            'M'
        };
        changes.push(Change::new(state, path, staged));
    }
    changes
}

/// What happened to a yak's own file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Created,
    /// Its `status:` changed, in place.
    Moved {
        from: String,
        to: String,
    },
    /// Edited in place (a field, the description or a note).
    Updated,
    Removed,
}

/// Everything that changed for one yak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YakChange {
    /// `None` when only the yak's attachments changed.
    pub kind: Option<Kind>,
    /// The yak file's path now (`None` when removed, or attachments only).
    pub path: Option<String>,
    /// The yak file's path at HEAD (`None` for a created yak, or attachments
    /// only).
    pub old_path: Option<String>,
    /// A file under `.yaks/<id>/` (an attachment) changed.
    pub artifacts: bool,
    /// Every changed path of this yak is staged.
    pub staged: bool,
}

/// The changes sorted into per-yak facts, plus farm files that belong to no
/// yak (`config.yaml`, ...).
#[derive(Debug, Default)]
pub struct Classified {
    pub yaks: BTreeMap<String, YakChange>,
    pub other: Vec<Change>,
}

/// Classify `changes` per yak. In the flat layout a status change is a
/// modified `<id>.md` whose `status:` differs between HEAD and the working
/// tree. Each yak gets one verb: a move, else a new file (`created`), an edit
/// in place (`Updated`), or a deleted file (`Removed`).
pub fn classify(changes: &[Change]) -> Classified {
    let mut out = Classified::default();
    for c in changes {
        let Some(id) = yak_id_of(&c.path) else {
            out.other.push(c.clone());
            continue;
        };
        let entry = out.yaks.entry(id.clone()).or_insert(YakChange {
            kind: None,
            path: None,
            old_path: None,
            artifacts: false,
            staged: true,
        });
        entry.staged &= c.staged;
        if is_attachment(&c.path) {
            entry.artifacts = true;
        }
    }
    for (id, entry) in out.yaks.iter_mut() {
        let mine = |state: char| {
            changes.iter().find(|c| {
                !is_attachment(&c.path)
                    && c.state == state
                    && yak_id_of(&c.path).as_deref() == Some(id.as_str())
            })
        };
        let (gone, now) = (mine('D'), mine('A').or_else(|| mine('M')));
        match (gone, now) {
            // One path per yak, so a deletion and an addition for the same id
            // cannot both be its own file; read it as an edit if it ever is.
            (Some(gone), Some(now)) => {
                entry.old_path = Some(gone.path.clone());
                entry.path = Some(now.path.clone());
                entry.kind = Some(Kind::Updated);
            }
            (None, Some(now)) if now.state == 'A' => {
                entry.kind = Some(Kind::Created);
                entry.path = Some(now.path.clone());
            }
            (None, Some(now)) => {
                entry.kind = Some(match (&now.status_before, &now.status_after) {
                    (Some(from), Some(to)) if from != to => Kind::Moved {
                        from: from.clone(),
                        to: to.clone(),
                    },
                    _ => Kind::Updated,
                });
                entry.old_path = Some(now.path.clone());
                entry.path = Some(now.path.clone());
            }
            (Some(gone), None) => {
                entry.kind = Some(Kind::Removed);
                entry.old_path = Some(gone.path.clone());
            }
            (None, None) => {} // attachments only
        }
    }
    out
}

/// `path` made relative to the farm root: everything after the last `.yaks`
/// component (a committed farm is always a `.yaks/` directory, as
/// [`yak_id_of`] relies on), or `path` itself when it has none.
fn farm_rel(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    match parts.iter().rposition(|p| *p == ".yaks") {
        Some(i) => parts[i + 1..].join("/"),
        None => path.to_string(),
    }
}

/// A file under `.yaks/<herd>/yaks/<id>/`.
fn is_attachment(path: &str) -> bool {
    store::attachment_owner(&farm_rel(path)).is_some()
}

/// A yak's own file: `.yaks/<herd>/yaks/<id>.md`.
fn is_flat_yak_file(path: &str) -> bool {
    let rel = farm_rel(path);
    store::path_owner(&rel).is_some() && store::attachment_owner(&rel).is_none()
}
