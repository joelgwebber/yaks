//! `yaks sheds`: every checkout of this repository (git worktrees and Delta
//! clones), this one included and marked, with its git state and the yaks it
//! has been writing (yaks-9eaa).
//!
//! Every checkout of a repository on one machine shares ONE farm (the farm
//! branch's worktree, yaks-6a34), so there is no per-shed copy of the farm to
//! compare. What a shed has been doing comes from the farm's local activity
//! log ([`crate::activity`]): every write of a yak file records the checkout
//! it ran in. A shed's row joins discovery (where the checkouts are) and git
//! (branch, ahead/behind, dirty, merged) to that log (which yaks it wrote,
//! newest first; which of those are `shaving`; when it last wrote).
//!
//! Plain git plus the filesystem; no Delta database, no Delta CLI. Strictly
//! read-only in the other checkouts: git is invoked as `git
//! --no-optional-locks -C <shed> ...` for read-only plumbing only.
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
//! The farm branch's own worktree is the farm, not a shed, and is left out
//! (`yaks status` reports its state). Everything found this way is the same
//! repository by construction (one object store, one pin namespace), so no
//! URL or name matching is needed. Limits: a clone on a shared-thread machine
//! cannot see the human's checkout (its host is the managed repo).

use crate::activity;
use crate::model::{Status, Task};
use crate::store;
use anyhow::{Context, Result, bail};
use chrono::DateTime;
#[cfg(test)]
use chrono::Utc;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
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

/// Whether anyone appears to be working in a shed right now, from its
/// activity log alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    /// It wrote to the farm within [`ACTIVE_WITHIN`].
    Active,
    /// It holds a claim (a yak it wrote that is `shaving`) but has not
    /// written for longer: a claim nobody has touched in a while.
    Stale,
    /// Neither: no recent write and no claim (the common case).
    Idle,
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

/// How recent a shed's last write must be for it to read `active`.
const ACTIVE_WITHIN: Duration = Duration::from_secs(2 * 60 * 60);

/// One yak a shed wrote, as the activity log and the farm say now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Touched {
    pub id: String,
    /// The yak's title, or `""` when it is no longer in the farm.
    pub title: String,
    /// Its status now; `None` when it is no longer in the farm.
    pub status: Option<Status>,
    /// Its `needs:` now.
    pub needs: Option<String>,
    /// The shed's newest write of it.
    pub last: String,
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
    /// The name [`Shed::name`] uses for this shed: the [`slug`]
    /// of `title`, made unique among the listed sheds by [`disambiguate`].
    pub slug: Option<String>,
    /// The repository's main checkout.
    pub primary: bool,
    /// The checkout `yaks sheds` was run in. Exactly one listed shed is
    /// current.
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
    /// How many of `dirty` are untracked (`??`). `None` exactly when `dirty`
    /// is.
    pub untracked: Option<usize>,
    /// The shed's HEAD is in the primary checkout's history (an ancestor of,
    /// or equal to, the primary's HEAD): its commits are all in (yaks-28dc).
    /// `None` for the primary itself, when the primary is not visible here,
    /// or when the shed's git state cannot be read. See [`state_line`].
    pub merged: Option<bool>,
    /// The yaks this checkout wrote (its activity log), newest write first.
    pub touched: Vec<Touched>,
    /// Ids of `touched` yaks that are `shaving` now: what it is working on.
    pub in_progress: Vec<String>,
    /// Its newest write (`None` when it never wrote).
    pub last_write: Option<String>,
    /// See [`Liveness`]. Every shed gets one.
    pub liveness: Liveness,
    /// Set when the shed's git state could not be read; the shed is still
    /// listed.
    pub error: Option<String>,
}

impl Shed {
    /// A short name for messages: `main` for the primary
    /// checkout, else the slug of its thread title, else its Delta dir id,
    /// else its branch, else the directory name.
    pub fn name(&self) -> String {
        if self.primary {
            return "main".into();
        }
        self.slug
            .clone()
            .or_else(|| self.dir.clone())
            .or_else(|| self.branch.clone())
            .unwrap_or_else(|| {
                self.path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| self.path.display().to_string())
            })
    }

    /// The touched yaks still waiting on a human.
    pub fn open_asks(&self) -> impl Iterator<Item = &Touched> {
        self.touched
            .iter()
            .filter(|t| t.needs.as_deref() == Some("human"))
    }
}

/// List every checkout of the repository containing `cwd`, this one included
/// (`current`). `farm_root` is the farm every checkout shares: its activity
/// log says what each shed wrote, and its own worktree is not listed. Sorted
/// by path.
#[cfg(test)]
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
    let farm_root = canonical(farm_root);
    // The independent first questions, asked at once: the checkouts and pins,
    // whether the repository is shallow, the farm's yaks, and our HEAD (so a
    // shed at the same commit needs no merge-base).
    let (gathered, shallow, tasks, top_head) = std::thread::scope(|s| {
        let found = s.spawn(|| gather(&top));
        let shallow = s.spawn(|| {
            git(&top, &["rev-parse", "--is-shallow-repository"]).is_ok_and(|s| s == "true")
        });
        let tasks = s.spawn(|| store::load(&farm_root, &EVERY).unwrap_or_default());
        let head = s.spawn(|| git(&top, &["rev-parse", "HEAD"]).ok());
        (
            found.join().unwrap(),
            shallow.join().unwrap(),
            tasks.join().unwrap(),
            head.join().unwrap(),
        )
    });
    let (found, pins) = gathered;
    // The farm branch's worktree holds the farm; it is not a shed.
    let farm_top = farm_root.parent().map(canonical);
    let found: Vec<(PathBuf, Found)> = found
        .into_iter()
        .filter(|(p, _)| Some(p) != farm_top.as_ref() || *p == top)
        .collect();
    let primary_head = found
        .iter()
        .find(|(_, f)| f.primary)
        .and_then(|(p, _)| git(p, &["rev-parse", "HEAD"]).ok())
        .filter(|h| !h.is_empty());
    let by_id: HashMap<&str, &Task> = tasks.iter().map(|t| (t.id.as_str(), t)).collect();
    let now = SystemTime::now();
    let mut sheds: Vec<Shed> = par_map(&found, |(path, found)| {
        let mut shed = inspect(
            &Ctx {
                top: &top,
                top_head: top_head.as_deref(),
                primary_head: primary_head.as_deref(),
            },
            path,
            found,
            *path == top,
        );
        shed.shallow_here = shallow;
        apply_activity(&mut shed, &activity::read(&farm_root, path), &by_id, now);
        shed
    });
    disambiguate(&mut sheds);
    let orphaned = pins.into_iter().filter(|p| p.checkout.is_err()).collect();
    Ok((sheds, orphaned))
}

/// Fill a shed's `touched`, `in_progress`, `last_write` and `liveness` from
/// its activity log `entries` and the farm's yaks now. Pure (given `now`).
fn apply_activity(
    shed: &mut Shed,
    entries: &[activity::Entry],
    tasks: &HashMap<&str, &Task>,
    now: SystemTime,
) {
    shed.touched = activity::latest_per_yak(entries)
        .into_iter()
        .map(|e| {
            let t = tasks.get(e.id.as_str());
            Touched {
                title: t.map(|t| t.title.clone()).unwrap_or_default(),
                status: t.map(|t| t.status),
                needs: t.and_then(|t| t.needs.clone()),
                id: e.id,
                last: e.ts,
            }
        })
        .collect();
    shed.in_progress = shed
        .touched
        .iter()
        .filter(|t| t.status == Some(Status::Shaving))
        .map(|t| t.id.clone())
        .collect();
    shed.last_write = shed.touched.first().map(|t| t.last.clone());
    let recent = shed
        .last_write
        .as_deref()
        .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
        .is_some_and(|t| {
            now.duration_since(SystemTime::from(t))
                .unwrap_or(Duration::ZERO)
                <= ACTIVE_WITHIN
        });
    shed.liveness = if recent {
        Liveness::Active
    } else if !shed.in_progress.is_empty() {
        Liveness::Stale
    } else {
        Liveness::Idle
    };
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

/// What every shed of one `discover` shares.
struct Ctx<'a> {
    top: &'a Path,
    /// Our own HEAD (full sha), when it resolves.
    top_head: Option<&'a str>,
    /// The primary checkout's HEAD (full sha), when it is visible and
    /// resolves: what [`Shed::merged`] is measured against.
    primary_head: Option<&'a str>,
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
/// Delta dir id appended (`<slug>-q7zz`), so every name is distinct. The
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
    /// The repository's main checkout: the first entry of a `git worktree
    /// list` when that entry is itself a checkout (the host checkout, or this
    /// repository's main working tree). A worktree added inside a Delta clone
    /// is never it: that list's first entry is the clone's git dir.
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
    // git lists the repository's main working tree first, so the primary is
    // entry 0 of a `git worktree list` WHEN it is itself a checkout. Where
    // entry 0 is not one (a Delta clone's separate git dir has no `.git`), the
    // list has no primary to name: counting only the entries that survive the
    // filter would crown the first plain worktree (yaks-93fe).
    let add_worktrees = |entries: Vec<(PathBuf, Result<(), String>)>,
                         found: &mut BTreeMap<PathBuf, Found>| {
        for (i, (p, verdict)) in entries.into_iter().enumerate() {
            if verdict.is_err() {
                continue;
            }
            let e = found.entry(canonical(&p)).or_insert(Found {
                kind: ShedKind::Worktree,
                dir: None,
                primary: false,
            });
            e.kind = ShedKind::Worktree;
            e.primary |= i == 0;
        }
    };
    let (here, hosted) = std::thread::scope(|s| {
        let here = s.spawn(|| worktree_entries(top));
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
            let entries = if primary.is_some() {
                entries
            } else {
                Vec::new()
            };
            Some((entries, pins))
        });
        (here.join().unwrap(), hosted.join().unwrap())
    });
    let mut found: BTreeMap<PathBuf, Found> = BTreeMap::new();
    add_worktrees(here, &mut found);
    let mut all_pins = Vec::new();
    if let Some((entries, pins)) = hosted {
        add_worktrees(entries, &mut found);
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

/// Gather one shed's git report. Never fails: unreadable parts become
/// `error`. The activity half is [`apply_activity`].
fn inspect(ctx: &Ctx, path: &Path, found: &Found, current: bool) -> Shed {
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
        untracked: None,
        merged: None,
        touched: Vec::new(),
        in_progress: Vec::new(),
        last_write: None,
        liveness: Liveness::Idle,
        error: None,
    };
    let probe = git_state(ctx, path, !found.primary);
    shed.title = probe.title;
    shed.slug = shed.title.as_deref().and_then(slug);
    match probe.state {
        Ok(g) => {
            shed.branch = g.branch;
            shed.head = Some(g.head);
            shed.ahead = g.ahead;
            shed.behind = g.behind;
            shed.dirty = Some(g.dirty);
            shed.untracked = Some(g.untracked);
            shed.merged = g.merged.filter(|_| !found.primary);
        }
        Err(e) => shed.error = Some(format!("{e:#}")),
    }
    shed
}

struct GitState {
    branch: Option<String>,
    head: String,
    ahead: Option<usize>,
    behind: Option<usize>,
    dirty: usize,
    untracked: usize,
    merged: Option<bool>,
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
/// merge-base/count chain in OUR repo) are asked side by side.
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
        let (ahead, behind) = if ctx.top_head == Some(full.as_str()) {
            (Some(0), Some(0))
        } else {
            let merge_base = git(top, &["merge-base", "HEAD", &full])
                .ok()
                .filter(|b| !b.is_empty());
            match &merge_base {
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
            }
        };
        // In the primary's history already? Cheap when the viewer is the
        // primary (nothing ahead of it means merged); else one git call.
        let merged = ctx.primary_head.map(|ph| {
            if ph == full {
                true
            } else if ctx.top_head == Some(ph) && ahead.is_some() {
                ahead == Some(0)
            } else {
                git(top, &["merge-base", "--is-ancestor", &full, ph]).is_ok()
            }
        });
        let status = status.join().unwrap();
        let state = status.map(|status| {
            let lines: Vec<&str> = status.lines().filter(|l| !l.trim().is_empty()).collect();
            GitState {
                branch: branch.join().unwrap(),
                head,
                ahead,
                behind,
                dirty: lines.len(),
                untracked: lines.iter().filter(|l| l.starts_with("??")).count(),
                merged,
            }
        });
        GitProbe {
            title: title.join().unwrap(),
            state,
        }
    })
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

/// The worktree of the repository containing `from` where `branch` is checked
/// out, if any: `git worktree list --porcelain` in the HOST ([`host`]: this
/// checkout's own repository, or the one its `alternates` names, so a Delta
/// clone finds the human's worktrees), matched on `branch refs/heads/<b>`.
/// Backs farm-branch discovery (`store::discover`). `None` outside git, or
/// when nothing has the branch checked out (a bare host lists only its own
/// worktrees). Read-only.
pub(crate) fn worktree_of_branch(from: &Path, branch: &str) -> Option<PathBuf> {
    let h = host(from)?;
    let text = git_raw(&h.git_dir, &["worktree", "list", "--porcelain"]).ok()?;
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

// -- rendering ----------------------------------------------------------------

/// JSON for `--json`: one object per shed with the same fields as the table.
pub fn to_json(sheds: &[Shed]) -> Value {
    Value::Array(
        sheds
            .iter()
            .map(|l| {
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
                    // Of `dirty`, the untracked files; `null` exactly when
                    // `dirty` is.
                    "untracked": l.untracked,
                    // HEAD already in the primary's history; `null` for the
                    // primary itself or when it cannot be told.
                    "merged": l.merged,
                    // From the farm's activity log: the yaks this checkout
                    // wrote, newest write first, as the farm has them now.
                    "touched": l.touched.iter().map(|t| json!({
                        "id": t.id,
                        "title": t.title,
                        "status": t.status.map(Status::as_str),
                        "needs": t.needs,
                        "last": t.last,
                    })).collect::<Vec<_>>(),
                    "in_progress": l.in_progress,
                    "last_write": l.last_write,
                    "liveness": l.liveness.as_str(),
                    "error": l.error,
                })
            })
            .collect(),
    )
}

/// The one-line activity summary: `shaving: a, b · 3 yaks · active`, each
/// part only when it has something; `None` for a shed that never wrote.
/// `idle` is silent: it is the common case.
pub fn label_line(l: &Shed) -> Option<String> {
    let mut parts = Vec::new();
    if !l.in_progress.is_empty() {
        parts.push(format!("shaving: {}", l.in_progress.join(", ")));
    }
    let asks = l.open_asks().count();
    if asks > 0 {
        parts.push(format!("{asks} asking"));
    }
    if !l.touched.is_empty() {
        let n = l.touched.len();
        parts.push(format!("{n} {}", if n == 1 { "yak" } else { "yaks" }));
    }
    match l.liveness {
        Liveness::Active => parts.push("active".into()),
        Liveness::Stale => parts.push("stale".into()),
        Liveness::Idle => {}
    }
    (!parts.is_empty()).then(|| parts.join(" \u{b7} "))
}

/// Whether anything still keeps a shed around (yaks-28dc), the text after
/// `state: ` in [`render`]: `merged, clean` when its HEAD is in the primary's
/// history and its working tree is clean (safe to clean up); otherwise what
/// remains: `merged` (or nothing, when not merged) plus `uncommitted: N
/// modified, M untracked`. `None` when there is nothing to say: the primary,
/// a shed that is not merged and is clean, or one whose git state cannot be
/// read. Shared with the TUI's Sheds tab.
pub fn state_line(l: &Shed) -> Option<String> {
    if l.primary {
        return None;
    }
    let merged = l.merged == Some(true);
    let uncommitted = match (l.dirty, l.untracked) {
        (Some(d), Some(u)) if d > 0 => {
            let m = d - u;
            let mut parts = Vec::new();
            if m > 0 {
                parts.push(format!("{m} modified"));
            }
            if u > 0 {
                parts.push(format!("{u} untracked"));
            }
            Some(format!("uncommitted: {}", parts.join(", ")))
        }
        _ => None,
    };
    match (merged, uncommitted) {
        (true, None) => Some("merged, clean".into()),
        (true, Some(u)) => Some(format!("merged \u{b7} {u}")),
        (false, Some(u)) => Some(u),
        (false, None) => None,
    }
}

/// The compact human table: one row per shed, then its name, label, state
/// and the yaks it touched most recently.
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
                l.last_write.clone().unwrap_or_else(|| "-".into()),
            ]
        })
        .collect();
    let heads = [
        "KIND",
        "BRANCH",
        "HEAD",
        "+AHEAD -BEHIND",
        "DIRTY",
        "LAST WRITE",
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
        if let Some(state) = state_line(l) {
            out.push_str(&format!("  state: {state}\n"));
        }
        for t in l.touched.iter().take(RECENT_SHOWN) {
            out.push_str(&format!("    {}\n", touched_line(t)));
        }
        if l.touched.len() > RECENT_SHOWN {
            out.push_str(&format!(
                "    (+{} more; --json lists them all)\n",
                l.touched.len() - RECENT_SHOWN
            ));
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
    // Orphaned pins are not a `Shed` row (no checkout to measure); one count
    // line says they exist (`yaks discover` lists each).
    if !orphaned.is_empty() {
        out.push_str(&format!(
            "\norphaned: {} pinned clone(s) with no checkout left (finished threads; \
             `yaks discover` lists them)\n",
            orphaned.len()
        ));
    }
    out
}

/// How many touched yaks the text table lists per shed.
const RECENT_SHOWN: usize = 5;

/// `<id>  <status>  <title>  (<last write>)`, `⏳` first when it is waiting
/// on a human: one touched yak in the table.
pub fn touched_line(t: &Touched) -> String {
    let ask = if t.needs.as_deref() == Some("human") {
        "\u{23f3} "
    } else {
        ""
    };
    let status = t.status.map_or("gone", Status::as_str);
    format!("{ask}{}  {status}  {}  ({})", t.id, t.title, t.last)
}

/// Pinned clones in `cwd`'s host whose checkout could not be resolved (a gone
/// git dir, a missing worktree, or similar — see [`pinned`]). Not a [`Shed`]:
/// there is no checkout to measure. `yaks sheds` and the TUI's Sheds tab use
/// [`discover_all`], which returns these from the same pass as the sheds;
/// this standalone form is what the tests compare it with.
#[cfg(test)]
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

    /// One yak in the current layout: `.yaks/<herd>/yaks/<id>.md` with a `status:` field.
    fn task_file(root: &Path, status: &str, id: &str, extra_fm: &str, body: &str) {
        let dir = root.join(".yaks");
        fs::create_dir_all(&dir).unwrap();
        // What `yaks init` writes: a flat farm is recognised by its marker.
        fs::write(dir.join("schema"), format!("{}\n", store::SCHEMA)).unwrap();
        let f = store::task_path(&dir, id);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(
            f,
            format!(
                "---\nid: {id}\ntitle: {id} title\nstatus: {status}\ntype: task\n\
                 priority: 3\ncreated: '2026-01-01T00:00:00Z'\n\
                 updated: '2026-01-01T00:00:00Z'\n{extra_fm}---\n{body}\n"
            ),
        )
        .unwrap();
    }

    /// Change a flat yak's `status:` in place (how a move is written now).
    fn set_status(root: &Path, id: &str, from: &str, to: &str) {
        let f = store::task_path(&root.join(".yaks"), id);
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

    /// Move a yak to another status in `repo` and commit it.
    fn commit_move(repo: &Path, id: &str, from: &str, to: &str) {
        set_status(repo, id, from, to);
        sh(repo, &["add", "-A"]);
        sh(repo, &["commit", "-q", "-m", &format!("move {id}")]);
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
        // Said once, at the end.
        assert!(text.ends_with(SHALLOW_NOTE), "{text}");
        assert_eq!(text.matches("history is shallow").count(), 1, "{text}");
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
    fn a_git_worktree_of_a_delta_clone_is_a_plain_shed_not_the_primary() {
        // yaks-93fe: `git worktree add` inside a Delta clone. In the clone's
        // `git worktree list` the first entry is its separate git dir (no
        // `.git`, dropped), so the worktree became the list's first CHECKOUT
        // and was taken for the primary: named `main`, measured vs origin.
        let b = base("clonewt");
        let t = b.join("repo");
        repo_with_farm(&t);
        let host = t.join(".git");
        let c1 = delta_clone(
            &host,
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "d1",
            "repo",
        );
        // The clone moves on (a thread's own work), then makes a worktree.
        task_file(&c1, "hairy", "yaks-dddd", "", "clone work");
        sh(&c1, &["add", "-A"]);
        sh(&c1, &["commit", "-q", "-m", "clone work"]);
        let wt = b.join("wt");
        sh(
            &c1,
            &["worktree", "add", "-q", "--detach", wt.to_str().unwrap()],
        );
        sh(&t, &["update-ref", "refs/remotes/origin/main", "HEAD"]);

        // From inside the clone or its worktree: one primary (the host
        // checkout), the worktree a plain shed. (The host's own list cannot
        // see a worktree registered in a clone's git dir; not this yak.)
        for from in [&c1] {
            let sheds = all_sheds_of(from);
            assert_eq!(paths(&sheds), vec![t.clone(), c1.clone(), wt.clone()]);
            let primaries: Vec<_> = sheds.iter().filter(|s| s.primary).collect();
            assert_eq!(primaries.len(), 1, "{:?}", paths(&sheds));
            assert_eq!(primaries[0].path, t);
            let w = shed_at(&sheds, &wt);
            assert!(!w.primary);
            assert_ne!(w.name(), "main");
            assert_eq!(w.name(), "wt");
        }
        // From the worktree itself: still not the primary, and not named
        // `main`. (Observed: it lists the host checkout and itself, not the
        // clone; that is how discovery already behaved, not this yak.)
        let from_wt = all_sheds_of(&wt);
        assert_eq!(from_wt.iter().filter(|s| s.primary).count(), 1);
        assert!(shed_at(&from_wt, &t).primary);
        assert!(!shed_at(&from_wt, &wt).primary);
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
        let f = wt.join(".yaks/yaks/yaks/yaks-aaaa.md");
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

    fn shed_at<'a>(sheds: &'a [Shed], path: &Path) -> &'a Shed {
        sheds.iter().find(|l| l.path == path).unwrap()
    }

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
        assert!(
            text.contains("\norphaned: 1 pinned clone(s) with no checkout left"),
            "{text}"
        );

        // The normal table is unaffected when there is nothing orphaned.
        assert!(!render(&sheds, &[]).contains("orphaned"));
    }

    /// yaks-28dc: a shed whose commits are in the primary says so, and a
    /// dirty one says what keeps it; `merged` is measured against the
    /// primary's HEAD (here the viewer), never set on the primary's own row.
    #[test]
    fn merged_and_uncommitted_say_what_keeps_a_shed() {
        let b = base("merged-state");
        let t = b.join("repo");
        repo_with_farm(&t);
        let w = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", w.to_str().unwrap()],
        );
        // Merged and clean: nothing keeps it.
        let sheds = all_sheds_of(&t);
        let l = shed_at(&sheds, &w);
        assert_eq!(l.merged, Some(true));
        assert_eq!(state_line(l).as_deref(), Some("merged, clean"));
        assert_eq!(shed_at(&sheds, &t).merged, None, "never on the primary");
        assert_eq!(state_line(shed_at(&sheds, &t)), None);
        let text = render(&sheds, &[]);
        assert!(text.contains("  state: merged, clean\n"), "{text}");
        let j = to_json(&sheds);
        let j = j
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["path"] == w.display().to_string())
            .unwrap();
        assert_eq!((&j["merged"], &j["untracked"]), (&json!(true), &json!(0)));

        // Merged, but uncommitted files keep it: one modified, one untracked.
        fs::write(w.join("README"), "x").unwrap();
        fs::write(store::task_path(&w.join(".yaks"), "yaks-bbbb"), "changed").unwrap();
        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &w);
        assert_eq!((l.dirty, l.untracked), (Some(2), Some(1)));
        assert!(
            state_line(l)
                .unwrap()
                .starts_with("merged \u{b7} uncommitted: 1 modified, 1 untracked"),
            "{:?}",
            state_line(l)
        );
        sh(&w, &["checkout", "-q", "--", ".yaks"]);
        fs::remove_file(w.join("README")).unwrap();

        // A commit of its own: not merged, and clean says nothing.
        fs::write(w.join("mine"), "x").unwrap();
        sh(&w, &["add", "-A"]);
        sh(&w, &["commit", "-q", "-m", "mine"]);
        let sheds = sheds_of(&t);
        let l = shed_at(&sheds, &w);
        assert_eq!(l.merged, Some(false));
        assert_eq!(state_line(l), None);
    }

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

    /// A linked-layout fixture: the primary `t`, a plain git worktree `wt`
    /// and a Delta clone `c`.
    fn three_checkouts(tag: &str) -> (PathBuf, PathBuf, PathBuf) {
        let b = base(tag);
        let t = b.join("repo");
        repo_with_farm(&t);
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        let c = delta_clone(
            &t.join(".git"),
            &t.join(".delta/clones"),
            &t.join(".delta/worktrees"),
            "q7zz",
            "repo",
        );
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
                assert_eq!(a.name(), b.name());
                assert_eq!(a.merged, b.merged, "{}", a.path.display());
            }
        }
        // The text says where you are.
        let from_c = &every[2];
        let text = render(from_c, &[]);
        assert!(text.contains("  name: q7zz (here)\n"), "{text}");
        assert!(text.contains("  name: main\n"), "{text}");
        assert_eq!(
            text.lines().filter(|l| l.starts_with("* ")).count(),
            1,
            "{text}"
        );
    }

    /// yaks-9eaa: a shed's row is the farm's activity log for that checkout
    /// joined to the farm's yaks now: newest write first, `shaving` ones in
    /// progress, an open ask counted, liveness from the last write.
    #[test]
    fn a_sheds_activity_comes_from_the_log_joined_to_the_farm() {
        let b = base("activity");
        let t = b.join("repo");
        repo_with_farm(&t);
        set_status(&t, "yaks-bbbb", "hairy", "shaving");
        let f = store::task_path(&t.join(".yaks"), "yaks-aaaa");
        let text = fs::read_to_string(&f).unwrap();
        fs::write(
            &f,
            text.replacen("type: task\n", "type: task\nneeds: human\n", 1),
        )
        .unwrap();
        let wt = b.join("wt");
        sh(
            &t,
            &["worktree", "add", "-q", "-b", "shed", wt.to_str().unwrap()],
        );
        let farm = t.join(".yaks");
        let now = Utc::now();
        let ago = |mins: i64| {
            (now - chrono::Duration::minutes(mins))
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
        };
        activity::record_for(&farm, &wt, "yaks-cccc", &ago(30)).unwrap();
        activity::record_for(&farm, &wt, "yaks-bbbb", &ago(20)).unwrap();
        activity::record_for(&farm, &wt, "yaks-aaaa", &ago(10)).unwrap();
        activity::record_for(&farm, &wt, "yaks-cccc", &ago(5)).unwrap();
        activity::record_for(&farm, &wt, "yaks-gone", &ago(40)).unwrap();
        let sheds = all_sheds_of(&t);
        let w = shed_at(&sheds, &wt);
        let ids: Vec<&str> = w.touched.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, ["yaks-cccc", "yaks-aaaa", "yaks-bbbb", "yaks-gone"]);
        assert_eq!(w.touched[0].status, Some(Status::Shorn));
        assert_eq!(w.touched[3].status, None, "a yak no longer in the farm");
        assert_eq!(w.in_progress, ["yaks-bbbb"]);
        assert_eq!(w.open_asks().count(), 1);
        assert_eq!(w.last_write.as_deref(), Some(ago(5).as_str()));
        assert_eq!(w.liveness, Liveness::Active);
        // The primary wrote nothing: idle, no label.
        let p = shed_at(&sheds, &t);
        assert!(p.touched.is_empty());
        assert_eq!((p.liveness, label_line(p)), (Liveness::Idle, None));
        let text = render(&sheds, &[]);
        assert!(
            text.contains("  shaving: yaks-bbbb \u{b7} 1 asking \u{b7} 4 yaks \u{b7} active\n"),
            "{text}"
        );
        assert!(
            text.contains("    \u{23f3} yaks-aaaa  hairy  yaks-aaaa title"),
            "{text}"
        );
        assert!(text.contains("    yaks-gone  gone"), "{text}");
        let j = to_json(&sheds);
        let jw = j
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["path"] == wt.display().to_string())
            .unwrap();
        assert_eq!(jw["touched"][0]["id"], "yaks-cccc");
        assert_eq!(jw["touched"][0]["status"], "shorn");
        assert_eq!(jw["liveness"], "active");
    }

    /// A claim with no recent write is stale; without a claim it is idle.
    #[test]
    fn liveness_is_active_stale_or_idle() {
        let mk = |last: &str, status: Status| {
            let tasks = [crate::model::Task {
                id: "y-1".into(),
                title: "t".into(),
                kind: "task".into(),
                priority: 3,
                status,
                created: None,
                updated: None,
                parent: None,
                labels: vec![],
                depends_on: vec![],
                source: None,
                needs: None,
                verify: None,
                extra: Vec::new(),
                body: String::new(),
            }];
            let by_id: HashMap<&str, &Task> = tasks.iter().map(|t| (t.id.as_str(), t)).collect();
            let mut s = Shed {
                path: PathBuf::from("/x"),
                kind: ShedKind::Worktree,
                dir: None,
                title: None,
                slug: None,
                primary: false,
                current: false,
                branch: None,
                head: None,
                ahead: None,
                behind: None,
                shallow_here: false,
                dirty: None,
                untracked: None,
                merged: None,
                touched: vec![],
                in_progress: vec![],
                last_write: None,
                liveness: Liveness::Idle,
                error: None,
            };
            let e = [activity::Entry {
                ts: last.into(),
                id: "y-1".into(),
            }];
            let now =
                SystemTime::from(DateTime::parse_from_rfc3339("2026-01-01T12:00:00Z").unwrap());
            apply_activity(&mut s, &e, &by_id, now);
            s.liveness
        };
        assert_eq!(mk("2026-01-01T11:00:00Z", Status::Hairy), Liveness::Active);
        assert_eq!(mk("2026-01-01T08:00:00Z", Status::Shaving), Liveness::Stale);
        assert_eq!(mk("2026-01-01T08:00:00Z", Status::Shorn), Liveness::Idle);
    }

    /// The farm branch's own worktree holds the farm: never a shed row.
    #[test]
    fn the_farm_branch_worktree_is_not_a_shed() {
        let b = base("farmwt");
        let t = b.join("repo");
        fs::create_dir_all(&t).unwrap();
        sh(&t, &["init", "-q"]);
        fs::write(t.join("README"), "code").unwrap();
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-q", "-m", "code"]);
        let farm_wt = b.join("farm");
        sh(
            &t,
            &[
                "worktree",
                "add",
                "-q",
                "--orphan",
                "-b",
                "yaks",
                farm_wt.to_str().unwrap(),
            ],
        );
        task_file(&farm_wt, "hairy", "yaks-aaaa", "", "x");
        let other = b.join("wt");
        sh(
            &t,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "shed",
                other.to_str().unwrap(),
            ],
        );
        let sheds = discover(&t, &farm_wt.join(".yaks")).unwrap();
        assert_eq!(
            paths(&sheds),
            vec![t.clone(), other.clone()],
            "{}",
            render(&sheds, &[])
        );
    }

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
    fn a_titled_clone_is_named_by_its_slug() {
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
        let j = to_json(&from_t);
        assert_eq!(j[0]["title"], "cflag-1: yaks -C global flag");
        assert_eq!(j[0]["name"], "cflag-1-yaks-c-global");
        assert_eq!(j[1]["title"], Value::Null);
        assert!(render(&from_t, &[]).contains(
            "  name: cflag-1-yaks-c-global \u{b7} title: cflag-1: yaks -C global flag\n"
        ));
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
}
