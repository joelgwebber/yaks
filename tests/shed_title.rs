//! A Delta clone records its thread title in its OWN git config the first
//! time a yaks command runs there with `DELTA_THREAD_TITLE` set, and
//! `yaks sheds` from the primary then names the clone by the title's slug.
//! Driven through the real binary against a Delta-shaped clone (a
//! `--separate-git-dir` shared clone pinned in the host as `refs/delta/...`).
//! `-C <clone>` records nothing (the env is the caller's thread, not the
//! clone's), and neither does a plain git worktree.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("yaks-shedtitle-{tag}-{}", std::process::id()));
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

/// `yaks <args>` in `cwd`, with the Delta thread env given.
fn yaks(cwd: &Path, title: Option<&str>, args: &[&str]) -> String {
    let mut cmd = Command::cargo_bin("yaks").unwrap();
    cmd.current_dir(cwd)
        .env_remove("YAKS_DIR")
        .env_remove("DELTA_THREAD_TITLE")
        .env_remove("DELTA_CURRENT_THREAD_ID")
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
        .args(args);
    if let Some(t) = title {
        cmd.env("DELTA_THREAD_TITLE", t)
            .env("DELTA_CURRENT_THREAD_ID", "thread-9");
    }
    let out = cmd.output().unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8(out.stdout).unwrap()
}

/// A repo with a committed farm and one Delta-shaped clone of it.
fn setup(tag: &str) -> (PathBuf, PathBuf) {
    let b = tmp(tag);
    let t = b.join("repo");
    std::fs::create_dir_all(&t).unwrap();
    git(&t, &["init", "-q"]);
    yaks(&t, None, &["init"]);
    git(&t, &["add", "."]);
    git(&t, &["commit", "-q", "-m", "init"]);
    let gd = t.join(".delta/clones/abc123/repo.git");
    let co = t.join(".delta/worktrees/abc123/repo");
    std::fs::create_dir_all(co.parent().unwrap()).unwrap();
    std::fs::create_dir_all(gd.parent().unwrap()).unwrap();
    git(
        &b,
        &[
            "clone",
            "-q",
            "--shared",
            "--separate-git-dir",
            gd.to_str().unwrap(),
            t.join(".git").to_str().unwrap(),
            co.to_str().unwrap(),
        ],
    );
    git(&co, &["config", "core.worktree", co.to_str().unwrap()]);
    let sha = git(&co, &["rev-parse", "HEAD"]);
    git(
        &t,
        &["update-ref", &format!("refs/delta/abc123/repo/{sha}"), &sha],
    );
    (t, co)
}

fn stored(dir: &Path, key: &str) -> String {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(["config", "--local", "--get", key])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn names(primary: &Path) -> Vec<String> {
    let j: Value = serde_json::from_str(&yaks(primary, None, &["sheds", "--json"])).unwrap();
    j.as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_command_in_the_clone_records_the_title_and_sheds_names_it_by_slug() {
    let (t, co) = setup("record");
    assert_eq!(names(&t), vec!["abc123"]);
    yaks(&co, Some("cflag-1: yaks -C global flag"), &["list"]);
    assert_eq!(
        stored(&co, "yaks.shed.title"),
        "cflag-1: yaks -C global flag"
    );
    assert_eq!(stored(&co, "yaks.shed.thread"), "thread-9");
    assert_eq!(names(&t), vec!["cflag-1-yaks-c-global"]);
    let text = yaks(&t, None, &["sheds"]);
    assert!(
        text.contains("name: cflag-1-yaks-c-global \u{b7} title: cflag-1: yaks -C global flag"),
        "{text}"
    );
    // The host's config is never stamped, and the farm is untouched.
    assert_eq!(stored(&t, "yaks.shed.title"), "");
    assert_eq!(
        git(&t, &["status", "--porcelain", "--untracked-files=no"]),
        ""
    );
}

#[test]
fn dash_c_into_the_clone_records_nothing() {
    let (t, co) = setup("dashc");
    let c = co.to_str().unwrap();
    yaks(&t, Some("Coordinator thread"), &["-C", c, "list"]);
    assert_eq!(stored(&co, "yaks.shed.title"), "");
    assert_eq!(stored(&t, "yaks.shed.title"), "");
    assert_eq!(names(&t), vec!["abc123"]);
}

#[test]
fn a_plain_git_worktree_records_nothing() {
    let (t, _co) = setup("worktree");
    let wt = t.parent().unwrap().join("wt");
    git(
        &t,
        &["worktree", "add", "-q", "-b", "w", wt.to_str().unwrap()],
    );
    yaks(&wt, Some("Worktree thread"), &["list"]);
    yaks(&t, Some("Primary thread"), &["list"]);
    assert_eq!(stored(&wt, "yaks.shed.title"), "");
    assert_eq!(stored(&t, "yaks.shed.title"), "");
}
