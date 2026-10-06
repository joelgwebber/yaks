//! `yaks sheds`: the OTHER checkouts of this repository (git worktrees and
//! Delta clones) and what each one's farm says relative to this checkout's.
//!
//! Plain git plus the filesystem; no Delta database, no Delta CLI. Everything
//! here is strictly read-only in the other checkouts: git is invoked as
//! `git --no-optional-locks -C <shed> ...` for read-only plumbing only (never
//! fetch/checkout/add), and farms are compared by reading `.yaks` directories
//! (see [`compare_farms`]).
//!
//! A shed's farm is compared with the farm at the commit the SHED FORKED from
//! (committed history, read with `git ls-tree`/`cat-file` into a temp dir of
//! our own), so a shed that is merely behind shows nothing of its own and what
//! it does show is exactly what it changed, committed or not. The fork point is
//! the oldest entry of the shed's HEAD reflog ([`fork_point`]); a worker forks
//! from its coordinator's thread, not from the checkout that runs `yaks sheds`,
//! so measuring from the merge-base with THIS checkout would credit the shed
//! with everything its coordinator did before spawning it. A shed synced with
//! this checkout after it forked is measured from that merge-base instead
//! when it is nearer ([`nearer_baseline`], `sync`): what it merged in is not
//! its own. When the reflog cannot give a usable fork point, the merge-base with this checkout is the
//! baseline instead and the output says so (`vs merge-base`); when there is no
//! base farm to read at all, the shed is compared with this checkout's working
//! tree and says so (`vs this checkout`).
//!
//! Discovery has two sources, merged by canonical path into one shed each:
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

/// How a shed was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShedKind {
    /// Listed by `git worktree list`.
    Worktree,
    /// Found only by scanning Delta's sibling-clone layout.
    Delta,
}

impl ShedKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ShedKind::Worktree => "worktree",
            ShedKind::Delta => "delta",
        }
    }
}

/// A yak whose status directory differs between the shed's farm and the one it
/// is compared with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moved {
    pub id: String,
    /// Status directory in the compared-with farm (merge-base or this one).
    pub from: &'static str,
    /// Status directory in the shed's farm.
    pub to: &'static str,
}

/// What the shed's farm has that the farm it is compared with lacks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FarmDelta {
    /// Yaks that exist only in the shed.
    pub added: Vec<String>,
    /// Yaks that exist in both with different status directories.
    pub moved: Vec<Moved>,
    /// `(id, count)` of note entries the shed has beyond ours.
    pub notes: Vec<(String, usize)>,
    /// `(id, value)` of shed yaks whose `needs:` is set and differs from ours.
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

/// Which commit a shed's own changes are measured from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseKind {
    /// Where the shed itself forked (oldest entry of its HEAD reflog): exactly
    /// the shed's own work.
    Fork,
    /// The merge-base with this checkout, chosen because it is NEARER than the
    /// fork point (the shed was synced with this checkout after it forked):
    /// everything before it is already in our history. As exact as `Fork`, so
    /// unmarked.
    Sync,
    /// The merge-base with this checkout: the fallback when the fork point is
    /// unknown. It also counts whatever the shed inherited from the thread that
    /// spawned it, so the output marks it.
    MergeBase,
}

impl BaseKind {
    /// The JSON `farm.vs` value.
    pub fn as_str(self) -> &'static str {
        match self {
            BaseKind::Fork => "fork",
            BaseKind::Sync => "sync",
            BaseKind::MergeBase => "merge-base",
        }
    }
}

/// The commit a shed's farm delta (and label) is relative to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FarmBase {
    pub kind: BaseKind,
    /// Short sha.
    pub sha: String,
}

/// The shed's farm relative to ours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShedFarm {
    /// The shed has a farm of its own; here is how it differs from ours.
    Own(FarmDelta),
    /// The shed resolves to the same farm directory as this checkout.
    Shared,
    /// Discovery found no farm for the shed (a private farm: set `YAKS_DIR`).
    None,
    /// The shed has a farm but it could not be read.
    Unreadable(String),
}

/// One other checkout of this repository.
#[derive(Debug, Clone)]
pub struct Shed {
    pub path: PathBuf,
    pub kind: ShedKind,
    /// `None` when HEAD is detached (or unreadable).
    pub branch: Option<String>,
    /// Abbreviated HEAD sha; `None` when the shed's git state cannot be read.
    pub head: Option<String>,
    /// Commits the shed's HEAD has that this checkout's HEAD lacks. `None`
    /// (unknown) when there is no merge-base between the two HEADs here:
    /// shallow history, unrelated histories, or a shed commit we do not have.
    pub ahead: Option<usize>,
    /// Commits this checkout's HEAD has that the shed's HEAD lacks; `None`
    /// exactly when `ahead` is.
    pub behind: Option<usize>,
    /// This checkout's repository is shallow (the same for every shed); only
    /// used by [`render`] to explain unknown `ahead`/`behind`.
    pub shallow_here: bool,
    /// Changed + untracked file count (`git status --porcelain`).
    pub dirty: Option<usize>,
    /// Newest mtime anywhere under the shed's farm: a liveness hint (finished
    /// threads' checkouts persist).
    pub farm_activity: Option<String>,
    pub farm: ShedFarm,
    /// The commit the farm delta is relative to; `None` means it is relative
    /// to this checkout's farm instead (no base farm could be read).
    pub farm_base: Option<FarmBase>,
    /// Who is working in the shed: the distinct actors on its OWN new note
    /// entries, first seen first (see [`label`]). Empty when the shed has no
    /// own entries or no base to tell which entries are its own.
    pub who: Vec<String>,
    /// Yaks the shed is working on: shaving in the shed, with own entries
    /// there (see [`label`]). Empty whenever `who` cannot be told.
    pub in_progress: Vec<String>,
    /// Set when the shed's git state could not be read; the shed is still
    /// listed.
    pub error: Option<String>,
}

/// List the other checkouts of the repository containing `cwd`, relative to
/// the farm at `farm_root` (this checkout's farm). Sorted by path.
pub fn discover(cwd: &Path, farm_root: &Path) -> Result<Vec<Shed>> {
    let top = PathBuf::from(
        git(cwd, &["rev-parse", "--show-toplevel"]).context("yaks sheds needs a git repository")?,
    );
    let top = canonical(&top);
    let own_common = common_dir(&top);

    // canonical path -> kind. Worktrees win over a Delta-only find.
    let mut found: BTreeMap<PathBuf, ShedKind> = BTreeMap::new();
    for p in worktree_paths(&top) {
        found.insert(canonical(&p), ShedKind::Worktree);
    }
    for p in delta_candidates(&top) {
        let p = canonical(&p);
        if found.contains_key(&p) {
            continue;
        }
        if own_common.as_deref().is_some_and(|c| same_repo(c, &p)) {
            found.insert(p, ShedKind::Delta);
        }
    }
    found.remove(&top);

    let ours = store::load(farm_root, &EVERY).unwrap_or_default();
    let our_farm = canonical(farm_root);
    let shallow = git(&top, &["rev-parse", "--is-shallow-repository"]).is_ok_and(|s| s == "true");
    Ok(found
        .into_iter()
        .map(|(path, kind)| {
            let mut shed = inspect(&top, &path, kind, &our_farm, &ours);
            shed.shallow_here = shallow;
            shed
        })
        .collect())
}

/// Gather one shed's report. Never fails: unreadable parts become `error`.
fn inspect(
    top: &Path,
    path: &Path,
    kind: ShedKind,
    our_farm: &Path,
    ours: &[crate::model::Task],
) -> Shed {
    let mut shed = Shed {
        path: path.to_path_buf(),
        kind,
        branch: None,
        head: None,
        ahead: None,
        behind: None,
        shallow_here: false,
        dirty: None,
        farm_activity: None,
        farm: ShedFarm::None,
        farm_base: None,
        who: Vec::new(),
        in_progress: Vec::new(),
        error: None,
    };
    let mut bases: Vec<(BaseKind, String)> = Vec::new();
    match git_state(top, path) {
        Ok(g) => {
            shed.branch = g.branch;
            shed.head = Some(g.head);
            shed.ahead = g.ahead;
            shed.behind = g.behind;
            shed.dirty = Some(g.dirty);
            bases.extend(g.fork);
            bases.extend(g.merge_base.map(|b| (BaseKind::MergeBase, b)));
        }
        Err(e) => shed.error = Some(format!("{e:#}")),
    }
    // The farm side is plain filesystem reads, so it is reported even when git
    // could not be read.
    match store::discover_with(path, None) {
        Err(_) => shed.farm = ShedFarm::None,
        Ok(d) => {
            let root = canonical(&d.root);
            if root == our_farm {
                shed.farm = ShedFarm::Shared;
            } else {
                shed.farm_activity = newest_mtime(&root).map(format_time);
                shed.farm = match store::load(&root, &EVERY) {
                    Ok(theirs) => {
                        // What the shed changed is relative to where it forked
                        // (else the merge-base), not to our (possibly newer)
                        // checkout.
                        let base = bases.iter().find_map(|(kind, b)| {
                            base_farm(top, b, path, &root)
                                .map(|(sha, tasks)| (FarmBase { kind: *kind, sha }, tasks))
                        });
                        match base {
                            Some((base, base_tasks)) => {
                                shed.farm_base = Some(base);
                                (shed.who, shed.in_progress) = label(&base_tasks, &theirs);
                                ShedFarm::Own(compare_farms(&base_tasks, &theirs))
                            }
                            None => ShedFarm::Own(compare_farms(ours, &theirs)),
                        }
                    }
                    Err(e) => ShedFarm::Unreadable(format!("{e:#}")),
                };
            }
        }
    }
    shed
}

struct GitState {
    branch: Option<String>,
    head: String,
    /// The exact baseline, when the shed's reflog gives a usable fork point
    /// (see [`fork_point`]): the fork point itself (`Fork`), or the merge-base
    /// when that is nearer (`Sync`, see [`nearer_baseline`]).
    fork: Option<(BaseKind, String)>,
    /// `git merge-base HEAD <shed HEAD>` in OUR repo; `None` when there is
    /// none (then `ahead`/`behind` are unknown too).
    merge_base: Option<String>,
    ahead: Option<usize>,
    behind: Option<usize>,
    dirty: usize,
}

fn git_state(top: &Path, shed: &Path) -> Result<GitState> {
    let full = git(shed, &["rev-parse", "HEAD"])?;
    let head = git(shed, &["rev-parse", "--short", "HEAD"])?;
    // Exit 1 from `symbolic-ref -q` means a detached HEAD.
    let branch = git(shed, &["symbolic-ref", "--short", "-q", "HEAD"])
        .ok()
        .filter(|b| !b.is_empty());
    // Counted in OUR repo, and only when the two HEADs have a merge-base here.
    // Without one (shallow history, unrelated histories, a shed commit we do
    // not have) rev-list would count a whole history, which says nothing.
    let merge_base = git(top, &["merge-base", "HEAD", &full])
        .ok()
        .filter(|b| !b.is_empty());
    let count = |range: String| -> Option<usize> {
        merge_base.as_ref()?;
        git(top, &["rev-list", "--count", &range])
            .ok()?
            .parse()
            .ok()
    };
    let ahead = count(format!("HEAD..{full}"));
    let behind = count(format!("{full}..HEAD"));
    // `--no-optional-locks` (see `git`) keeps status from refreshing the index.
    let status = git_raw(shed, &["status", "--porcelain", "--untracked-files=all"])?;
    let dirty = status.lines().filter(|l| !l.trim().is_empty()).count();
    Ok(GitState {
        branch,
        head,
        fork: fork_point(top, shed, &full).map(|f| nearer_baseline(top, f, merge_base.as_deref())),
        merge_base,
        ahead,
        behind,
        dirty,
    })
}

/// The commit `shed` was forked from: the second field of the first (oldest)
/// line of its HEAD reflog (`<git-dir>/logs/HEAD`). For a Delta clone that is
/// the coordinator's HEAD at spawn time, for a `git worktree add -b` shed the
/// branch creation. `None` (the caller falls back to the merge-base) when there
/// is no reflog, it is expired or unparseable, or the commit is not usable as a
/// baseline: unknown to our object store, or not an ancestor of the shed's HEAD
/// `full` (e.g. the shed was rebased since), which would make "since the fork"
/// meaningless.
fn fork_point(top: &Path, shed: &Path, full: &str) -> Option<String> {
    use std::io::{BufRead, BufReader};
    let dir = git(shed, &["rev-parse", "--absolute-git-dir"]).ok()?;
    let file = fs::File::open(Path::new(&dir).join("logs").join("HEAD")).ok()?;
    let mut first = String::new();
    BufReader::new(file).read_line(&mut first).ok()?;
    // `<old> <new> <identity> <time> <tz>\t<message>`: `new` is where HEAD
    // pointed after the entry.
    let sha = first.split_whitespace().nth(1)?;
    if sha.len() < 40
        || !sha.bytes().all(|b| b.is_ascii_hexdigit())
        || sha.bytes().all(|b| b == b'0')
    {
        return None;
    }
    git(top, &["merge-base", "--is-ancestor", sha, full]).ok()?;
    Some(sha.to_string())
}

/// The nearer of the validated fork point `fork` and the merge-base `mb` with
/// this checkout. When the merge-base descends from the fork point (the shed
/// was synced with this checkout after it forked) everything between the two
/// is already in our history, so it is not what the shed has that we lack: the
/// merge-base wins (`Sync`). Otherwise (the merge-base is the fork point, an
/// ancestor of it, or unrelated: a worker forked from a coordinator whose
/// commits are not in our history, e.g. after a squash landing) the fork point
/// wins.
fn nearer_baseline(top: &Path, fork: String, mb: Option<&str>) -> (BaseKind, String) {
    match mb {
        Some(mb) if mb != fork && git(top, &["merge-base", "--is-ancestor", &fork, mb]).is_ok() => {
            (BaseKind::Sync, mb.to_string())
        }
        _ => (BaseKind::Fork, fork),
    }
}

/// A temp directory of ours, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Result<TempDir> {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "yaks-sheds-base-{}-{}",
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

/// The shed's farm as of `base`, the commit the shed forked from or else its
/// merge-base with this checkout (see [`GitState`]): `(short sha, tasks)`. `None` when there is nothing
/// trustworthy to compare against (a farm outside the shed's checkout, or no
/// farm committed at the base); the caller then tries its next base, and with
/// none left (no fork point, no common history, or the shed's commit unknown
/// here) falls back to comparing with this checkout's working tree.
///
/// Only our own repo is read (Delta clones share objects with it, and git
/// worktrees are the same repo); the shed is never touched. The base's yak
/// files are materialized into a temp dir of ours and loaded like any farm.
fn base_farm(
    top: &Path,
    base: &str,
    shed: &Path,
    shed_farm: &Path,
) -> Option<(String, Vec<crate::model::Task>)> {
    let rel = shed_farm.strip_prefix(shed).ok()?;
    let rel = rel.to_str()?.replace('\\', "/");
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

/// Compare a shed's tasks to ours (see [`FarmDelta`]). Pure; ids sorted.
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
        // farms already carry is not news from this shed.
        let ours_needs = mine.get(t.id.as_str()).and_then(|m| m.needs.as_ref());
        if let Some(n) = t.needs.as_ref().filter(|n| Some(*n) != ours_needs) {
            d.needs.push((t.id.clone(), n.clone()));
        }
    }
    d
}

/// Label a shed from its OWN new note entries: `(who, in_progress)`. Pure.
///
/// A shed's own entries are the notes its yaks have beyond the base's
/// (notes are append-only, so that is the tail past the base's count; every
/// note of a yak the base lacks), committed or not. A `moved:` transition is
/// one of those notes, so a claim or shear made in the shed counts like any
/// other entry.
///
/// - `who`: the distinct actors on those entries, in first-seen order by
///   timestamp. An entry with no actor names no one.
/// - `in_progress`: the yaks that are in `shaving` in the shed and have own
///   entries there, sorted by id. One rule, deliberately not "the shed moved
///   it to shaving": a worker's checkout usually forks AFTER the coordinator's
///   claim, so the claim is in the base and the worker's own entries are
///   plain notes. A yak the shed has shorn, or never touched, is not in
///   progress.
fn label(base: &[crate::model::Task], theirs: &[crate::model::Task]) -> (Vec<String>, Vec<String>) {
    let known: BTreeMap<&str, usize> = base
        .iter()
        .map(|t| (t.id.as_str(), store::parse_notes(&t.body).len()))
        .collect();
    let mut theirs: Vec<&crate::model::Task> = theirs.iter().collect();
    theirs.sort_by(|a, b| a.id.cmp(&b.id));
    let mut entries: Vec<store::NoteEntry> = Vec::new();
    let mut in_progress = Vec::new();
    for t in theirs {
        let skip = known.get(t.id.as_str()).copied().unwrap_or(0);
        let before = entries.len();
        entries.extend(store::parse_notes(&t.body).into_iter().skip(skip));
        if entries.len() > before && t.status == Status::Shaving {
            in_progress.push(t.id.clone());
        }
    }
    entries.sort_by(|a, b| a.ts.cmp(&b.ts));
    let mut who: Vec<String> = Vec::new();
    for actor in entries.into_iter().filter_map(|e| e.actor) {
        if !who.contains(&actor) {
            who.push(actor);
        }
    }
    (who, in_progress)
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
        // checkout itself is a shed too.
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

fn farm_message(f: &ShedFarm) -> Option<String> {
    match f {
        ShedFarm::Own(_) => None,
        ShedFarm::Shared => Some("shares this farm".into()),
        ShedFarm::None => Some("no farm here (private? set YAKS_DIR)".into()),
        ShedFarm::Unreadable(e) => Some(format!("farm unreadable: {e}")),
    }
}

/// JSON for `--json`: one object per shed with the same fields as the table.
pub fn to_json(sheds: &[Shed]) -> Value {
    Value::Array(
        sheds
            .iter()
            .map(|l| {
                let farm = match &l.farm {
                    ShedFarm::Own(d) => json!({
                        "state": "own",
                        // What the delta is relative to (`base` = its short
                        // sha): where the shed forked (`fork`), or this
                        // checkout's merge-base with it when it was synced
                        // since (`sync`), else that merge-base as a fallback
                        // (`merge-base`), else,
                        // when neither can be read, this checkout's farm.
                        "vs": match &l.farm_base {
                            Some(b) => b.kind.as_str(),
                            None => "checkout",
                        },
                        "base": l.farm_base.as_ref().map(|b| &b.sha),
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
                            ShedFarm::Shared => "shared",
                            ShedFarm::None => "none",
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
                    // `null` = unknown (no merge-base here).
                    "ahead": l.ahead,
                    "behind": l.behind,
                    "dirty": l.dirty,
                    "farm_activity": l.farm_activity,
                    "farm": farm,
                    "who": l.who,
                    "in_progress": l.in_progress,
                    "error": l.error,
                })
            })
            .collect(),
    )
}

/// What the table says about a baseline other than the shed's own fork point,
/// so a number that may include inherited work is never silent: ` (vs
/// merge-base)` or ` (vs this checkout)`. Empty for the fork point.
fn vs_marker(l: &Shed) -> &'static str {
    match l.farm_base.as_ref().map(|b| b.kind) {
        Some(BaseKind::Fork | BaseKind::Sync) => "",
        Some(BaseKind::MergeBase) => " (vs merge-base)",
        None => " (vs this checkout)",
    }
}

/// The one-line shed label: `who: a, b · shaving: id, id`, each half only
/// when it has something; `None` for an unlabeled shed. Measured from a
/// merge-base it carries the `vs merge-base` marker.
fn label_line(l: &Shed) -> Option<String> {
    let mut parts = Vec::new();
    if !l.who.is_empty() {
        parts.push(format!("who: {}", l.who.join(", ")));
    }
    if !l.in_progress.is_empty() {
        parts.push(format!("shaving: {}", l.in_progress.join(", ")));
    }
    (!parts.is_empty()).then(|| format!("{}{}", parts.join(" \u{b7} "), vs_marker(l)))
}

/// The compact human table: one row per shed, then its label and farm detail.
pub fn render(sheds: &[Shed]) -> String {
    if sheds.is_empty() {
        return "No other checkouts found.\n".into();
    }
    let rows: Vec<[String; 6]> = sheds
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
                    let n = |c: Option<usize>| c.map_or_else(|| "?".into(), |c| c.to_string());
                    format!("+{} -{}", n(l.ahead), n(l.behind))
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
    for (l, r) in sheds.iter().zip(&rows) {
        out.push_str(&line(
            r.iter().map(String::as_str).collect(),
            &l.path.display().to_string(),
        ));
        if let Some(e) = &l.error {
            out.push_str(&format!("  error: {e}\n"));
        }
        if let Some(label) = label_line(l) {
            out.push_str(&format!("  {label}\n"));
        }
        match &l.farm {
            ShedFarm::Own(d) if d.is_empty() && l.farm_base.is_some() => {
                out.push_str(&format!("  farm: no changes of its own{}\n", vs_marker(l)))
            }
            ShedFarm::Own(d) if d.is_empty() => {
                out.push_str("  farm: no differences (vs this checkout)\n")
            }
            ShedFarm::Own(d) => {
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
                out.push_str(&format!("  farm: {}{}\n", parts.join(", "), vs_marker(l)));
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
    // One line for the whole table, and only when it explains a `?`.
    if sheds
        .iter()
        .any(|l| l.shallow_here && l.head.is_some() && l.ahead.is_none())
    {
        out.push_str(
            "note: history is shallow here, so some sheds cannot be compared exactly; \
             git fetch --unshallow fixes it\n",
        );
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
            "yaks-sheds-{tag}-{}-{}",
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

    fn sheds_of(top: &Path) -> Vec<Shed> {
        discover(top, &top.join(".yaks")).unwrap()
    }

    fn paths(sheds: &[Shed]) -> Vec<PathBuf> {
        sheds.iter().map(|l| l.path.clone()).collect()
    }

    #[test]
    fn no_siblings_is_an_empty_list() {
        let b = base("none");
        let t = b.join("repo");
        repo_with_farm(&t);
        assert!(sheds_of(&t).is_empty());
        assert_eq!(render(&[]), "No other checkouts found.\n");
    }

    #[test]
    fn git_worktree_shed_reports_git_state_and_farm_delta() {
        let b = base("wt");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );

        // A commit the shed has and we lack; an uncommitted file.
        fs::write(wt.join("x.txt"), "x").unwrap();
        sh(&wt, &["add", "x.txt"]);
        sh(&wt, &["commit", "-q", "-m", "shed work"]);
        fs::write(wt.join("untracked.txt"), "u").unwrap();

        // Farm changes in the shed: an added yak, a moved yak, an added note,
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

        let sheds = sheds_of(&t);
        assert_eq!(paths(&sheds), vec![wt.clone()]);
        let l = &sheds[0];
        assert_eq!(l.kind, ShedKind::Worktree);
        assert_eq!(l.branch.as_deref(), Some("shed"));
        assert_eq!(
            l.head.as_deref(),
            Some(&*sh(&wt, &["rev-parse", "--short", "HEAD"]))
        );
        assert_eq!((l.ahead, l.behind), (Some(1), Some(0)));
        assert!(l.farm_base.is_some());
        // untracked.txt plus the yak edits (new files, a rename, a modified file).
        assert!(l.dirty.unwrap() >= 5, "dirty = {:?}", l.dirty);
        assert!(l.farm_activity.is_some());
        assert!(l.error.is_none());
        let ShedFarm::Own(d) = &l.farm else {
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
        let text = render(&sheds);
        assert!(text.contains("worktree"), "{text}");
        assert!(text.contains("yaks-aaaa hairy -> shaving"), "{text}");
        assert!(text.contains("yaks-cccc needs: human"), "{text}");
        let j = to_json(&sheds);
        assert_eq!(j[0]["branch"], "shed");
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
        let sheds = sheds_of(&t);
        assert_eq!(sheds[0].branch, None, "detached HEAD has no branch");
        assert_eq!(sheds[0].farm, ShedFarm::Own(FarmDelta::default()));
        assert!(render(&sheds).contains("(detached)"));
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
    fn a_shed_that_is_only_behind_has_no_farm_changes_of_its_own() {
        let b = base("behind");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        // We move on (and shear a yak); the shed stays where it was.
        commit_move(&t, "yaks-cccc", "shorn", "hairy");
        commit_move(&t, "yaks-aaaa", "hairy", "shorn");

        let sheds = sheds_of(&t);
        let l = &sheds[0];
        assert_eq!((l.ahead, l.behind), (Some(0), Some(2)));
        assert!(l.farm_base.is_some());
        assert_eq!(l.farm, ShedFarm::Own(FarmDelta::default()));
        let text = render(&sheds);
        assert!(text.contains("+0 -2"), "{text}");
        assert!(text.contains("farm: no changes of its own"), "{text}");
        assert!(!text.contains("->"), "no phantom moves: {text}");
        let j = to_json(&sheds);
        assert_eq!(j[0]["behind"], 2);
        assert_eq!(j[0]["farm"]["vs"], "fork");
        assert_eq!(j[0]["farm"]["base"], l.farm_base.clone().unwrap().sha);
        assert!(
            !text.contains("(vs "),
            "the fork point needs no marker: {text}"
        );
    }

    #[test]
    fn a_shed_behind_with_its_own_changes_shows_only_its_own() {
        let b = base("both-ways");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        // The shed COMMITS one yak move, then leaves the rest uncommitted.
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

        let sheds = sheds_of(&t);
        let l = &sheds[0];
        assert_eq!((l.ahead, l.behind), (Some(1), Some(2)));
        let ShedFarm::Own(d) = &l.farm else {
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
        let text = render(&sheds);
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
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        // Unrelated history: the shed's branch is an orphan.
        sh(&wt, &["checkout", "-q", "--orphan", "orphan"]);
        task_file(&wt, "shorn", "yaks-aaaa", "", "first");
        sh(&wt, &["add", "-A"]);
        sh(&wt, &["commit", "-q", "-m", "orphan"]);

        let sheds = sheds_of(&t);
        let l = &sheds[0];
        assert_eq!(l.farm_base, None);
        let ShedFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        // Compared with this checkout, as before.
        assert_eq!(d.moved.len(), 1);
        let text = render(&sheds);
        assert!(text.contains("(vs this checkout)"), "{text}");
        assert_eq!(to_json(&sheds)[0]["farm"]["vs"], "checkout");
        assert_eq!(to_json(&sheds)[0]["farm"]["base"], Value::Null);
        // No merge-base: the counts are unknown, not the size of a history.
        assert_eq!((l.ahead, l.behind), (None, None));
        assert!(text.contains("+? -?"), "{text}");
        assert_eq!(to_json(&sheds)[0]["ahead"], Value::Null);
        assert_eq!(to_json(&sheds)[0]["behind"], Value::Null);
        // Not a shallow repository, so there is nothing to unshallow.
        assert!(!text.contains("history is shallow"), "{text}");
    }

    const SHALLOW_NOTE: &str = "note: history is shallow here, so some sheds cannot be compared exactly; git fetch --unshallow fixes it\n";

    /// A depth-1 clone of a three-commit repo as the primary, plus a Delta shed
    /// (a full clone of the same origin) checked out at `shed_at`. Returns
    /// `(primary, shed)`.
    fn shallow_primary_with_shed(tag: &str, shed_at: &str) -> (PathBuf, PathBuf) {
        let b = base(tag);
        let origin = b.join("origin");
        repo_with_farm(&origin);
        commit_move(&origin, "yaks-cccc", "shorn", "hairy");
        commit_move(&origin, "yaks-aaaa", "hairy", "shorn");
        let origin_url = format!("file://{}", origin.display());
        let t = b.join("repo");
        sh(
            &b,
            &[
                "clone",
                "-q",
                "--depth",
                "1",
                &origin_url,
                t.to_str().unwrap(),
            ],
        );
        assert_eq!(sh(&t, &["rev-parse", "--is-shallow-repository"]), "true");
        let shed = t.join(".delta/worktrees/xyz/repo");
        fs::create_dir_all(shed.parent().unwrap()).unwrap();
        sh(&b, &["clone", "-q", &origin_url, shed.to_str().unwrap()]);
        sh(&shed, &["checkout", "-q", shed_at]);
        (t, shed)
    }

    #[test]
    fn a_shallow_primary_cannot_place_a_shed_older_than_its_boundary() {
        let (t, shed) = shallow_primary_with_shed("shallow", "HEAD~2");
        let sheds = sheds_of(&t);
        assert_eq!(paths(&sheds), vec![shed]);
        let l = &sheds[0];
        assert_eq!((l.ahead, l.behind), (None, None));
        let text = render(&sheds);
        assert!(text.contains("+? -?"), "{text}");
        // Said once, at the end, and the farm still falls back.
        assert!(text.ends_with(SHALLOW_NOTE), "{text}");
        assert_eq!(text.matches("history is shallow").count(), 1, "{text}");
        assert!(text.contains("(vs this checkout)"), "{text}");
        // JSON keeps its shape (a top-level array of sheds) and says null.
        let j = to_json(&sheds);
        assert_eq!(j.as_array().unwrap().len(), 1);
        assert_eq!(j[0]["ahead"], Value::Null);
        assert_eq!(j[0]["behind"], Value::Null);
    }

    #[test]
    fn the_same_repo_unshallowed_shows_numbers_and_no_note() {
        let (t, _shed) = shallow_primary_with_shed("unshallowed", "HEAD~2");
        // Our own temp clone, so fetching is fine here.
        sh(&t, &["fetch", "-q", "--unshallow"]);
        let sheds = sheds_of(&t);
        assert_eq!((sheds[0].ahead, sheds[0].behind), (Some(0), Some(2)));
        let text = render(&sheds);
        assert!(text.contains("+0 -2"), "{text}");
        assert!(
            !text.contains("history is shallow") && !text.contains('?'),
            "{text}"
        );
        assert!(sheds[0].farm_base.is_some());
    }

    #[test]
    fn a_shallow_primary_is_silent_when_every_shed_is_comparable() {
        // The shed sits on the boundary commit itself: merge-base exists.
        let (t, _shed) = shallow_primary_with_shed("shallow-ok", "HEAD");
        let sheds = sheds_of(&t);
        assert_eq!((sheds[0].ahead, sheds[0].behind), (Some(0), Some(0)));
        let text = render(&sheds);
        assert!(text.contains("+0 -0"), "{text}");
        assert!(!text.contains("history is shallow"), "{text}");
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
    fn managed_layout_sibling_is_a_delta_shed_and_look_alikes_are_rejected() {
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

        let sheds = sheds_of(&me);
        assert_eq!(paths(&sheds), vec![sib, by_url]);
        assert!(sheds.iter().all(|l| l.kind == ShedKind::Delta));
        assert!(!paths(&sheds).contains(&me), "never lists self");
    }

    #[test]
    fn laptop_layout_clones_under_the_checkout_are_delta_sheds() {
        let b = base("laptop");
        let t = b.join("repo");
        init_commit(&t, Some(&url(&b, "r")));
        let shed = t.join(".delta/worktrees/xyz/repo");
        fs::create_dir_all(shed.parent().unwrap()).unwrap();
        sh(
            &b,
            &["clone", "-q", t.to_str().unwrap(), shed.to_str().unwrap()],
        );
        sh(&shed, &["remote", "set-url", "origin", &url(&b, "r")]);
        let sheds = sheds_of(&t);
        assert_eq!(paths(&sheds), vec![shed.clone()]);
        assert_eq!(sheds[0].kind, ShedKind::Delta);

        // And from the clone, the repository checkout that holds it is a shed.
        let back = sheds_of(&shed);
        assert_eq!(paths(&back), vec![t]);
    }

    #[test]
    fn found_by_both_sources_is_one_shed_of_kind_worktree() {
        let b = base("both");
        let t = b.join("repo");
        repo_with_farm(&t);
        sh(&t, &["remote", "add", "origin", &url(&b, "r")]);
        // A linked worktree that also matches the Delta scan (T/.delta/worktrees/*/repo).
        let wt = t.join(".delta/worktrees/xyz/repo");
        fs::create_dir_all(wt.parent().unwrap()).unwrap();
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        let sheds = sheds_of(&t);
        assert_eq!(paths(&sheds), vec![wt]);
        assert_eq!(sheds[0].kind, ShedKind::Worktree);
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

        let sheds = sheds_of(&t);
        let by = |p: &Path| sheds.iter().find(|l| l.path == p).unwrap();
        assert_eq!(by(&private).farm, ShedFarm::None);
        assert_eq!(by(&shared).farm, ShedFarm::Shared);
        let text = render(&sheds);
        assert!(
            text.contains("no farm here (private? set YAKS_DIR)"),
            "{text}"
        );
        assert!(text.contains("shares this farm"), "{text}");
        let j = to_json(&sheds);
        let states: Vec<&str> = j
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l["farm"]["state"].as_str().unwrap())
            .collect();
        assert!(states.contains(&"none") && states.contains(&"shared"));
    }

    #[test]
    fn unreadable_shed_is_listed_with_its_error() {
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

        let sheds = sheds_of(&t);
        assert_eq!(paths(&sheds), vec![good, wt]);
        assert!(sheds[0].error.is_none());
        let bad = &sheds[1];
        assert!(bad.error.is_some());
        assert!(bad.head.is_none() && bad.dirty.is_none());
        assert!(render(&sheds).contains("  error: "));
        assert!(to_json(&sheds)[1]["error"].is_string());
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
    fn inspecting_a_shed_changes_nothing_in_it() {
        let b = base("readonly");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "w", wt.to_str().unwrap()],
        );
        // Make the shed's index stale (same bytes, newer mtime) so an ordinary
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
        let sheds = sheds_of(&t);
        assert_eq!(sheds.len(), 1);
        let _ = (render(&sheds), to_json(&sheds));
        let after = (
            snapshot(&wt),
            snapshot(&admin),
            sh(&wt, &["--no-optional-locks", "status", "--porcelain"]),
        );
        assert_eq!(before, after);
    }

    // -- shed labels (who is working in a shed, and on which yaks) ------------

    /// `body` with timestamped note entries appended: `(ts, actor, text)`.
    fn notes(body: &str, entries: &[(&str, Option<&str>, &str)]) -> String {
        entries
            .iter()
            .fold(body.to_string(), |b, (ts, actor, text)| {
                store::append_note(&b, ts, *actor, text)
            })
    }

    /// A repo whose committed farm has `yaks-aaaa` already claimed (shaving,
    /// with the coordinator's claim note) and `yaks-bbbb` still hairy, plus a
    /// linked worktree `wt` forked from it.
    fn claimed_repo_with_shed(tag: &str) -> (PathBuf, PathBuf) {
        let b = base(tag);
        let t = b.join("repo");
        fs::create_dir_all(&t).unwrap();
        sh(&t, &["init", "-q"]);
        let claim = notes(
            "first",
            &[(
                "2026-03-01T00:00:00Z",
                Some("coord"),
                "moved: hairy -> shaving",
            )],
        );
        task_file(&t, "shaving", "yaks-aaaa", "", &claim);
        task_file(&t, "hairy", "yaks-bbbb", "", "second");
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "init"]);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        (t, wt)
    }

    /// Rewrite `yaks-aaaa` (shaving) in the shed with extra note entries; the
    /// claim from the base stays first.
    fn shed_notes_on_aaaa(wt: &Path, entries: &[(&str, Option<&str>, &str)]) {
        let claim = notes(
            "first",
            &[(
                "2026-03-01T00:00:00Z",
                Some("coord"),
                "moved: hairy -> shaving",
            )],
        );
        task_file(wt, "shaving", "yaks-aaaa", "", &notes(&claim, entries));
    }

    fn label_of(t: &Path) -> (Vec<String>, Vec<String>, String, Value) {
        let sheds = sheds_of(t);
        assert_eq!(sheds.len(), 1);
        let l = &sheds[0];
        (
            l.who.clone(),
            l.in_progress.clone(),
            render(&sheds),
            to_json(&sheds),
        )
    }

    fn strs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_worker_named_shed_is_labeled_with_the_worker_and_its_yak() {
        let (t, wt) = claimed_repo_with_shed("label-worker");
        // The claim is in the base; the worker's own entry is a plain note.
        shed_notes_on_aaaa(&wt, &[("2026-03-02T00:00:00Z", Some("lbl-1"), "started")]);

        let (who, in_progress, text, json) = label_of(&t);
        assert_eq!(who, strs(&["lbl-1"]), "the base's claim actor is not ours");
        assert_eq!(in_progress, strs(&["yaks-aaaa"]));
        assert!(
            text.contains("  who: lbl-1 \u{b7} shaving: yaks-aaaa\n"),
            "{text}"
        );
        assert_eq!(json[0]["who"], json!(["lbl-1"]));
        assert_eq!(json[0]["in_progress"], json!(["yaks-aaaa"]));
        // The existing fields are untouched.
        assert_eq!(json[0]["farm"]["notes"][0]["new"], 1);
    }

    #[test]
    fn a_delta_thread_shed_is_labeled_with_its_delta_title() {
        let (t, wt) = claimed_repo_with_shed("label-delta");
        shed_notes_on_aaaa(
            &wt,
            &[(
                "2026-03-02T00:00:00Z",
                Some("delta:Fix attribution"),
                "on it",
            )],
        );
        let (who, _, text, json) = label_of(&t);
        assert_eq!(who, strs(&["delta:Fix attribution"]));
        assert!(
            text.contains("  who: delta:Fix attribution \u{b7} shaving: yaks-aaaa\n"),
            "{text}"
        );
        assert_eq!(json[0]["who"], json!(["delta:Fix attribution"]));
    }

    #[test]
    fn a_git_user_fallback_actor_labels_the_shed_like_any_other() {
        let (t, wt) = claimed_repo_with_shed("label-git-user");
        shed_notes_on_aaaa(&wt, &[("2026-03-02T00:00:00Z", Some("Jane Doe"), "note")]);
        let (who, in_progress, ..) = label_of(&t);
        assert_eq!(who, strs(&["Jane Doe"]));
        assert_eq!(in_progress, strs(&["yaks-aaaa"]));
    }

    #[test]
    fn an_idle_shed_is_unlabeled() {
        let (t, _wt) = claimed_repo_with_shed("label-idle");
        // Behind with no entries of its own: the base's claim names no one here.
        commit_move(&t, "yaks-bbbb", "hairy", "shorn");
        let (who, in_progress, text, json) = label_of(&t);
        assert!(who.is_empty() && in_progress.is_empty());
        assert!(
            !text.contains("who:") && !text.contains("shaving:"),
            "{text}"
        );
        assert_eq!(json[0]["who"], json!([]));
        assert_eq!(json[0]["in_progress"], json!([]));
    }

    #[test]
    fn entries_without_an_actor_are_not_a_who() {
        let (t, wt) = claimed_repo_with_shed("label-bare");
        shed_notes_on_aaaa(&wt, &[("2026-03-02T00:00:00Z", None, "bare note")]);
        let (who, in_progress, text, _) = label_of(&t);
        assert!(who.is_empty());
        // Still in progress, so the label says what, not who.
        assert_eq!(in_progress, strs(&["yaks-aaaa"]));
        assert!(text.contains("  shaving: yaks-aaaa\n"), "{text}");
        assert!(!text.contains("who:"), "{text}");
    }

    #[test]
    fn two_actors_in_one_shed_are_listed_once_each_in_first_seen_order() {
        let (t, wt) = claimed_repo_with_shed("label-two");
        // `yaks-bbbb` sorts after `yaks-aaaa`, but its note is the earliest.
        task_file(
            &wt,
            "hairy",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[
                    ("2026-03-02T00:00:00Z", Some("sheds-2"), "look"),
                    ("2026-03-04T00:00:00Z", Some("sheds-2"), "again"),
                ],
            ),
        );
        shed_notes_on_aaaa(
            &wt,
            &[
                ("2026-03-03T00:00:00Z", Some("delta:Fix attribution"), "hi"),
                ("2026-03-05T00:00:00Z", Some("sheds-2"), "bye"),
            ],
        );
        let (who, in_progress, text, json) = label_of(&t);
        assert_eq!(who, strs(&["sheds-2", "delta:Fix attribution"]));
        // `yaks-bbbb` is hairy in the shed: noted, not in progress.
        assert_eq!(in_progress, strs(&["yaks-aaaa"]));
        assert!(
            text.contains("  who: sheds-2, delta:Fix attribution \u{b7} shaving: yaks-aaaa\n"),
            "{text}"
        );
        assert_eq!(json[0]["who"], json!(["sheds-2", "delta:Fix attribution"]));
    }

    #[test]
    fn a_yak_shorn_within_the_shed_is_not_in_progress() {
        let (t, wt) = claimed_repo_with_shed("label-shorn");
        shed_notes_on_aaaa(&wt, &[("2026-03-02T00:00:00Z", Some("w1"), "working")]);
        // `yaks-bbbb`: claimed, worked and shorn inside the shed.
        fs::remove_file(wt.join(".yaks/hairy/yaks-bbbb.md")).unwrap();
        task_file(
            &wt,
            "shorn",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[
                    (
                        "2026-03-02T00:10:00Z",
                        Some("w2"),
                        "moved: hairy -> shaving",
                    ),
                    (
                        "2026-03-02T00:20:00Z",
                        Some("w2"),
                        "moved: shaving -> shorn",
                    ),
                ],
            ),
        );
        // A brand-new yak the shed created and left hairy.
        task_file(
            &wt,
            "hairy",
            "yaks-cccc",
            "",
            &notes("new", &[("2026-03-02T00:30:00Z", Some("w1"), "filed")]),
        );
        let (who, in_progress, text, json) = label_of(&t);
        assert_eq!(who, strs(&["w1", "w2"]));
        assert_eq!(in_progress, strs(&["yaks-aaaa"]));
        assert_eq!(json[0]["in_progress"], json!(["yaks-aaaa"]));
        assert!(
            text.contains("  who: w1, w2 \u{b7} shaving: yaks-aaaa\n"),
            "{text}"
        );
    }

    #[test]
    fn a_shed_compared_vs_checkout_is_unlabeled() {
        let b = base("label-nobase");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        // Unrelated history: no merge-base, so no telling which entries are its own.
        sh(&wt, &["checkout", "-q", "--orphan", "orphan"]);
        task_file(
            &wt,
            "shaving",
            "yaks-aaaa",
            "",
            &notes("first", &[("2026-03-02T00:00:00Z", Some("lbl-1"), "hi")]),
        );
        sh(&wt, &["add", "-A"]);
        sh(&wt, &["commit", "-q", "-m", "orphan"]);

        let (who, in_progress, text, json) = label_of(&t);
        assert_eq!(json[0]["farm"]["vs"], "checkout");
        assert!(who.is_empty() && in_progress.is_empty());
        assert!(
            !text.contains("who:") && !text.contains("shaving:"),
            "{text}"
        );
        assert_eq!(json[0]["who"], json!([]));
    }

    // -- the baseline is where the shed forked, not the viewer's merge-base ----

    /// Commit `yaks-aaaa` in `dir` with the base claim plus `entries`.
    fn commit_notes_on_aaaa(dir: &Path, entries: &[(&str, Option<&str>, &str)]) {
        shed_notes_on_aaaa(dir, entries);
        sh(dir, &["add", "-A"]);
        sh(dir, &["commit", "-q", "-m", "notes"]);
    }

    /// A repo `t` (the viewer) whose `coord` shed already has its own notes
    /// (`coordx`), plus a worker shed `name` forked from that commit.
    fn worker_of(t: &Path, coord: &Path, name: &str) -> PathBuf {
        let wt = t.parent().unwrap().join(name);
        sh(
            t,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                name,
                wt.to_str().unwrap(),
                &sh(coord, &["rev-parse", "HEAD"]),
            ],
        );
        wt
    }

    fn coordinator_with_own_notes(tag: &str) -> (PathBuf, PathBuf) {
        let (t, coord) = claimed_repo_with_shed(tag);
        commit_notes_on_aaaa(
            &coord,
            &[("2026-03-02T00:00:00Z", Some("coordx"), "coordinator note")],
        );
        (t, coord)
    }

    fn shed_at<'a>(sheds: &'a [Shed], path: &Path) -> &'a Shed {
        sheds.iter().find(|l| l.path == path).unwrap()
    }

    #[test]
    fn a_worker_forked_from_a_coordinator_with_notes_shows_only_the_workers_entries() {
        let (t, coord) = coordinator_with_own_notes("fork-worker");
        let w = worker_of(&t, &coord, "w1");
        commit_notes_on_aaaa(
            &w,
            &[
                ("2026-03-02T00:00:00Z", Some("coordx"), "coordinator note"),
                ("2026-03-03T00:00:00Z", Some("w-1"), "worker note"),
            ],
        );
        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &w);
        assert_eq!(l.who, strs(&["w-1"]), "coordx predates the fork");
        assert_eq!(l.in_progress, strs(&["yaks-aaaa"]));
        let fork = sh(&coord, &["rev-parse", "--short", "HEAD"]);
        assert_eq!(
            l.farm_base,
            Some(FarmBase {
                kind: BaseKind::Fork,
                sha: fork.clone()
            })
        );
        let ShedFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert_eq!(d.notes, vec![("yaks-aaaa".to_string(), 1)]);
        // Ahead/behind still describe the shed relative to the viewer.
        assert_eq!((l.ahead, l.behind), (Some(2), Some(0)));
        let j = to_json(&sheds);
        let jw = j
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["path"] == w.display().to_string().as_str())
            .unwrap();
        assert_eq!(jw["who"], json!(["w-1"]));
        assert_eq!(jw["farm"]["vs"], "fork");
        assert_eq!(jw["farm"]["base"], fork.as_str());
        assert!(
            render(&sheds).contains("  who: w-1 \u{b7} shaving: yaks-aaaa\n"),
            "{}",
            render(&sheds)
        );
    }

    #[test]
    fn a_freshly_spawned_worker_has_no_changes_and_no_label() {
        let (t, coord) = coordinator_with_own_notes("fork-fresh");
        let w = worker_of(&t, &coord, "w1");
        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &w);
        assert!(l.who.is_empty() && l.in_progress.is_empty());
        assert_eq!(l.farm, ShedFarm::Own(FarmDelta::default()));
        assert!(
            render(&sheds).contains("farm: no changes of its own\n"),
            "{}",
            render(&sheds)
        );
    }

    #[test]
    fn two_sheds_forked_from_the_same_coordinator_share_no_labels() {
        let (t, coord) = coordinator_with_own_notes("fork-two");
        let (w1, w2) = (worker_of(&t, &coord, "w1"), worker_of(&t, &coord, "w2"));
        let base_entries = ("2026-03-02T00:00:00Z", Some("coordx"), "coordinator note");
        commit_notes_on_aaaa(
            &w1,
            &[base_entries, ("2026-03-03T00:00:00Z", Some("w-1"), "one")],
        );
        commit_notes_on_aaaa(
            &w2,
            &[base_entries, ("2026-03-03T00:00:00Z", Some("w-2"), "two")],
        );
        let sheds = sheds_of(&t);
        assert_eq!(shed_at(&sheds, &w1).who, strs(&["w-1"]));
        assert_eq!(shed_at(&sheds, &w2).who, strs(&["w-2"]));
    }

    #[test]
    fn a_landed_squashed_shed_shows_its_own_commit_not_the_history_before_it() {
        let (t, coord) = coordinator_with_own_notes("fork-squash");
        let w = worker_of(&t, &coord, "w1");
        commit_notes_on_aaaa(
            &w,
            &[
                ("2026-03-02T00:00:00Z", Some("coordx"), "coordinator note"),
                ("2026-03-03T00:00:00Z", Some("w-1"), "worker note"),
            ],
        );
        // The viewer lands the coordinator and the worker as ONE squash
        // commit: neither shed's commits are ancestors of the viewer's HEAD.
        sh(&t, &["merge", "--squash", "-q", "w1"]);
        sh(&t, &["commit", "-q", "-m", "squash"]);
        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &w);
        assert_eq!(l.who, strs(&["w-1"]));
        // The viewer's merge-base with the worker is OLDER than the fork point
        // (the squash left the coordinator's commits out of the viewer's
        // history), so the fork point still wins: no `sync`.
        let fork = sh(&coord, &["rev-parse", "--short", "HEAD"]);
        assert_eq!(
            l.farm_base,
            Some(FarmBase {
                kind: BaseKind::Fork,
                sha: fork
            })
        );
        let ShedFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert_eq!(d.notes, vec![("yaks-aaaa".to_string(), 1)]);
    }

    /// The shed `l`'s JSON object from `to_json`.
    fn json_of(sheds: &[Shed], l: &Path) -> Value {
        to_json(sheds)
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["path"] == l.display().to_string().as_str())
            .unwrap()
            .clone()
    }

    /// What a shed that has been synced with the viewer must show: only what
    /// came after the sync, exact (no `vs` marker), `farm.vs` = `sync`.
    fn assert_only_post_sync(t: &Path, w: &Path, base_sha: &str) {
        let sheds = sheds_of(t);
        let l = shed_at(&sheds, w);
        assert_eq!(
            l.farm_base,
            Some(FarmBase {
                kind: BaseKind::Sync,
                sha: base_sha.to_string()
            })
        );
        assert_eq!(l.who, strs(&["new-1"]), "oldx and viewx predate the sync");
        assert_eq!(l.in_progress, strs(&["yaks-aaaa"]));
        let ShedFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert_eq!(d.notes, vec![("yaks-aaaa".to_string(), 1)]);
        assert!(d.added.is_empty() && d.moved.is_empty(), "{d:?}");
        let text = render(&sheds);
        assert!(
            text.contains("  who: new-1 \u{b7} shaving: yaks-aaaa\n"),
            "{text}"
        );
        assert!(
            !text.contains("(vs "),
            "sync is exact, not a fallback: {text}"
        );
        let jw = json_of(&sheds, w);
        assert_eq!(jw["farm"]["vs"], "sync");
        assert_eq!(jw["farm"]["base"], base_sha);
        assert_eq!(jw["who"], json!(["new-1"]));
    }

    #[test]
    fn a_shed_the_viewer_merged_shows_only_what_came_after_that() {
        let (t, w) = claimed_repo_with_shed("sync-viewer-merged");
        let old = ("2026-03-02T00:00:00Z", Some("oldx"), "old note");
        commit_notes_on_aaaa(&w, &[old]);
        // The viewer merges the shed (a real merge commit), then the shed
        // carries on.
        sh(&t, &["merge", "--no-ff", "-q", "-m", "land", "shed"]);
        let synced = sh(&w, &["rev-parse", "--short", "HEAD"]);
        commit_notes_on_aaaa(
            &w,
            &[old, ("2026-03-03T00:00:00Z", Some("new-1"), "new note")],
        );
        assert_only_post_sync(&t, &w, &synced);
        // ahead/behind are untouched: still relative to the viewer's HEAD.
        let sheds = sheds_of(&t);
        assert_eq!(
            (shed_at(&sheds, &w).ahead, shed_at(&sheds, &w).behind),
            (Some(1), Some(1))
        );
    }

    #[test]
    fn a_shed_that_merged_the_viewer_shows_only_what_came_after_that() {
        let (t, w) = claimed_repo_with_shed("sync-shed-merged");
        // The viewer moves on (a note by `viewx`) and the shed pulls it in.
        let old = ("2026-03-02T12:00:00Z", Some("viewx"), "viewer note");
        commit_notes_on_aaaa(&t, &[old]);
        let branch = sh(&t, &["symbolic-ref", "--short", "HEAD"]);
        sh(&w, &["merge", "--no-ff", "-q", "-m", "sync", &branch]);
        let synced = sh(&t, &["rev-parse", "--short", "HEAD"]);
        commit_notes_on_aaaa(
            &w,
            &[old, ("2026-03-03T00:00:00Z", Some("new-1"), "new note")],
        );
        assert_only_post_sync(&t, &w, &synced);
    }

    #[test]
    fn a_shed_that_never_synced_keeps_its_fork_point_even_when_the_viewer_moved_on() {
        let (t, w) = claimed_repo_with_shed("sync-none");
        let fork = sh(&w, &["rev-parse", "--short", "HEAD"]);
        commit_notes_on_aaaa(&w, &[("2026-03-03T00:00:00Z", Some("new-1"), "new note")]);
        task_file(
            &t,
            "hairy",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[("2026-03-02T12:00:00Z", Some("viewx"), "viewer note")],
            ),
        );
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "viewer note"]);
        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &w);
        assert_eq!(
            l.farm_base,
            Some(FarmBase {
                kind: BaseKind::Fork,
                sha: fork.clone()
            })
        );
        assert_eq!(l.who, strs(&["new-1"]));
        assert_eq!(json_of(&sheds, &w)["farm"]["vs"], "fork");
    }

    /// `<git-dir>/logs/HEAD` of the worktree `name` of `t`.
    fn reflog_of(t: &Path, name: &str) -> PathBuf {
        t.join(".git/worktrees").join(name).join("logs/HEAD")
    }

    #[test]
    fn a_missing_reflog_falls_back_to_the_merge_base_and_says_so() {
        let (t, coord) = coordinator_with_own_notes("fork-noreflog");
        let w = worker_of(&t, &coord, "w1");
        commit_notes_on_aaaa(
            &w,
            &[
                ("2026-03-02T00:00:00Z", Some("coordx"), "coordinator note"),
                ("2026-03-03T00:00:00Z", Some("w-1"), "worker note"),
            ],
        );
        assert!(
            reflog_of(&t, "w1").exists(),
            "the test needs a reflog to remove"
        );
        fs::remove_file(reflog_of(&t, "w1")).unwrap();
        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &w);
        assert_eq!(l.farm_base.as_ref().unwrap().kind, BaseKind::MergeBase);
        // Today's behaviour, now marked: the coordinator's entry is inherited.
        assert_eq!(l.who, strs(&["coordx", "w-1"]));
        let text = render(&sheds);
        assert!(
            text.contains("  who: coordx, w-1 \u{b7} shaving: yaks-aaaa (vs merge-base)\n"),
            "{text}"
        );
        assert!(text.contains("new notes (vs merge-base)"), "{text}");
        let j = to_json(&sheds);
        let jw = j
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["path"] == w.display().to_string().as_str())
            .unwrap();
        assert_eq!(jw["farm"]["vs"], "merge-base");
    }

    #[test]
    fn an_unusable_reflog_falls_back_to_the_merge_base() {
        let (t, coord) = coordinator_with_own_notes("fork-badreflog");
        let w = worker_of(&t, &coord, "w1");
        commit_notes_on_aaaa(
            &w,
            &[
                ("2026-03-02T00:00:00Z", Some("coordx"), "coordinator note"),
                ("2026-03-03T00:00:00Z", Some("w-1"), "worker note"),
            ],
        );
        let log = reflog_of(&t, "w1");
        let good = fs::read_to_string(&log).unwrap();
        let first_unknown = |sha: &str| {
            let mut lines = good.lines();
            let first = lines.next().unwrap();
            let mut f: Vec<&str> = first.splitn(3, ' ').collect();
            f[1] = sha;
            let mut out = f.join(" ");
            for l in lines {
                out.push('\n');
                out.push_str(l);
            }
            out.push('\n');
            out
        };
        for bad in [
            // A commit our object store does not have.
            first_unknown("1234567890123456789012345678901234567890"),
            // Not a sha at all.
            first_unknown("not-a-sha"),
            String::new(),
            "garbage\n".to_string(),
        ] {
            fs::write(&log, bad).unwrap();
            let sheds = sheds_of(&t);
            let l = shed_at(&sheds, &w);
            assert_eq!(l.farm_base.as_ref().unwrap().kind, BaseKind::MergeBase);
        }
    }

    #[test]
    fn a_fork_point_that_is_not_an_ancestor_of_the_shed_falls_back() {
        let (t, coord) = coordinator_with_own_notes("fork-rebased");
        let w = worker_of(&t, &coord, "w1");
        // The shed rewrites its history onto an unrelated root: its fork point
        // is no longer part of it, so "since the fork" means nothing.
        sh(&w, &["checkout", "-q", "--orphan", "orphan"]);
        commit_notes_on_aaaa(&w, &[("2026-03-03T00:00:00Z", Some("w-1"), "x")]);
        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &w);
        assert_ne!(
            l.farm_base.as_ref().map(|b| b.kind),
            Some(BaseKind::Fork),
            "{l:?}"
        );
    }
}
