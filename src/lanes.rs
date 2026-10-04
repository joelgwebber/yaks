//! `yaks lanes`: the OTHER checkouts of this repository (git worktrees and
//! Delta clones) and what each one's farm says relative to this checkout's.
//!
//! Plain git plus the filesystem; no Delta database, no Delta CLI. Everything
//! here is strictly read-only in the other checkouts: git is invoked as
//! `git --no-optional-locks -C <lane> ...` for read-only plumbing only (never
//! fetch/checkout/add), and farms are compared by reading `.yaks` directories
//! (see [`compare_farms`]).
//!
//! A lane's farm is compared with the farm at its MERGE-BASE with this checkout
//! (committed history, read with `git ls-tree`/`cat-file` into a temp dir of
//! our own), so a lane that is merely behind shows nothing of its own and what
//! it does show is exactly what it changed, committed or not. When there is no
//! merge-base farm to read, the lane is compared with this checkout's working
//! tree instead and says so (`vs this checkout`).
//!
//! Discovery has two sources, merged by canonical path into one lane each:
//!
//! 1. `git worktree list --porcelain` (plain `git worktree add` checkouts).
//! 2. Delta clones. Inside Delta `git worktree list` shows only the current
//!    clone, so sibling directories are scanned too. A Delta checkout is
//!    `<root>/<dir>/<name>` in both layouts (laptop
//!    `<repo>/.delta/worktrees/<dir>/<name>`, managed machine
//!    `~/.local/share/delta/worktrees/<dir>/<name>`). With `T` this checkout's
//!    git top-level and `name` its basename the candidates are
//!    `dirname(dirname(T))/*/<name>`, `T/.delta/worktrees/*/<name>`, and, when
//!    `T` is itself a laptop-layout clone, the repository checkout that holds
//!    it. A candidate is accepted only if it has its own `.git` and is the same
//!    repository ([`same_repo`]).

use crate::model::Status;
use crate::store;
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

const EVERY: [Status; 4] = [Status::Hairy, Status::Shaving, Status::Shorn, Status::Dead];

/// How a lane was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneKind {
    /// Listed by `git worktree list`.
    Worktree,
    /// Found only by scanning Delta's sibling-clone layout.
    Delta,
}

impl LaneKind {
    pub fn as_str(self) -> &'static str {
        match self {
            LaneKind::Worktree => "worktree",
            LaneKind::Delta => "delta",
        }
    }
}

/// A yak whose status directory differs between the lane's farm and the one it
/// is compared with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moved {
    pub id: String,
    /// Status directory in the compared-with farm (merge-base or this one).
    pub from: &'static str,
    /// Status directory in the lane's farm.
    pub to: &'static str,
}

/// What the lane's farm has that the farm it is compared with lacks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FarmDelta {
    /// Yaks that exist only in the lane.
    pub added: Vec<String>,
    /// Yaks that exist in both with different status directories.
    pub moved: Vec<Moved>,
    /// `(id, count)` of note entries the lane has beyond ours.
    pub notes: Vec<(String, usize)>,
    /// `(id, value)` of lane yaks whose `needs:` is set and differs from ours.
    pub needs: Vec<(String, String)>,
}

impl FarmDelta {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.moved.is_empty()
            && self.notes.is_empty()
            && self.needs.is_empty()
    }
}

/// The lane's farm relative to ours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaneFarm {
    /// The lane has a farm of its own; here is how it differs from ours.
    Own(FarmDelta),
    /// The lane resolves to the same farm directory as this checkout.
    Shared,
    /// Discovery found no farm for the lane (a private farm: set `YAKS_DIR`).
    None,
    /// The lane has a farm but it could not be read.
    Unreadable(String),
}

/// One other checkout of this repository.
#[derive(Debug, Clone)]
pub struct Lane {
    pub path: PathBuf,
    pub kind: LaneKind,
    /// `None` when HEAD is detached (or unreadable).
    pub branch: Option<String>,
    /// Abbreviated HEAD sha; `None` when the lane's git state cannot be read.
    pub head: Option<String>,
    /// Commits the lane's HEAD has that this checkout's HEAD lacks (0 when the
    /// object is unknown here).
    pub ahead: usize,
    /// Commits this checkout's HEAD has that the lane's HEAD lacks (0 when the
    /// object is unknown here).
    pub behind: usize,
    /// Changed + untracked file count (`git status --porcelain`).
    pub dirty: Option<usize>,
    /// Newest mtime anywhere under the lane's farm: a liveness hint (finished
    /// threads' checkouts persist).
    pub farm_activity: Option<String>,
    pub farm: LaneFarm,
    /// Short sha of the merge-base the farm delta is relative to; `None` means
    /// it is relative to this checkout's farm instead (no merge-base farm).
    pub farm_base: Option<String>,
    /// Set when the lane's git state could not be read; the lane is still
    /// listed.
    pub error: Option<String>,
}

/// List the other checkouts of the repository containing `cwd`, relative to
/// the farm at `farm_root` (this checkout's farm). Sorted by path.
pub fn discover(cwd: &Path, farm_root: &Path) -> Result<Vec<Lane>> {
    let top = PathBuf::from(
        git(cwd, &["rev-parse", "--show-toplevel"]).context("yaks lanes needs a git repository")?,
    );
    let top = canonical(&top);
    let own_common = common_dir(&top);

    // canonical path -> kind. Worktrees win over a Delta-only find.
    let mut found: BTreeMap<PathBuf, LaneKind> = BTreeMap::new();
    for p in worktree_paths(&top) {
        found.insert(canonical(&p), LaneKind::Worktree);
    }
    for p in delta_candidates(&top) {
        let p = canonical(&p);
        if found.contains_key(&p) {
            continue;
        }
        if own_common.as_deref().is_some_and(|c| same_repo(c, &p)) {
            found.insert(p, LaneKind::Delta);
        }
    }
    found.remove(&top);

    let ours = store::load(farm_root, &EVERY).unwrap_or_default();
    let our_farm = canonical(farm_root);
    Ok(found
        .into_iter()
        .map(|(path, kind)| inspect(&top, &path, kind, &our_farm, &ours))
        .collect())
}

/// Gather one lane's report. Never fails: unreadable parts become `error`.
fn inspect(
    top: &Path,
    path: &Path,
    kind: LaneKind,
    our_farm: &Path,
    ours: &[crate::model::Task],
) -> Lane {
    let mut lane = Lane {
        path: path.to_path_buf(),
        kind,
        branch: None,
        head: None,
        ahead: 0,
        behind: 0,
        dirty: None,
        farm_activity: None,
        farm: LaneFarm::None,
        farm_base: None,
        error: None,
    };
    let mut full_head = None;
    match git_state(top, path) {
        Ok(g) => {
            lane.branch = g.branch;
            lane.head = Some(g.head);
            lane.ahead = g.ahead;
            lane.behind = g.behind;
            lane.dirty = Some(g.dirty);
            full_head = Some(g.full);
        }
        Err(e) => lane.error = Some(format!("{e:#}")),
    }
    // The farm side is plain filesystem reads, so it is reported even when git
    // could not be read.
    match store::discover_with(path, None) {
        Err(_) => lane.farm = LaneFarm::None,
        Ok(d) => {
            let root = canonical(&d.root);
            if root == our_farm {
                lane.farm = LaneFarm::Shared;
            } else {
                lane.farm_activity = newest_mtime(&root).map(format_time);
                lane.farm = match store::load(&root, &EVERY) {
                    Ok(theirs) => {
                        // What the lane changed is relative to the merge-base,
                        // not to our (possibly newer) checkout.
                        let base = full_head
                            .as_deref()
                            .and_then(|h| base_farm(top, h, path, &root));
                        match base {
                            Some((sha, base_tasks)) => {
                                lane.farm_base = Some(sha);
                                LaneFarm::Own(compare_farms(&base_tasks, &theirs))
                            }
                            None => LaneFarm::Own(compare_farms(ours, &theirs)),
                        }
                    }
                    Err(e) => LaneFarm::Unreadable(format!("{e:#}")),
                };
            }
        }
    }
    lane
}

struct GitState {
    branch: Option<String>,
    head: String,
    full: String,
    ahead: usize,
    behind: usize,
    dirty: usize,
}

fn git_state(top: &Path, lane: &Path) -> Result<GitState> {
    let full = git(lane, &["rev-parse", "HEAD"])?;
    let head = git(lane, &["rev-parse", "--short", "HEAD"])?;
    // Exit 1 from `symbolic-ref -q` means a detached HEAD.
    let branch = git(lane, &["symbolic-ref", "--short", "-q", "HEAD"])
        .ok()
        .filter(|b| !b.is_empty());
    // Counted in OUR repo: a lane commit we do not have is unknown here => 0.
    let count = |range: String| {
        git(top, &["rev-list", "--count", &range])
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(0)
    };
    let ahead = count(format!("HEAD..{full}"));
    let behind = count(format!("{full}..HEAD"));
    // `--no-optional-locks` (see `git`) keeps status from refreshing the index.
    let status = git_raw(lane, &["status", "--porcelain", "--untracked-files=all"])?;
    let dirty = status.lines().filter(|l| !l.trim().is_empty()).count();
    Ok(GitState {
        branch,
        head,
        full,
        ahead,
        behind,
        dirty,
    })
}

/// A temp directory of ours, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Result<TempDir> {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "yaks-lanes-base-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).with_context(|| format!("creating {}", p.display()))?;
        Ok(TempDir(p))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The lane's farm as of its merge-base with this checkout: `(short sha,
/// tasks)`. `None` when there is nothing trustworthy to compare against (no
/// common history, the lane's commit unknown here, a farm outside the lane's
/// checkout, or no farm committed at the base); the caller then falls back to
/// comparing with this checkout's working tree.
///
/// Only our own repo is read (the base is an ancestor of our HEAD, and Delta
/// clones share objects with it); the lane is never touched. The base's yak
/// files are materialized into a temp dir of ours and loaded like any farm.
fn base_farm(
    top: &Path,
    lane_head: &str,
    lane: &Path,
    lane_farm: &Path,
) -> Option<(String, Vec<crate::model::Task>)> {
    let rel = lane_farm.strip_prefix(lane).ok()?;
    let rel = rel.to_str()?.replace('\\', "/");
    let base = git(top, &["merge-base", "HEAD", lane_head]).ok()?;
    let short = git(top, &["rev-parse", "--short", &base]).ok()?;

    // Status-dir task files only: `<rel>/<status>/<id>.md`.
    let tree = git_raw(top, &["ls-tree", "-r", "-z", &base, "--", &rel]).ok()?;
    let mut files: Vec<(&str, String)> = Vec::new(); // (blob sha, "<status>/<file>")
    for entry in tree.split('\0').filter(|e| !e.is_empty()) {
        let (meta, path) = entry.split_once('\t')?;
        let mut m = meta.split(' ');
        if m.nth(1) != Some("blob") {
            continue;
        }
        let sha = m.next()?;
        let Some(under) = path.strip_prefix(&rel).and_then(|p| p.strip_prefix('/')) else {
            continue;
        };
        let mut parts = under.split('/');
        let (Some(dir), Some(file), None) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        if file.ends_with(".md") && EVERY.iter().any(|s| s.dir() == dir) {
            files.push((sha, format!("{dir}/{file}")));
        }
    }
    if files.is_empty() {
        return None;
    }

    let tmp = TempDir::new().ok()?;
    let farm = tmp.0.join("farm");
    // One `cat-file --batch` for all blobs; its stdin is a file so there is no
    // pipe to deadlock on.
    let input = tmp.0.join("blobs");
    let list: String = files.iter().map(|(sha, _)| format!("{sha}\n")).collect();
    fs::write(&input, list).ok()?;
    let out = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(top)
        .args(["cat-file", "--batch"])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .stdin(fs::File::open(&input).ok()?)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    // `<sha> blob <size>\n<content>\n`, one per input line, in order.
    let mut rest = &out.stdout[..];
    for (_, name) in &files {
        let nl = rest.iter().position(|&b| b == b'\n')?;
        let size: usize = std::str::from_utf8(&rest[..nl])
            .ok()?
            .rsplit(' ')
            .next()?
            .parse()
            .ok()?;
        let body = rest.get(nl + 1..nl + 1 + size)?;
        rest = rest.get(nl + 1 + size + 1..)?;
        let dest = farm.join(name);
        fs::create_dir_all(dest.parent()?).ok()?;
        fs::write(dest, body).ok()?;
    }
    let tasks = store::load(&farm, &EVERY).ok()?;
    Some((short, tasks))
}

/// Compare a lane's tasks to ours (see [`FarmDelta`]). Pure; ids sorted.
pub fn compare_farms(ours: &[crate::model::Task], theirs: &[crate::model::Task]) -> FarmDelta {
    let mine: BTreeMap<&str, &crate::model::Task> =
        ours.iter().map(|t| (t.id.as_str(), t)).collect();
    let mut d = FarmDelta::default();
    let mut theirs: Vec<&crate::model::Task> = theirs.iter().collect();
    theirs.sort_by(|a, b| a.id.cmp(&b.id));
    for t in theirs {
        match mine.get(t.id.as_str()) {
            None => d.added.push(t.id.clone()),
            Some(m) => {
                if m.status != t.status {
                    d.moved.push(Moved {
                        id: t.id.clone(),
                        from: m.status.dir(),
                        to: t.status.dir(),
                    });
                }
                let new = store::parse_notes(&t.body)
                    .len()
                    .saturating_sub(store::parse_notes(&m.body).len());
                if new > 0 {
                    d.notes.push((t.id.clone(), new));
                }
            }
        }
        // Only a `needs:` that is new or changed relative to ours: one both
        // farms already carry is not news from this lane.
        let ours_needs = mine.get(t.id.as_str()).and_then(|m| m.needs.as_ref());
        if let Some(n) = t.needs.as_ref().filter(|n| Some(*n) != ours_needs) {
            d.needs.push((t.id.clone(), n.clone()));
        }
    }
    d
}

// -- git / filesystem plumbing ---------------------------------------------

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

fn canonical(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// Real checkouts from `git worktree list --porcelain`: skips bare, prunable,
/// missing and `.git`-less entries. (Inside Delta the entry is the clone's
/// separate git dir, which holds no `.git`, so it drops out here.)
fn worktree_paths(top: &Path) -> Vec<PathBuf> {
    let Ok(text) = git_raw(top, &["worktree", "list", "--porcelain"]) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for block in text.split("\n\n") {
        let mut path: Option<&str> = None;
        let mut skip = false;
        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(p);
            } else if line == "bare" || line.starts_with("prunable") {
                skip = true;
            }
        }
        if let (Some(p), false) = (path, skip) {
            let p = PathBuf::from(p);
            if p.join(".git").exists() {
                out.push(p);
            }
        }
    }
    out
}

/// Delta-layout sibling candidates for the checkout at `top` (see the module
/// docs). Not yet checked for being the same repository.
fn delta_candidates(top: &Path) -> Vec<PathBuf> {
    let Some(name) = top.file_name() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let root = top.parent().and_then(Path::parent);
    if let Some(root) = root {
        out.extend(scan_siblings(root, name));
        // Laptop layout: `<repo>/.delta/worktrees/<dir>/<name>`; the repository
        // checkout itself is a lane too.
        if root.ends_with(".delta/worktrees") {
            if let Some(repo) = root.parent().and_then(Path::parent) {
                if repo.join(".git").exists() {
                    out.push(repo.to_path_buf());
                }
            }
        }
    }
    out.extend(scan_siblings(&top.join(".delta/worktrees"), name));
    out
}

/// `<base>/*/<name>` entries that have their own `.git`.
fn scan_siblings(base: &Path, name: &std::ffi::OsStr) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(base) else {
        return Vec::new();
    };
    rd.flatten()
        .map(|e| e.path().join(name))
        .filter(|c| c.join(".git").exists())
        .collect()
}

/// The shared (common) git dir of the checkout at `checkout`.
fn common_dir(checkout: &Path) -> Option<PathBuf> {
    let d = git(checkout, &["rev-parse", "--git-common-dir"]).ok()?;
    let d = PathBuf::from(d);
    Some(canonical(&if d.is_absolute() {
        d
    } else {
        checkout.join(d)
    }))
}

/// Is the checkout at `cand` the same repository as the one whose common git
/// dir is `own`? Same `objects/info/alternates`, else the same
/// `remote.origin.url`. A same-named but different repository is rejected.
fn same_repo(own: &Path, cand: &Path) -> bool {
    let Some(theirs) = common_dir(cand) else {
        return false;
    };
    if theirs == own {
        return true;
    }
    let alternates = |c: &Path| {
        fs::read_to_string(c.join("objects/info/alternates"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    if let (Some(a), Some(b)) = (alternates(own), alternates(&theirs)) {
        return a == b;
    }
    let origin = |c: &Path| {
        let cfg = c.join("config");
        git(
            c,
            &[
                "config",
                "--file",
                &cfg.display().to_string(),
                "--get",
                "remote.origin.url",
            ],
        )
        .ok()
        .filter(|s| !s.is_empty())
    };
    matches!((origin(own), origin(&theirs)), (Some(a), Some(b)) if a == b)
}

/// Newest mtime of anything under `root` (symlinks not followed).
fn newest_mtime(root: &Path) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let Ok(md) = fs::symlink_metadata(e.path()) else {
                continue;
            };
            if let Ok(m) = md.modified() {
                if newest.is_none_or(|n| m > n) {
                    newest = Some(m);
                }
            }
            if md.is_dir() {
                stack.push(e.path());
            }
        }
    }
    newest
}

fn format_time(t: SystemTime) -> String {
    DateTime::<Utc>::from(t)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

// -- rendering ----------------------------------------------------------------

fn farm_message(f: &LaneFarm) -> Option<String> {
    match f {
        LaneFarm::Own(_) => None,
        LaneFarm::Shared => Some("shares this farm".into()),
        LaneFarm::None => Some("no farm here (private? set YAKS_DIR)".into()),
        LaneFarm::Unreadable(e) => Some(format!("farm unreadable: {e}")),
    }
}

/// JSON for `--json`: one object per lane with the same fields as the table.
pub fn to_json(lanes: &[Lane]) -> Value {
    Value::Array(
        lanes
            .iter()
            .map(|l| {
                let farm = match &l.farm {
                    LaneFarm::Own(d) => json!({
                        "state": "own",
                        // What the delta is relative to: the lane's merge-base
                        // with this checkout (`base` = its short sha), or, when
                        // that cannot be read, this checkout's farm.
                        "vs": if l.farm_base.is_some() { "merge-base" } else { "checkout" },
                        "base": l.farm_base,
                        "added": d.added,
                        "moved": d.moved.iter()
                            .map(|m| json!({"id": m.id, "from": m.from, "to": m.to}))
                            .collect::<Vec<_>>(),
                        "notes": d.notes.iter()
                            .map(|(id, n)| json!({"id": id, "new": n}))
                            .collect::<Vec<_>>(),
                        "needs": d.needs.iter()
                            .map(|(id, v)| json!({"id": id, "needs": v}))
                            .collect::<Vec<_>>(),
                    }),
                    other => json!({
                        "state": match other {
                            LaneFarm::Shared => "shared",
                            LaneFarm::None => "none",
                            _ => "unreadable",
                        },
                        "message": farm_message(other),
                    }),
                };
                json!({
                    "path": l.path.display().to_string(),
                    "kind": l.kind.as_str(),
                    "branch": l.branch,
                    "head": l.head,
                    "ahead": l.ahead,
                    "behind": l.behind,
                    "dirty": l.dirty,
                    "farm_activity": l.farm_activity,
                    "farm": farm,
                    "error": l.error,
                })
            })
            .collect(),
    )
}

/// The compact human table: one row per lane, then indented farm detail.
pub fn render(lanes: &[Lane]) -> String {
    if lanes.is_empty() {
        return "No other checkouts found.\n".into();
    }
    let rows: Vec<[String; 6]> = lanes
        .iter()
        .map(|l| {
            [
                l.kind.as_str().to_string(),
                match (&l.branch, &l.head) {
                    (Some(b), _) => b.clone(),
                    (None, Some(_)) => "(detached)".into(),
                    (None, None) => "-".into(),
                },
                l.head.clone().unwrap_or_else(|| "-".into()),
                if l.head.is_some() {
                    format!("+{} -{}", l.ahead, l.behind)
                } else {
                    "-".into()
                },
                l.dirty.map_or_else(|| "-".into(), |d| d.to_string()),
                l.farm_activity.clone().unwrap_or_else(|| "-".into()),
            ]
        })
        .collect();
    let heads = [
        "KIND",
        "BRANCH",
        "HEAD",
        "+AHEAD -BEHIND",
        "DIRTY",
        "FARM ACTIVE",
    ];
    let widths: Vec<usize> = (0..6)
        .map(|i| {
            rows.iter()
                .map(|r| r[i].chars().count())
                .chain([heads[i].len()])
                .max()
                .unwrap_or(0)
        })
        .collect();
    let line = |cells: Vec<&str>, path: &str| {
        let mut s = String::new();
        for (c, w) in cells.iter().zip(&widths) {
            s.push_str(&format!("{c:<w$}  "));
        }
        s.push_str(path);
        s.push('\n');
        s
    };
    let mut out = line(heads.to_vec(), "PATH");
    for (l, r) in lanes.iter().zip(&rows) {
        out.push_str(&line(
            r.iter().map(String::as_str).collect(),
            &l.path.display().to_string(),
        ));
        if let Some(e) = &l.error {
            out.push_str(&format!("  error: {e}\n"));
        }
        match &l.farm {
            LaneFarm::Own(d) if d.is_empty() && l.farm_base.is_some() => {
                out.push_str("  farm: no changes of its own\n")
            }
            LaneFarm::Own(d) if d.is_empty() => {
                out.push_str("  farm: no differences (vs this checkout)\n")
            }
            LaneFarm::Own(d) => {
                let n: usize = d.notes.iter().map(|(_, n)| n).sum();
                let mut parts = Vec::new();
                if !d.added.is_empty() {
                    parts.push(format!("{} new", d.added.len()));
                }
                if !d.moved.is_empty() {
                    parts.push(format!("{} moved", d.moved.len()));
                }
                if n > 0 {
                    parts.push(format!("{n} new notes"));
                }
                if !d.needs.is_empty() {
                    parts.push(format!("{} needs", d.needs.len()));
                }
                let vs = if l.farm_base.is_some() {
                    ""
                } else {
                    " (vs this checkout)"
                };
                out.push_str(&format!("  farm: {}{vs}\n", parts.join(", ")));
                for id in &d.added {
                    out.push_str(&format!("    + {id}\n"));
                }
                for m in &d.moved {
                    out.push_str(&format!("    {} {} -> {}\n", m.id, m.from, m.to));
                }
                for (id, n) in &d.notes {
                    out.push_str(&format!("    {id} +{n} notes\n"));
                }
                for (id, v) in &d.needs {
                    out.push_str(&format!("    {id} needs: {v}\n"));
                }
            }
            other => {
                if let Some(m) = farm_message(other) {
                    out.push_str(&format!("  farm: {m}\n"));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn base(tag: &str) -> PathBuf {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "yaks-lanes-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        canonical(&p)
    }

    /// An origin URL unique to one test's temp dir, so concurrently running
    /// tests (siblings under the shared temp dir) never look like one repo.
    fn url(base: &Path, name: &str) -> String {
        format!(
            "https://example.com/{}/{name}.git",
            base.file_name().unwrap().to_string_lossy()
        )
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

    fn task_file(root: &Path, status: &str, id: &str, extra_fm: &str, body: &str) {
        let dir = root.join(".yaks").join(status);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntitle: {id} title\ntype: task\npriority: 3\n\
                 created: '2026-01-01T00:00:00Z'\nupdated: '2026-01-01T00:00:00Z'\n\
                 {extra_fm}---\n{body}\n"
            ),
        )
        .unwrap();
    }

    /// A repo at `dir` with a committed farm of three yaks.
    fn repo_with_farm(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        sh(dir, &["init", "-q"]);
        task_file(dir, "hairy", "yaks-aaaa", "", "first");
        task_file(dir, "hairy", "yaks-bbbb", "", "second");
        task_file(dir, "shorn", "yaks-cccc", "", "third");
        sh(dir, &["add", "-A"]);
        sh(dir, &["commit", "-q", "-m", "init"]);
    }

    fn lanes_of(top: &Path) -> Vec<Lane> {
        discover(top, &top.join(".yaks")).unwrap()
    }

    fn paths(lanes: &[Lane]) -> Vec<PathBuf> {
        lanes.iter().map(|l| l.path.clone()).collect()
    }

    #[test]
    fn no_siblings_is_an_empty_list() {
        let b = base("none");
        let t = b.join("repo");
        repo_with_farm(&t);
        assert!(lanes_of(&t).is_empty());
        assert_eq!(render(&[]), "No other checkouts found.\n");
    }

    #[test]
    fn git_worktree_lane_reports_git_state_and_farm_delta() {
        let b = base("wt");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "lane", wt.to_str().unwrap()],
        );

        // A commit the lane has and we lack; an uncommitted file.
        fs::write(wt.join("x.txt"), "x").unwrap();
        sh(&wt, &["add", "x.txt"]);
        sh(&wt, &["commit", "-q", "-m", "lane work"]);
        fs::write(wt.join("untracked.txt"), "u").unwrap();

        // Farm changes in the lane: an added yak, a moved yak, an added note,
        // and `needs:` on an existing and on a new yak.
        task_file(&wt, "hairy", "yaks-dddd", "", "new");
        fs::create_dir_all(wt.join(".yaks/shaving")).unwrap();
        fs::rename(
            wt.join(".yaks/hairy/yaks-aaaa.md"),
            wt.join(".yaks/shaving/yaks-aaaa.md"),
        )
        .unwrap();
        task_file(
            &wt,
            "hairy",
            "yaks-bbbb",
            "",
            "second\n\n---\n\u{25b8} 2026-02-02T00:00:00Z [w]\nfound it",
        );
        task_file(&wt, "shorn", "yaks-cccc", "needs: human\n", "third");
        task_file(&wt, "hairy", "yaks-eeee", "needs: human\n", "asks");

        let lanes = lanes_of(&t);
        assert_eq!(paths(&lanes), vec![wt.clone()]);
        let l = &lanes[0];
        assert_eq!(l.kind, LaneKind::Worktree);
        assert_eq!(l.branch.as_deref(), Some("lane"));
        assert_eq!(
            l.head.as_deref(),
            Some(&*sh(&wt, &["rev-parse", "--short", "HEAD"]))
        );
        assert_eq!((l.ahead, l.behind), (1, 0));
        assert!(l.farm_base.is_some());
        // untracked.txt plus the yak edits (new files, a rename, a modified file).
        assert!(l.dirty.unwrap() >= 5, "dirty = {:?}", l.dirty);
        assert!(l.farm_activity.is_some());
        assert!(l.error.is_none());
        let LaneFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert_eq!(d.added, vec!["yaks-dddd", "yaks-eeee"]);
        assert_eq!(
            d.moved,
            vec![Moved {
                id: "yaks-aaaa".into(),
                from: "hairy",
                to: "shaving"
            }]
        );
        assert_eq!(d.notes, vec![("yaks-bbbb".to_string(), 1)]);
        assert_eq!(
            d.needs,
            vec![
                ("yaks-cccc".to_string(), "human".to_string()),
                ("yaks-eeee".to_string(), "human".to_string())
            ]
        );

        // Both renderings carry the same facts.
        let text = render(&lanes);
        assert!(text.contains("worktree"), "{text}");
        assert!(text.contains("yaks-aaaa hairy -> shaving"), "{text}");
        assert!(text.contains("yaks-cccc needs: human"), "{text}");
        let j = to_json(&lanes);
        assert_eq!(j[0]["branch"], "lane");
        assert_eq!(j[0]["farm"]["moved"][0]["to"], "shaving");
        assert_eq!(j[0]["farm"]["notes"][0]["new"], 1);
        assert_eq!(j[0]["error"], Value::Null);
    }

    #[test]
    fn needs_already_in_our_farm_is_not_news() {
        let b = base("needs-same");
        let t = b.join("repo");
        fs::create_dir_all(&t).unwrap();
        sh(&t, &["init", "-q"]);
        task_file(&t, "hairy", "yaks-aaaa", "needs: human\n", "x");
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "init"]);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "--detach", wt.to_str().unwrap()],
        );
        let lanes = lanes_of(&t);
        assert_eq!(lanes[0].branch, None, "detached HEAD has no branch");
        assert_eq!(lanes[0].farm, LaneFarm::Own(FarmDelta::default()));
        assert!(render(&lanes).contains("(detached)"));
    }

    /// Move a yak between status dirs in `repo` and commit it.
    fn commit_move(repo: &Path, id: &str, from: &str, to: &str) {
        fs::create_dir_all(repo.join(".yaks").join(to)).unwrap();
        fs::rename(
            repo.join(format!(".yaks/{from}/{id}.md")),
            repo.join(format!(".yaks/{to}/{id}.md")),
        )
        .unwrap();
        sh(repo, &["add", "-A"]);
        sh(repo, &["commit", "-q", "-m", &format!("move {id}")]);
    }

    #[test]
    fn base_temp_dir_is_removed_when_dropped() {
        let t = TempDir::new().unwrap();
        let p = t.0.clone();
        fs::write(p.join("f"), "x").unwrap();
        assert!(p.is_dir());
        drop(t);
        assert!(!p.exists());
    }

    #[test]
    fn a_lane_that_is_only_behind_has_no_farm_changes_of_its_own() {
        let b = base("behind");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "lane", wt.to_str().unwrap()],
        );
        // We move on (and shear a yak); the lane stays where it was.
        commit_move(&t, "yaks-cccc", "shorn", "hairy");
        commit_move(&t, "yaks-aaaa", "hairy", "shorn");

        let lanes = lanes_of(&t);
        let l = &lanes[0];
        assert_eq!((l.ahead, l.behind), (0, 2));
        assert!(l.farm_base.is_some());
        assert_eq!(l.farm, LaneFarm::Own(FarmDelta::default()));
        let text = render(&lanes);
        assert!(text.contains("+0 -2"), "{text}");
        assert!(text.contains("farm: no changes of its own"), "{text}");
        assert!(!text.contains("->"), "no phantom moves: {text}");
        let j = to_json(&lanes);
        assert_eq!(j[0]["behind"], 2);
        assert_eq!(j[0]["farm"]["vs"], "merge-base");
        assert_eq!(j[0]["farm"]["base"], l.farm_base.clone().unwrap());
    }

    #[test]
    fn a_lane_behind_with_its_own_changes_shows_only_its_own() {
        let b = base("both-ways");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "lane", wt.to_str().unwrap()],
        );
        // The lane COMMITS one yak move, then leaves the rest uncommitted.
        commit_move(&wt, "yaks-bbbb", "hairy", "shaving");
        task_file(&wt, "hairy", "yaks-dddd", "", "new");
        task_file(
            &wt,
            "hairy",
            "yaks-aaaa",
            "needs: human\n",
            "first\n\n---\n\u{25b8} 2026-02-02T00:00:00Z [w]\nnote",
        );
        // Meanwhile we moved two other yaks, and a third one is new here.
        commit_move(&t, "yaks-cccc", "shorn", "dead");
        task_file(&t, "hairy", "yaks-ffff", "", "ours only");
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "ours"]);

        let lanes = lanes_of(&t);
        let l = &lanes[0];
        assert_eq!((l.ahead, l.behind), (1, 2));
        let LaneFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert_eq!(d.added, vec!["yaks-dddd"]);
        assert_eq!(
            d.moved,
            vec![Moved {
                id: "yaks-bbbb".into(),
                from: "hairy",
                to: "shaving"
            }]
        );
        assert_eq!(d.notes, vec![("yaks-aaaa".to_string(), 1)]);
        assert_eq!(
            d.needs,
            vec![("yaks-aaaa".to_string(), "human".to_string())]
        );
        let text = render(&lanes);
        assert!(text.contains("+1 -2"), "{text}");
        assert!(
            !text.contains("yaks-cccc") && !text.contains("yaks-ffff"),
            "{text}"
        );
        assert!(!text.contains("vs this checkout"), "{text}");
    }

    #[test]
    fn no_merge_base_falls_back_to_this_checkout_and_says_so() {
        let b = base("nobase");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "lane", wt.to_str().unwrap()],
        );
        // Unrelated history: the lane's branch is an orphan.
        sh(&wt, &["checkout", "-q", "--orphan", "orphan"]);
        task_file(&wt, "shorn", "yaks-aaaa", "", "first");
        sh(&wt, &["add", "-A"]);
        sh(&wt, &["commit", "-q", "-m", "orphan"]);

        let lanes = lanes_of(&t);
        let l = &lanes[0];
        assert_eq!(l.farm_base, None);
        let LaneFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        // Compared with this checkout, as before.
        assert_eq!(d.moved.len(), 1);
        let text = render(&lanes);
        assert!(text.contains("(vs this checkout)"), "{text}");
        assert_eq!(to_json(&lanes)[0]["farm"]["vs"], "checkout");
        assert_eq!(to_json(&lanes)[0]["farm"]["base"], Value::Null);
    }

    /// Give `repo` (a checkout dir) an `objects/info/alternates` naming `store`.
    fn set_alternates(repo: &Path, store: &Path) {
        let info = repo.join(".git/objects/info");
        fs::create_dir_all(&info).unwrap();
        fs::write(
            info.join("alternates"),
            format!("{}\n", store.join("objects").display()),
        )
        .unwrap();
    }

    fn init_commit(dir: &Path, origin: Option<&str>) {
        fs::create_dir_all(dir).unwrap();
        sh(dir, &["init", "-q"]);
        if let Some(url) = origin {
            sh(dir, &["remote", "add", "origin", url]);
        }
        task_file(dir, "hairy", "yaks-aaaa", "", "x");
        sh(dir, &["add", "-A"]);
        sh(dir, &["commit", "-q", "-m", "init"]);
    }

    #[test]
    fn managed_layout_sibling_is_a_delta_lane_and_look_alikes_are_rejected() {
        let b = base("managed");
        let store = b.join("store.git");
        fs::create_dir_all(&store).unwrap();
        sh(&store, &["init", "-q", "--bare"]);
        let root = b.join("worktrees");

        // Us, and a sibling clone sharing our object store (alternates).
        let me = root.join("aaa/name");
        let sib = root.join("bbb/name");
        init_commit(&me, Some(&url(&b, "r")));
        init_commit(&sib, Some(&url(&b, "r")));
        set_alternates(&me, &store);
        set_alternates(&sib, &store);
        // Same repo by origin url only (no alternates): accepted.
        let by_url = root.join("ccc/name");
        init_commit(&by_url, Some(&url(&b, "r")));
        // Same basename, different repository: rejected.
        let other = root.join("ddd/name");
        init_commit(&other, Some(&url(&b, "other")));
        // A same-named directory with no `.git` of its own: rejected.
        let bare_dir = root.join("eee/name");
        fs::create_dir_all(&bare_dir).unwrap();
        // A differently named sibling is never a candidate.
        let renamed = root.join("fff/other-name");
        init_commit(&renamed, Some(&url(&b, "r")));

        let lanes = lanes_of(&me);
        assert_eq!(paths(&lanes), vec![sib, by_url]);
        assert!(lanes.iter().all(|l| l.kind == LaneKind::Delta));
        assert!(!paths(&lanes).contains(&me), "never lists self");
    }

    #[test]
    fn laptop_layout_clones_under_the_checkout_are_delta_lanes() {
        let b = base("laptop");
        let t = b.join("repo");
        init_commit(&t, Some(&url(&b, "r")));
        let lane = t.join(".delta/worktrees/xyz/repo");
        fs::create_dir_all(lane.parent().unwrap()).unwrap();
        sh(
            &b,
            &["clone", "-q", t.to_str().unwrap(), lane.to_str().unwrap()],
        );
        sh(&lane, &["remote", "set-url", "origin", &url(&b, "r")]);
        let lanes = lanes_of(&t);
        assert_eq!(paths(&lanes), vec![lane.clone()]);
        assert_eq!(lanes[0].kind, LaneKind::Delta);

        // And from the clone, the repository checkout that holds it is a lane.
        let back = lanes_of(&lane);
        assert_eq!(paths(&back), vec![t]);
    }

    #[test]
    fn found_by_both_sources_is_one_lane_of_kind_worktree() {
        let b = base("both");
        let t = b.join("repo");
        repo_with_farm(&t);
        sh(&t, &["remote", "add", "origin", &url(&b, "r")]);
        // A linked worktree that also matches the Delta scan (T/.delta/worktrees/*/repo).
        let wt = t.join(".delta/worktrees/xyz/repo");
        fs::create_dir_all(wt.parent().unwrap()).unwrap();
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "lane", wt.to_str().unwrap()],
        );
        let lanes = lanes_of(&t);
        assert_eq!(paths(&lanes), vec![wt]);
        assert_eq!(lanes[0].kind, LaneKind::Worktree);
    }

    #[test]
    fn private_and_shared_farms_are_named_not_diffed() {
        let b = base("farms");
        let t = b.join("repo");
        repo_with_farm(&t);
        let private = b.join("private");
        sh(
            &t,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "p",
                private.to_str().unwrap(),
            ],
        );
        fs::remove_dir_all(private.join(".yaks")).unwrap();
        let shared = b.join("shared");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "s", shared.to_str().unwrap()],
        );
        fs::remove_dir_all(shared.join(".yaks")).unwrap();
        fs::write(
            shared.join(".yaks"),
            format!("path: {}\n", t.join(".yaks").display()),
        )
        .unwrap();

        let lanes = lanes_of(&t);
        let by = |p: &Path| lanes.iter().find(|l| l.path == p).unwrap();
        assert_eq!(by(&private).farm, LaneFarm::None);
        assert_eq!(by(&shared).farm, LaneFarm::Shared);
        let text = render(&lanes);
        assert!(
            text.contains("no farm here (private? set YAKS_DIR)"),
            "{text}"
        );
        assert!(text.contains("shares this farm"), "{text}");
        let j = to_json(&lanes);
        let states: Vec<&str> = j
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l["farm"]["state"].as_str().unwrap())
            .collect();
        assert!(states.contains(&"none") && states.contains(&"shared"));
    }

    #[test]
    fn unreadable_lane_is_listed_with_its_error() {
        let b = base("broken");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "w", wt.to_str().unwrap()],
        );
        let good = b.join("good");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "g", good.to_str().unwrap()],
        );
        fs::write(wt.join(".git"), "garbage\n").unwrap();

        let lanes = lanes_of(&t);
        assert_eq!(paths(&lanes), vec![good, wt]);
        assert!(lanes[0].error.is_none());
        let bad = &lanes[1];
        assert!(bad.error.is_some());
        assert!(bad.head.is_none() && bad.dirty.is_none());
        assert!(render(&lanes).contains("  error: "));
        assert!(to_json(&lanes)[1]["error"].is_string());
    }

    /// (relative path, mtime, len) of everything under `root`.
    fn snapshot(root: &Path) -> Vec<(PathBuf, SystemTime, u64)> {
        let mut out = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(d) = stack.pop() {
            for e in fs::read_dir(&d).unwrap().flatten() {
                let md = fs::symlink_metadata(e.path()).unwrap();
                out.push((e.path(), md.modified().unwrap(), md.len()));
                if md.is_dir() {
                    stack.push(e.path());
                }
            }
        }
        out.sort();
        out
    }

    #[test]
    fn inspecting_a_lane_changes_nothing_in_it() {
        let b = base("readonly");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "w", wt.to_str().unwrap()],
        );
        // Make the lane's index stale (same bytes, newer mtime) so an ordinary
        // `git status` would want to refresh it.
        std::thread::sleep(std::time::Duration::from_millis(30));
        let f = wt.join(".yaks/hairy/yaks-aaaa.md");
        let text = fs::read_to_string(&f).unwrap();
        fs::write(&f, text).unwrap();
        task_file(&wt, "hairy", "yaks-zzzz", "", "new");

        let admin = t.join(".git/worktrees");
        let before = (
            snapshot(&wt),
            snapshot(&admin),
            sh(&wt, &["--no-optional-locks", "status", "--porcelain"]),
        );
        let lanes = lanes_of(&t);
        assert_eq!(lanes.len(), 1);
        let _ = (render(&lanes), to_json(&lanes));
        let after = (
            snapshot(&wt),
            snapshot(&admin),
            sh(&wt, &["--no-optional-locks", "status", "--porcelain"]),
        );
        assert_eq!(before, after);
    }
}
