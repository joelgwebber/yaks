//! `yaks -C <path>`: run any command as if started in <path> (like `git -C`),
//! through the real binary from an unrelated cwd.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("yaks-dashc-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // Resolve symlinks (macOS /var -> /private/var) so paths compare equal.
    dir.canonicalize().unwrap()
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

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(cwd)
        .env_remove("YAKS_DIR")
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
        .args(args)
        .output()
        .unwrap()
}

fn yaks(cwd: &Path, args: &[&str]) -> String {
    let out = run(cwd, args);
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// A repo with a committed farm holding one yak, plus a sub-directory; returns
/// (repo, sub-dir, yak id).
fn repo(tag: &str) -> (PathBuf, PathBuf, String) {
    let dir = tmp(tag);
    git(&dir, &["init", "-q"]);
    yaks(&dir, &["init"]);
    let created = yaks(&dir, &["create", "dash c yak"]);
    let id = created.split([' ', ':']).nth(1).unwrap().to_string();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "init"]);
    let sub = dir.join("sub/deeper");
    std::fs::create_dir_all(&sub).unwrap();
    (dir, sub, id)
}

#[test]
fn list_from_an_unrelated_cwd() {
    let (repo, _, _) = repo("list");
    let elsewhere = tmp("list-elsewhere");
    // Without -C the unrelated cwd has no farm.
    assert!(!run(&elsewhere, &["list"]).status.success());
    let out = yaks(&elsewhere, &["-C", repo.to_str().unwrap(), "list"]);
    assert!(out.contains("dash c yak"), "{out}");
}

#[test]
fn show_from_a_sub_directory_of_the_repo() {
    let (repo, sub, id) = repo("show");
    let elsewhere = tmp("show-elsewhere");
    let out = yaks(&elsewhere, &["-C", sub.to_str().unwrap(), "show", &id]);
    assert!(out.contains("dash c yak"), "{out}");
    // `repo` only ever appears as the anchor of the discovery walk.
    assert!(repo.exists());
}

#[test]
fn sheds_json_matches_running_in_that_checkout() {
    let (repo, _, _) = repo("sheds");
    // A linked worktree of the repo is a second checkout with its own farm.
    let wt = repo
        .parent()
        .unwrap()
        .join(format!("yaks-dashc-sheds-wt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&wt);
    git(
        &repo,
        &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "wt"],
    );
    let wt = wt.canonicalize().unwrap();
    let elsewhere = tmp("sheds-elsewhere");
    let direct = yaks(&wt, &["sheds", "--json"]);
    let via_c = yaks(&elsewhere, &["-C", wt.to_str().unwrap(), "sheds", "--json"]);
    assert_eq!(direct, via_c);
    assert!(via_c.contains(repo.to_str().unwrap()), "{via_c}");
}

#[test]
fn missing_path_is_an_error_naming_it() {
    let elsewhere = tmp("missing");
    let out = run(&elsewhere, &["-C", "no/such/dir", "list"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("no/such/dir"), "{err}");
}

#[test]
fn flag_after_the_subcommand_also_works() {
    let (repo, _, _) = repo("after");
    let elsewhere = tmp("after-elsewhere");
    let out = yaks(&elsewhere, &["list", "-C", repo.to_str().unwrap()]);
    assert!(out.contains("dash c yak"), "{out}");
}

#[test]
fn repeated_c_composes_relative_to_the_previous() {
    let (repo, _, _) = repo("compose");
    let parent = repo.parent().unwrap().to_path_buf();
    let name = repo.file_name().unwrap().to_str().unwrap().to_string();
    let elsewhere = tmp("compose-elsewhere");
    let out = yaks(
        &elsewhere,
        &["-C", parent.to_str().unwrap(), "-C", &name, "list"],
    );
    assert!(out.contains("dash c yak"), "{out}");
}

#[test]
fn init_and_discover_honor_c() {
    let dir = tmp("init");
    git(&dir, &["init", "-q"]);
    let elsewhere = tmp("init-elsewhere");
    yaks(&elsewhere, &["-C", dir.to_str().unwrap(), "init"]);
    assert!(dir.join(".yaks/hairy").is_dir());
    assert!(!elsewhere.join(".yaks").exists());
    let out = yaks(
        &elsewhere,
        &["-C", dir.to_str().unwrap(), "discover", "--json"],
    );
    assert!(out.contains(dir.to_str().unwrap()), "{out}");
}
