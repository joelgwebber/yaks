//! `yaks status` through the real binary, in a temp git repo with a committed
//! (team) farm: the text, `--check` exit codes and `--json`.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

/// A repo with a committed farm holding one yak; returns it and the yak's id.
fn repo(tag: &str) -> (PathBuf, String) {
    let dir = std::env::temp_dir().join(format!("yaks-status-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "-q"]);
    // `yaks commit` runs a plain `git commit`, which needs an identity; a CI
    // runner has no global one, so the repo carries its own.
    git(&dir, &["config", "user.name", "t"]);
    git(&dir, &["config", "user.email", "t@t"]);
    yaks(&dir, &["init"]);
    let created = yaks(&dir, &["create", "first"]);
    let id = created.split([' ', ':']).nth(1).unwrap().to_string();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "init"]);
    (dir, id)
}

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(dir)
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
        .args(args)
        .output()
        .unwrap()
}

fn yaks(dir: &Path, args: &[&str]) -> String {
    let out = run(dir, args);
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn clean_then_dirty_with_check_exit_codes() {
    let (dir, id) = repo("check");
    assert_eq!(yaks(&dir, &["status"]), "farm clean: nothing to commit\n");
    assert_eq!(run(&dir, &["status", "--check"]).status.code(), Some(0));

    yaks(&dir, &["update", &id, "--note", "hello"]);
    yaks(&dir, &["shave", &id]);
    let out = run(&dir, &["status", "--check"]);
    assert_eq!(out.status.code(), Some(1));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.contains(&format!("  {id}  moved hairy -> shaving, notes +1")),
        "{text}"
    );
    assert!(text.contains(&format!(
        "run `yaks commit` to commit these as: yaks: shaving {id}"
    )));
    // Without --check the exit code is 0 either way.
    assert_eq!(run(&dir, &["status"]).status.code(), Some(0));

    yaks(&dir, &["commit"]);
    assert_eq!(run(&dir, &["status", "--check"]).status.code(), Some(0));
}

#[test]
fn json_names_the_yak_and_what_changed() {
    let (dir, id) = repo("json");
    yaks(&dir, &["update", &id, "--note", "hello"]);
    let v: serde_json::Value = serde_json::from_str(&yaks(&dir, &["status", "--json"])).unwrap();
    assert_eq!(v["clean"], false);
    assert_eq!(v["merge_in_progress"], false);
    assert_eq!(v["yaks"][0]["id"], id.as_str());
    assert_eq!(v["yaks"][0]["changes"], serde_json::json!(["notes +1"]));
    assert_eq!(v["yaks"][0]["staged"], false);
    assert_eq!(v["other"], serde_json::json!([]));
}
