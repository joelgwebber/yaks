//! `yaks discover`: the shed-discovery chain `yaks sheds` uses, shown step by
//! step from one checkout, so you can see what is found and why (yaks-c635).
//!
//! The chain (see the `sheds` module docs): this checkout's own git dir; the
//! host repository its `objects/info/alternates` names; every
//! `refs/delta/<dir>/<name>/*` pin group there, resolved to a checkout through
//! the clone's git dir `core.worktree` (or reported gone); and, when the host
//! is a checkout, that checkout's `git worktree list`. Git data only: nothing
//! is searched for, so it must be run from inside a checkout of the repo.
//! Read-only, and it needs no farm.

use crate::sheds::{
    Host, Pinned, ShedKind, canonical, host, host_checkout, origin_url, pinned, shed_paths,
    worktree_entries,
};
use crate::store;
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What `yaks discover` saw from one checkout.
#[derive(Debug)]
pub struct Report {
    pub anchor: PathBuf,
    pub top: PathBuf,
    pub layout: String,
    pub origin: Option<String>,
    /// `local` remote URL, and whether it is a non-bare checkout.
    pub local: Option<(String, Option<bool>)>,
    /// The farm discovery resolves from the anchor, or why none.
    pub farm: Result<PathBuf, String>,
    pub host: Option<Host>,
    /// The host's checkout, when the host is not bare.
    pub host_checkout: Option<PathBuf>,
    /// `git worktree list` here: each entry and why it is not a checkout.
    pub worktrees_here: Vec<(PathBuf, Result<(), String>)>,
    /// `git worktree list` in the host checkout (empty when it is this repo).
    pub worktrees_host: Vec<(PathBuf, Result<(), String>)>,
    pub pinned: Vec<Pinned>,
    /// The resulting sheds, exactly as `yaks sheds` finds them.
    pub sheds: BTreeMap<PathBuf, ShedKind>,
}

/// Run the chain from `anchor` (any directory inside a checkout).
pub fn run(anchor: &Path, yaks_dir: Option<&str>) -> Result<Report> {
    let anchor = canonical(anchor);
    let Some(top) = git(&anchor, &["rev-parse", "--show-toplevel"]) else {
        bail!(
            "{} is not inside a git checkout. yaks discovers sheds from inside one \
             (a primary checkout, a git worktree, or a Delta clone); cd into one or pass its path",
            anchor.display()
        );
    };
    let top = canonical(Path::new(&top));
    let farm = store::discover_with(&anchor, yaks_dir)
        .map(|d| d.root)
        .map_err(|e| format!("{e:#}"));
    let local = git(&top, &["config", "--get", "remote.local.url"]).map(|u| {
        let bare = git(Path::new(&u), &["rev-parse", "--is-bare-repository"]);
        (u.clone(), bare.map(|b| b == "false"))
    });
    let host = host(&top);
    let host_checkout = host.as_ref().and_then(host_checkout);
    let worktrees_host = match &host_checkout {
        Some(h) if host.as_ref().is_some_and(|h| h.via_alternates) => worktree_entries(h),
        _ => Vec::new(),
    };
    Ok(Report {
        layout: layout(&top, host.as_ref()),
        origin: host.as_ref().and_then(|h| origin_url(&h.own_git_dir)),
        pinned: host.as_ref().map(pinned).unwrap_or_default(),
        worktrees_here: worktree_entries(&top),
        sheds: shed_paths(&top),
        anchor,
        top,
        local,
        farm,
        host,
        host_checkout,
        worktrees_host,
    })
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

/// A one-line description of what kind of checkout `top` is.
fn layout(top: &Path, host: Option<&Host>) -> String {
    let Some(h) = host else {
        return "unreadable git state".into();
    };
    if top.join(".git").is_dir() {
        return "primary checkout (.git is a directory)".into();
    }
    if h.via_alternates {
        return if h.bare {
            "Delta clone; host is a bare repo (Delta-managed: a shared-thread machine)".into()
        } else {
            "Delta clone; host is a checkout (the project was added from a folder)".into()
        };
    }
    match crate::sheds::common_dir(top) {
        Some(c) if c != h.own_git_dir => {
            format!("git worktree of {}", c.parent().unwrap_or(&c).display())
        }
        _ => "checkout with a separate git dir".into(),
    }
}

// -- rendering ----------------------------------------------------------------

fn entry_line(s: &mut String, top: &Path, p: &Path, v: &Result<(), String>) {
    let p = canonical(p);
    let (mark, why) = match v {
        _ if p == top => ("-", "this checkout".to_string()),
        Ok(()) => ("+", "a checkout".to_string()),
        Err(e) => ("-", e.clone()),
    };
    let _ = writeln!(s, "  {mark} {}\n      {why}", p.display());
}

pub fn render(r: &Report) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "CHECKOUT  {}", r.top.display());
    if r.anchor != r.top {
        let _ = writeln!(s, "  from        {}", r.anchor.display());
    }
    let _ = writeln!(s, "  layout      {}", r.layout);
    let _ = writeln!(
        s,
        "  origin      {}",
        r.origin.as_deref().unwrap_or("(none)")
    );
    if let Some((u, checkout)) = &r.local {
        let what = match checkout {
            Some(true) => "a checkout",
            Some(false) => "BARE",
            None => "unreachable",
        };
        let _ = writeln!(s, "  local       {u}  ({what})");
    }
    match &r.farm {
        Ok(f) => {
            let _ = writeln!(s, "  farm        {}", f.display());
        }
        Err(e) => {
            let _ = writeln!(s, "  farm        (none: {e})");
        }
    }

    let _ = writeln!(s, "\n1. GIT WORKTREE LIST (here)");
    for (p, v) in &r.worktrees_here {
        entry_line(&mut s, &r.top, p, v);
    }

    let _ = writeln!(s, "\n2. HOST");
    match &r.host {
        None => {
            let _ = writeln!(s, "  (could not read this checkout's git dir)");
        }
        Some(h) => {
            let _ = writeln!(s, "  own git dir {}", h.own_git_dir.display());
            let how = if h.via_alternates {
                "named by objects/info/alternates"
            } else {
                "this repository: no alternates"
            };
            let kind = if h.bare { "bare" } else { "a checkout" };
            let _ = writeln!(s, "  host        {}  ({how}; {kind})", h.git_dir.display());
            match &h.store {
                Some(st) => {
                    let _ = writeln!(s, "  clone store {}", st.display());
                }
                None => {
                    let _ = writeln!(s, "  clone store (unknown: run from inside a Delta clone)");
                }
            }
        }
    }

    let live = r.pinned.iter().filter(|p| p.checkout.is_ok()).count();
    let _ = writeln!(
        s,
        "\n3. DELTA PINS in the host  ({} pinned dirs: {live} live, {} not)",
        r.pinned.len(),
        r.pinned.len() - live
    );
    for p in &r.pinned {
        let label = format!("{}/{}  ({} pins)", p.dir, p.name, p.pins);
        match &p.checkout {
            Ok(c) if c == &r.top => {
                let _ = writeln!(s, "  - {label}\n      this checkout");
            }
            Ok(c) => {
                let _ = writeln!(s, "  + {label}\n      {}", c.display());
            }
            Err(e) => {
                let _ = writeln!(s, "  - {label}\n      {e}");
            }
        }
    }

    if let Some(hc) = &r.host_checkout {
        if r.host.as_ref().is_some_and(|h| h.via_alternates) {
            let _ = writeln!(s, "\n4. GIT WORKTREE LIST (host checkout {})", hc.display());
            for (p, v) in &r.worktrees_host {
                entry_line(&mut s, &r.top, p, v);
            }
        }
    }

    let _ = writeln!(s, "\nSHEDS  ({}, this checkout excluded)", r.sheds.len());
    for (p, k) in &r.sheds {
        let _ = writeln!(s, "  {}  [{}]", p.display(), k.as_str());
    }
    s
}

pub fn to_json(r: &Report) -> Value {
    let entries = |v: &[(PathBuf, Result<(), String>)]| {
        v.iter()
            .map(|(p, e)| {
                json!({
                    "path": p.display().to_string(),
                    "checkout": e.is_ok(),
                    "reason": e.as_ref().err(),
                })
            })
            .collect::<Vec<_>>()
    };
    json!({
        "checkout": r.top.display().to_string(),
        "anchor": r.anchor.display().to_string(),
        "layout": r.layout,
        "origin": r.origin,
        "local": r.local.as_ref().map(|(u, c)| json!({"url": u, "checkout": c})),
        "farm": r.farm.as_ref().ok().map(|f| f.display().to_string()),
        "farm_error": r.farm.as_ref().err(),
        "host": r.host.as_ref().map(|h| json!({
            "own_git_dir": h.own_git_dir.display().to_string(),
            "git_dir": h.git_dir.display().to_string(),
            "via_alternates": h.via_alternates,
            "bare": h.bare,
            "store": h.store.as_ref().map(|s| s.display().to_string()),
            "checkout": r.host_checkout.as_ref().map(|c| c.display().to_string()),
        })),
        "worktrees_here": entries(&r.worktrees_here),
        "worktrees_host": entries(&r.worktrees_host),
        "pins": r.pinned.iter().map(|p| json!({
            "dir": p.dir,
            "name": p.name,
            "pins": p.pins,
            "checkout": p.checkout.as_ref().ok().map(|c| c.display().to_string()),
            "reason": p.checkout.as_ref().err(),
        })).collect::<Vec<_>>(),
        "sheds": r.sheds.iter().map(|(p, k)| json!({
            "path": p.display().to_string(),
            "kind": k.as_str(),
        })).collect::<Vec<_>>(),
    })
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
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// A primary `b/repo` with one Delta clone `d1` (pinned, live) and one
    /// pin group `gone1` whose clone no longer exists.
    fn linked(b: &Path) -> (PathBuf, PathBuf) {
        let t = b.join("repo");
        fs::create_dir_all(&t).unwrap();
        sh(&t, &["init", "-q"]);
        fs::write(t.join("f"), "x").unwrap();
        sh(&t, &["add", "-A"]);
        sh(&t, &["commit", "-qm", "init"]);
        let head = sh(&t, &["rev-parse", "HEAD"]);
        let co = t.join(".delta/worktrees/d1/repo");
        let gd = t.join(".delta/clones/d1/repo.git");
        fs::create_dir_all(co.parent().unwrap()).unwrap();
        fs::create_dir_all(gd.parent().unwrap()).unwrap();
        sh(
            b,
            &[
                "clone",
                "-q",
                "--shared",
                "--separate-git-dir",
                gd.to_str().unwrap(),
                t.to_str().unwrap(),
                co.to_str().unwrap(),
            ],
        );
        sh(&co, &["config", "core.worktree", co.to_str().unwrap()]);
        for d in ["d1", "gone1"] {
            sh(
                &t,
                &["update-ref", &format!("refs/delta/{d}/repo/{head}"), &head],
            );
        }
        (t, co)
    }

    #[test]
    fn from_a_clone_the_chain_names_host_pins_and_the_primary() {
        let b = base("chain");
        let (t, co) = linked(&b);
        // From a subdirectory of the clone.
        fs::create_dir_all(co.join("sub")).unwrap();
        let r = run(&co.join("sub"), Some("")).unwrap();
        assert_eq!(r.top, co);
        let h = r.host.as_ref().unwrap();
        assert!(h.via_alternates && !h.bare);
        assert_eq!(h.git_dir, canonical(&t.join(".git")));
        assert_eq!(r.host_checkout.as_deref(), Some(t.as_path()));
        assert!(r.layout.starts_with("Delta clone; host is a checkout"));
        let pins: Vec<_> = r
            .pinned
            .iter()
            .map(|p| (p.dir.as_str(), p.checkout.is_ok()))
            .collect();
        assert_eq!(pins, vec![("d1", true), ("gone1", false)]);
        // The sheds are exactly what `yaks sheds` would list.
        assert_eq!(r.sheds.keys().cloned().collect::<Vec<_>>(), vec![t.clone()]);
        let text = render(&r);
        assert!(
            text.contains("4. GIT WORKTREE LIST (host checkout"),
            "{text}"
        );
        assert!(
            text.contains("gone (its git dir no longer exists)"),
            "{text}"
        );
        let j = to_json(&r);
        assert_eq!(j["pins"][1]["checkout"], Value::Null);
        assert_eq!(j["sheds"][0]["kind"], "worktree");
    }

    #[test]
    fn from_the_primary_there_is_no_host_step_and_the_clone_is_found() {
        let b = base("primary");
        let (t, co) = linked(&b);
        let r = run(&t, Some("")).unwrap();
        assert!(!r.host.as_ref().unwrap().via_alternates);
        assert!(r.worktrees_host.is_empty());
        assert_eq!(r.sheds.keys().cloned().collect::<Vec<_>>(), vec![co]);
        assert!(!render(&r).contains("4. GIT WORKTREE LIST"));
    }

    #[test]
    fn outside_a_checkout_it_refuses_and_says_where_to_run_it() {
        let b = base("outside");
        let e = run(&b, Some("")).unwrap_err().to_string();
        assert!(e.contains("not inside a git checkout"), "{e}");
    }
}
