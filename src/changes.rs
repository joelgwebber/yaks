//! The farm's uncommitted changes, read from git and classified per yak.
//!
//! The one place that turns `git status` under the farm into yak terms. Both
//! `yaks commit` (which builds its message from it) and `yaks status` (which
//! lists it) use [`classify`], so the two cannot disagree about what happened
//! to a yak.

use crate::preflight::yak_id_of;
use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::path::Path;
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
}

/// The farm's changes at `root` (paths relative to the repository top), or
/// `None` for a private farm: nothing under it is tracked by git.
pub fn read(root: &Path) -> Result<Option<Vec<Change>>> {
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
    let status = check(
        git(
            root,
            &[
                "status",
                "--porcelain",
                "-z",
                "--untracked-files=all",
                "--",
                ".",
            ],
        )?,
        "git status",
    )?;
    Ok(Some(parse_status(&String::from_utf8_lossy(&status))))
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
    for (head, what) in [("MERGE_HEAD", "merge"), ("CHERRY_PICK_HEAD", "cherry-pick")] {
        if git(root, &["rev-parse", "-q", "--verify", head])?
            .status
            .success()
        {
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
                changes.push(Change {
                    state: 'D',
                    path: from.to_string(),
                    staged,
                });
            }
            changes.push(Change {
                state: 'A',
                path,
                staged,
            });
            continue;
        }
        let state = if xy == "??" || x == 'A' {
            'A'
        } else if x == 'D' || y == 'D' {
            'D'
        } else {
            'M'
        };
        changes.push(Change {
            state,
            path,
            staged,
        });
    }
    changes
}

/// What happened to a yak's own file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Created,
    /// The file vanished from one status directory and appeared in another.
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
    /// `None` when only files under `artifacts/<id>/` changed.
    pub kind: Option<Kind>,
    /// The yak file's path now (`None` when removed, or artifacts only).
    pub path: Option<String>,
    /// The yak file's path at HEAD (`None` for a created yak, or artifacts only).
    pub old_path: Option<String>,
    /// A file under `artifacts/<id>/` changed.
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

/// Classify `changes` per yak. A yak whose file vanished from one status
/// directory and appeared in another moved, so both sides are looked at per id
/// before the verb is chosen: a move, else a new file (`created`), an edit in
/// place (`Updated`), or a deleted file (`Removed`).
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
        if is_artifact(&c.path) {
            entry.artifacts = true;
        }
    }
    for (id, entry) in out.yaks.iter_mut() {
        let mine = |state: char| {
            changes.iter().find(|c| {
                !is_artifact(&c.path)
                    && c.state == state
                    && yak_id_of(&c.path).as_deref() == Some(id.as_str())
            })
        };
        let (gone, now) = (mine('D'), mine('A').or_else(|| mine('M')));
        match (gone, now) {
            (Some(gone), Some(now)) => {
                entry.kind = Some(Kind::Moved {
                    from: status_dir(&gone.path).to_string(),
                    to: status_dir(&now.path).to_string(),
                });
                entry.old_path = Some(gone.path.clone());
                entry.path = Some(now.path.clone());
            }
            (None, Some(now)) if now.state == 'A' => {
                entry.kind = Some(Kind::Created);
                entry.path = Some(now.path.clone());
            }
            (None, Some(now)) => {
                entry.kind = Some(Kind::Updated);
                entry.old_path = Some(now.path.clone());
                entry.path = Some(now.path.clone());
            }
            (Some(gone), None) => {
                entry.kind = Some(Kind::Removed);
                entry.old_path = Some(gone.path.clone());
            }
            (None, None) => {} // artifacts only
        }
    }
    out
}

fn is_artifact(path: &str) -> bool {
    path.split('/').any(|p| p == "artifacts")
}

/// The status directory a yak file path sits in (`shorn` for `.yaks/shorn/x.md`).
fn status_dir(path: &str) -> &str {
    let mut parts = path.rsplit('/');
    parts.next();
    parts.next().unwrap_or("")
}
