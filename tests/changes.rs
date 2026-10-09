//! `yaks changes <shed>` through the real binary: a worker shed's per-yak
//! changes with actors, and `resolve`'s candidate list (exit 1) for a name that
//! matches no shed.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
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

fn yaks(dir: &Path, actor: &str, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(dir)
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", actor)
        .env_remove("YAKS_DIR")
        .args(args)
        .output()
        .unwrap()
}

fn ok(dir: &Path, actor: &str, args: &[&str]) -> String {
    let out = yaks(dir, actor, args);
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// A committed repo with two yaks (`first` shaving, `second` hairy) and a
/// worktree shed `wt` on branch `work`.
fn repo(tag: &str) -> (PathBuf, PathBuf, String, String) {
    let base = std::env::temp_dir().join(format!("yaks-changes-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let t = base.join("repo");
    std::fs::create_dir_all(&t).unwrap();
    let t = t.canonicalize().unwrap();
    git(&t, &["init", "-q"]);
    ok(&t, "coord", &["init"]);
    let id = |title: &str| {
        let v: serde_json::Value =
            serde_json::from_str(&ok(&t, "coord", &["create", title, "--json"])).unwrap();
        v["id"].as_str().unwrap().to_string()
    };
    let (first, second) = (id("first"), id("second"));
    ok(&t, "coord", &["shave", &first]);
    git(&t, &["add", "-A"]);
    git(&t, &["commit", "-q", "-m", "init"]);
    let wt = t.parent().unwrap().join("wt");
    git(
        &t,
        &["worktree", "add", "-q", "-b", "work", wt.to_str().unwrap()],
    );
    (t, wt, first, second)
}

#[test]
fn a_shed_that_created_moved_and_noted_is_listed_per_yak_with_actors() {
    let (t, wt, first, second) = repo("per-yak");
    ok(
        &wt,
        "w-1",
        &["update", &first, "--note", "worker one\nsecond line"],
    );
    ok(&wt, "w-2", &["update", &first, "--note", "worker two"]);
    ok(&wt, "w-2", &["shave", &second]);
    let new: serde_json::Value =
        serde_json::from_str(&ok(&wt, "w-1", &["create", "made here", "--json"])).unwrap();
    let new = new["id"].as_str().unwrap();

    let out = ok(&t, "me", &["changes", "work"]);
    assert!(
        out.contains(&format!("  {first}  first  [shaving]\n")),
        "{out}"
    );
    assert!(out.contains("  w-1  worker one\n"), "{out}");
    assert!(out.contains("  w-2  worker two\n"), "{out}");
    assert!(!out.contains("second line"), "{out}");
    assert!(
        out.contains(&format!("  {second}  second  [hairy -> shaving]\n")),
        "{out}"
    );
    assert!(out.contains("  w-2  moved: hairy -> shaving\n"), "{out}");
    assert!(
        out.contains(&format!("  {new}  made here  [new, hairy]\n")),
        "{out}"
    );
    assert!(
        !out.contains("coord"),
        "the coordinator's claim is before the fork: {out}"
    );

    let j: Vec<serde_json::Value> =
        serde_json::from_str(&ok(&t, "me", &["changes", "w-1", "--json"])).unwrap();
    assert_eq!(j.len(), 3, "{j:?}");
    let actors: Vec<_> = j.iter().find(|y| y["id"] == first.as_str()).unwrap()["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["actor"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(actors, ["w-1", "w-2"]);

    // From the shed, `main` is the primary checkout: nothing of its own yet.
    let out = ok(&wt, "me", &["changes", "main"]);
    assert!(out.contains("no changes of its own"), "{out}");
}

#[test]
fn an_unknown_shed_lists_the_candidates_and_exits_1() {
    let (t, _wt, _, _) = repo("unknown");
    let out = yaks(&t, "me", &["changes", "nope"]);
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(
        err.contains("no shed matches `nope`; the sheds are:"),
        "{err}"
    );
    assert!(err.contains("/wt"), "{err}");
    assert!(out.stdout.is_empty());
}
