//! `yaks sheds` builds a shed's row from the farm's local activity log
//! (yaks-9eaa): a write from a checkout records that checkout under
//! `<farm>/.activity/`, never committed, and `yaks sheds` lists the yaks each
//! checkout wrote. Driven through the real binary on a farm-branch repo (the
//! farm in its own worktree, shared by every checkout), as on a real machine.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("yaks-shedact-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn yaks(cwd: &Path, args: &[&str]) -> String {
    let out = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(cwd)
        .env_remove("YAKS_DIR")
        .env_remove("DELTA_THREAD_TITLE")
        .env_remove("DELTA_CURRENT_THREAD_ID")
        .env_remove("YAKS_NO_ACTIVITY")
        .env("YAKS_NO_BOOTSTRAP", "1")
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8(out.stdout).unwrap()
}

/// A code repo `t` whose farm is the orphan branch `yaks` checked out at
/// `farm`, plus a second checkout `wt` (a git worktree on a branch).
fn setup(tag: &str) -> (PathBuf, PathBuf, PathBuf) {
    let b = tmp(tag);
    let t = b.join("repo");
    std::fs::create_dir_all(&t).unwrap();
    git(&t, &["init", "-q"]);
    std::fs::write(t.join("README"), "code").unwrap();
    git(&t, &["add", "-A"]);
    git(&t, &["commit", "-q", "-m", "code"]);
    let farm = b.join("farm");
    git(
        &t,
        &[
            "worktree",
            "add",
            "-q",
            "--orphan",
            "-b",
            "yaks",
            farm.to_str().unwrap(),
        ],
    );
    yaks(&farm, &["init", "--herd", "y"]);
    git(&farm, &["add", "-A"]);
    git(&farm, &["commit", "-q", "-m", "farm"]);
    let wt = b.join("wt");
    git(
        &t,
        &["worktree", "add", "-q", "-b", "work", wt.to_str().unwrap()],
    );
    (t, farm, wt)
}

fn id_of(create_json: &str) -> String {
    let v: Value = serde_json::from_str(create_json).unwrap();
    v["id"].as_str().unwrap().to_string()
}

fn row<'a>(j: &'a Value, path: &Path) -> &'a Value {
    j.as_array()
        .unwrap()
        .iter()
        .find(|s| s["path"] == path.display().to_string())
        .unwrap_or_else(|| panic!("no row for {}: {j}", path.display()))
}

#[test]
fn each_checkout_is_credited_with_its_own_writes() {
    let (t, farm, wt) = setup("credit");
    let a = id_of(&yaks(&t, &["create", "from main", "--json"]));
    let b = id_of(&yaks(&wt, &["create", "from the worktree", "--json"]));
    yaks(&wt, &["shave", &b]);
    yaks(&wt, &["ask", &b, "--note", "which way?"]);

    let j: Value = serde_json::from_str(&yaks(&t, &["sheds", "--json"])).unwrap();
    // The farm's own worktree is not a shed.
    assert_eq!(j.as_array().unwrap().len(), 2, "{j}");
    let main = row(&j, &t);
    let ids = |r: &Value| -> Vec<String> {
        r["touched"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["id"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(ids(main), [a]);
    let w = row(&j, &wt);
    assert_eq!(ids(w), std::slice::from_ref(&b));
    assert_eq!(w["in_progress"], serde_json::json!([b]));
    assert_eq!(w["touched"][0]["needs"], "human");
    assert_eq!(w["liveness"], "active");

    // The same answer from the other checkout.
    let from_wt: Value = serde_json::from_str(&yaks(&wt, &["sheds", "--json"])).unwrap();
    assert_eq!(ids(row(&from_wt, &wt)), std::slice::from_ref(&b));
    assert_eq!(row(&from_wt, &wt)["current"], true);

    let text = yaks(&t, &["sheds"]);
    assert!(
        text.contains(&format!(
            "shaving: {b} \u{b7} 1 asking \u{b7} 1 yak \u{b7} active"
        )),
        "{text}"
    );
    assert!(
        text.contains(&format!("\u{23f3} {b}  shaving  from the worktree")),
        "{text}"
    );
    assert!(!text.contains(&farm.display().to_string()), "{text}");

    // The log is local: git in the farm worktree never sees it.
    let status = git(&farm, &["status", "--porcelain", "--untracked-files=all"]);
    assert!(!status.contains(".activity"), "{status}");
    assert!(farm.join(".yaks/.activity").is_dir());
    assert!(
        yaks(&t, &["status"])
            .lines()
            .all(|l| !l.contains(".activity"))
    );
}
