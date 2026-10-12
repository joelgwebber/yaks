//! The farm's uncommitted changes, read from git and classified per yak.
//!
//! The one place that turns `git status` under the farm into yak terms. Both
//! `yaks commit` (which builds its message from it) and `yaks status` (which
//! lists it) use [`classify`], so the two cannot disagree about what happened
//! to a yak.

use crate::model::Status;
use crate::preflight::yak_id_of;
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
    let mut changes = parse_status(&String::from_utf8_lossy(&status));
    if changes.iter().any(|c| is_flat_yak_file(&c.path)) {
        let top = check(
            git(root, &["rev-parse", "--show-toplevel"])?,
            "git rev-parse",
        )?;
        let top = PathBuf::from(String::from_utf8_lossy(&top).trim());
        for c in changes.iter_mut().filter(|c| is_flat_yak_file(&c.path)) {
            read_statuses(root, &top, c);
        }
    }
    Ok(Some(changes))
}

/// Fill `c`'s `status:` fields from HEAD's text and the working-tree file (a
/// flat yak file is the only place a status change is visible to git).
fn read_statuses(root: &Path, top: &Path, c: &mut Change) {
    let status_of =
        |text: &str| store::parse_task(text, Status::Hairy).map(|t| t.status.dir().to_string());
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
    /// Its `status:` changed (in place), or (an old-layout farm in history)
    /// the file vanished from one status directory and appeared in another.
    Moved {
        from: String,
        to: String,
    },
    /// The farm layout migration: the old-layout file was deleted and the flat
    /// file added with the same status. Never listed per yak; see
    /// [`Classified::migrated`].
    Migrated,
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

/// The farm layout migration in the changes: how many yaks it moved, and
/// whether all of it (their files, attachments and `schema`) is staged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub yaks: usize,
    pub staged: bool,
}

impl Migration {
    /// The one line `yaks status` shows for it.
    pub fn summary(&self) -> String {
        format!("farm layout migrated: {} yaks", self.yaks)
    }

    /// The part of `yaks commit`'s message that names it.
    pub fn message(&self) -> String {
        format!("migrate the farm to the flat layout ({} yaks)", self.yaks)
    }
}

/// The changes sorted into per-yak facts, plus farm files that belong to no
/// yak (`config.yaml`, ...).
#[derive(Debug, Default)]
pub struct Classified {
    /// Per yak, except the ones the migration moved (see `migrated`).
    pub yaks: BTreeMap<String, YakChange>,
    pub other: Vec<Change>,
    pub migrated: Option<Migration>,
}

/// Classify `changes` per yak. In the flat layout a status change is a
/// modified `<id>.md` whose `status:` differs between HEAD and the working
/// tree. Each yak gets one verb: a move, else a new file (`created`), an edit
/// in place (`Updated`), or a deleted file (`Removed`). The old-layout pairing
/// (a deleted and an added path for one id) is [`legacy_pair`]. A yak the
/// layout migration moved is counted in `migrated` instead of listed, and the
/// `schema` file change that comes with it is folded in.
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
            (Some(gone), Some(now)) => legacy_pair(entry, gone, now),
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
    let migrated: Vec<String> = out
        .yaks
        .iter()
        .filter(|(_, y)| y.kind == Some(Kind::Migrated))
        .map(|(id, _)| id.clone())
        .collect();
    if !migrated.is_empty() {
        let mut staged = true;
        for id in &migrated {
            staged &= out.yaks.remove(id).is_some_and(|y| y.staged);
        }
        out.other.retain(|c| {
            let is_schema = farm_rel(&c.path) == "schema";
            if is_schema {
                staged &= c.staged;
            }
            !is_schema
        });
        out.migrated = Some(Migration {
            yaks: migrated.len(),
            staged,
        });
    }
    out
}

/// TRANSITIONAL (yaks-bbda deletes it with the old layout): a yak with both a
/// deleted and an added/modified file moved between status directories (a
/// commit from before the flat layout) or, when the added file is the flat
/// `<id>.md` carrying the status the old directory named, was moved by the
/// layout migration. A flat side's status comes from its `status:` field.
fn legacy_pair(entry: &mut YakChange, gone: &Change, now: &Change) {
    entry.old_path = Some(gone.path.clone());
    entry.path = Some(now.path.clone());
    let from = status_at(gone);
    let to = status_at(now);
    entry.kind = Some(match (from, to) {
        (Some(from), Some(to)) if from == to && is_legacy_path(&gone.path) && now.state == 'A' => {
            Kind::Migrated
        }
        (Some(from), Some(to)) => Kind::Moved { from, to },
        _ => Kind::Updated,
    });
}

/// The status a yak file's change puts it in: the directory an old-layout path
/// sits in, or a flat file's `status:` (at HEAD for a deletion).
fn status_at(c: &Change) -> Option<String> {
    if is_legacy_path(&c.path) {
        return farm_rel(&c.path).split('/').next().map(String::from);
    }
    if c.state == 'D' {
        c.status_before.clone()
    } else {
        c.status_after.clone()
    }
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

/// A file under `.yaks/<id>/` (or, transitionally, `.yaks/artifacts/<id>/`).
fn is_attachment(path: &str) -> bool {
    store::attachment_owner(&farm_rel(path)).is_some()
}

/// A yak's own flat file: `.yaks/<id>.md`.
fn is_flat_yak_file(path: &str) -> bool {
    let rel = farm_rel(path);
    !rel.contains('/') && rel.ends_with(".md")
}

/// An old-layout yak file: `.yaks/<status>/<id>.md`.
fn is_legacy_path(path: &str) -> bool {
    let rel = farm_rel(path);
    let mut parts = rel.split('/');
    matches!((parts.next(), parts.next(), parts.next()),
        (Some(dir), Some(file), None) if store::legacy::is_status_dir(dir) && file.ends_with(".md"))
}
