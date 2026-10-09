//! `yaks discover`: a diagnostic that shows what every shed-discovery source
//! finds from one anchor, and why each candidate was accepted or rejected.
//!
//! It exists to check the discovery model against real machines before the
//! sources are wired into `yaks sheds` / a `--shed` selector (yaks-7eea,
//! yaks-2177). Read-only, and it needs no farm: point it at any directory.
//!
//! Sources, each reported separately:
//! 1. `git worktree list` from the anchor's checkout.
//! 2. The Delta sibling scan `yaks sheds` uses today
//!    ([`crate::sheds::delta_candidates`]).
//! 3. The Delta roots: the platform's managed-checkout root
//!    ([`crate::sheds::default_delta_roots`]) plus any `--root`. Every entry
//!    is listed; when the anchor is in a repo, each is matched against it.
//!
//! Every candidate goes through the one rule `yaks sheds` uses
//! ([`crate::sheds::repo_match`]), so the verdicts here are the verdicts there.

use crate::sheds::{
    alternates, canonical, common_dir, default_delta_roots, delta_candidates, delta_root_entries,
    normalize_url, origin_url, repo_match, worktree_entries,
};
use crate::store;
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What the anchor directory is.
#[derive(Debug)]
pub struct Anchor {
    pub path: PathBuf,
    /// The git top-level, when the anchor is inside a checkout.
    pub top: Option<PathBuf>,
    pub common: Option<PathBuf>,
    pub layout: Option<String>,
    pub origin: Option<String>,
    pub alternates: Option<PathBuf>,
    /// `local` remote URL, and whether it is a non-bare checkout.
    pub local: Option<(String, Option<bool>)>,
    /// The farm discovery resolves from the anchor, or why none.
    pub farm: Result<PathBuf, String>,
}

/// One candidate a source produced, with its verdict.
#[derive(Debug)]
pub struct Candidate {
    pub path: PathBuf,
    /// `Ok(how)` = a shed of the anchor's repo; `Err(why)` = rejected.
    pub verdict: Result<String, String>,
    pub origin: Option<String>,
}

#[derive(Debug)]
pub struct Source {
    pub name: &'static str,
    /// Where the source looked (roots, the command).
    pub looked: Vec<String>,
    pub candidates: Vec<Candidate>,
}

#[derive(Debug)]
pub struct Report {
    pub anchor: Anchor,
    pub sources: Vec<Source>,
}

/// Run every source from `anchor`, adding `extra_roots` to the Delta roots.
pub fn run(anchor: &Path, extra_roots: &[PathBuf], yaks_dir: Option<&str>) -> Report {
    let mut roots = default_delta_roots();
    roots.extend(extra_roots.iter().cloned());
    let anchor = inspect_anchor(anchor, yaks_dir, &roots);
    let mut sources = Vec::new();

    if let (Some(top), Some(common)) = (&anchor.top, &anchor.common) {
        let wt = worktree_entries(top)
            .into_iter()
            .map(|(p, v)| {
                let p = canonical(&p);
                let verdict = if &p == top {
                    Err("the anchor itself".into())
                } else {
                    v.map(|()| "listed by git".to_string())
                };
                Candidate {
                    origin: None,
                    path: p,
                    verdict,
                }
            })
            .collect();
        sources.push(Source {
            name: "git worktree list",
            looked: vec![format!("git -C {} worktree list", top.display())],
            candidates: wt,
        });

        let mut sib: Vec<Candidate> = delta_candidates(top)
            .into_iter()
            .map(|p| canonical(&p))
            .filter(|p| p != top)
            .map(|p| judge(common, p))
            .collect();
        sib.sort_by(|a, b| a.path.cmp(&b.path));
        sib.dedup_by(|a, b| a.path == b.path);
        sources.push(Source {
            name: "Delta sibling scan",
            looked: sibling_globs(top),
            candidates: sib,
        });
    }

    let mut looked = Vec::new();
    let mut cands = Vec::new();
    for r in &roots {
        let exists = r.is_dir();
        looked.push(format!(
            "{}{}",
            r.display(),
            if exists { "" } else { "  (does not exist)" }
        ));
        for e in delta_root_entries(r) {
            let e = canonical(&e);
            let c = match (&anchor.top, &anchor.common) {
                (Some(top), _) if &e == top => Candidate {
                    origin: None,
                    path: e,
                    verdict: Err("the anchor itself".into()),
                },
                (_, Some(common)) => judge(common, e),
                _ => unanchored(e),
            };
            cands.push(c);
        }
    }
    sources.push(Source {
        name: "Delta roots",
        looked,
        candidates: cands,
    });

    Report { anchor, sources }
}

fn judge(common: &Path, path: PathBuf) -> Candidate {
    let origin = common_dir(&path).and_then(|c| origin_url(&c));
    let verdict = if !path.join(".git").exists() {
        Err("no .git".into())
    } else {
        repo_match(common, &path).map(|how| format!("same repo ({how})"))
    };
    Candidate {
        path,
        verdict,
        origin,
    }
}

/// With no anchor repo there is nothing to match: report each entry's repo.
fn unanchored(path: PathBuf) -> Candidate {
    if !path.join(".git").exists() {
        return Candidate {
            path,
            verdict: Err("no .git".into()),
            origin: None,
        };
    }
    let origin = common_dir(&path).and_then(|c| origin_url(&c));
    Candidate {
        verdict: Ok("a checkout (no anchor repo to match)".into()),
        path,
        origin,
    }
}

fn sibling_globs(top: &Path) -> Vec<String> {
    let name = top
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut v = Vec::new();
    if let Some(root) = top.parent().and_then(Path::parent) {
        v.push(format!("{}/*/{name}", root.display()));
        if root.ends_with(".delta/worktrees") {
            if let Some(repo) = root.parent().and_then(Path::parent) {
                v.push(format!(
                    "{}  (the checkout holding this clone)",
                    repo.display()
                ));
            }
        }
    }
    v.push(format!("{}/.delta/worktrees/*/{name}", top.display()));
    v
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn inspect_anchor(path: &Path, yaks_dir: Option<&str>, roots: &[PathBuf]) -> Anchor {
    let path = canonical(path);
    let farm = store::discover_with(&path, yaks_dir)
        .map(|d| d.root)
        .map_err(|e| format!("{e:#}"));
    let top = git(&path, &["rev-parse", "--show-toplevel"]).map(|t| canonical(Path::new(&t)));
    let Some(top) = top else {
        return Anchor {
            path,
            top: None,
            common: None,
            layout: None,
            origin: None,
            alternates: None,
            local: None,
            farm,
        };
    };
    let common = common_dir(&top);
    let local = git(&top, &["config", "--get", "remote.local.url"]).map(|u| {
        let checkout = Path::new(&u);
        let bare = git(checkout, &["rev-parse", "--is-bare-repository"]);
        (u.clone(), bare.map(|b| b == "false"))
    });
    Anchor {
        layout: Some(layout(&top, common.as_deref(), roots)),
        origin: common.as_deref().and_then(origin_url),
        alternates: common.as_deref().and_then(alternates),
        path,
        top: Some(top),
        common,
        local,
        farm,
    }
}

/// A one-line description of what kind of checkout `top` is.
fn layout(top: &Path, common: Option<&Path>, roots: &[PathBuf]) -> String {
    let dot = top.join(".git");
    if dot.is_dir() {
        return "primary checkout (.git is a directory)".into();
    }
    let git_dir = git(top, &["rev-parse", "--absolute-git-dir"]).map(|d| canonical(Path::new(&d)));
    if let (Some(g), Some(c)) = (&git_dir, common) {
        if g != c {
            let main = c.parent().unwrap_or(c);
            return format!("git worktree of {}", main.display());
        }
    }
    let s = top.display().to_string();
    if let Some(i) = s.find("/.delta/worktrees/") {
        return format!("Delta clone, linked layout (under {})", &s[..i]);
    }
    if roots.iter().any(|r| top.starts_with(canonical(r))) {
        return "Delta clone, managed layout".into();
    }
    "checkout with a separate git dir".into()
}

// -- rendering ----------------------------------------------------------------

pub fn render(r: &Report) -> String {
    let mut s = String::new();
    let a = &r.anchor;
    let _ = writeln!(s, "ANCHOR  {}", a.path.display());
    match &a.top {
        None => {
            let _ = writeln!(s, "  not in a git repository: only the Delta roots apply");
        }
        Some(top) => {
            let _ = writeln!(s, "  top-level   {}", top.display());
            if let Some(l) = &a.layout {
                let _ = writeln!(s, "  layout      {l}");
            }
            if let Some(c) = &a.common {
                let _ = writeln!(s, "  git dir     {}", c.display());
            }
            let _ = writeln!(
                s,
                "  origin      {}",
                a.origin.as_deref().unwrap_or("(none)")
            );
            if let Some(alt) = &a.alternates {
                let _ = writeln!(s, "  alternates  {}", alt.display());
            }
            if let Some((u, checkout)) = &a.local {
                let what = match checkout {
                    Some(true) => "a checkout",
                    Some(false) => "BARE",
                    None => "unreachable",
                };
                let _ = writeln!(s, "  local       {u}  ({what})");
            }
        }
    }
    match &a.farm {
        Ok(f) => {
            let _ = writeln!(s, "  farm        {}", f.display());
        }
        Err(e) => {
            let _ = writeln!(s, "  farm        (none: {e})");
        }
    }

    let anchored = a.top.is_some();
    for src in &r.sources {
        let found = src.candidates.iter().filter(|c| c.verdict.is_ok()).count();
        let _ = writeln!(
            s,
            "\n{}  ({found} found, {} rejected)",
            src.name.to_uppercase(),
            src.candidates.len() - found
        );
        for l in &src.looked {
            let _ = writeln!(s, "  looked in  {l}");
        }
        for c in &src.candidates {
            let (mark, why) = match &c.verdict {
                Ok(how) => ("+", how.as_str()),
                Err(why) => ("-", why.as_str()),
            };
            let _ = writeln!(s, "  {mark} {}", c.path.display());
            let origin = match (&c.origin, anchored) {
                (Some(o), false) => format!("  origin {o}"),
                _ => String::new(),
            };
            let _ = writeln!(s, "      {why}{origin}");
        }
    }

    let union = union(r);
    if anchored {
        let _ = writeln!(s, "\nSHEDS  ({} distinct, anchor excluded)", union.len());
        for (p, by) in &union {
            let _ = writeln!(s, "  {}  [{}]", p.display(), by.join(", "));
        }
    } else {
        let groups = by_origin(r);
        let _ = writeln!(s, "\nREPOS  ({} by origin)", groups.len());
        for (o, ps) in &groups {
            let _ = writeln!(s, "  {o}");
            for p in ps {
                let _ = writeln!(s, "    {}", p.display());
            }
        }
    }
    s
}

/// Accepted candidates, one per path, with the sources that found each.
fn union(r: &Report) -> BTreeMap<PathBuf, Vec<&'static str>> {
    let mut m: BTreeMap<PathBuf, Vec<&'static str>> = BTreeMap::new();
    for src in &r.sources {
        for c in src.candidates.iter().filter(|c| c.verdict.is_ok()) {
            m.entry(c.path.clone()).or_default().push(src.name);
        }
    }
    m
}

/// Unanchored: accepted checkouts grouped by normalized origin URL.
fn by_origin(r: &Report) -> BTreeMap<String, Vec<PathBuf>> {
    let mut m: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for c in r.sources.iter().flat_map(|s| &s.candidates) {
        if c.verdict.is_ok() {
            let key = c
                .origin
                .as_deref()
                .map(normalize_url)
                .unwrap_or_else(|| "(no origin)".into());
            m.entry(key).or_default().push(c.path.clone());
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn base(tag: &str) -> PathBuf {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "yaks-discover-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        canonical(&p)
    }

    fn sh(dir: &Path, args: &[&str]) {
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
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// A primary checkout at `b/repo` (origin `url`), a managed bare repo, and
    /// a managed-layout clone `b/root/m1/local_u1` whose git dir sits beside
    /// it and whose origin is spelled `managed_url`.
    fn layout(b: &Path, url: &str, managed_url: &str) -> (PathBuf, PathBuf, PathBuf) {
        let repo = b.join("repo");
        fs::create_dir_all(&repo).unwrap();
        sh(&repo, &["init", "-q"]);
        sh(&repo, &["remote", "add", "origin", url]);
        fs::write(repo.join("f"), "x").unwrap();
        sh(&repo, &["add", "-A"]);
        sh(&repo, &["commit", "-qm", "init"]);
        let bare = b.join("managed.git");
        sh(
            b,
            &[
                "clone",
                "-q",
                "--bare",
                repo.to_str().unwrap(),
                bare.to_str().unwrap(),
            ],
        );
        let root = b.join("root");
        let clone = root.join("m1/local_u1");
        fs::create_dir_all(clone.parent().unwrap()).unwrap();
        let gd = root.join("m1/local_u1.git");
        sh(
            b,
            &[
                "clone",
                "-q",
                "--shared",
                "--separate-git-dir",
                gd.to_str().unwrap(),
                bare.to_str().unwrap(),
                clone.to_str().unwrap(),
            ],
        );
        sh(&clone, &["remote", "set-url", "origin", managed_url]);
        (repo, root, clone)
    }

    /// Accepted paths from `source`, limited to the test's own `b` (the real
    /// platform Delta root is always scanned too).
    fn accepted(r: &Report, source: &str, b: &Path) -> Vec<PathBuf> {
        r.sources
            .iter()
            .filter(|s| s.name == source)
            .flat_map(|s| &s.candidates)
            .filter(|c| c.verdict.is_ok() && c.path.starts_with(b))
            .map(|c| c.path.clone())
            .collect()
    }

    #[test]
    fn the_root_scan_finds_a_managed_clone_from_the_primary_checkout() {
        let b = base("primary");
        let url = format!(
            "https://example.com/{}/r.git",
            b.file_name().unwrap().to_string_lossy()
        );
        // Delta records the URL without `.git`; still the same repository.
        let (repo, root, clone) = layout(&b, &url, url.strip_suffix(".git").unwrap());
        let r = run(&repo, &[root.clone()], Some(""));
        assert_eq!(accepted(&r, "Delta roots", &b), vec![clone.clone()]);
        // The sibling scan alone (what `yaks sheds` uses today) misses it.
        assert!(!accepted(&r, "Delta sibling scan", &b).contains(&clone));
        // The `.git` dir beside the checkout is never a candidate.
        let all: Vec<_> = r.sources.iter().flat_map(|s| &s.candidates).collect();
        assert!(
            all.iter()
                .all(|c| !c.path.to_string_lossy().ends_with(".git"))
        );
    }

    #[test]
    fn differing_alternates_fall_through_to_origin() {
        let b = base("alt");
        let url = format!(
            "https://example.com/{}/r.git",
            b.file_name().unwrap().to_string_lossy()
        );
        let (repo, _root, clone) = layout(&b, &url, &url);
        // A linked-layout clone borrows the primary's objects, the managed
        // clone the managed bare repo's: different alternates, one repo.
        let linked = b.join("linked");
        sh(
            &b,
            &[
                "clone",
                "-q",
                "--shared",
                repo.to_str().unwrap(),
                linked.to_str().unwrap(),
            ],
        );
        sh(&linked, &["remote", "set-url", "origin", &url]);
        let own = common_dir(&linked).unwrap();
        assert_eq!(repo_match(&own, &clone), Ok("origin"));
    }

    #[test]
    fn a_different_repository_is_rejected_with_its_origin() {
        let b = base("other");
        let url = format!(
            "https://example.com/{}/r.git",
            b.file_name().unwrap().to_string_lossy()
        );
        let (repo, root, _clone) = layout(&b, &url, "https://example.com/elsewhere/other");
        let r = run(&repo, &[root], Some(""));
        assert!(accepted(&r, "Delta roots", &b).is_empty());
        let src = r.sources.iter().find(|s| s.name == "Delta roots").unwrap();
        let c = src
            .candidates
            .iter()
            .find(|c| c.path.starts_with(&b))
            .unwrap();
        assert!(c.verdict.as_ref().unwrap_err().contains("elsewhere/other"));
    }

    #[test]
    fn outside_a_repository_every_checkout_is_listed_by_origin() {
        let b = base("anywhere");
        let url = format!(
            "https://example.com/{}/r.git",
            b.file_name().unwrap().to_string_lossy()
        );
        let (_repo, root, clone) = layout(&b, &url, &url);
        let orphan = root.join("m2/local_u2");
        fs::create_dir_all(&orphan).unwrap();
        let nowhere = b.join("nowhere");
        fs::create_dir_all(&nowhere).unwrap();
        let r = run(&nowhere, &[root], Some(""));
        assert!(r.anchor.top.is_none());
        assert_eq!(accepted(&r, "Delta roots", &b), vec![clone.clone()]);
        let groups = by_origin(&r);
        // Keyed by normalized origin; real checkouts may add other groups.
        assert_eq!(groups.get(&normalize_url(&url)), Some(&vec![clone]));
        let text = render(&r);
        assert!(text.contains("REPOS") && !text.contains("SHEDS"));
    }
}

pub fn to_json(r: &Report) -> Value {
    let a = &r.anchor;
    let p = |p: &Option<PathBuf>| p.as_ref().map(|p| p.display().to_string());
    json!({
        "anchor": {
            "path": a.path.display().to_string(),
            "top": p(&a.top),
            "layout": a.layout,
            "git_dir": p(&a.common),
            "origin": a.origin,
            "alternates": p(&a.alternates),
            "local": a.local.as_ref().map(|(u, c)| json!({"url": u, "checkout": c})),
            "farm": a.farm.as_ref().ok().map(|f| f.display().to_string()),
            "farm_error": a.farm.as_ref().err(),
        },
        "sources": r.sources.iter().map(|s| json!({
            "name": s.name,
            "looked": s.looked,
            "candidates": s.candidates.iter().map(|c| json!({
                "path": c.path.display().to_string(),
                "accepted": c.verdict.is_ok(),
                "reason": match &c.verdict { Ok(x) | Err(x) => x },
                "origin": c.origin,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "sheds": union(r).into_iter().map(|(p, by)| json!({
            "path": p.display().to_string(),
            "sources": by,
        })).collect::<Vec<_>>(),
    })
}
