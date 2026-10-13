//! `yaks sheds`: every checkout of this repository (git worktrees and Delta
//! clones), this one included and marked, and what each one's farm changed.
//! Every row is measured the same way from every viewer; the viewer only
//! decides which row is `current` (yaks-ad6d).
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
//! the oldest entry of the shed's HEAD reflog ([`fork_point`]) and it is the
//! baseline from EVERY viewer (yaks-e9bb), except that the PRIMARY has no fork
//! point worth the name and is measured from its `origin/main` instead (what
//! it has not published; [`origin_base`]; its fork point only when there is no
//! usable origin ref). Either way the answer about a shed never
//! depends on which checkout runs `yaks sheds`. A worker forks from its
//! coordinator's thread, not from the checkout that runs `yaks sheds`, so
//! measuring from the merge-base with THIS checkout would credit the shed with
//! everything its coordinator did before spawning it. What a shed merged in
//! after forking (say its coordinator's `main`) is credited to it like anything
//! else it holds: a stable baseline is preferred to a viewer-dependent one.
//! Only when the reflog cannot give a usable fork point is the merge-base with
//! this checkout the baseline instead; that fallback IS viewer-relative, so the
//! output says so (`vs merge-base`). When there is no base farm to read at all,
//! the shed is compared with this checkout's working tree and says so (`vs this
//! checkout`).
//!
//! Discovery reads only what git records, from inside any checkout of the
//! repository; it never searches the filesystem (yaks-c635):
//!
//! 1. `git worktree list --porcelain` here (plain `git worktree add`
//!    checkouts; inside a Delta clone it shows only that clone).
//! 2. The HOST ([`host`]): the repository named by this checkout's
//!    `objects/info/alternates` (the human's `.git` on a machine where the
//!    project was added from a folder; Delta's managed BARE repo on a machine
//!    that only had the thread shared to it), else this repository itself.
//! 3. Delta clones ([`pinned`]): Delta pins every clone's commits in the host
//!    as `refs/delta/<dir>/<name>/<sha>`, from the moment the clone exists.
//!    Each pin group resolves to a checkout through the clone's git dir
//!    `<store>/<dir>/<name>.git` (`store` is the parent of our own git dir's
//!    parent when we are a clone, else `<host checkout>/.delta/clones`) and its
//!    `core.worktree`. A pin whose git dir is gone is a finished thread.
//! 4. When the host is a checkout (not bare), that checkout and its own
//!    `git worktree list`, so a Delta clone sees the primary's git worktrees.
//!
//! Everything found this way is the same repository by construction (one
//! object store, one pin namespace), so no URL or name matching is needed.
//! Limits: a clone on a shared-thread machine cannot see the human's checkout
//! (its host is the managed repo), and pins say nothing about which thread
//! owns a clone (the `who:` label does that).

use crate::model::Status;
use crate::store;
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime};

const EVERY: [Status; 4] = [Status::Hairy, Status::Shaving, Status::Shorn, Status::Dead];

/// How a shed was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShedKind {
    /// Listed by `git worktree list` (here, or in the host checkout).
    Worktree,
    /// A Delta clone, found through its `refs/delta` pins in the host.
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
    /// Yaks in both whose title, fields or description the shed rewrote (see
    /// [`store::edited`]).
    pub edited: Vec<String>,
    /// Touched ids that the REFERENCE farm (see the module docs and
    /// [`reference_tasks`]) already has in full: nothing outstanding for the
    /// shed to do about that yak, whether because the shed's state there
    /// exactly matches the reference or because the reference is further
    /// along (the shed's work was squashed/cherry-picked in under a
    /// different commit, or the shed is simply behind the reference on that
    /// one yak). Empty whenever there is no usable reference, and always for
    /// the primary's own row (see [`Shed::primary`]).
    pub landed: Vec<String>,
}

impl FarmDelta {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.moved.is_empty()
            && self.notes.is_empty()
            && self.needs.is_empty()
            && self.edited.is_empty()
    }

    /// Every distinct id this delta touches (new, moved, noted, `needs:`
    /// newly set, or edited), deduped: a yak can land in more than one of
    /// `added`/`moved`/`notes`/`needs`/`edited` at once. Compared with
    /// `landed` by the render layer to tell "all of it landed" from "none of
    /// it" from "some of it".
    pub fn touched(&self) -> Vec<String> {
        let mut ids: BTreeSet<String> = BTreeSet::new();
        ids.extend(self.added.iter().cloned());
        ids.extend(self.moved.iter().map(|m| m.id.clone()));
        ids.extend(self.notes.iter().map(|(id, _)| id.clone()));
        ids.extend(self.needs.iter().map(|(id, _)| id.clone()));
        ids.extend(self.edited.iter().cloned());
        ids.into_iter().collect()
    }
}

/// Which commit a shed's own changes are measured from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseKind {
    /// The primary checkout's `origin/main` (its remote-tracking ref, else
    /// `origin/HEAD`'s target): what the primary has not published. Only the
    /// primary is measured this way, from every viewer; it has no fork point
    /// worth the name (its reflog starts when the repository did).
    Origin,
    /// Where the shed itself forked (oldest entry of its HEAD reflog): exactly
    /// the shed's own work.
    Fork,
    /// The merge-base with this checkout: the fallback when the fork point is
    /// unknown. Unlike `Fork` it depends on which checkout is looking, and it
    /// also counts whatever the shed inherited from the thread that spawned
    /// it, so the output marks it.
    MergeBase,
}

impl BaseKind {
    /// The JSON `farm.vs` value.
    pub fn as_str(self) -> &'static str {
        match self {
            BaseKind::Origin => "origin",
            BaseKind::Fork => "fork",
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

/// Whether anyone appears to be actively working in a shed right now:
/// orthogonal to `farm`'s `landed` (yaks-ed5e) rather than a third value of
/// it, since the two questions ("is anything outstanding" and "is anyone
/// still touching it") have independent answers — a shed can be
/// landed-and-idle (the common case), landed-and-stale (nothing outstanding,
/// but its own idea of `shaving` disagrees with the reference and nobody has
/// touched it in a while), outstanding-and-active, or outstanding-and-stale
/// (abandoned mid-work). Computed the same way for every shed, the primary
/// included: unlike `landed` this is never a comparison to anything else
/// (yaks-9277).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    /// `in_progress` (see [`Shed::in_progress`]) is non-empty and the shed's
    /// farm activity is within [`STALE_AFTER`].
    Active,
    /// No claim at all, or a claim whose farm activity cannot be told
    /// (`farm_activity` unknown): no signal is not evidence of abandonment,
    /// so this is the default, not `Stale`.
    Idle,
    /// `in_progress` is non-empty but the shed's farm activity is older than
    /// [`STALE_AFTER`]: a claim nobody has touched in a while.
    Stale,
}

impl Liveness {
    /// The JSON `liveness` value.
    pub fn as_str(self) -> &'static str {
        match self {
            Liveness::Active => "active",
            Liveness::Idle => "idle",
            Liveness::Stale => "stale",
        }
    }
}

/// How long a shed with a claimed yak (`in_progress` non-empty) can go with
/// no farm activity before it reads `stale` instead of `active`. Hardcoded,
/// not configurable yet (yaks-9277: "revisit only if it misleads"); a config
/// override is a natural follow-up if 2h turns out to be wrong in practice.
const STALE_AFTER: Duration = Duration::from_secs(2 * 60 * 60);

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

/// One checkout of this repository.
#[derive(Debug, Clone)]
pub struct Shed {
    pub path: PathBuf,
    pub kind: ShedKind,
    /// The Delta directory id, for a pinned clone (see [`Found`]).
    pub dir: Option<String>,
    /// The Delta thread title the clone recorded in its own git config
    /// (`yaks.shed.title`, see [`record_thread`]); `None` for the primary and
    /// for any clone that never ran yaks in a thread.
    pub title: Option<String>,
    /// The name [`Shed::name`] and `resolve` use for this shed: the [`slug`]
    /// of `title`, made unique among the listed sheds by [`disambiguate`].
    pub slug: Option<String>,
    /// The repository's main checkout.
    pub primary: bool,
    /// The checkout `yaks sheds` was run in (the default shed when none is
    /// named). Exactly one listed shed is current.
    pub current: bool,
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
    /// Whether anyone appears to be actively working here right now; see
    /// [`Liveness`]. Every shed gets one, including the primary.
    pub liveness: Liveness,
    /// Set when the shed's git state could not be read; the shed is still
    /// listed.
    pub error: Option<String>,
}

impl Shed {
    /// A short name for messages and selectors: `main` for the primary
    /// checkout, else the slug of its thread title, else the first actor in
    /// `who`, else the Delta dir id, else the branch, else the directory name.
    pub fn name(&self) -> String {
        if self.primary {
            return "main".into();
        }
        self.slug
            .clone()
            .or_else(|| self.who.first().cloned())
            .or_else(|| self.dir.clone())
            .or_else(|| self.branch.clone())
            .unwrap_or_else(|| {
                self.path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| self.path.display().to_string())
            })
    }
}

/// List every checkout of the repository containing `cwd`, this one included
/// (`current`), each measured the same way. `farm_root` is this checkout's
/// farm: where a shed is compared with "this checkout" and what makes another
/// shed `Shared`. Sorted by path.
pub fn discover(cwd: &Path, farm_root: &Path) -> Result<Vec<Shed>> {
    Ok(discover_all(cwd, farm_root)?.0)
}

/// [`discover`] plus the orphaned pins ([`orphaned_pins`]) from the same pass:
/// the host is read and the pin groups are resolved once, not twice.
pub fn discover_all(cwd: &Path, farm_root: &Path) -> Result<(Vec<Shed>, Vec<Pinned>)> {
    let top = PathBuf::from(
        git(cwd, &["rev-parse", "--show-toplevel"]).context("yaks sheds needs a git repository")?,
    );
    let top = canonical(&top);
    // The four independent first questions, asked at once: the checkouts and
    // pins (every checkout including this one, yaks-ad6d: the answer about a
    // shed never depends on where you run `yaks sheds`, only which row is
    // marked `current`), whether the repository is shallow, our own farm, and
    // our HEAD (so a shed at the same commit needs no merge-base).
    let (gathered, shallow, ours, top_head) = std::thread::scope(|s| {
        let found = s.spawn(|| gather(&top));
        let shallow = s.spawn(|| {
            git(&top, &["rev-parse", "--is-shallow-repository"]).is_ok_and(|s| s == "true")
        });
        let ours = s.spawn(|| load_farm(farm_root).unwrap_or_default());
        let head = s.spawn(|| git(&top, &["rev-parse", "HEAD"]).ok());
        (
            found.join().unwrap(),
            shallow.join().unwrap(),
            ours.join().unwrap(),
            head.join().unwrap(),
        )
    });
    let (found, pins) = gathered;
    let our_farm = canonical(farm_root);
    // The REFERENCE every shed's `landed` is measured against (same from
    // every viewer): the primary's path, found the cheap way (it is already
    // in `found`, no second discovery walk).
    let primary_path = found
        .iter()
        .find(|(_, f)| f.primary)
        .map(|(p, _)| p.clone());
    let reference = reference_tasks(primary_path.as_deref(), &top, &our_farm);
    let found: Vec<(PathBuf, Found)> = found.into_iter().collect();
    let cache = BaseCache::default();
    let mut sheds: Vec<Shed> = par_map(&found, |(path, found)| {
        let mut shed = inspect(
            &Ctx {
                top: &top,
                our_farm: &our_farm,
                ours: &ours,
                reference: reference.as_deref(),
                top_head: top_head.as_deref(),
                cache: &cache,
            },
            path,
            found,
            *path == top,
        );
        shed.shallow_here = shallow;
        shed
    });
    disambiguate(&mut sheds);
    let orphaned = pins.into_iter().filter(|p| p.checkout.is_err()).collect();
    Ok((sheds, orphaned))
}

/// Apply `f` to every item on a small pool of threads, results in item order.
/// What dominates a shed's cost is waiting on `git` child processes, so the
/// sheds (and the pin groups) are measured side by side.
fn par_map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    const WORKERS: usize = 8;
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<Option<R>>> = Mutex::new(items.iter().map(|_| None).collect());
    std::thread::scope(|s| {
        for _ in 0..WORKERS.min(items.len()) {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = items.get(i) else { break };
                    let r = f(item);
                    out.lock().unwrap()[i] = Some(r);
                }
            });
        }
    });
    out.into_inner()
        .unwrap()
        .into_iter()
        .map(|r| r.expect("every item was mapped"))
        .collect()
}

/// What every shed of one `discover` shares: read once, shown to all.
struct Ctx<'a> {
    top: &'a Path,
    our_farm: &'a Path,
    ours: &'a [crate::model::Task],
    reference: Option<&'a [crate::model::Task]>,
    /// Our own HEAD (full sha), when it resolves.
    top_head: Option<&'a str>,
    cache: &'a BaseCache,
}

/// [`base_farm`] results for one `discover`, by (base commit, farm path in the
/// checkout): sheds forked from one commit (every worker a coordinator spawned
/// at once) read that commit's farm once.
#[derive(Default)]
struct BaseCache(Mutex<HashMap<(String, String), BaseCell>>);

type BaseCell = Arc<OnceLock<Option<(String, Arc<Vec<crate::model::Task>>)>>>;

impl BaseCache {
    fn get(
        &self,
        top: &Path,
        base: &str,
        shed: &Path,
        shed_farm: &Path,
    ) -> Option<(String, Arc<Vec<crate::model::Task>>)> {
        let rel = shed_farm
            .strip_prefix(shed)
            .ok()?
            .to_str()?
            .replace('\\', "/");
        let cell = {
            let mut m = self.0.lock().unwrap();
            m.entry((base.to_string(), rel)).or_default().clone()
        };
        cell.get_or_init(|| {
            base_farm(top, base, shed, shed_farm).map(|(sha, tasks)| (sha, Arc::new(tasks)))
        })
        .clone()
    }
}

/// Longest slug [`slug`] returns.
const MAX_SLUG: usize = 24;

/// A git-worktree-like name for a thread title: lowercase, every run of
/// characters outside `[a-z0-9]` one `-`, no leading or trailing `-`, cut to
/// at most [`MAX_SLUG`] characters at a `-` boundary when there is one.
/// `None` when nothing is left (a title of only punctuation or emoji).
pub fn slug(title: &str) -> Option<String> {
    let mut s = String::new();
    for c in title.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            s.push(c);
        } else if !s.ends_with('-') {
            s.push('-');
        }
    }
    // Pure ASCII from here, so byte offsets are character offsets.
    let s = s.trim_matches('-');
    let cut = if s.len() <= MAX_SLUG {
        s
    } else if s.as_bytes()[MAX_SLUG] == b'-' {
        &s[..MAX_SLUG]
    } else {
        let head = &s[..MAX_SLUG];
        head.rfind('-').map_or(head, |i| &head[..i])
    };
    let cut = cut.trim_matches('-');
    (!cut.is_empty()).then(|| cut.to_string())
}

/// Sheds whose title slugs are equal all get the first 4 characters of their
/// Delta dir id appended (`<slug>-q7zz`), so a name picks out one shed. The
/// primary is `main` whatever its title, so it takes no part.
fn disambiguate(sheds: &mut [Shed]) {
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for s in sheds.iter().filter(|s| !s.primary) {
        if let Some(slug) = &s.slug {
            *count.entry(slug.clone()).or_default() += 1;
        }
    }
    for s in sheds.iter_mut().filter(|s| !s.primary) {
        let (Some(slug), Some(dir)) = (&mut s.slug, &s.dir) else {
            continue;
        };
        if count[slug.as_str()] > 1 {
            slug.push('-');
            slug.extend(dir.chars().take(4));
        }
    }
}

/// How one shed was found, and the identifiers discovery knows for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub kind: ShedKind,
    /// The Delta directory id (`refs/delta/<dir>/...`), for a pinned clone.
    pub dir: Option<String>,
    /// The repository's main checkout (the host checkout, or the main
    /// working tree `git worktree list` names first).
    pub primary: bool,
}

/// Every OTHER checkout of the repository at `top` (see [`checkout_paths`]).
pub(crate) fn shed_paths(top: &Path) -> BTreeMap<PathBuf, Found> {
    let mut found = checkout_paths(top);
    found.remove(top);
    found
}

/// Every checkout of the repository at `top`, `top` itself included, by
/// canonical path, with how it was found (see the module docs). A checkout
/// found both ways is a `Worktree` and keeps its Delta dir id. `top` is listed
/// even when nothing records it (the answer never depends on where you are).
pub(crate) fn checkout_paths(top: &Path) -> BTreeMap<PathBuf, Found> {
    gather(top).0
}

/// [`checkout_paths`] plus every resolved pin group of the host (live and
/// orphaned): everything discovery reads from git about WHERE the checkouts
/// are, in one pass. The independent questions (`git worktree list` here, the
/// host and its pins, the host checkout's worktrees) are asked side by side,
/// and the pin groups are resolved side by side; they are merged in the order
/// they always were, so the result is the same.
fn gather(top: &Path) -> (BTreeMap<PathBuf, Found>, Vec<Pinned>) {
    let add_worktrees = |paths: Vec<PathBuf>, found: &mut BTreeMap<PathBuf, Found>| {
        for (i, p) in paths.into_iter().enumerate() {
            let e = found.entry(canonical(&p)).or_insert(Found {
                kind: ShedKind::Worktree,
                dir: None,
                primary: false,
            });
            e.kind = ShedKind::Worktree;
            // git lists the main working tree first.
            e.primary |= i == 0;
        }
    };
    let (here, hosted) = std::thread::scope(|s| {
        let here = s.spawn(|| worktree_paths(top));
        let hosted = s.spawn(|| {
            let h = host(top)?;
            // The host checkout's `git worktree list`, asked once: its first
            // entry IS the host checkout, and the list is the same from there.
            let (entries, pins) = std::thread::scope(|s| {
                let pins = s.spawn(|| pinned(&h));
                let entries = if h.bare {
                    Vec::new()
                } else {
                    worktree_entries(&h.git_dir)
                };
                (entries, pins.join().unwrap())
            });
            let primary = host_checkout_of(&entries);
            let paths = if primary.is_some() {
                entries_paths(entries)
            } else {
                Vec::new()
            };
            Some((paths, pins))
        });
        (here.join().unwrap(), hosted.join().unwrap())
    });
    let mut found: BTreeMap<PathBuf, Found> = BTreeMap::new();
    add_worktrees(here, &mut found);
    let mut all_pins = Vec::new();
    if let Some((paths, pins)) = hosted {
        add_worktrees(paths, &mut found);
        for p in &pins {
            if let Ok(path) = &p.checkout {
                found
                    .entry(path.clone())
                    .or_insert(Found {
                        kind: ShedKind::Delta,
                        dir: None,
                        primary: false,
                    })
                    .dir = Some(p.dir.clone());
            }
        }
        all_pins = pins;
    }
    found.entry(top.to_path_buf()).or_insert(Found {
        kind: ShedKind::Worktree,
        dir: None,
        primary: false,
    });
    (found, all_pins)
}

/// Pick one shed by `query`, in this order: its path (as given or
/// canonical), its Delta dir id, its slug (the name `yaks sheds` shows for a
/// clone with a thread title), `main` for the primary checkout, its
/// branch, an actor in its `who`, then a unique case-insensitive substring of
/// any of those, its slug, or its full title. `main` names the primary before
/// any branch called `main` (every Delta clone usually is on one). Ambiguity
/// and no match are errors that list the candidates, so a caller can print
/// them as is.
pub fn resolve<'a>(sheds: &'a [Shed], query: &str) -> Result<&'a Shed, String> {
    let q = query.trim();
    if q.is_empty() {
        return Err("empty shed name".into());
    }
    // `main` names the primary checkout and nothing else: when it is not in
    // the list (a clone on a machine where the primary is not visible),
    // falling through to "a branch called `main`" would silently pick a Delta
    // clone.
    if q == "main" && !sheds.iter().any(|s| s.primary) {
        return Err("`main` is the primary checkout, and it is not visible from here".into());
    }
    let qpath = canonical(Path::new(q));
    let rules: [&dyn Fn(&Shed) -> bool; 7] = [
        &|s| s.path == qpath || s.path == Path::new(q),
        &|s| s.dir.as_deref() == Some(q),
        &|s| s.slug.as_deref() == Some(q),
        &|s| q == "main" && s.primary,
        &|s| s.branch.as_deref() == Some(q),
        &|s| s.who.iter().any(|w| w == q),
        &|s| {
            let ql = q.to_lowercase();
            let hit = |t: &str| t.to_lowercase().contains(&ql);
            hit(&s.path.display().to_string())
                || s.dir.as_deref().is_some_and(hit)
                || s.slug.as_deref().is_some_and(hit)
                || s.title.as_deref().is_some_and(hit)
                || s.who.iter().any(|w| hit(w))
        },
    ];
    for rule in rules {
        let hits: Vec<&Shed> = sheds.iter().filter(|s| rule(s)).collect();
        match hits.len() {
            0 => continue,
            1 => return Ok(hits[0]),
            _ => {
                return Err(format!(
                    "`{q}` names {} sheds; be more specific:\n{}",
                    hits.len(),
                    candidates(&hits)
                ));
            }
        }
    }
    let all: Vec<&Shed> = sheds.iter().collect();
    Err(if all.is_empty() {
        format!("no shed matches `{q}`: no checkouts found")
    } else {
        format!(
            "no shed matches `{q}`; the sheds are:\n{}",
            candidates(&all)
        )
    })
}

/// One line per shed: its short name, then its path.
fn candidates(sheds: &[&Shed]) -> String {
    sheds
        .iter()
        .map(|s| format!("  {}  {}\n", s.name(), s.path.display()))
        .collect()
}

/// Gather one shed's report. Never fails: unreadable parts become `error`.
fn inspect(ctx: &Ctx, path: &Path, found: &Found, current: bool) -> Shed {
    let (top, our_farm, ours, reference) = (ctx.top, ctx.our_farm, ctx.ours, ctx.reference);
    let mut shed = Shed {
        path: path.to_path_buf(),
        kind: found.kind,
        dir: found.dir.clone(),
        title: None,
        slug: None,
        primary: found.primary,
        current,
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
        liveness: Liveness::Idle,
        error: None,
    };
    let mut bases: Vec<(BaseKind, String)> = Vec::new();
    // The primary is measured from what it has not published, whoever looks.
    let (origin, probe) = std::thread::scope(|s| {
        let origin = s.spawn(|| found.primary.then(|| origin_base(path)).flatten());
        let probe = git_state(ctx, path, !found.primary);
        (origin.join().unwrap(), probe)
    });
    bases.extend(origin.map(|o| (BaseKind::Origin, o)));
    shed.title = probe.title;
    shed.slug = shed.title.as_deref().and_then(slug);
    match probe.state {
        Ok(g) => {
            shed.branch = g.branch;
            shed.head = Some(g.head);
            shed.ahead = g.ahead;
            shed.behind = g.behind;
            shed.dirty = Some(g.dirty);
            bases.extend(g.fork.map(|f| (BaseKind::Fork, f)));
            bases.extend(g.merge_base.map(|b| (BaseKind::MergeBase, b)));
        }
        Err(e) => shed.error = Some(format!("{e:#}")),
    }
    // The farm side is plain filesystem reads, so it is reported even when git
    // could not be read.
    let mut activity: Option<SystemTime> = None;
    match store::discover_with(path, None) {
        Err(_) => shed.farm = ShedFarm::None,
        Ok(d) => {
            let root = canonical(&d.root);
            // "Shares this farm" says ANOTHER shed uses the viewer's farm; the
            // viewer's own checkout is measured like any other shed.
            if root == our_farm && !current {
                shed.farm = ShedFarm::Shared;
            } else {
                activity = newest_mtime(&root);
                shed.farm_activity = activity.map(format_time);
                shed.farm = match load_farm(&root) {
                    Ok(theirs) => {
                        // What the shed changed is relative to where it forked
                        // (else the merge-base), never to our (possibly newer)
                        // checkout, so every viewer gets the same answer.
                        let base = bases.iter().find_map(|(kind, b)| {
                            ctx.cache
                                .get(top, b, path, &root)
                                .map(|(sha, tasks)| (FarmBase { kind: *kind, sha }, tasks))
                        });
                        match base {
                            Some((base, base_tasks)) => {
                                shed.farm_base = Some(base);
                                (shed.who, shed.in_progress) = label(&base_tasks, &theirs);
                                let mut d = compare_farms(&base_tasks, &theirs);
                                if !found.primary {
                                    d.landed = landed_ids(&base_tasks, &theirs, reference);
                                }
                                ShedFarm::Own(d)
                            }
                            None => {
                                let mut d = compare_farms(ours, &theirs);
                                if !found.primary {
                                    d.landed = landed_ids(ours, &theirs, reference);
                                }
                                ShedFarm::Own(d)
                            }
                        }
                    }
                    Err(e) => ShedFarm::Unreadable(format!("{e:#}")),
                };
            }
        }
    }
    shed.liveness = liveness(&shed.in_progress, activity);
    shed
}

/// [`Liveness`] from the two raw signals: whether the shed has claimed a yak
/// with own entries there (`in_progress`, see [`label`]) and when its farm
/// was last touched (`None` when unknown). Pure, so it is easy to hit all
/// three cases in a unit test without a real filesystem/git fixture.
fn liveness(in_progress: &[String], activity: Option<SystemTime>) -> Liveness {
    if in_progress.is_empty() {
        return Liveness::Idle;
    }
    match activity {
        // No signal is not evidence of abandonment.
        None => Liveness::Idle,
        Some(t) => {
            let elapsed = SystemTime::now()
                .duration_since(t)
                .unwrap_or(Duration::ZERO);
            if elapsed > STALE_AFTER {
                Liveness::Stale
            } else {
                Liveness::Active
            }
        }
    }
}

/// The REFERENCE farm's tasks (see the module docs): what every shed's
/// `landed` is measured against, the same from every viewer. The PRIMARY
/// checkout's live working tree when it is visible here and its farm can be
/// read (a plain, fresh read: `Own`, `Shared` and even a farm that happens to
/// equal ours all count, since they are all readable), else the primary's
/// `origin/main` (read via `top`, since every checkout of one repository
/// shares the same remote-tracking refs), so a Delta-managed repo with no
/// visible primary (`local` is a bare repo with no branches) still has a
/// reference. `None` when neither works: callers then skip `landed` for
/// every shed, never guessing. `farm_root` is `top`'s own farm (used only for
/// the `origin/main` fallback, to read `top`'s `.yaks` as it existed there).
fn reference_tasks(
    primary_path: Option<&Path>,
    top: &Path,
    farm_root: &Path,
) -> Option<Vec<crate::model::Task>> {
    primary_path
        .and_then(|p| {
            let d = store::discover_with(p, None).ok()?;
            load_farm(&canonical(&d.root)).ok()
        })
        .or_else(|| {
            let commit = origin_base(top)?;
            base_farm(top, &commit, top, farm_root).map(|(_, tasks)| tasks)
        })
}

/// Where a status sits in the `hairy -> shaving -> shorn` (then
/// `dead`) progression, for telling a FORWARD move (a yak further along)
/// from a backward one (a yak not yet caught up): `EVERY`'s order.
fn status_rank(dir: &str) -> usize {
    EVERY
        .iter()
        .position(|s| s.as_str() == dir)
        .unwrap_or_default()
}

/// The ids of `yak_changes(reference, theirs)` that are genuinely
/// OUTSTANDING: the shed has something beyond the reference, not merely
/// behind it. A plain membership test would also flag the SUPERSEDED
/// direction (the shed's copy is an earlier state of the reference's: fewer
/// notes, or a status earlier in the hairy->shaving->shorn progression),
/// because [`yak_changes`]'s `moved` is a plain inequality with no direction
/// of its own (its `notes` already resolves this correctly by construction:
/// `.skip(known)` past a shorter list of theirs' notes is simply empty).
/// So a backward-only move (the reference's status ranks at or beyond
/// theirs') is not outstanding by itself; any other signal (new to the
/// reference, extra notes, a new `needs:`, a forward move, or edited) is.
/// A new `needs:` is itself superseded when the reference already resolved
/// it (see [`needs_superseded`]). Pure.
fn outstanding_ids(
    reference: &[crate::model::Task],
    theirs: &[crate::model::Task],
) -> BTreeSet<String> {
    let reference_by_id: BTreeMap<&str, &crate::model::Task> =
        reference.iter().map(|t| (t.id.as_str(), t)).collect();
    let theirs_by_id: BTreeMap<&str, &crate::model::Task> =
        theirs.iter().map(|t| (t.id.as_str(), t)).collect();
    yak_changes(reference, theirs)
        .into_iter()
        .filter(|c| {
            let id = c.id.as_str();
            let needs_outstanding = c.needs.is_some()
                && !reference_by_id
                    .get(id)
                    .zip(theirs_by_id.get(id))
                    .is_some_and(|(r, t)| needs_superseded(r, t));
            c.added
                || !c.notes.is_empty()
                || needs_outstanding
                || c.edited
                || c.moved
                    .is_some_and(|(from, to)| status_rank(to) > status_rank(from))
        })
        .map(|c| c.id)
        .collect()
}

/// The shed's `needs:` on `theirs` is SUPERSEDED by `reference` (its copy of
/// the same yak): the reference carries no `needs:` itself, already holds the
/// shed's latest ask note entry (same timestamp, actor and text), and has
/// either shipped the yak (shorn) or a later note entry after that ask: the
/// question was asked, then answered or picked up, and the `needs:` cleared.
/// A reference that never saw the ask (including a hand-set `needs:` with no
/// ask entry) or still carries a `needs:` does not supersede. Pure.
fn needs_superseded(reference: &crate::model::Task, theirs: &crate::model::Task) -> bool {
    if reference.needs.is_some() {
        return false;
    }
    let theirs_notes = store::parse_notes(&theirs.body);
    let Some(ask) = theirs_notes.iter().rfind(|n| store::is_ask_text(&n.text)) else {
        return false;
    };
    let notes = store::parse_notes(&reference.body);
    notes
        .iter()
        .position(|n| n == ask)
        .is_some_and(|at| reference.status == Status::Shorn || at + 1 < notes.len())
}

/// Which ids `yak_changes(base, theirs)` touches that the `reference` farm
/// already has in full too (see [`FarmDelta::landed`]): nothing outstanding
/// for the shed to do about them, whether because its state there matches
/// the reference exactly or the reference is further along (see
/// [`outstanding_ids`]). Empty with no `reference` or nothing touched. Pure.
fn landed_ids(
    base: &[crate::model::Task],
    theirs: &[crate::model::Task],
    reference: Option<&[crate::model::Task]>,
) -> Vec<String> {
    let Some(reference) = reference else {
        return Vec::new();
    };
    let touched: Vec<String> = yak_changes(base, theirs)
        .into_iter()
        .map(|c| c.id)
        .collect();
    if touched.is_empty() {
        return Vec::new();
    }
    let outstanding = outstanding_ids(reference, theirs);
    touched
        .into_iter()
        .filter(|id| !outstanding.contains(id))
        .collect()
}

/// The primary's `origin/main`: the remote-tracking ref `refs/remotes/origin/
/// main`, else the target of `refs/remotes/origin/HEAD`. `None` with no
/// `origin`, or when it was never fetched; the caller then falls back to the
/// fork point.
fn origin_base(primary: &Path) -> Option<String> {
    let commit = |r: &str| {
        git(
            primary,
            &["rev-parse", "--verify", "-q", &format!("{r}^{{commit}}")],
        )
        .ok()
        .filter(|s| !s.is_empty())
    };
    commit("refs/remotes/origin/main").or_else(|| {
        let target = git(primary, &["symbolic-ref", "-q", "refs/remotes/origin/HEAD"]).ok()?;
        commit(&target)
    })
}

struct GitState {
    branch: Option<String>,
    head: String,
    /// The shed's fork point, when its reflog gives a usable one (see
    /// [`fork_point`]): the same commit from every viewer.
    fork: Option<String>,
    /// `git merge-base HEAD <shed HEAD>` in OUR repo; `None` when there is
    /// none (then `ahead`/`behind` are unknown too).
    merge_base: Option<String>,
    ahead: Option<usize>,
    behind: Option<usize>,
    dirty: usize,
}

/// What one shed's git questions came to: its thread `title` (asked when
/// `want_title`; independent of whether the rest could be read) and the git
/// `state`.
struct GitProbe {
    title: Option<String>,
    state: Result<GitState>,
}

/// One shed's git state. After the one probe that names its git dir, full and
/// short HEAD, the independent questions (title, branch, `status`, and the
/// merge-base/count/fork-point chain in OUR repo) are asked side by side.
fn git_state(ctx: &Ctx, shed: &Path, want_title: bool) -> GitProbe {
    let top = ctx.top;
    // `rev-parse --absolute-git-dir HEAD --short HEAD`: git dir, full, short.
    let probe = git(
        shed,
        &["rev-parse", "--absolute-git-dir", "HEAD", "--short", "HEAD"],
    );
    let (git_dir, full, head) = match &probe {
        Ok(out) => {
            let mut l = out.lines();
            match (l.next(), l.next(), l.next()) {
                (Some(d), Some(f), Some(h)) => (d.to_string(), f.to_string(), h.to_string()),
                _ => (String::new(), String::new(), String::new()),
            }
        }
        Err(_) => (String::new(), String::new(), String::new()),
    };
    if full.is_empty() {
        // HEAD does not resolve: the error is the answer for the git state,
        // the title (a config read) is independent of it.
        let title = want_title
            .then(|| shed_git_dir(shed).and_then(|gd| config_get(&gd, TITLE_KEY)))
            .flatten();
        // The message is the one a plain `git rev-parse HEAD` gives (what
        // this has always reported), not the combined probe's.
        let e = match git(shed, &["rev-parse", "HEAD"]) {
            Err(e) => e,
            Ok(_) => probe
                .err()
                .unwrap_or_else(|| anyhow::anyhow!("git rev-parse HEAD: no output")),
        };
        return GitProbe {
            title,
            state: Err(e),
        };
    }
    std::thread::scope(|s| {
        let title = s.spawn(|| {
            want_title
                .then(|| config_get(&canonical(Path::new(&git_dir)), TITLE_KEY))
                .flatten()
        });
        // Exit 1 from `symbolic-ref -q` means a detached HEAD.
        let branch = s.spawn(|| {
            git(shed, &["symbolic-ref", "--short", "-q", "HEAD"])
                .ok()
                .filter(|b| !b.is_empty())
        });
        // `--no-optional-locks` (see `git`) keeps status from refreshing the index.
        let status = s.spawn(|| git_raw(shed, &["status", "--porcelain", "--untracked-files=all"]));
        // Counted in OUR repo, and only when the two HEADs have a merge-base
        // here. Without one (shallow history, unrelated histories, a shed
        // commit we do not have) rev-list would count a whole history, which
        // says nothing. The same commit as ours is its own merge-base, 0 and 0.
        let (merge_base, ahead, behind) = if ctx.top_head == Some(full.as_str()) {
            (Some(full.clone()), Some(0), Some(0))
        } else {
            let merge_base = git(top, &["merge-base", "HEAD", &full])
                .ok()
                .filter(|b| !b.is_empty());
            let (ahead, behind) = match &merge_base {
                None => (None, None),
                // `HEAD...full` counts `<left>\t<right>` = commits only we
                // have (the shed is BEHIND by) and only the shed has (AHEAD).
                Some(_) => git(
                    top,
                    &[
                        "rev-list",
                        "--left-right",
                        "--count",
                        &format!("HEAD...{full}"),
                    ],
                )
                .ok()
                .and_then(|c| {
                    let (l, r) = c.split_once('\t')?;
                    Some((r.trim().parse().ok()?, l.trim().parse().ok()?))
                })
                .unzip(),
            };
            (merge_base, ahead, behind)
        };
        let fork = fork_point(top, &git_dir, &full, merge_base.as_deref());
        let status = status.join().unwrap();
        let state = status.map(|status| GitState {
            branch: branch.join().unwrap(),
            head,
            fork,
            merge_base,
            ahead,
            behind,
            dirty: status.lines().filter(|l| !l.trim().is_empty()).count(),
        });
        GitProbe {
            title: title.join().unwrap(),
            state,
        }
    })
}

/// The commit `shed` was forked from: the second field of the first (oldest)
/// line of its HEAD reflog (`<git-dir>/logs/HEAD`). For a Delta clone that is
/// the coordinator's HEAD at spawn time, for a `git worktree add -b` shed the
/// branch creation. `None` (the caller falls back to the merge-base) when there
/// is no reflog, it is expired or unparseable, or the commit is not usable as a
/// baseline: unknown to our object store, or not an ancestor of the shed's HEAD
/// `full` (e.g. the shed was rebased since), which would make "since the fork"
/// meaningless. `git_dir` is the shed's absolute git dir; a fork point equal to
/// `full` or to the shed's `merge_base` with us is an ancestor by definition,
/// so it is not asked of git.
fn fork_point(top: &Path, git_dir: &str, full: &str, merge_base: Option<&str>) -> Option<String> {
    use std::io::{BufRead, BufReader};
    let file = fs::File::open(Path::new(git_dir).join("logs").join("HEAD")).ok()?;
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
    let known_ancestor = merge_base.is_some_and(|mb| sha == full || sha == mb);
    if !known_ancestor {
        git(top, &["merge-base", "--is-ancestor", sha, full]).ok()?;
    }
    Some(sha.to_string())
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
    // The short sha is independent of the tree read: ask both at once.
    let short = std::thread::scope(|s| {
        let short = s.spawn(|| git(top, &["rev-parse", "--short", base]).ok());
        let r = base_files(top, base, &rel);
        (short.join().unwrap(), r)
    });
    let (short, files) = short;
    let short = short?;
    let files = files?;
    base_farm_tasks(top, short, files)
}

/// The task files (`(blob sha, name under the farm)`) of the farm at `rel` in
/// commit `base`; `None` when there are none.
fn base_files(top: &Path, base: &str, rel: &str) -> Option<Vec<(String, String)>> {
    // Task files only: `<rel>/<id>.md`; attachments are skipped. A commit from
    // before the farm migrated has none (its yaks sat in status directories),
    // so there is nothing to read and the caller falls back to its next base.
    let tree = git_raw(top, &["ls-tree", "-r", "-z", base, "--", rel]).ok()?;
    let mut files: Vec<(String, String)> = Vec::new(); // (blob sha, path under the farm)
    for entry in tree.split('\0').filter(|e| !e.is_empty()) {
        let (meta, path) = entry.split_once('\t')?;
        let mut m = meta.split(' ');
        if m.nth(1) != Some("blob") {
            continue;
        }
        let sha = m.next()?;
        let Some(under) = path.strip_prefix(rel).and_then(|p| p.strip_prefix('/')) else {
            continue;
        };
        if !under.contains('/') && under.ends_with(".md") {
            files.push((sha.to_string(), under.to_string()));
        }
    }
    (!files.is_empty()).then_some(files)
}

/// Materialize `files` (blobs of our repo) into a temp dir and load them as a
/// farm: `(short sha, tasks)`.
fn base_farm_tasks(
    top: &Path,
    short: String,
    files: Vec<(String, String)>,
) -> Option<(String, Vec<crate::model::Task>)> {
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
    let tasks = load_farm(&farm).ok()?;
    Some((short, tasks))
}

/// The tasks of the farm at `root`, for comparing. A farm on the old layout
/// (an old-layout clone, from before 0.0.16) is an error here, never an empty
/// farm: reading it as one would look like it deleted every yak.
fn load_farm(root: &Path) -> Result<Vec<crate::model::Task>> {
    if store::old_layout(root) {
        anyhow::bail!(
            "old layout (pre-0.0.16 clone): its farm keeps yaks in status directories, which \
             this yaks does not read, so it is not read; {}",
            store::OLD_LAYOUT_FIX
        );
    }
    store::load(root, &EVERY)
}

/// Compare a shed's tasks to ours (see [`FarmDelta`]): the counts of
/// [`yak_changes`], which owns the rules. A created yak's notes count as new
/// notes too, so `farm:` agrees with `yaks changes`, which lists them. Pure;
/// ids sorted.
pub fn compare_farms(ours: &[crate::model::Task], theirs: &[crate::model::Task]) -> FarmDelta {
    let mut d = FarmDelta::default();
    for c in yak_changes(ours, theirs) {
        if c.added {
            d.added.push(c.id.clone());
        }
        if !c.notes.is_empty() {
            d.notes.push((c.id.clone(), c.notes.len()));
        }
        if let Some((from, to)) = c.moved {
            d.moved.push(Moved {
                id: c.id.clone(),
                from,
                to,
            });
        }
        if c.edited {
            d.edited.push(c.id.clone());
        }
        if let Some(n) = c.needs {
            d.needs.push((c.id, n));
        }
    }
    d
}

/// What one shed did to one yak since the baseline (see [`yak_changes`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YakChange {
    pub id: String,
    pub title: String,
    /// The yak's status directory in the shed.
    pub status: &'static str,
    /// The yak exists only in the shed.
    pub added: bool,
    /// `(from, to)` status directories, when the yak is in another status than
    /// at the baseline.
    pub moved: Option<(&'static str, &'static str)>,
    /// The note entries the shed appended (all of them for a created yak). A
    /// `moved:` transition is one of these, with its actor.
    pub notes: Vec<store::NoteEntry>,
    /// The shed's `needs:`, when it is new or differs from the baseline's.
    pub needs: Option<String>,
    /// The yak is in both and the shed rewrote its title, a field or its
    /// description ([`store::edited`]). Never set with `added`.
    pub edited: bool,
    /// The REFERENCE farm already has this yak's change in full (see
    /// [`FarmDelta::landed`]); always `false` from [`yak_changes`] itself
    /// (it has no reference to compare with) and only ever set by
    /// [`shed_yak_changes`], never for the primary's own row.
    pub landed: bool,
}

/// What the shed did to each yak it touched, per yak: one that is new, in
/// another status, has more notes, has a new `needs:` (one both farms already
/// carry is not news), or was edited (title, fields or description, as
/// `yaks status` reads it: [`store::edited`]; `updated:` alone is not an
/// edit). The one place these rules live;
/// [`compare_farms`] counts them. Pure; ids sorted; yaks the shed did not
/// touch are left out.
pub fn yak_changes(base: &[crate::model::Task], theirs: &[crate::model::Task]) -> Vec<YakChange> {
    let mine: BTreeMap<&str, &crate::model::Task> =
        base.iter().map(|t| (t.id.as_str(), t)).collect();
    let mut theirs: Vec<&crate::model::Task> = theirs.iter().collect();
    theirs.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = Vec::new();
    for t in theirs {
        let m = mine.get(t.id.as_str());
        let known = m.map_or(0, |m| store::parse_notes(&m.body).len());
        let change = YakChange {
            id: t.id.clone(),
            title: t.title.clone(),
            status: t.status.as_str(),
            added: m.is_none(),
            moved: m
                .filter(|m| m.status != t.status)
                .map(|m| (m.status.as_str(), t.status.as_str())),
            notes: store::parse_notes(&t.body)
                .into_iter()
                .skip(known)
                .collect(),
            needs: t
                .needs
                .clone()
                .filter(|n| Some(n) != m.and_then(|m| m.needs.as_ref())),
            edited: m.is_some_and(|m| store::edited(m, t)),
            landed: false,
        };
        if change.added
            || change.moved.is_some()
            || !change.notes.is_empty()
            || change.needs.is_some()
            || change.edited
        {
            out.push(change);
        }
    }
    out
}

/// One shed's own changes, per yak (`yaks changes`).
#[derive(Debug, Clone)]
pub struct Changes {
    pub name: String,
    pub path: PathBuf,
    /// The shed is the primary checkout (see [`Shed::primary`]).
    pub primary: bool,
    /// What the changes are relative to, as in [`Shed::farm_base`]; `None`
    /// means this checkout's farm.
    pub base: Option<FarmBase>,
    /// The changed yaks, or why there is nothing to list (a shared, absent or
    /// unreadable farm).
    pub yaks: Result<Vec<YakChange>, String>,
}

/// The per-yak changes of `shed` (one of [`discover`]'s), measured from the
/// same baseline `discover` chose for it. `cwd` and `farm_root` are what
/// `discover` was given; `sheds` is `discover`'s full result (used only to
/// find the primary's path for [`reference_tasks`], without a second
/// discovery walk). Read-only, like the rest of this module.
pub fn changes(shed: &Shed, cwd: &Path, farm_root: &Path, sheds: &[Shed]) -> Result<Changes> {
    let yaks = match &shed.farm {
        ShedFarm::Own(_) => {
            let top = canonical(&PathBuf::from(
                git(cwd, &["rev-parse", "--show-toplevel"])
                    .context("yaks changes needs a git repository")?,
            ));
            shed_yak_changes(shed, &top, farm_root, sheds)
        }
        other => Err(farm_message(other).unwrap_or_default()),
    };
    Ok(Changes {
        name: shed.name(),
        path: shed.path.clone(),
        primary: shed.primary,
        base: shed.farm_base.clone(),
        yaks,
    })
}

fn shed_yak_changes(
    shed: &Shed,
    top: &Path,
    farm_root: &Path,
    sheds: &[Shed],
) -> Result<Vec<YakChange>, String> {
    let root = store::discover_with(&shed.path, None)
        .map_err(|e| format!("{e:#}"))?
        .root;
    let root = canonical(&root);
    let theirs = load_farm(&root).map_err(|e| format!("{e:#}"))?;
    // The shed's own baseline when it has one (as `inspect` found it), else
    // this checkout's farm, exactly as `compare_farms` was fed.
    let base = shed
        .farm_base
        .as_ref()
        .and_then(|b| base_farm(top, &b.sha, &shed.path, &root))
        .map(|(_, tasks)| tasks)
        .unwrap_or_else(|| load_farm(farm_root).unwrap_or_default());
    let mut changes = yak_changes(&base, &theirs);
    if !shed.primary {
        let primary_path = sheds.iter().find(|s| s.primary).map(|s| s.path.as_path());
        if let Some(reference) = reference_tasks(primary_path, top, farm_root) {
            let outstanding = outstanding_ids(&reference, &theirs);
            for c in &mut changes {
                c.landed = !outstanding.contains(&c.id);
            }
        }
    }
    Ok(changes)
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

pub(crate) fn canonical(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// Real checkouts from `git worktree list --porcelain`: skips bare, prunable,
/// missing and `.git`-less entries. (Inside Delta the entry is the clone's
/// separate git dir, which holds no `.git`, so it drops out here.)
fn worktree_paths(top: &Path) -> Vec<PathBuf> {
    entries_paths(worktree_entries(top))
}

fn entries_paths(entries: Vec<(PathBuf, Result<(), String>)>) -> Vec<PathBuf> {
    entries
        .into_iter()
        .filter_map(|(p, verdict)| verdict.ok().map(|()| p))
        .collect()
}

/// Every entry of `git worktree list --porcelain` in `top`, each with why it
/// is not a checkout (`Err`) or `Ok` when it is one.
pub(crate) fn worktree_entries(top: &Path) -> Vec<(PathBuf, Result<(), String>)> {
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

/// The repository whose object store the checkout at `top` borrows, and
/// everything this machine's Delta records there (see the module docs).
#[derive(Debug, Clone)]
pub struct Host {
    /// This checkout's own git dir (`--absolute-git-dir`).
    pub own_git_dir: PathBuf,
    /// The host's git dir: the parent of the first `objects/info/alternates`
    /// line, else this checkout's own common git dir (a primary checkout, a
    /// plain git worktree, a non-Delta clone).
    pub git_dir: PathBuf,
    /// `true` when found through `alternates`.
    pub via_alternates: bool,
    /// The host is a bare repository (Delta's managed repo on a machine with
    /// no linked checkout); `false` means a real checkout.
    pub bare: bool,
    /// Where Delta keeps clone git dirs (`<store>/<dir>/<name>.git`), when
    /// known: beside our own git dir when we are a clone, else the host
    /// checkout's `.delta/clones`.
    pub store: Option<PathBuf>,
}

/// One `refs/delta/<dir>/<name>/*` pin group in the host, resolved.
#[derive(Debug, Clone)]
pub struct Pinned {
    pub dir: String,
    pub name: String,
    /// Pinned commits.
    pub pins: usize,
    /// The clone's checkout (its git dir's `core.worktree`), or why the pin
    /// does not resolve to a live checkout.
    pub checkout: Result<PathBuf, String>,
}

/// Find the host of the checkout at `top`.
pub fn host(top: &Path) -> Option<Host> {
    // One probe for both: this checkout's git dir and its common one.
    let probe = git(
        top,
        &["rev-parse", "--absolute-git-dir", "--git-common-dir"],
    )
    .ok()?;
    let mut lines = probe.lines();
    let own_git_dir = canonical(Path::new(lines.next()?));
    let common = resolve_common_dir(top, lines.next()?);
    let (git_dir, via_alternates) = match alternates(&common) {
        // `<host>/objects`; a host with no `objects` dir is not one we read.
        Some(objects) if objects.ends_with("objects") && objects.is_dir() => {
            (canonical(objects.parent()?), true)
        }
        _ => (common.clone(), false),
    };
    let bare = git(&git_dir, &["rev-parse", "--is-bare-repository"]).is_ok_and(|b| b == "true");
    let mut h = Host {
        own_git_dir,
        git_dir,
        via_alternates,
        bare,
        store: None,
    };
    h.store = if via_alternates {
        h.own_git_dir
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
    } else {
        host_checkout(&h).map(|checkout| checkout.join(".delta/clones"))
    };
    Some(h)
}

/// Every pin group in the host, resolved to its checkout through the sibling
/// git dir's `core.worktree`. Sorted by `<dir>/<name>`.
pub fn pinned(host: &Host) -> Vec<Pinned> {
    let Ok(refs) = git_raw(
        &host.git_dir,
        &["for-each-ref", "--format=%(refname)", "refs/delta/"],
    ) else {
        return Vec::new();
    };
    let mut groups: BTreeMap<(String, String), usize> = BTreeMap::new();
    for r in refs.lines() {
        let mut parts = r.trim().splitn(5, '/');
        // refs / delta / <dir> / <name> / <sha>
        if let (Some("refs"), Some("delta"), Some(dir), Some(name), Some(_)) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            *groups
                .entry((dir.to_string(), name.to_string()))
                .or_default() += 1;
        }
    }
    // Most groups are finished threads whose git dir is gone (a directory
    // test); each live one costs a `git config`, so those run side by side.
    let groups: Vec<_> = groups.into_iter().collect();
    par_map(&groups, |((dir, name), pins)| Pinned {
        dir: dir.clone(),
        name: name.clone(),
        pins: *pins,
        checkout: resolve_pin(host, dir, name),
    })
}

fn resolve_pin(host: &Host, dir: &str, name: &str) -> Result<PathBuf, String> {
    let store = host
        .store
        .as_ref()
        .ok_or("no clone store known here (run from inside a Delta clone)")?;
    let gd = store.join(dir).join(format!("{name}.git"));
    if !gd.is_dir() {
        return Err("gone (its git dir no longer exists)".into());
    }
    let cfg = gd.join("config");
    let wt = git(
        &gd,
        &[
            "config",
            "--file",
            &cfg.display().to_string(),
            "--get",
            "core.worktree",
        ],
    )
    .map_err(|_| "its git dir has no core.worktree".to_string())?;
    let wt = PathBuf::from(wt);
    let wt = if wt.is_absolute() { wt } else { gd.join(wt) };
    if !wt.join(".git").exists() {
        return Err(format!("checkout missing ({})", wt.display()));
    }
    Ok(canonical(&wt))
}

/// The host's main checkout, when the host is not bare (the human's checkout
/// on a machine where the project was added from a folder): the first entry
/// of `git worktree list` run in the host's git dir, which git always lists
/// as the main working tree.
pub(crate) fn host_checkout(host: &Host) -> Option<PathBuf> {
    if host.bare {
        return None;
    }
    host_checkout_of(&worktree_entries(&host.git_dir))
}

/// The main checkout among `git worktree list` entries: the first, when it is
/// a real checkout.
fn host_checkout_of(entries: &[(PathBuf, Result<(), String>)]) -> Option<PathBuf> {
    let (main, verdict) = entries.first()?;
    verdict.as_ref().ok()?;
    Some(canonical(main))
}

/// The shared (common) git dir of the checkout at `checkout`.
pub(crate) fn common_dir(checkout: &Path) -> Option<PathBuf> {
    let d = git(checkout, &["rev-parse", "--git-common-dir"]).ok()?;
    Some(resolve_common_dir(checkout, &d))
}

/// `git rev-parse --git-common-dir` output, made absolute against `checkout`.
fn resolve_common_dir(checkout: &Path, d: &str) -> PathBuf {
    let d = PathBuf::from(d);
    canonical(&if d.is_absolute() { d } else { checkout.join(d) })
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

/// The git config keys a Delta clone records its thread under, in its OWN git
/// dir's config: per clone, never committed, rebuildable (so not a second
/// source of truth for anything in `.yaks/`).
const TITLE_KEY: &str = "yaks.shed.title";
const THREAD_KEY: &str = "yaks.shed.thread";

/// The absolute git dir of the checkout at `checkout`.
fn shed_git_dir(checkout: &Path) -> Option<PathBuf> {
    git(checkout, &["rev-parse", "--absolute-git-dir"])
        .ok()
        .map(|d| canonical(Path::new(&d)))
}

/// `key` from `<git_dir>/config` alone (never the user's or system config).
fn config_get(git_dir: &Path, key: &str) -> Option<String> {
    let cfg = git_dir.join("config");
    let v = git(
        git_dir,
        &["config", "--file", &cfg.display().to_string(), "--get", key],
    )
    .ok()?;
    (!v.is_empty()).then_some(v)
}

/// Record this checkout's Delta thread (`$DELTA_THREAD_TITLE`, and
/// `$DELTA_CURRENT_THREAD_ID` when set) in its own git config, so
/// `yaks sheds` can name the clone by its thread. Called once per command
/// (see `main`); best effort and silent: any failure just leaves the name as
/// it was. See [`record_thread_in`] for when it writes.
pub fn record_thread() {
    let var = |k: &str| std::env::var(k).ok();
    if let Ok(cwd) = std::env::current_dir() {
        record_thread_in(
            &cwd,
            var("DELTA_THREAD_TITLE").as_deref(),
            var("DELTA_CURRENT_THREAD_ID").as_deref(),
        );
    }
}

/// The testable core of [`record_thread`]. Writes only in a Delta clone (its
/// objects come through `objects/info/alternates`, and `cwd`'s git dir is the
/// clone's own: a plain git worktree shares its config with the primary, and
/// a worktree of a clone has no config of its own), only a non-empty title,
/// and only a value that differs from what is stored, so most commands write
/// nothing. The title has its whitespace collapsed.
pub fn record_thread_in(cwd: &Path, title: Option<&str>, thread: Option<&str>) {
    let tidy = |s: Option<&str>| {
        let s = s?.split_whitespace().collect::<Vec<_>>().join(" ");
        (!s.is_empty()).then_some(s)
    };
    let Some(title) = tidy(title) else { return };
    let Some(h) = host(cwd) else { return };
    if !h.via_alternates || common_dir(cwd).is_none_or(|c| c != h.own_git_dir) {
        return;
    }
    let cfg = h.own_git_dir.join("config");
    let set = |key: &str, value: &str| {
        if config_get(&h.own_git_dir, key).as_deref() != Some(value) {
            let _ = git(
                &h.own_git_dir,
                &["config", "--file", &cfg.display().to_string(), key, value],
            );
        }
    };
    set(TITLE_KEY, &title);
    if let Some(thread) = tidy(thread) {
        set(THREAD_KEY, &thread);
    }
}

/// `remote.origin.url` from the repository whose git dir is `git_dir`.
pub(crate) fn origin_url(git_dir: &Path) -> Option<String> {
    let cfg = git_dir.join("config");
    git(
        git_dir,
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
                        // sha): the primary's `origin/main` (`origin`),
                        // where the shed forked (`fork`), else this
                        // checkout's merge-base with it as a fallback
                        // (`merge-base`), else, when neither can be read,
                        // this checkout's farm.
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
                        "edited": d.edited,
                        "landed": d.landed,
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
                    // The short name `resolve` and messages use, the Delta
                    // dir id of a pinned clone, and whether it is the main
                    // checkout.
                    "name": l.name(),
                    "dir": l.dir,
                    // The thread title the clone recorded (`null` if none).
                    "title": l.title,
                    "primary": l.primary,
                    // The checkout this was run in: exactly one row.
                    "current": l.current,
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
                    // Always present, including on the primary's own row
                    // (unlike `farm.landed`, this is never a comparison to
                    // anything, so there is nothing vacuous about it here).
                    "liveness": l.liveness.as_str(),
                    "error": l.error,
                })
            })
            .collect(),
    )
}

/// What the table says about a baseline other than the shed's own fork point,
/// so a number that may include inherited work is never silent: ` (vs
/// origin/main)` (the primary), ` (vs fork)` (the primary without one), ` (vs
/// merge-base)` or ` (vs this checkout)`. Empty for a shed's fork point.
fn vs_marker(l: &Shed) -> &'static str {
    base_marker(l.farm_base.as_ref(), l.primary)
}

/// The primary is expected to be measured from `origin/main`, so when it is
/// measured from its fork point instead (no usable `origin` ref) that is said
/// too; for any other shed the fork point is the plain, unmarked case.
fn base_marker(base: Option<&FarmBase>, primary: bool) -> &'static str {
    match base.map(|b| b.kind) {
        Some(BaseKind::Origin) => " (vs origin/main)",
        Some(BaseKind::Fork) if primary => " (vs fork)",
        Some(BaseKind::Fork) => "",
        Some(BaseKind::MergeBase) => " (vs merge-base)",
        None => " (vs this checkout)",
    }
}

/// The first non-empty line of a note's text: what the per-yak view shows.
fn first_line(text: &str) -> &str {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
}

/// The human per-yak view of `yaks changes`: one block per changed yak, then
/// its new note entries (timestamp, actor, first line) and `needs:`.
pub fn render_changes(c: &Changes) -> String {
    let mut out = format!("{}  {}\n", c.name, c.path.display());
    let marker = base_marker(c.base.as_ref(), c.primary);
    let yaks = match &c.yaks {
        Err(why) => {
            out.push_str(&format!("  nothing to show: {why}\n"));
            return out;
        }
        Ok(y) => y,
    };
    if yaks.is_empty() {
        out.push_str(&if c.base.is_some() {
            format!("  no changes of its own{marker}\n")
        } else {
            "  no differences (vs this checkout)\n".to_string()
        });
        return out;
    }
    out.push_str(&format!("  changes{marker}:\n"));
    for y in yaks {
        let mut what = match (y.added, y.moved) {
            (true, _) => format!("new, {}", y.status),
            (false, Some((from, to))) => format!("{from} -> {to}"),
            (false, None) => y.status.to_string(),
        };
        if y.edited {
            what.push_str(", edited");
        }
        if y.landed {
            what.push_str(", landed");
        }
        out.push_str(&format!("  {}  {}  [{what}]\n", y.id, y.title));
        for n in &y.notes {
            out.push_str(&format!(
                "    {}  {}  {}\n",
                n.ts,
                n.actor.as_deref().unwrap_or("-"),
                first_line(&n.text)
            ));
        }
        if let Some(v) = &y.needs {
            out.push_str(&format!("    needs: {v}\n"));
        }
    }
    out
}

/// JSON for `yaks changes --json`: one object per changed yak. `vs`/`base`
/// are as in `yaks sheds --json` `farm`. Empty when there is nothing to list.
pub fn changes_json(c: &Changes) -> Value {
    let vs = c.base.as_ref().map_or("checkout", |b| b.kind.as_str());
    let base = c.base.as_ref().map(|b| &b.sha);
    Value::Array(
        c.yaks
            .iter()
            .flatten()
            .map(|y| {
                json!({
                    "id": y.id,
                    "title": y.title,
                    "status": y.status,
                    "added": y.added,
                    "moved": y.moved.map(|(from, to)| json!({"from": from, "to": to})),
                    "notes": y.notes.iter()
                        .map(|n| json!({"ts": n.ts, "actor": n.actor, "text": n.text}))
                        .collect::<Vec<_>>(),
                    "needs": y.needs,
                    "edited": y.edited,
                    "landed": y.landed,
                    "vs": vs,
                    "base": base,
                })
            })
            .collect(),
    )
}

/// The one-line shed label: `who: a, b · shaving: id, id · active`, each
/// part only when it has something; `None` for an unlabeled, idle shed.
/// `idle` ([`Liveness`]) is silent — it is the common case, so it is never
/// noise on the line; `active`/`stale` show (and only ever appear alongside
/// `shaving:`, since both require a non-empty `in_progress`). Measured from
/// a merge-base it carries the `vs merge-base` marker.
pub fn label_line(l: &Shed) -> Option<String> {
    let mut parts = Vec::new();
    if !l.who.is_empty() {
        parts.push(format!("who: {}", l.who.join(", ")));
    }
    if !l.in_progress.is_empty() {
        parts.push(format!("shaving: {}", l.in_progress.join(", ")));
    }
    match l.liveness {
        Liveness::Active => parts.push("active".into()),
        Liveness::Stale => parts.push("stale".into()),
        Liveness::Idle => {}
    }
    (!parts.is_empty()).then(|| format!("{}{}", parts.join(" \u{b7} "), vs_marker(l)))
}

/// The shed's farm has something outstanding: a delta of its own that is not
/// all `landed`. What the TUI's Sheds tab counts and sorts by, and when
/// [`render`] itemizes the delta under the `farm:` line.
pub fn is_outstanding(l: &Shed) -> bool {
    match &l.farm {
        // `landed` is a subset of `touched`, so equal lengths = all landed.
        ShedFarm::Own(d) => !d.is_empty() && d.landed.len() != d.touched().len(),
        _ => false,
    }
}

/// The one-line summary of a shed's farm, the text after `farm: ` in
/// [`render`] (`no changes of its own`, `landed (2 yaks)`, `1 moved, 3 new
/// notes (1 landed)`, `farm unreadable: ...`), with its baseline marker.
/// Shared with the TUI's Sheds tab so the two never word it differently.
pub fn farm_line(l: &Shed) -> String {
    match &l.farm {
        ShedFarm::Own(d) if d.is_empty() && l.farm_base.is_some() => {
            format!("no changes of its own{}", vs_marker(l))
        }
        ShedFarm::Own(d) if d.is_empty() => "no differences (vs this checkout)".into(),
        // Every touched yak is landed: nothing outstanding to list.
        ShedFarm::Own(d) if !is_outstanding(l) => {
            let n = d.landed.len();
            let yak = if n == 1 { "yak" } else { "yaks" };
            format!("landed ({n} {yak}){}", vs_marker(l))
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
            if !d.edited.is_empty() {
                parts.push(format!("{} edited", d.edited.len()));
            }
            // A shed with some yaks landed and some still outstanding: the
            // summary keeps every yak's full counts (the totals since the
            // fork), but `render`'s itemized lines list only what is still
            // outstanding (per the spec: "lists the outstanding ones and
            // says how many are landed").
            let landed_note = if d.landed.is_empty() {
                String::new()
            } else {
                format!(" ({} landed)", d.landed.len())
            };
            format!("{}{landed_note}{}", parts.join(", "), vs_marker(l))
        }
        other => farm_message(other).unwrap_or_default(),
    }
}

/// The compact human table: one row per shed, then its label and farm detail.
pub fn render(sheds: &[Shed], orphaned: &[Pinned]) -> String {
    if sheds.is_empty() {
        return "No checkouts found.\n".into();
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
    // `*` marks the checkout you are in.
    let line = |mark: &str, cells: Vec<&str>, path: &str| {
        let mut s = format!("{mark} ");
        for (c, w) in cells.iter().zip(&widths) {
            s.push_str(&format!("{c:<w$}  "));
        }
        s.push_str(path);
        s.push('\n');
        s
    };
    let mut out = line(" ", heads.to_vec(), "PATH");
    for (l, r) in sheds.iter().zip(&rows) {
        out.push_str(&line(
            if l.current { "*" } else { " " },
            r.iter().map(String::as_str).collect(),
            &l.path.display().to_string(),
        ));
        let here = if l.current { " (here)" } else { "" };
        match &l.title {
            Some(t) => out.push_str(&format!("  name: {}{here} \u{b7} title: {t}\n", l.name())),
            None => out.push_str(&format!("  name: {}{here}\n", l.name())),
        }
        if let Some(e) = &l.error {
            out.push_str(&format!("  error: {e}\n"));
        }
        if let Some(label) = label_line(l) {
            out.push_str(&format!("  {label}\n"));
        }
        out.push_str(&format!("  farm: {}\n", farm_line(l)));
        match &l.farm {
            ShedFarm::Own(d) if is_outstanding(l) => {
                let outstanding = |id: &str| !d.landed.iter().any(|l| l == id);
                for id in d.added.iter().filter(|id| outstanding(id)) {
                    out.push_str(&format!("    + {id}\n"));
                }
                for m in d.moved.iter().filter(|m| outstanding(&m.id)) {
                    out.push_str(&format!("    {} {} -> {}\n", m.id, m.from, m.to));
                }
                for (id, n) in d.notes.iter().filter(|(id, _)| outstanding(id)) {
                    out.push_str(&format!("    {id} +{n} notes\n"));
                }
                for (id, v) in d.needs.iter().filter(|(id, _)| outstanding(id)) {
                    out.push_str(&format!("    {id} needs: {v}\n"));
                }
                for id in d.edited.iter().filter(|id| outstanding(id)) {
                    out.push_str(&format!("    {id} edited\n"));
                }
            }
            _ => {}
        }
    }
    // One line for the whole table, and only when it explains a `?`.
    if sheds
        .iter()
        .any(|l| !l.current && l.shallow_here && l.head.is_some() && l.ahead.is_none())
    {
        out.push_str(
            "note: history is shallow here, so some sheds cannot be compared exactly; \
             git fetch --unshallow fixes it\n",
        );
    }
    // Orphaned pins are not a `Shed` row (no checkout to measure: see
    // `orphaned_pins`), so they get one informational, text-only footer
    // instead, reusing exactly the `Err(...)` entries `yaks discover`'s PINS
    // section already shows — not a second "which pins are gone" check.
    if !orphaned.is_empty() {
        out.push_str(&format!(
            "\norphaned ({}): a pinned clone whose checkout could not be resolved \
             (most often a gone git dir — a finished thread)\n",
            orphaned.len()
        ));
        for p in orphaned {
            out.push_str(&format!(
                "  {}/{}: {}\n",
                p.dir,
                p.name,
                p.checkout.as_ref().unwrap_err()
            ));
        }
    }
    out
}

/// Pinned clones in `cwd`'s host whose checkout could not be resolved (a gone
/// git dir, a missing worktree, or similar — see [`pinned`]): what `render`'s
/// `orphaned` footer lists. Not a [`Shed`]: there is no checkout to measure
/// (no git state, no farm), so these are never in `discover`'s sheds array or
/// `sheds --json`, only in the text table's footer (yaks-9277).
///
/// `yaks sheds` itself uses [`discover_all`], which returns these from the same
/// pass as the sheds; this standalone form is for a caller that wants only them.
#[allow(dead_code)]
pub fn orphaned_pins(cwd: &Path) -> Vec<Pinned> {
    let Some(h) = host(cwd) else {
        return Vec::new();
    };
    pinned(&h)
        .into_iter()
        .filter(|p| p.checkout.is_err())
        .collect()
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

    /// One yak in the FLAT layout: `.yaks/<id>.md` with a `status:` field.
    fn task_file(root: &Path, status: &str, id: &str, extra_fm: &str, body: &str) {
        let dir = root.join(".yaks");
        fs::create_dir_all(&dir).unwrap();
        // What `yaks init` writes: a flat farm is recognised by its marker.
        fs::write(dir.join("schema"), format!("{}\n", store::SCHEMA)).unwrap();
        fs::write(
            dir.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntitle: {id} title\nstatus: {status}\ntype: task\n\
                 priority: 3\ncreated: '2026-01-01T00:00:00Z'\n\
                 updated: '2026-01-01T00:00:00Z'\n{extra_fm}---\n{body}\n"
            ),
        )
        .unwrap();
    }

    /// One yak in the OLD layout: `.yaks/<status>/<id>.md`, no `status:` field.
    fn old_task_file(root: &Path, status: &str, id: &str, extra_fm: &str, body: &str) {
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

    /// Change a flat yak's `status:` in place (how a move is written now).
    fn set_status(root: &Path, id: &str, from: &str, to: &str) {
        let f = root.join(format!(".yaks/{id}.md"));
        let text = fs::read_to_string(&f).unwrap();
        let old = format!("status: {from}\n");
        assert!(text.contains(&old), "{old} not in {text}");
        fs::write(&f, text.replacen(&old, &format!("status: {to}\n"), 1)).unwrap();
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

    /// Every checkout as `top` sees them, `top` itself included (`current`).
    fn all_sheds_of(top: &Path) -> Vec<Shed> {
        discover(top, &top.join(".yaks")).unwrap()
    }

    /// The OTHER checkouts as `top` sees them: most tests are about a row
    /// seen from somewhere else; the current row has its own tests.
    fn sheds_of(top: &Path) -> Vec<Shed> {
        let mut sheds = all_sheds_of(top);
        sheds.retain(|s| !s.current);
        sheds
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
        assert_eq!(render(&[], &[]), "No checkouts found.\n");
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
        set_status(&wt, "yaks-aaaa", "hairy", "shaving");
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
        let text = render(&sheds, &[]);
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
        assert!(render(&sheds, &[]).contains("(detached)"));
    }

    /// Move a yak to another status in `repo` and commit it.
    fn commit_move(repo: &Path, id: &str, from: &str, to: &str) {
        set_status(repo, id, from, to);
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
        let text = render(&sheds, &[]);
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
        let text = render(&sheds, &[]);
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
        let text = render(&sheds, &[]);
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
        // The shed's git dir sits in the primary's clone store and is pinned
        // there, like a Delta clone; it is a full clone, so it knows commits
        // the shallow primary does not.
        let shed = t.join(".delta/worktrees/xyz/repo");
        let gd = t.join(".delta/clones/xyz/repo.git");
        fs::create_dir_all(shed.parent().unwrap()).unwrap();
        fs::create_dir_all(gd.parent().unwrap()).unwrap();
        sh(
            &b,
            &[
                "clone",
                "-q",
                "--separate-git-dir",
                gd.to_str().unwrap(),
                &origin_url,
                shed.to_str().unwrap(),
            ],
        );
        sh(&shed, &["config", "core.worktree", shed.to_str().unwrap()]);
        sh(&shed, &["checkout", "-q", shed_at]);
        let head = sh(&t, &["rev-parse", "HEAD"]);
        pin(&t.join(".git"), "xyz", "repo", &head);
        (t, shed)
    }

    #[test]
    fn a_shallow_primary_cannot_place_a_shed_older_than_its_boundary() {
        let (t, shed) = shallow_primary_with_shed("shallow", "HEAD~2");
        let sheds = sheds_of(&t);
        assert_eq!(paths(&sheds), vec![shed]);
        let l = &sheds[0];
        assert_eq!((l.ahead, l.behind), (None, None));
        let text = render(&sheds, &[]);
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
        let text = render(&sheds, &[]);
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
        let text = render(&sheds, &[]);
        assert!(text.contains("+0 -0"), "{text}");
        assert!(!text.contains("history is shallow"), "{text}");
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

    /// Make a Delta-shaped clone of `host` (a git dir: the human's `.git` or
    /// a managed bare repo) the way Delta does: the checkout at
    /// `<checkouts>/<dir>/<name>`, its git dir at `<store>/<dir>/<name>.git`
    /// with `core.worktree` set and `objects/info/alternates` naming the
    /// host's objects, and the starting commit pinned in the host as
    /// `refs/delta/<dir>/<name>/<sha>`. Returns the checkout.
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
        pin(host, dir, name, &sh(&co, &["rev-parse", "HEAD"]));
        co
    }

    /// Pin `sha` in `host` as Delta does.
    fn pin(host: &Path, dir: &str, name: &str, sha: &str) {
        sh(
            host,
            &["update-ref", &format!("refs/delta/{dir}/{name}/{sha}"), sha],
        );
    }

    #[test]
    fn linked_layout_clones_find_each_other_the_primary_and_its_worktrees() {
        // The human's machine: the project was added from a folder, so the
        // host is the human's checkout and clones live under `.delta/`.
        let b = base("linked");
        let t = b.join("repo");
        repo_with_farm(&t);
        let host = t.join(".git");
        let store = t.join(".delta/clones");
        let co = t.join(".delta/worktrees");
        let c1 = delta_clone(&host, &store, &co, "d1", "repo");
        let c2 = delta_clone(&host, &store, &co, "d2", "repo");
        // A plain git worktree of the primary, outside the repo.
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "w", wt.to_str().unwrap()],
        );
        // A finished thread: pinned, its clone removed by Delta.
        pin(&host, "gone1", "repo", &sh(&t, &["rev-parse", "HEAD"]));

        // From a clone: the other clone, the primary, and the primary's
        // worktree (which `git worktree list` in a clone cannot show).
        let from_c1 = sheds_of(&c1);
        assert_eq!(paths(&from_c1), vec![t.clone(), c2.clone(), wt.clone()]);
        assert_eq!(shed_at(&from_c1, &c2).kind, ShedKind::Delta);
        assert_eq!(shed_at(&from_c1, &t).kind, ShedKind::Worktree);
        // From the primary: both clones and the worktree.
        let from_t = sheds_of(&t);
        assert_eq!(paths(&from_t), vec![c1.clone(), c2.clone(), wt.clone()]);
        // From the plain worktree: the same set, seen from there.
        assert_eq!(paths(&sheds_of(&wt)), vec![t.clone(), c1, c2]);

        // The gone pin is reported, never listed as a shed.
        let h = host_of(&t);
        let gone = pinned(&h).into_iter().find(|p| p.dir == "gone1").unwrap();
        assert!(gone.checkout.unwrap_err().starts_with("gone"));
    }

    fn host_of(t: &Path) -> Host {
        host(t).unwrap()
    }

    #[test]
    fn managed_layout_clones_find_each_other_whatever_their_names() {
        // A machine that only had the thread shared to it: the host is
        // Delta's managed BARE repo and clone names need not match (seen:
        // `j15r` beside `local_<uuid>`).
        let b = base("managed");
        let src = b.join("src");
        repo_with_farm(&src);
        let host = b.join("managed/repository.git");
        sh(
            &b,
            &[
                "clone",
                "-q",
                "--bare",
                src.to_str().unwrap(),
                host.to_str().unwrap(),
            ],
        );
        let root = b.join("worktrees");
        let c1 = delta_clone(&host, &root, &root, "m1", "local_u1");
        let c2 = delta_clone(&host, &root, &root, "m2", "proj");

        let from_c1 = sheds_of(&c1);
        assert_eq!(paths(&from_c1), vec![c2.clone()]);
        assert_eq!(from_c1[0].kind, ShedKind::Delta);
        assert_eq!(paths(&sheds_of(&c2)), vec![c1]);
        let h = host_of(&c2);
        assert!(h.bare && h.via_alternates);
        // The unrelated `src` repo is never seen: no pins point at it.
        assert!(!paths(&from_c1).contains(&src));
    }

    #[test]
    fn a_new_clone_is_a_shed_before_its_first_commit() {
        // Delta pins a clone's starting commit when it creates the clone
        // (observed: a probe worker that never committed had pins).
        let b = base("fresh");
        let t = b.join("repo");
        repo_with_farm(&t);
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "new",
            "repo",
        );
        assert_eq!(
            sh(&c, &["rev-parse", "HEAD"]),
            sh(&t, &["rev-parse", "HEAD"])
        );
        assert_eq!(paths(&sheds_of(&t)), vec![c]);
    }

    #[test]
    fn look_alikes_without_pins_are_never_sheds() {
        // Directory shapes alone prove nothing: an unpinned same-named clone
        // and a same-named clone of another repo are both ignored.
        let b = base("lookalike");
        let t = b.join("repo");
        repo_with_farm(&t);
        let unpinned = t.join(".delta/worktrees/zzz/repo");
        fs::create_dir_all(unpinned.parent().unwrap()).unwrap();
        sh(
            &b,
            &[
                "clone",
                "-q",
                t.to_str().unwrap(),
                unpinned.to_str().unwrap(),
            ],
        );
        let sib = b.join("other/x/repo");
        init_commit(&sib, Some(&url(&b, "r")));
        assert!(sheds_of(&t).is_empty());
    }

    #[test]
    fn found_by_both_sources_is_one_shed_of_kind_worktree() {
        let b = base("both");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        // A pin group whose clone store entry points at the same checkout.
        let gd = t.join(".delta/clones/xyz/repo.git");
        fs::create_dir_all(&gd).unwrap();
        fs::write(
            gd.join("config"),
            format!("[core]\n\tworktree = {}\n", wt.display()),
        )
        .unwrap();
        pin(
            &t.join(".git"),
            "xyz",
            "repo",
            &sh(&t, &["rev-parse", "HEAD"]),
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
        let text = render(&sheds, &[]);
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
        assert!(render(&sheds, &[]).contains("  error: "));
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
        let f = wt.join(".yaks/yaks-aaaa.md");
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
        let _ = (render(&sheds, &[]), to_json(&sheds));
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
            render(&sheds, &[]),
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
            text.contains("  who: lbl-1 \u{b7} shaving: yaks-aaaa \u{b7} active\n"),
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
            text.contains("  who: delta:Fix attribution \u{b7} shaving: yaks-aaaa \u{b7} active\n"),
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
        assert!(
            text.contains("  shaving: yaks-aaaa \u{b7} active\n"),
            "{text}"
        );
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
            text.contains(
                "  who: sheds-2, delta:Fix attribution \u{b7} shaving: yaks-aaaa \u{b7} active\n"
            ),
            "{text}"
        );
        assert_eq!(json[0]["who"], json!(["sheds-2", "delta:Fix attribution"]));
    }

    #[test]
    fn a_yak_shorn_within_the_shed_is_not_in_progress() {
        let (t, wt) = claimed_repo_with_shed("label-shorn");
        shed_notes_on_aaaa(&wt, &[("2026-03-02T00:00:00Z", Some("w1"), "working")]);
        // `yaks-bbbb`: claimed, worked and shorn inside the shed.
        fs::remove_file(wt.join(".yaks/yaks-bbbb.md")).unwrap();
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
            text.contains("  who: w1, w2 \u{b7} shaving: yaks-aaaa \u{b7} active\n"),
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
            render(&sheds, &[]).contains("  who: w-1 \u{b7} shaving: yaks-aaaa \u{b7} active\n"),
            "{}",
            render(&sheds, &[])
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
            render(&sheds, &[]).contains("farm: no changes of its own\n"),
            "{}",
            render(&sheds, &[])
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
        // history); the fork point is the baseline from any viewer.
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

    // -- landed (yaks-ed5e): a shed whose changes are all already in the
    // reference farm reads `landed`, not as outstanding work -----------------

    /// The exact motivating case: a shed's commit was squashed into the
    /// reference under a different sha, so `ahead`/`behind` (by commit) does
    /// not catch it, but its farm content is now identical.
    #[test]
    fn a_squashed_shed_reads_landed_once_the_reference_has_everything_it_has() {
        let (t, coord) = coordinator_with_own_notes("landed-squash");
        let w = worker_of(&t, &coord, "w1");
        commit_notes_on_aaaa(
            &w,
            &[
                ("2026-03-02T00:00:00Z", Some("coordx"), "coordinator note"),
                ("2026-03-03T00:00:00Z", Some("w-1"), "worker note"),
            ],
        );
        sh(&t, &["merge", "--squash", "-q", "w1"]);
        sh(&t, &["commit", "-q", "-m", "squash"]);
        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &w);
        let ShedFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert_eq!(d.landed, ["yaks-aaaa"]);
        let text = render(&sheds, &[]);
        assert!(text.contains("farm: landed (1 yak)"), "{text}");
        assert!(!text.contains("+1 notes"), "no itemized line: {text}");
        let j = json_of(&sheds, &w);
        assert_eq!(j["farm"]["landed"], json!(["yaks-aaaa"]));
    }

    /// A shed with one note the reference lacks is NOT landed, even though
    /// the reference has moved on in other ways.
    #[test]
    fn a_shed_with_an_extra_note_the_reference_lacks_is_not_landed() {
        let (t, wt) = claimed_repo_with_shed("landed-extra-note");
        shed_notes_on_aaaa(
            &wt,
            &[("2026-03-02T00:00:00Z", Some("w-1"), "still working")],
        );
        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &wt);
        let ShedFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert!(d.landed.is_empty(), "{d:?}");
        let text = render(&sheds, &[]);
        // Exact line: no `(N landed)` suffix (checking the literal "landed"
        // substring would also match this test's own temp-dir path).
        assert!(text.contains("  farm: 1 new notes\n"), "{text}");
        assert!(text.contains("    yaks-aaaa +1 notes\n"), "{text}");
    }

    /// The SECOND real-machine case (the 2026-10-10 note): a shed's copy of a
    /// yak is an EARLIER state of the reference's (here: `shaving` with no
    /// notes, while the reference already sheared it with notes). The shed
    /// did touch the yak relative to its OWN fork (it is new there), but the
    /// reference already has everything it has and more: landed.
    #[test]
    fn a_shed_whose_copy_is_superseded_by_the_reference_reads_landed() {
        let b = base("landed-superseded");
        let t = b.join("repo");
        // A real initial commit (so the shed's fork point has a readable
        // farm WITHOUT `yaks-dddd`): otherwise there is no base to read and
        // `inspect` falls back to comparing against this checkout's CURRENT
        // farm, which would already include the finished `yaks-dddd` below
        // and make it look like a plain backward `moved`, not `added`.
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        // The shed creates the yak itself, uncommitted, still `shaving` with
        // no notes (a Delta-applied copy of work the primary has since
        // finished, per the real case).
        task_file(&wt, "shaving", "yaks-dddd", "", "new");
        // The primary finishes and shears it, with notes, in its own commit.
        task_file(
            &t,
            "shorn",
            "yaks-dddd",
            "",
            &notes(
                "new",
                &[
                    (
                        "2026-03-01T00:00:00Z",
                        Some("coord"),
                        "moved: hairy -> shaving",
                    ),
                    ("2026-03-02T00:00:00Z", Some("coord"), "done"),
                    (
                        "2026-03-03T00:00:00Z",
                        Some("coord"),
                        "moved: shaving -> shorn",
                    ),
                ],
            ),
        );
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "finish dddd"]);

        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &wt);
        let ShedFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert_eq!(
            d.added,
            ["yaks-dddd"],
            "new relative to the shed's own fork"
        );
        assert_eq!(d.landed, ["yaks-dddd"]);
        let text = render(&sheds, &[]);
        assert!(text.contains("farm: landed (1 yak)"), "{text}");
    }

    /// A shed with some yaks landed and some still outstanding: the summary
    /// keeps the full counts, adds `(K landed)`, and the itemized lines list
    /// only what is still outstanding.
    #[test]
    fn a_shed_with_some_yaks_landed_and_some_outstanding_lists_only_the_outstanding() {
        let (t, wt) = claimed_repo_with_shed("landed-mixed");
        // The shed: an extra note on the already-claimed yaks-aaaa (stays
        // outstanding, the reference never sees it), and yaks-bbbb moved to
        // shaving with a note (will land: the primary makes the same move,
        // with its own actor/text — note COUNT is the rule, see `yak_changes`).
        shed_notes_on_aaaa(
            &wt,
            &[("2026-03-02T00:00:00Z", Some("w-1"), "still working")],
        );
        fs::remove_file(wt.join(".yaks/yaks-bbbb.md")).unwrap();
        task_file(
            &wt,
            "shaving",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[(
                    "2026-03-03T00:00:00Z",
                    Some("w-1"),
                    "moved: hairy -> shaving",
                )],
            ),
        );
        // The primary moves bbbb the same way, independently, after the shed forked.
        fs::remove_file(t.join(".yaks/yaks-bbbb.md")).unwrap();
        task_file(
            &t,
            "shaving",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[(
                    "2026-03-04T00:00:00Z",
                    Some("coord"),
                    "moved: hairy -> shaving",
                )],
            ),
        );
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "primary also moves bbbb"]);

        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &wt);
        let ShedFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert_eq!(d.landed, ["yaks-bbbb"]);
        assert_eq!(d.touched().len(), 2, "{d:?}");
        let text = render(&sheds, &[]);
        assert!(
            text.contains("farm: 1 moved, 2 new notes (1 landed)"),
            "{text}"
        );
        assert!(text.contains("    yaks-aaaa +1 notes\n"), "{text}");
        // `yaks-bbbb` still appears on the `shaving:` label line (it IS
        // being worked on there); only the itemized farm-detail lines
        // (4-space indent, `moved`/`notes`) must drop it.
        assert!(
            !text.contains("    yaks-bbbb"),
            "no itemized line for the landed yak: {text}"
        );
    }

    // -- liveness (yaks-9277): active/idle/stale, orthogonal to `landed` -----

    /// Sets every entry under `dir` (files and directories, recursively, to
    /// match what [`newest_mtime`] reads) to `age` ago, so a test can make a
    /// shed's farm activity read as old without waiting for real time to
    /// pass.
    fn backdate(dir: &Path, age: Duration) {
        let when = SystemTime::now() - age;
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            for entry in fs::read_dir(&d).unwrap() {
                let entry = entry.unwrap();
                let p = entry.path();
                let _ = fs::File::open(&p).and_then(|f| f.set_modified(when));
                if entry.file_type().unwrap().is_dir() {
                    stack.push(p);
                }
            }
            let _ = fs::File::open(&d).and_then(|f| f.set_modified(when));
        }
    }

    #[test]
    fn liveness_is_active_with_a_claim_and_recent_activity() {
        assert_eq!(
            liveness(&["yaks-aaaa".to_string()], Some(SystemTime::now())),
            Liveness::Active
        );
    }

    #[test]
    fn liveness_is_stale_with_a_claim_and_old_activity() {
        let old = SystemTime::now() - STALE_AFTER - Duration::from_secs(1);
        assert_eq!(
            liveness(&["yaks-aaaa".to_string()], Some(old)),
            Liveness::Stale
        );
    }

    #[test]
    fn liveness_is_idle_with_no_claim_or_with_unknown_activity() {
        // No claim at all, even with fresh activity.
        assert_eq!(liveness(&[], Some(SystemTime::now())), Liveness::Idle);
        // A claim, but activity that cannot be told: no signal is not
        // evidence of abandonment, so this is `idle`, never `stale`.
        assert_eq!(liveness(&["yaks-aaaa".to_string()], None), Liveness::Idle);
    }

    #[test]
    fn the_primary_gets_a_liveness_value_unlike_landed() {
        // Unlike `farm.landed` (yaks-ed5e, never set for the primary's own
        // row since comparing it with itself is vacuous), liveness is not a
        // comparison to anything, so the primary gets one too.
        let b = base("primary-liveness");
        let t = b.join("repo");
        repo_with_farm(&t);
        // The primary claims a yak itself, uncommitted, with its own note.
        fs::remove_file(t.join(".yaks/yaks-aaaa.md")).unwrap();
        task_file(
            &t,
            "shaving",
            "yaks-aaaa",
            "",
            &notes(
                "first",
                &[(
                    "2026-03-01T00:00:00Z",
                    Some("coord"),
                    "moved: hairy -> shaving",
                )],
            ),
        );
        let sheds = all_sheds_of(&t);
        let primary = sheds.iter().find(|s| s.primary).unwrap();
        assert!(primary.current);
        assert_eq!(primary.in_progress, strs(&["yaks-aaaa"]));
        assert_eq!(primary.liveness, Liveness::Active);
        let ShedFarm::Own(d) = &primary.farm else {
            panic!("expected own farm, got {:?}", primary.farm)
        };
        assert!(d.landed.is_empty(), "never set for the primary's own row");
        let j = to_json(&sheds);
        let jp = j
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["primary"] == true)
            .unwrap();
        assert_eq!(jp["liveness"], "active");
    }

    /// The real j15r case from this yak's own notes: two clones agree on HEAD
    /// and on their uncommitted farm files, and the primary already made the
    /// same move independently (so the shed reads `landed`, yaks-ed5e) --
    /// `landed` and liveness are orthogonal (decision A), so a shed can show
    /// BOTH at once: nothing outstanding to act on, but nobody has touched
    /// the shed itself in a while.
    #[test]
    fn a_shed_that_is_landed_and_stale_shows_both_independently() {
        let (t, wt) = claimed_repo_with_shed("landed-stale");
        // The shed claims yaks-bbbb, with its own note, uncommitted.
        fs::remove_file(wt.join(".yaks/yaks-bbbb.md")).unwrap();
        task_file(
            &wt,
            "shaving",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[(
                    "2026-03-03T00:00:00Z",
                    Some("w-1"),
                    "moved: hairy -> shaving",
                )],
            ),
        );
        // The primary makes the identical move independently, after the fork.
        fs::remove_file(t.join(".yaks/yaks-bbbb.md")).unwrap();
        task_file(
            &t,
            "shaving",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[(
                    "2026-03-04T00:00:00Z",
                    Some("coord"),
                    "moved: hairy -> shaving",
                )],
            ),
        );
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "primary also moves bbbb"]);
        // Nobody has touched the shed's farm files in a long time.
        backdate(&wt.join(".yaks"), STALE_AFTER + Duration::from_secs(60));

        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &wt);
        assert_eq!(l.in_progress, strs(&["yaks-bbbb"]));
        assert_eq!(
            l.liveness,
            Liveness::Stale,
            "activity: {:?}",
            l.farm_activity
        );
        let ShedFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert_eq!(d.landed, ["yaks-bbbb"]);
        let text = render(&sheds, &[]);
        assert!(text.contains("farm: landed (1 yak)"), "{text}");
        assert!(text.contains("shaving: yaks-bbbb \u{b7} stale"), "{text}");
        let j = json_of(&sheds, &wt);
        assert_eq!(j["liveness"], "stale");
        assert_eq!(j["farm"]["landed"], json!(["yaks-bbbb"]));
    }

    // -- orphaned pins (yaks-9277): a gone Delta clone, listed but never a
    // `Shed` row ---------------------------------------------------------

    #[test]
    fn render_lists_a_gone_pin_in_a_text_only_orphaned_footer() {
        let b = base("orphan");
        let t = b.join("repo");
        repo_with_farm(&t);
        let host = t.join(".git");
        let store = t.join(".delta/clones");
        let co = t.join(".delta/worktrees");
        // A live clone (so the host's clone store exists and is known).
        let _c1 = delta_clone(&host, &store, &co, "d1", "repo");
        // A finished thread: pinned, its clone directory removed by Delta.
        pin(&host, "gone1", "repo", &sh(&t, &["rev-parse", "HEAD"]));

        let sheds = sheds_of(&t);
        assert!(
            sheds.iter().all(|s| s.dir.as_deref() != Some("gone1")),
            "a gone pin is never a Shed row"
        );
        let orphaned = orphaned_pins(&t);
        assert_eq!(orphaned.len(), 1);
        assert_eq!(
            (orphaned[0].dir.as_str(), orphaned[0].name.as_str()),
            ("gone1", "repo")
        );
        assert!(orphaned[0].checkout.is_err());

        let text = render(&sheds, &orphaned);
        assert!(text.contains("orphaned (1):"), "{text}");
        assert!(text.contains("gone1/repo"), "{text}");

        // The normal table is unaffected when there is nothing orphaned.
        assert!(!render(&sheds, &[]).contains("orphaned"));
    }

    /// No visible primary and no usable `origin` ref: `reference_tasks`
    /// (and so `landed`) is `None`/empty, never a guess. Exercised directly
    /// against the function `discover` relies on: a Delta-managed repo with
    /// no visible primary (a bare `local`, see the module docs) falls
    /// through exactly this path.
    #[test]
    fn landed_is_empty_with_no_visible_primary_and_no_usable_origin_ref() {
        let b = base("landed-no-reference");
        let t = b.join("repo");
        repo_with_farm(&t); // no `origin` remote configured at all
        assert_eq!(reference_tasks(None, &t, &t.join(".yaks")), None);

        // And a shed with a real change still reports no `landed` for it.
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        task_file(&wt, "hairy", "yaks-dddd", "", "new");
        let base_tasks = store::load(&t.join(".yaks"), &EVERY).unwrap();
        let theirs = store::load(&wt.join(".yaks"), &EVERY).unwrap();
        assert_eq!(landed_ids(&base_tasks, &theirs, None), Vec::<String>::new());
    }

    /// Two non-primary viewers (siblings, neither the primary) agree on
    /// whether a THIRD shed is landed: the reference is read the same way
    /// regardless of who asks.
    #[test]
    fn two_non_primary_viewers_agree_on_whether_a_third_shed_is_landed() {
        let (t, w) = claimed_repo_with_shed("landed-viewers");
        // `w`'s change lands: the primary makes the identical move after.
        fs::remove_file(w.join(".yaks/yaks-bbbb.md")).unwrap();
        task_file(
            &w,
            "shaving",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[(
                    "2026-03-02T00:00:00Z",
                    Some("w-1"),
                    "moved: hairy -> shaving",
                )],
            ),
        );
        fs::remove_file(t.join(".yaks/yaks-bbbb.md")).unwrap();
        task_file(
            &t,
            "shaving",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[(
                    "2026-03-03T00:00:00Z",
                    Some("coord"),
                    "moved: hairy -> shaving",
                )],
            ),
        );
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "primary also moves bbbb"]);
        // Two sibling worktrees, neither the primary, forked after that commit.
        let sib1 = t.parent().unwrap().join("sib1");
        let sib2 = t.parent().unwrap().join("sib2");
        sh(
            &t,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "sib1",
                sib1.to_str().unwrap(),
            ],
        );
        sh(
            &t,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "sib2",
                sib2.to_str().unwrap(),
            ],
        );
        for viewer in [&sib1, &sib2] {
            let sheds = all_sheds_of(viewer);
            let l = shed_at(&sheds, &w);
            let ShedFarm::Own(d) = &l.farm else {
                panic!("expected own farm, got {:?}", l.farm)
            };
            assert_eq!(d.landed, ["yaks-bbbb"], "from {}", viewer.display());
        }
    }

    /// `yaks changes <shed>` marks each yak `landed` when it is: text (the
    /// bracket list, like `edited`) and JSON.
    #[test]
    fn changes_marks_a_landed_yak_text_and_json() {
        let (t, wt) = claimed_repo_with_shed("landed-changes");
        fs::remove_file(wt.join(".yaks/yaks-bbbb.md")).unwrap();
        task_file(
            &wt,
            "shaving",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[(
                    "2026-03-02T00:00:00Z",
                    Some("w-1"),
                    "moved: hairy -> shaving",
                )],
            ),
        );
        shed_notes_on_aaaa(
            &wt,
            &[("2026-03-02T00:00:00Z", Some("w-1"), "still working")],
        );
        // The primary makes bbbb's move too, independently; aaaa's extra note
        // stays outstanding.
        fs::remove_file(t.join(".yaks/yaks-bbbb.md")).unwrap();
        task_file(
            &t,
            "shaving",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[(
                    "2026-03-03T00:00:00Z",
                    Some("coord"),
                    "moved: hairy -> shaving",
                )],
            ),
        );
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "primary also moves bbbb"]);

        let c = changes_of(&t, "shed");
        let ys = c.yaks.as_ref().unwrap();
        let landed: Vec<(&str, bool)> = ys.iter().map(|y| (y.id.as_str(), y.landed)).collect();
        assert_eq!(
            landed,
            [("yaks-aaaa", false), ("yaks-bbbb", true)],
            "{ys:?}"
        );
        let out = render_changes(&c);
        assert!(out.contains("[hairy -> shaving, landed]"), "{out}");
        assert!(!out.contains("[shaving, landed]"), "{out}");
        let j = changes_json(&c);
        assert_eq!(j[0]["landed"], false);
        assert_eq!(j[1]["landed"], true);
    }

    /// The pure building blocks, directly: `outstanding_ids` excludes a
    /// backward move (superseded) but includes a forward one, extra notes, a
    /// new `needs:`, an edit, or a brand-new yak; `landed_ids` is then
    /// `touched - outstanding`.
    #[test]
    fn outstanding_ids_is_direction_aware() {
        let b = base("outstanding-ids");
        let (reference, theirs) = (b.join("reference"), b.join("theirs"));
        task_file(&reference, "shorn", "yaks-back", "", "x");
        task_file(&theirs, "shaving", "yaks-back", "", "x"); // superseded: behind
        task_file(&reference, "hairy", "yaks-fwd", "", "x");
        task_file(&theirs, "shaving", "yaks-fwd", "", "x"); // genuinely ahead
        task_file(&reference, "hairy", "yaks-note", "", "x");
        task_file(
            &theirs,
            "hairy",
            "yaks-note",
            "",
            &notes("x", &[("2026-02-02T00:00:00Z", Some("w"), "extra")]),
        );
        let load = |root: &Path| store::load(&root.join(".yaks"), &EVERY).unwrap();
        let (reference, theirs) = (load(&reference), load(&theirs));

        let outstanding = outstanding_ids(&reference, &theirs);
        assert_eq!(
            outstanding,
            BTreeSet::from(["yaks-fwd".to_string(), "yaks-note".to_string()])
        );
        // `landed_ids` relative to an empty base: every id is "touched" (all
        // new), so landed is exactly what's left once outstanding is removed.
        let empty: Vec<crate::model::Task> = Vec::new();
        let landed = landed_ids(&empty, &theirs, Some(&reference));
        assert_eq!(landed, ["yaks-back".to_string()]);
    }

    /// `outstanding_ids` contains `yaks-ask` for a shed copy that asks
    /// (`needs: human` plus the ask entry) against a reference copy built by
    /// the caller: `(status, front matter, note texts after the ask)`, or
    /// `None` for a reference that never saw the ask.
    fn outstanding_after_ask(tag: &str, reference: Option<(&str, &str, &[&str])>) -> bool {
        let ask = ("2026-02-01T00:00:00Z", Some("w"), "asked: needs human");
        let b = base(tag);
        let (r, t) = (b.join("reference"), b.join("theirs"));
        task_file(
            &t,
            "shaving",
            "yaks-ask",
            "needs: human\n",
            &notes("x", &[ask]),
        );
        match reference {
            Some((status, fm, after)) => {
                let mut entries = vec![ask];
                entries.extend(
                    after
                        .iter()
                        .map(|text| ("2026-02-02T00:00:00Z", Some("coord"), *text)),
                );
                task_file(&r, status, "yaks-ask", fm, &notes("x", &entries));
            }
            None => task_file(&r, "shaving", "yaks-ask", "", "x"),
        }
        let load = |root: &Path| store::load(&root.join(".yaks"), &EVERY).unwrap();
        outstanding_ids(&load(&r), &load(&t)).contains("yaks-ask")
    }

    /// yaks-8814: a shed's `needs:` that the reference already resolved is
    /// superseded, not outstanding.
    #[test]
    fn outstanding_ids_supersedes_a_needs_the_reference_resolved() {
        // (a) the land-1 case: ask, answer, pickup, `needs:` cleared.
        assert!(!outstanding_after_ask(
            "needs-resolved",
            Some(("shaving", "", &["answered: yes", "picked up"])),
        ));
        // (b) the reference shipped it (shorn, no `needs:`), ask present.
        assert!(!outstanding_after_ask(
            "needs-shorn",
            Some(("shorn", "", &["moved: shaving -> shorn"])),
        ));
        // (c) the reference never saw the ask: still outstanding.
        assert!(outstanding_after_ask("needs-unseen", None));
        // (d) the reference has `needs: human` too: unchanged (not a new
        // `needs:`, so nothing outstanding on that account).
        assert!(!outstanding_after_ask(
            "needs-same",
            Some(("shaving", "needs: human\n", &[])),
        ));
        // The ask is present but nothing followed it and `needs:` is
        // cleared: nothing resolved it, so still outstanding.
        assert!(outstanding_after_ask(
            "needs-no-follow-up",
            Some(("shaving", "", &[])),
        ));
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

    /// What a shed that merged the viewer's work after forking must show from
    /// EVERY viewer (yaks-e9bb): measured from its fork point, so what it
    /// merged in is credited to it like anything else it holds, exact (no `vs`
    /// marker), `farm.vs` = `fork`.
    fn assert_since_the_fork(t: &Path, w: &Path, fork_sha: &str) {
        let sheds = sheds_of(t);
        let l = shed_at(&sheds, w);
        assert_eq!(
            l.farm_base,
            Some(FarmBase {
                kind: BaseKind::Fork,
                sha: fork_sha.to_string()
            })
        );
        assert_eq!(
            l.who,
            strs(&["oldx", "new-1"]),
            "the merged note is its own"
        );
        assert_eq!(l.in_progress, strs(&["yaks-aaaa"]));
        let ShedFarm::Own(d) = &l.farm else {
            panic!("expected own farm, got {:?}", l.farm)
        };
        assert_eq!(d.notes, vec![("yaks-aaaa".to_string(), 2)]);
        assert!(d.added.is_empty() && d.moved.is_empty(), "{d:?}");
        let text = render(&sheds, &[]);
        assert!(
            text.contains("  who: oldx, new-1 \u{b7} shaving: yaks-aaaa \u{b7} active\n"),
            "{text}"
        );
        // (A primary without an `origin` ref says `(vs fork)`, see below.)
        assert!(
            !text.contains("(vs merge-base)") && !text.contains("(vs this checkout)"),
            "the fork point is exact: {text}"
        );
        let jw = json_of(&sheds, w);
        assert_eq!(jw["farm"]["vs"], "fork");
        assert_eq!(jw["farm"]["base"], fork_sha);
        assert_eq!(jw["who"], json!(["oldx", "new-1"]));
    }

    #[test]
    fn a_shed_the_viewer_merged_still_shows_everything_since_its_fork() {
        let (t, w) = claimed_repo_with_shed("fork-viewer-merged");
        let fork = sh(&w, &["rev-parse", "--short", "HEAD"]);
        let old = ("2026-03-02T00:00:00Z", Some("oldx"), "old note");
        commit_notes_on_aaaa(&w, &[old]);
        // The viewer merges the shed (a real merge commit), then the shed
        // carries on.
        sh(&t, &["merge", "--no-ff", "-q", "-m", "land", "shed"]);
        commit_notes_on_aaaa(
            &w,
            &[old, ("2026-03-03T00:00:00Z", Some("new-1"), "new note")],
        );
        assert_since_the_fork(&t, &w, &fork);
        // ahead/behind are untouched: still relative to the viewer's HEAD.
        let sheds = sheds_of(&t);
        assert_eq!(
            (shed_at(&sheds, &w).ahead, shed_at(&sheds, &w).behind),
            (Some(1), Some(1))
        );
    }

    #[test]
    fn a_shed_that_merged_the_viewer_still_shows_everything_since_its_fork() {
        let (t, w) = claimed_repo_with_shed("fork-shed-merged");
        let fork = sh(&w, &["rev-parse", "--short", "HEAD"]);
        // The viewer moves on (a note by `oldx`) and the shed pulls it in.
        let old = ("2026-03-02T00:00:00Z", Some("oldx"), "viewer note");
        commit_notes_on_aaaa(&t, &[old]);
        let branch = sh(&t, &["symbolic-ref", "--short", "HEAD"]);
        sh(&w, &["merge", "--no-ff", "-q", "-m", "sync", &branch]);
        commit_notes_on_aaaa(
            &w,
            &[old, ("2026-03-03T00:00:00Z", Some("new-1"), "new note")],
        );
        assert_since_the_fork(&t, &w, &fork);
    }

    #[test]
    fn a_shed_that_merged_the_viewers_main_reads_the_same_from_every_viewer() {
        // `w` forks from F, then merges the PRIMARY's main (a note by
        // `oldx`), then adds its own. Seen from the primary the merge-base is
        // main's tip, seen from a sibling shed it is F: the baseline, the
        // changes and the label must not depend on which.
        let (t, w) = claimed_repo_with_shed("fork-two-viewers");
        let fork = sh(&w, &["rev-parse", "--short", "HEAD"]);
        let sibling = t.parent().unwrap().join("sibling");
        sh(
            &t,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "sibling",
                sibling.to_str().unwrap(),
                &fork,
            ],
        );
        let old = ("2026-03-02T00:00:00Z", Some("oldx"), "viewer note");
        commit_notes_on_aaaa(&t, &[old]);
        let branch = sh(&t, &["symbolic-ref", "--short", "HEAD"]);
        sh(&w, &["merge", "--no-ff", "-q", "-m", "sync", &branch]);
        commit_notes_on_aaaa(
            &w,
            &[old, ("2026-03-03T00:00:00Z", Some("new-1"), "new note")],
        );
        // From the primary, and from the sibling (whose merge-base with the
        // shed is the fork point itself).
        assert_since_the_fork(&t, &w, &fork);
        assert_since_the_fork(&sibling, &w, &fork);
        let (from_t, from_sibling) = (sheds_of(&t), sheds_of(&sibling));
        assert_eq!(shed_at(&from_t, &w).farm, shed_at(&from_sibling, &w).farm);
        assert_eq!(
            json_of(&from_t, &w)["farm"],
            json_of(&from_sibling, &w)["farm"]
        );
    }

    #[test]
    fn a_delta_clone_is_measured_from_its_first_reflog_entry_from_every_viewer() {
        // The clone's first reflog entry is the HEAD it was cloned at (F);
        // it then merges the primary's moved-on main. A worktree sibling and
        // the primary itself must agree on F.
        let b = base("fork-clone");
        let t = b.join("repo");
        repo_with_farm(&t);
        let fork = sh(&t, &["rev-parse", "--short", "HEAD"]);
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "d1",
            "repo",
        );
        let sibling = b.join("sibling");
        sh(
            &t,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "sib",
                sibling.to_str().unwrap(),
            ],
        );
        task_file(
            &t,
            "hairy",
            "yaks-dddd",
            "",
            &notes(
                "main's own",
                &[("2026-03-02T00:00:00Z", Some("viewx"), "n")],
            ),
        );
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "main moves on"]);
        let branch = sh(&t, &["symbolic-ref", "--short", "HEAD"]);
        // The clone merges the primary's main (the clone's `origin`).
        sh(&c, &["fetch", "-q", "origin", &branch]);
        sh(&c, &["merge", "--no-ff", "-q", "-m", "sync", "FETCH_HEAD"]);
        task_file(&c, "hairy", "yaks-eeee", "", "own");
        sh(&c, &["add", "-A"]);
        sh(&c, &["commit", "-q", "-m", "own"]);
        // Delta pins the clone's commits in the host as it goes, which is what
        // lets every viewer read the clone's history.
        let head = sh(&c, &["rev-parse", "HEAD"]);
        sh(
            &t,
            &[
                "fetch",
                "-q",
                c.to_str().unwrap(),
                &format!("{head}:refs/delta/d1/repo/{head}"),
            ],
        );
        for viewer in [&t, &sibling] {
            let sheds = sheds_of(viewer);
            let l = shed_at(&sheds, &c);
            assert_eq!(
                l.farm_base,
                Some(FarmBase {
                    kind: BaseKind::Fork,
                    sha: fork.clone()
                }),
                "viewer {}",
                viewer.display()
            );
            let ShedFarm::Own(d) = &l.farm else {
                panic!("expected own farm, got {:?}", l.farm)
            };
            assert_eq!(
                d.added,
                vec!["yaks-dddd".to_string(), "yaks-eeee".to_string()],
                "viewer {}",
                viewer.display()
            );
        }
    }

    #[test]
    fn a_shed_keeps_its_fork_point_even_when_the_viewer_moved_on() {
        let (t, w) = claimed_repo_with_shed("fork-viewer-moved");
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
        let text = render(&sheds, &[]);
        assert!(
            text.contains(
                "  who: coordx, w-1 \u{b7} shaving: yaks-aaaa \u{b7} active (vs merge-base)\n"
            ),
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

    // -- naming and resolving a shed -------------------------------------------

    #[test]
    fn sheds_carry_their_dir_id_and_the_primary_is_main() {
        let b = base("names");
        let t = b.join("repo");
        repo_with_farm(&t);
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "abc123",
            "repo",
        );
        let from_c = sheds_of(&c);
        let main = shed_at(&from_c, &t);
        assert!(main.primary && main.name() == "main");
        let from_t = sheds_of(&t);
        let clone = shed_at(&from_t, &c);
        assert_eq!(clone.dir.as_deref(), Some("abc123"));
        assert!(!clone.primary);
        assert_eq!(clone.name(), "abc123");
        let j = to_json(&from_t);
        assert_eq!(j[0]["dir"], "abc123");
        assert_eq!(j[0]["name"], "abc123");
        assert!(render(&from_t, &[]).contains("  name: abc123\n"));
    }

    #[test]
    fn resolve_by_path_dir_branch_actor_main_and_substring() {
        let (t, wt) = claimed_repo_with_shed("resolve");
        shed_notes_on_aaaa(&wt, &[("2026-03-03T00:00:00Z", Some("sheds-2"), "hi")]);
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "q7zz",
            "repo",
        );
        let from_c = sheds_of(&c);
        let p = |s: &Shed| s.path.clone();
        assert_eq!(p(resolve(&from_c, "main").unwrap()), t);
        assert_eq!(p(resolve(&from_c, wt.to_str().unwrap()).unwrap()), wt);
        assert_eq!(p(resolve(&from_c, "sheds-2").unwrap()), wt);
        // `main` is the primary even though the clone is on a branch `main`.
        assert_eq!(shed_at(&from_c, &t).branch.as_deref(), Some("main"));
        let from_wt = sheds_of(&wt);
        assert_eq!(shed_at(&from_wt, &c).branch.as_deref(), Some("main"));
        assert_eq!(p(resolve(&from_wt, "main").unwrap()), t);
        let from_t = sheds_of(&t);
        // From the primary itself `main` is still the primary (not the clone
        // that is on a branch called `main`), see the selectors test below.
        assert_eq!(shed_at(&from_t, &c).branch.as_deref(), Some("main"));
        assert_eq!(p(resolve(&all_sheds_of(&t), "main").unwrap()), t);
        assert_eq!(p(resolve(&from_t, "q7zz").unwrap()), c);
        assert_eq!(p(resolve(&from_t, "q7").unwrap()), c, "unique substring");
        let branch = shed_at(&from_t, &wt).branch.clone().unwrap();
        assert_eq!(p(resolve(&from_t, &branch).unwrap()), wt);
        // No match lists every shed; ambiguity lists the candidates.
        let e = resolve(&from_t, "nope").unwrap_err();
        assert!(
            e.contains("no shed matches `nope`") && e.contains("q7zz"),
            "{e}"
        );
        let e = resolve(&from_t, "/").unwrap_err();
        assert!(e.contains("names 2 sheds"), "{e}");
    }

    // -- the current shed, and the primary's origin/main (yaks-ad6d) ----------

    /// A linked-layout fixture: the primary `t`, a plain git worktree `wt`
    /// and a Delta clone `c`, each with a farm change of its own.
    fn three_checkouts(tag: &str) -> (PathBuf, PathBuf, PathBuf) {
        let (t, wt) = claimed_repo_with_shed(tag);
        shed_notes_on_aaaa(&wt, &[("2026-03-03T00:00:00Z", Some("w-1"), "wt note")]);
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "q7zz",
            "repo",
        );
        task_file(&c, "hairy", "yaks-cccc", "", "clone yak");
        (t, wt, c)
    }

    #[test]
    fn every_viewer_lists_every_checkout_and_marks_only_itself() {
        let (t, wt, c) = three_checkouts("three-viewers");
        let every = [&t, &wt, &c].map(|v| all_sheds_of(v));
        for (viewer, sheds) in [&t, &wt, &c].into_iter().zip(&every) {
            assert_eq!(sheds.len(), 3, "{}", render(sheds, &[]));
            let here: Vec<&Path> = sheds
                .iter()
                .filter(|s| s.current)
                .map(|s| s.path.as_path())
                .collect();
            assert_eq!(here, [viewer.as_path()], "exactly the viewer is current");
            let j = to_json(sheds);
            let flags: Vec<bool> = j
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x["current"].as_bool().unwrap())
                .collect();
            assert_eq!(flags.iter().filter(|f| **f).count(), 1);
        }
        // The same checkouts, and the same answer about each, from anywhere:
        // `cwd` only picks which row is current.
        for other in &every[1..] {
            assert_eq!(paths(&every[0]), paths(other));
            for (a, b) in every[0].iter().zip(other) {
                assert_eq!(a.farm, b.farm, "{}", a.path.display());
                assert_eq!(a.farm_base, b.farm_base, "{}", a.path.display());
                assert_eq!((&a.who, &a.in_progress), (&b.who, &b.in_progress));
                assert_eq!(a.name(), b.name());
            }
        }
        // The viewer's row is measured like any other: the clone's own yak.
        let from_c = &every[2];
        let ShedFarm::Own(d) = &shed_at(from_c, &c).farm else {
            panic!("the current shed has an own farm")
        };
        assert_eq!(d.added, ["yaks-cccc"]);
        // And the text says where you are.
        let text = render(from_c, &[]);
        assert!(text.contains("  name: q7zz (here)\n"), "{text}");
        assert!(text.contains("  name: main\n"), "{text}");
        assert_eq!(
            text.lines().filter(|l| l.starts_with("* ")).count(),
            1,
            "{text}"
        );
    }

    #[test]
    fn every_selector_accepts_the_current_shed_and_main_is_the_primary_from_the_primary() {
        let (t, wt, c) = three_checkouts("selectors");
        let from_t = all_sheds_of(&t);
        let p = |s: &Shed| s.path.clone();
        // From the primary, `main` is the primary itself, current or not.
        assert_eq!(p(resolve(&from_t, "main").unwrap()), t);
        assert!(resolve(&from_t, "main").unwrap().current);
        assert_eq!(p(resolve(&from_t, "q7zz").unwrap()), c);
        // From a clone, its own name, dir id, path and branch all work.
        let from_c = all_sheds_of(&c);
        assert_eq!(p(resolve(&from_c, "q7zz").unwrap()), c);
        assert!(resolve(&from_c, "q7zz").unwrap().current);
        assert_eq!(p(resolve(&from_c, c.to_str().unwrap()).unwrap()), c);
        assert_eq!(p(resolve(&from_c, "main").unwrap()), t);
        assert_eq!(p(resolve(&all_sheds_of(&wt), "shed").unwrap()), wt);
        // `main` still errors, rather than picking a clone that is on a
        // branch called `main`, when no primary is visible at all.
        let no_primary: Vec<Shed> = from_c.iter().filter(|s| !s.primary).cloned().collect();
        let e = resolve(&no_primary, "main").unwrap_err();
        assert!(e.contains("not visible from here"), "{e}");
    }

    /// `repo_with_farm`, then a second commit adding `yaks-dddd` and a pointer
    /// at it as `origin/main`, then a third adding `yaks-eeee` and an
    /// uncommitted `yaks-ffff`: only eeee and ffff are unpublished.
    fn primary_ahead_of_origin(tag: &str, origin_ref: &str) -> (PathBuf, String) {
        let b = base(tag);
        let t = b.join("repo");
        repo_with_farm(&t);
        task_file(&t, "hairy", "yaks-dddd", "", "published");
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "published"]);
        let published = sh(&t, &["rev-parse", "--short", "HEAD"]);
        sh(&t, &["update-ref", origin_ref, "HEAD"]);
        task_file(&t, "hairy", "yaks-eeee", "", "committed, not pushed");
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "unpublished"]);
        task_file(&t, "hairy", "yaks-ffff", "", "not even committed");
        (t, published)
    }

    #[test]
    fn the_primary_is_measured_from_origin_main_whoever_looks() {
        let (t, published) = primary_ahead_of_origin("origin-main", "refs/remotes/origin/main");
        let sibling = t.parent().unwrap().join("sibling");
        sh(
            &t,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "sib",
                sibling.to_str().unwrap(),
            ],
        );
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "d1",
            "repo",
        );
        for viewer in [&t, &sibling, &c] {
            let sheds = all_sheds_of(viewer);
            let main = sheds.iter().find(|s| s.primary).unwrap();
            assert_eq!(
                main.farm_base,
                Some(FarmBase {
                    kind: BaseKind::Origin,
                    sha: published.clone()
                }),
                "{}",
                viewer.display()
            );
            let ShedFarm::Own(d) = &main.farm else {
                panic!("the primary has its own farm")
            };
            assert_eq!(d.added, ["yaks-eeee", "yaks-ffff"], "no flood: {d:?}");
            let text = render(&sheds, &[]);
            assert!(text.contains("farm: 2 new (vs origin/main)\n"), "{text}");
            let j = json_of(&sheds, &t);
            assert_eq!(
                (&j["farm"]["vs"], &j["farm"]["base"]),
                (&json!("origin"), &json!(published))
            );
        }
    }

    #[test]
    fn the_primary_falls_back_to_origin_heads_target_then_to_its_fork_point() {
        // No `origin/main`, but `origin/HEAD` points at another branch.
        let (t, published) = primary_ahead_of_origin("origin-head", "refs/remotes/origin/trunk");
        sh(
            &t,
            &[
                "symbolic-ref",
                "refs/remotes/origin/HEAD",
                "refs/remotes/origin/trunk",
            ],
        );
        let sheds = all_sheds_of(&t);
        let main = shed_at(&sheds, &t);
        assert_eq!(
            main.farm_base,
            Some(FarmBase {
                kind: BaseKind::Origin,
                sha: published
            })
        );
        // No usable origin ref at all (no remote, never fetched): the fork
        // point, and the text says `(vs fork)` rather than passing for
        // "unpublished".
        let (t, _) = primary_ahead_of_origin("no-origin", "refs/heads/elsewhere");
        let sheds = all_sheds_of(&t);
        let main = shed_at(&sheds, &t);
        assert_eq!(main.farm_base.as_ref().unwrap().kind, BaseKind::Fork);
        let text = render(&sheds, &[]);
        assert!(text.contains("(vs fork)"), "{text}");
        assert_eq!(json_of(&sheds, &t)["farm"]["vs"], "fork");
    }

    // -- thread titles as shed names ----------------------------------------------

    #[test]
    fn slug_is_a_git_worktree_like_name() {
        let slug = |t: &str| super::slug(t);
        assert_eq!(
            slug("cflag-1: yaks -C global flag").as_deref(),
            Some("cflag-1-yaks-c-global")
        );
        assert_eq!(
            slug("Delta :Yaks (cont'd)").as_deref(),
            Some("delta-yaks-cont-d")
        );
        assert_eq!(
            slug("  --Hello,   World!--  ").as_deref(),
            Some("hello-world")
        );
        // At most 24 chars, cut at a `-` boundary when there is one.
        assert_eq!(
            slug("alpha beta gamma delta epsilon").as_deref(),
            Some("alpha-beta-gamma-delta")
        );
        // A `-` right after the 24th character: kept whole.
        assert_eq!(
            slug("abcdefghijklmnopqrstuvwx yz").as_deref(),
            Some("abcdefghijklmnopqrstuvwx")
        );
        // A cut that lands inside a word drops that word.
        assert_eq!(
            slug("alpha beta gamma delta eps").as_deref(),
            Some("alpha-beta-gamma-delta")
        );
        assert_eq!(
            slug("abcdefghij klmnopqrst uvwxyz").as_deref(),
            Some("abcdefghij-klmnopqrst")
        );
        // No boundary to cut at: a hard cut.
        assert_eq!(
            slug(&"x".repeat(40)).as_deref(),
            Some("xxxxxxxxxxxxxxxxxxxxxxxx")
        );
        // Nothing left: no slug, so the name falls through.
        assert_eq!(slug("!!! \u{1f980}"), None);
        assert_eq!(slug(""), None);
    }

    /// Give the clone `c` a recorded thread title the way `record_thread_in`
    /// does, without needing the environment.
    fn titled(c: &Path, title: &str) {
        record_thread_in(c, Some(title), None);
    }

    #[test]
    fn a_titled_clone_is_named_by_its_slug_and_resolves_by_it() {
        let b = base("titled");
        let t = b.join("repo");
        repo_with_farm(&t);
        let host = t.join(".git");
        let (store, co) = (t.join(".delta/clones"), t.join(".delta/worktrees"));
        let c1 = delta_clone(&host, &store, &co, "abc123", "repo");
        let c2 = delta_clone(&host, &store, &co, "q7zz00", "repo");
        titled(&c1, "cflag-1: yaks -C global flag");

        let from_t = sheds_of(&t);
        let s1 = shed_at(&from_t, &c1);
        assert_eq!(s1.title.as_deref(), Some("cflag-1: yaks -C global flag"));
        assert_eq!(s1.name(), "cflag-1-yaks-c-global");
        // An untitled clone keeps its dir id.
        assert_eq!(shed_at(&from_t, &c2).name(), "q7zz00");
        let p = |q: &str| resolve(&from_t, q).unwrap().path.clone();
        assert_eq!(p("cflag-1-yaks-c-global"), c1);
        assert_eq!(p("abc123"), c1, "the dir id still resolves");
        assert_eq!(p("global flag"), c1, "substring of the full title");
        assert_eq!(p("YAKS -C GLOBAL"), c1, "case-insensitive");
        let j = to_json(&from_t);
        assert_eq!(j[0]["title"], "cflag-1: yaks -C global flag");
        assert_eq!(j[0]["name"], "cflag-1-yaks-c-global");
        assert_eq!(j[1]["title"], Value::Null);
        assert!(render(&from_t, &[]).contains(
            "  name: cflag-1-yaks-c-global \u{b7} title: cflag-1: yaks -C global flag\n"
        ));
    }

    #[test]
    fn the_slug_beats_the_actor_and_the_dir_id_in_a_name() {
        let (t, wt) = claimed_repo_with_shed("slugfirst");
        shed_notes_on_aaaa(&wt, &[("2026-03-03T00:00:00Z", Some("sheds-2"), "hi")]);
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "q7zz",
            "repo",
        );
        titled(&c, "Some Title");
        let from_t = sheds_of(&t);
        assert_eq!(shed_at(&from_t, &wt).name(), "sheds-2");
        assert_eq!(shed_at(&from_t, &c).name(), "some-title");
    }

    #[test]
    fn sheds_with_the_same_slug_get_their_dir_id_prefix() {
        let b = base("collide");
        let t = b.join("repo");
        repo_with_farm(&t);
        let host = t.join(".git");
        let (store, co) = (t.join(".delta/clones"), t.join(".delta/worktrees"));
        let c1 = delta_clone(&host, &store, &co, "q7zzaa", "repo");
        let c2 = delta_clone(&host, &store, &co, "k3mmbb", "repo");
        let c3 = delta_clone(&host, &store, &co, "xxxxcc", "repo");
        titled(&c1, "Fix the thing");
        titled(&c2, "fix the thing!");
        titled(&c3, "Fix another thing");
        let from_t = sheds_of(&t);
        assert_eq!(shed_at(&from_t, &c1).name(), "fix-the-thing-q7zz");
        assert_eq!(shed_at(&from_t, &c2).name(), "fix-the-thing-k3mm");
        assert_eq!(shed_at(&from_t, &c3).name(), "fix-another-thing");
        let p = |q: &str| resolve(&from_t, q).unwrap().path.clone();
        assert_eq!(p("fix-the-thing-q7zz"), c1);
        assert_eq!(p("fix-the-thing-k3mm"), c2);
        assert!(resolve(&from_t, "fix-the-thing").is_err(), "ambiguous");
    }

    #[test]
    fn record_thread_writes_only_in_a_delta_clone_and_only_a_changed_value() {
        let b = base("record");
        let t = b.join("repo");
        repo_with_farm(&t);
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "abc123",
            "repo",
        );
        let get = |dir: &Path, key: &str| {
            Command::new("git")
                .arg("-C")
                .arg(dir)
                .args(["config", "--local", "--get", key])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap()
        };
        record_thread_in(&c, Some("  A  title \n"), Some("thr-1"));
        assert_eq!(get(&c, "yaks.shed.title"), "A title");
        assert_eq!(get(&c, "yaks.shed.thread"), "thr-1");
        // Nothing leaked to the host.
        assert_eq!(get(&t, "yaks.shed.title"), "");
        // Same value: no write (the config file is untouched).
        let gd = shed_git_dir(&c).unwrap();
        let before = fs::metadata(gd.join("config")).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        record_thread_in(&c, Some("A title"), Some("thr-1"));
        let after = fs::metadata(gd.join("config")).unwrap().modified().unwrap();
        assert_eq!(before, after);
        // A new title replaces the old one; no title leaves it alone.
        record_thread_in(&c, Some("Next"), None);
        assert_eq!(get(&c, "yaks.shed.title"), "Next");
        record_thread_in(&c, None, Some("thr-2"));
        record_thread_in(&c, Some("  "), Some("thr-2"));
        assert_eq!(get(&c, "yaks.shed.title"), "Next");
        assert_eq!(get(&c, "yaks.shed.thread"), "thr-1");

        // The primary checkout and a plain git worktree record nothing.
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "w", wt.to_str().unwrap()],
        );
        record_thread_in(&t, Some("Primary"), None);
        record_thread_in(&wt, Some("Worktree"), None);
        assert_eq!(get(&t, "yaks.shed.title"), "");
        assert_eq!(get(&wt, "yaks.shed.title"), "");
    }

    // -- yaks changes: the per-yak expansion of the `farm:` line ---------------

    /// The worker shed `wt` of `claimed_repo_with_shed`: it creates `yaks-cccc`
    /// (a note by w-1), moves `yaks-bbbb` hairy -> shaving (the `moved:` note by
    /// w-2), and appends two notes by two actors to the claimed `yaks-aaaa`.
    fn shed_with_one_of_each(wt: &Path) {
        shed_notes_on_aaaa(
            wt,
            &[
                (
                    "2026-03-03T00:00:00Z",
                    Some("w-1"),
                    "first line\nsecond line stays out",
                ),
                ("2026-03-04T00:00:00Z", Some("w-2"), "other actor"),
            ],
        );
        task_file(
            wt,
            "hairy",
            "yaks-cccc",
            "",
            &notes("new", &[("2026-03-03T01:00:00Z", Some("w-1"), "made it")]),
        );
        fs::remove_file(wt.join(".yaks/yaks-bbbb.md")).unwrap();
        task_file(
            wt,
            "shaving",
            "yaks-bbbb",
            "",
            &notes(
                "second",
                &[(
                    "2026-03-05T00:00:00Z",
                    Some("w-2"),
                    "moved: hairy -> shaving",
                )],
            ),
        );
    }

    fn changes_of(viewer: &Path, query: &str) -> Changes {
        // The full list (current included): `changes` needs the primary's
        // path to find the reference, even when the viewer IS the primary.
        let sheds = all_sheds_of(viewer);
        let shed = resolve(&sheds, query).unwrap();
        changes(shed, viewer, &viewer.join(".yaks"), &sheds).unwrap()
    }

    #[test]
    fn changes_lists_one_shed_per_yak_with_actors() {
        let (t, wt) = claimed_repo_with_shed("changes-per-yak");
        shed_with_one_of_each(&wt);
        let c = changes_of(&t, "shed");
        let ys = c.yaks.as_ref().unwrap();
        let ids: Vec<&str> = ys.iter().map(|y| y.id.as_str()).collect();
        assert_eq!(ids, ["yaks-aaaa", "yaks-bbbb", "yaks-cccc"]);
        let actors = |y: &YakChange| -> Vec<Option<String>> {
            y.notes.iter().map(|n| n.actor.clone()).collect()
        };
        // Notes: only the shed's two, not the coordinator's claim.
        assert_eq!(ys[0].notes.len(), 2);
        assert_eq!(
            actors(&ys[0]),
            [Some("w-1".to_string()), Some("w-2".to_string())]
        );
        assert!(!ys[0].added && ys[0].moved.is_none());
        // Moved.
        assert_eq!(ys[1].moved, Some(("hairy", "shaving")));
        assert_eq!(ys[1].notes[0].text, "moved: hairy -> shaving");
        assert_eq!(actors(&ys[1]), [Some("w-2".to_string())]);
        // Created.
        assert!(ys[2].added && ys[2].moved.is_none());
        assert_eq!(
            (ys[2].title.as_str(), ys[2].status),
            ("yaks-cccc title", "hairy")
        );
        assert_eq!(actors(&ys[2]), [Some("w-1".to_string())]);

        let out = render_changes(&c);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], format!("{}  {}", c.name, wt.display()));
        assert_eq!(lines[1], "  changes:");
        assert_eq!(lines[2], "  yaks-aaaa  yaks-aaaa title  [shaving]");
        assert_eq!(lines[3], "    2026-03-03T00:00:00Z  w-1  first line");
        assert_eq!(lines[4], "    2026-03-04T00:00:00Z  w-2  other actor");
        assert_eq!(lines[5], "  yaks-bbbb  yaks-bbbb title  [hairy -> shaving]");
        assert_eq!(
            lines[6],
            "    2026-03-05T00:00:00Z  w-2  moved: hairy -> shaving"
        );
        assert_eq!(lines[7], "  yaks-cccc  yaks-cccc title  [new, hairy]");
        assert_eq!(lines[8], "    2026-03-03T01:00:00Z  w-1  made it");
        assert_eq!(lines.len(), 9, "{out}");
        assert!(!out.contains("second line stays out"));

        let j = changes_json(&c);
        let j = j.as_array().unwrap();
        assert_eq!(j.len(), 3);
        assert_eq!(j[1]["moved"], json!({"from": "hairy", "to": "shaving"}));
        assert_eq!(j[2]["added"], true);
        assert_eq!(j[0]["notes"][1]["actor"], "w-2");
        assert_eq!(j[0]["vs"], "fork");
        // It expands what `sheds` counts: 1 new, 1 moved, and every new note
        // (2 on aaaa + 1 on bbbb + the 1 on the created cccc).
        let sheds = sheds_of(&t);
        assert!(
            render(&sheds, &[]).contains("farm: 1 new, 1 moved, 4 new notes"),
            "{}",
            render(&sheds, &[])
        );
    }

    #[test]
    fn changes_reports_needs_newly_set() {
        let (t, wt) = claimed_repo_with_shed("changes-needs");
        task_file(&wt, "hairy", "yaks-bbbb", "needs: agent\n", "second");
        let c = changes_of(&t, "shed");
        let ys = c.yaks.as_ref().unwrap();
        assert_eq!(ys.len(), 1);
        assert_eq!(ys[0].needs.as_deref(), Some("agent"));
        assert!(render_changes(&c).contains("    needs: agent\n"));
    }

    /// Rewrite a yak file of the shed in place with `from` replaced by `to`.
    fn rewrite(root: &Path, status: &str, id: &str, from: &str, to: &str) {
        let _ = status; // the flat layout has one path per yak
        let f = root.join(format!(".yaks/{id}.md"));
        let text = fs::read_to_string(&f).unwrap();
        assert!(text.contains(from), "{from} not in {text}");
        fs::write(&f, text.replacen(from, to, 1)).unwrap();
    }

    /// A shed that only rewrote a yak's description, or only its title, has
    /// an `edited` yak: `changes` lists it (text and JSON) and `farm:` counts
    /// it.
    #[test]
    fn changes_reports_an_edited_yak() {
        let (t, wt) = claimed_repo_with_shed("changes-edited");
        rewrite(&wt, "hairy", "yaks-bbbb", "second", "second, rewritten");
        rewrite(
            &wt,
            "shaving",
            "yaks-aaaa",
            "title: yaks-aaaa title",
            "title: new title",
        );
        let c = changes_of(&t, "shed");
        let ys = c.yaks.as_ref().unwrap();
        let ids: Vec<(&str, bool)> = ys.iter().map(|y| (y.id.as_str(), y.edited)).collect();
        assert_eq!(ids, [("yaks-aaaa", true), ("yaks-bbbb", true)]);
        assert!(
            ys.iter()
                .all(|y| !y.added && y.moved.is_none() && y.notes.is_empty())
        );

        let out = render_changes(&c);
        assert!(
            out.contains("  yaks-aaaa  new title  [shaving, edited]\n"),
            "{out}"
        );
        assert!(
            out.contains("  yaks-bbbb  yaks-bbbb title  [hairy, edited]\n"),
            "{out}"
        );
        let j = changes_json(&c);
        assert_eq!(j[0]["edited"], true);
        assert_eq!(j[1]["edited"], true);
        let sheds = render(&sheds_of(&t), &[]);
        assert!(sheds.contains("farm: 2 edited"), "{sheds}");
        assert!(sheds.contains("    yaks-bbbb edited\n"), "{sheds}");
        let j = &to_json(&sheds_of(&t))[0]["farm"]["edited"];
        assert_eq!(j, &json!(["yaks-aaaa", "yaks-bbbb"]));
    }

    /// `updated:` alone is not an edit, and a yak that only gained notes,
    /// moved or set `needs:` is not also `edited`; a created yak is `new`.
    #[test]
    fn changes_edited_ignores_stamps_notes_moves_and_needs() {
        let (t, wt) = claimed_repo_with_shed("changes-plain");
        // aaaa: only the updated stamp. bbbb: moves, gains a note, sets needs.
        rewrite(
            &wt,
            "shaving",
            "yaks-aaaa",
            "updated: '2026-01-01",
            "updated: '2026-09-09",
        );
        shed_with_one_of_each(&wt);
        task_file(
            &wt,
            "shaving",
            "yaks-bbbb",
            "needs: agent\n",
            &notes(
                "second",
                &[
                    (
                        "2026-03-05T00:00:00Z",
                        Some("w-2"),
                        "moved: hairy -> shaving",
                    ),
                    ("2026-03-06T00:00:00Z", Some("w-2"), "more"),
                ],
            ),
        );
        shed_notes_on_aaaa(&wt, &[("2026-03-03T00:00:00Z", Some("w-1"), "a note")]);
        rewrite(
            &wt,
            "shaving",
            "yaks-aaaa",
            "updated: '2026-01-01",
            "updated: '2026-09-09",
        );
        let c = changes_of(&t, "shed");
        let ys = c.yaks.as_ref().unwrap();
        assert_eq!(ys.len(), 3);
        assert!(ys.iter().all(|y| !y.edited), "{ys:?}");
        assert!(
            !render_changes(&c).contains("edited"),
            "{}",
            render_changes(&c)
        );
        assert!(
            !render(&sheds_of(&t), &[]).contains("edited"),
            "{}",
            render(&sheds_of(&t), &[])
        );
    }

    /// An `updated:`-only change lists nothing at all.
    #[test]
    fn changes_ignores_an_updated_only_change() {
        let (t, wt) = claimed_repo_with_shed("changes-updated-only");
        rewrite(
            &wt,
            "hairy",
            "yaks-bbbb",
            "updated: '2026-01-01",
            "updated: '2026-09-09",
        );
        let c = changes_of(&t, "shed");
        assert!(c.yaks.as_ref().unwrap().is_empty());
    }

    /// An edit alongside a move or notes reads `edited` as well.
    #[test]
    fn changes_edited_is_reported_with_a_move_and_notes() {
        let (t, wt) = claimed_repo_with_shed("changes-edited-and");
        shed_with_one_of_each(&wt);
        rewrite(&wt, "shaving", "yaks-bbbb", "priority: 3", "priority: 1");
        rewrite(&wt, "shaving", "yaks-aaaa", "first", "first, reworded");
        let c = changes_of(&t, "shed");
        let out = render_changes(&c);
        assert!(out.contains("[shaving, edited]"), "{out}");
        assert!(out.contains("[hairy -> shaving, edited]"), "{out}");
        assert!(out.contains("[new, hairy]"), "{out}");
        assert!(!out.contains("[new, hairy, edited]"));
    }

    /// `yaks status` and `yaks sheds`/`changes` read an edit the same way: the
    /// same pair of files gets the same verdict from both.
    #[test]
    fn status_and_changes_agree_on_edited() {
        let base = "---\nid: yak-1\ntitle: t\ntype: task\npriority: 3\n\
                    created: '2026-01-01T00:00:00Z'\nupdated: '2026-01-01T00:00:00Z'\n---\nbody";
        let n = "\n\n---\n\u{25b8} 2026-02-02T00:00:00Z [w]\nnote";
        let cases = [
            (base.to_string(), false),
            (
                base.replace("2026-01-01T00:00:00Z'\n---", "2026-05-05T00:00:00Z'\n---"),
                false,
            ),
            (format!("{base}{n}"), false),
            (base.replace("title: t", "title: u"), true),
            (base.replace("priority: 3", "priority: 1"), true),
            (base.replace("type: task", "type: bug"), true),
            (
                base.replace("priority: 3", "priority: 3\nlabels: [x]"),
                true,
            ),
            (
                base.replace("priority: 3", "priority: 3\nparent: yak-0"),
                true,
            ),
            (
                base.replace("priority: 3", "priority: 3\nverify: true"),
                true,
            ),
            (base.replace("priority: 3", "priority: 3\nsource: s"), true),
            (base.replace("priority: 3", "priority: 3\nmystery: 1"), true),
            (base.replace("\nbody", "\nbody changed"), true),
            (
                format!("{}{n}", base.replace("\nbody", "\nbody changed")),
                true,
            ),
        ];
        let parse = |t: &str| store::parse_task(t, crate::model::Status::Hairy).unwrap();
        for (new, want) in cases {
            let (_, status_says) = crate::status::describe(base, &new);
            let changes_says = yak_changes(&[parse(base)], &[parse(&new)])
                .iter()
                .any(|y| y.edited);
            assert_eq!((status_says, changes_says), (want, want), "{new}");
        }
    }

    /// A created yak's notes count as new notes (`yaks changes` lists them,
    /// so `farm:` must agree).
    #[test]
    fn compare_farms_counts_a_created_yaks_notes() {
        let b = base("compare-created-notes");
        let (ours, theirs) = (b.join("ours"), b.join("theirs"));
        let note = "\n\n---\n\u{25b8} 2026-02-02T00:00:00Z [w]\nn";
        task_file(&ours, "hairy", "yaks-aaaa", "", "a");
        task_file(&theirs, "hairy", "yaks-aaaa", "", "a");
        task_file(&theirs, "hairy", "yaks-bbbb", "", &format!("b{note}{note}"));
        let load = |root: &Path| store::load(&root.join(".yaks"), &EVERY).unwrap();

        let d = compare_farms(&load(&ours), &load(&theirs));

        assert_eq!(d.added, ["yaks-bbbb"]);
        assert_eq!(d.notes, [("yaks-bbbb".to_string(), 2)]);
    }

    /// `compare_farms` is `yak_changes` counted: both give the same answer on
    /// a fixture with a created yak, a moved one, extra notes, and `needs:`
    /// new, changed and unchanged.
    #[test]
    fn compare_farms_is_yak_changes_counted() {
        let b = base("compare-vs-changes");
        let (ours, theirs) = (b.join("ours"), b.join("theirs"));
        let note = "\n\n---\n\u{25b8} 2026-02-02T00:00:00Z [w]\nn";
        // ours: aaaa moves, bbbb gains a note, cccc's needs changes, dddd's
        // needs is already carried, eeee is untouched.
        task_file(&ours, "hairy", "yaks-aaaa", "", "a");
        task_file(&ours, "hairy", "yaks-bbbb", "", "b");
        task_file(&ours, "hairy", "yaks-cccc", "needs: agent\n", "c");
        task_file(&ours, "hairy", "yaks-dddd", "needs: human\n", "d");
        task_file(&ours, "hairy", "yaks-eeee", "", "e");
        task_file(&theirs, "shaving", "yaks-aaaa", "", "a");
        task_file(&theirs, "hairy", "yaks-bbbb", "", &format!("b{note}{note}"));
        task_file(&theirs, "hairy", "yaks-cccc", "needs: human\n", "c");
        task_file(&theirs, "hairy", "yaks-dddd", "needs: human\n", "d");
        task_file(&theirs, "hairy", "yaks-eeee", "", "e");
        // A created yak, with a note and a `needs:`.
        task_file(
            &theirs,
            "hairy",
            "yaks-ffff",
            "needs: agent\n",
            &format!("f{note}"),
        );
        let load = |root: &Path| store::load(&root.join(".yaks"), &EVERY).unwrap();
        let (ours, theirs) = (load(&ours), load(&theirs));

        let d = compare_farms(&ours, &theirs);

        let mut counted = FarmDelta::default();
        for c in yak_changes(&ours, &theirs) {
            if c.added {
                counted.added.push(c.id.clone());
            }
            if !c.notes.is_empty() {
                counted.notes.push((c.id.clone(), c.notes.len()));
            }
            if let Some((from, to)) = c.moved {
                counted.moved.push(Moved {
                    id: c.id.clone(),
                    from,
                    to,
                });
            }
            if c.edited {
                counted.edited.push(c.id.clone());
            }
            if let Some(n) = c.needs {
                counted.needs.push((c.id, n));
            }
        }
        assert_eq!(d, counted);

        // And the facts themselves, so the rules can't drift together.
        assert_eq!(d.added, ["yaks-ffff"]);
        assert_eq!(
            d.moved,
            [Moved {
                id: "yaks-aaaa".into(),
                from: "hairy",
                to: "shaving"
            }]
        );
        // The created yak's own note counts, like the extra notes on bbbb.
        assert_eq!(
            d.notes,
            [("yaks-bbbb".to_string(), 2), ("yaks-ffff".to_string(), 1)]
        );
        assert_eq!(
            d.needs,
            [
                ("yaks-cccc".to_string(), "human".to_string()),
                ("yaks-ffff".to_string(), "agent".to_string())
            ]
        );
    }

    #[test]
    fn changes_main_from_a_shed_shows_the_main_checkouts_own_changes() {
        let (t, wt) = claimed_repo_with_shed("changes-main");
        shed_notes_on_aaaa(&wt, &[("2026-03-03T00:00:00Z", Some("w-1"), "shed note")]);
        // Nothing of main's own since the fork yet.
        let none = changes_of(&wt, "main");
        assert_eq!(none.name, "main");
        assert_eq!(none.yaks.as_ref().unwrap(), &vec![]);
        assert_eq!(
            render_changes(&none),
            // A fixture primary has no `origin`: measured from its fork, said so.
            format!("main  {}\n  no changes of its own (vs fork)\n", t.display())
        );
        // Main then lands its own yak and a note after the shed forked.
        task_file(
            &t,
            "hairy",
            "yaks-dddd",
            "",
            &notes(
                "main's",
                &[("2026-03-06T00:00:00Z", Some("human"), "from main")],
            ),
        );
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "main moves on"]);
        let c = changes_of(&wt, "main");
        let ys = c.yaks.as_ref().unwrap();
        assert_eq!(ys.len(), 1, "{ys:?}");
        assert!(ys[0].added && ys[0].id == "yaks-dddd");
        assert_eq!(ys[0].notes[0].actor.as_deref(), Some("human"));
        assert!(
            !render_changes(&c).contains("shed note"),
            "the shed's own notes are not main's"
        );
    }

    #[test]
    fn changes_says_why_a_shared_farm_has_nothing_to_show() {
        let b = base("changes-shared");
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "s", wt.to_str().unwrap()],
        );
        let sheds = sheds_of(&t);
        let shed = shed_at(&sheds, &wt);
        // A farm outside the checkout, both resolve to it.
        let shared = ShedFarm::Shared;
        let mut s = shed.clone();
        s.farm = shared;
        let c = changes(&s, &t, &t.join(".yaks"), &sheds).unwrap();
        assert_eq!(c.yaks, Err("shares this farm".to_string()));
        assert!(render_changes(&c).contains("nothing to show: shares this farm"));
        assert_eq!(changes_json(&c), json!([]));
    }

    // -- the old layout -----------------------------------------

    #[test]
    fn base_farm_of_a_commit_from_before_the_migration_finds_nothing() {
        let b = base("base-old");
        let t = b.join("repo");
        fs::create_dir_all(&t).unwrap();
        sh(&t, &["init", "-q"]);
        old_task_file(&t, "hairy", "yaks-aaaa", "", "first");
        old_task_file(&t, "shorn", "yaks-cccc", "", "third");
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "old layout"]);
        let old_sha = sh(&t, &["rev-parse", "HEAD"]);
        fs::remove_dir_all(t.join(".yaks")).unwrap();
        task_file(&t, "hairy", "yaks-aaaa", "", "first");
        task_file(&t, "shorn", "yaks-cccc", "", "third");
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "flat layout"]);
        let flat_sha = sh(&t, &["rev-parse", "HEAD"]);

        let farm = t.join(".yaks");
        // No flat file at the old commit: `None`, the caller's cue to try its
        // next base (never an empty base, which would make every yak "new").
        assert!(base_farm(&t, &old_sha, &t, &farm).is_none());
        let (_, flat) = base_farm(&t, &flat_sha, &t, &farm).unwrap();
        assert_eq!(flat.len(), 2);
    }

    #[test]
    fn an_old_layout_farm_is_an_error_not_an_empty_farm() {
        let b = base("is-old");
        let old = b.join("old");
        let flat = b.join("flat");
        let late = b.join("late");
        old_task_file(&old, "hairy", "yaks-aaaa", "", "x");
        fs::write(old.join(".yaks/schema"), "3\n").unwrap();
        task_file(&flat, "hairy", "yaks-aaaa", "", "x");
        // Schema 4 with a yak an older binary left in a status directory.
        task_file(&late, "hairy", "yaks-bbbb", "", "x");
        old_task_file(&late, "hairy", "yaks-aaaa", "", "x");
        for old in [&old, &late] {
            let err = load_farm(&old.join(".yaks")).unwrap_err().to_string();
            assert!(err.contains("old layout (pre-0.0.16 clone)"), "{err}");
            assert!(err.contains("not read"), "{err}");
        }
        assert_eq!(load_farm(&flat.join(".yaks")).unwrap().len(), 1);
    }
}
