//! `yaks sheds` builds one row per ACTOR from the farm's notes (yaks-da23):
//! who wrote what, the claim on a `shaving` yak, asks and liveness, whatever
//! checkout a command ran in. Driven through the real binary on a farm-branch
//! repo (the farm in its own worktree, shared by every checkout), as on a
//! real machine.

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

fn yaks_as(actor: &str, cwd: &Path, args: &[&str]) -> String {
    let out = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(cwd)
        .env_remove("YAKS_DIR")
        .env_remove("DELTA_THREAD_TITLE")
        .env_remove("DELTA_CURRENT_THREAD_ID")
        .env("YAKS_NO_BOOTSTRAP", "1")
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", actor)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8(out.stdout).unwrap()
}

fn yaks(cwd: &Path, args: &[&str]) -> String {
    yaks_as("tester", cwd, args)
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

fn shed<'a>(j: &'a Value, name: &str) -> &'a Value {
    j["sheds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == name)
        .unwrap_or_else(|| panic!("no shed {name}: {j}"))
}

#[test]
fn each_actor_is_credited_with_its_own_writes_from_any_checkout() {
    let (t, farm, wt) = setup("credit");
    // `boss` only creates a yak (no note, no move: not a row); `w1` writes
    // from the other checkout and asks; `solo` shaves a different yak.
    let a = id_of(&yaks_as("boss", &t, &["create", "claimed", "--json"]));
    let b = id_of(&yaks_as("solo", &wt, &["create", "own", "--json"]));
    yaks_as("solo", &wt, &["shave", &b]);
    yaks_as("w1", &wt, &["update", &a, "--note", "started"]);
    yaks_as("w1", &wt, &["ask", &a, "--note", "which way?"]);

    let j: Value = serde_json::from_str(&yaks(&t, &["sheds", "--json"])).unwrap();
    let names: Vec<_> = j["sheds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    // Open ask first, then the other active actors by most recent write.
    assert_eq!(names[0], "w1", "{names:?}");
    assert_eq!(names, ["w1", "solo"], "{names:?}");
    // No upstream: no farm freshness.
    assert!(j["farm"].is_null(), "{j}");

    let w1 = shed(&j, "w1");
    assert_eq!(w1["asking"], serde_json::json!([a]));
    assert_eq!(w1["touched"][0]["needs"], "human");
    assert_eq!(w1["liveness"], "active");
    // `solo` holds its own claim; the yak `a` has no `shaving` move, so
    // nobody holds it.
    assert_eq!(shed(&j, "solo")["working"], serde_json::json!([b]));
    assert_eq!(w1["working"], serde_json::json!([]));

    // The same answer from the other checkout, and from the farm's own.
    let from_wt = yaks(&wt, &["sheds", "--json"]);
    assert_eq!(from_wt, yaks(&t, &["sheds", "--json"]));
    assert_eq!(from_wt, yaks(&farm, &["sheds", "--json"]));

    let text = yaks(&t, &["sheds"]);
    assert!(text.starts_with("w1  "), "{text}");
    assert!(
        text.contains(&format!("asking   {a}  hairy  claimed")),
        "{text}"
    );
    assert!(text.contains(&format!("shaving  {b}  own")), "{text}");
    assert!(!text.contains(&wt.display().to_string()), "{text}");

    // Writes leave no machine-local log behind (the activity log is gone).
    assert!(!farm.join(".yaks/.activity").exists());
}
