//! `yaks --shed <name>`: `-C` by name. Run any command in another checkout of
//! this repository picked like `yaks changes <shed>`, through the real binary:
//! a plain git worktree shed (to read its farm) and a Delta-shaped clone (to
//! check the caller's thread title is not recorded in the target).

use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("yaks-shedflag-{tag}-{}", std::process::id()));
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

fn run(cwd: &Path, title: Option<&str>, args: &[&str]) -> std::process::Output {
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
    cmd.output().unwrap()
}

fn yaks(cwd: &Path, args: &[&str]) -> String {
    let out = run(cwd, None, args);
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8(out.stdout).unwrap()
}

/// A repo with a committed farm, and a git worktree shed on branch `feat`
/// whose farm has a yak the primary lacks.
fn repo_with_shed(tag: &str) -> (PathBuf, PathBuf) {
    let b = tmp(tag);
    let t = b.join("repo");
    std::fs::create_dir_all(&t).unwrap();
    git(&t, &["init", "-q"]);
    yaks(&t, &["init"]);
    yaks(&t, &["create", "in both"]);
    git(&t, &["add", "."]);
    git(&t, &["commit", "-q", "-m", "init"]);
    let wt = b.join("wt");
    git(
        &t,
        &["worktree", "add", "-q", "-b", "feat", wt.to_str().unwrap()],
    );
    yaks(&wt, &["create", "only the shed has this"]);
    (t, wt)
}

#[test]
fn shed_before_the_subcommand_lists_the_sheds_farm() {
    let (t, _wt) = repo_with_shed("before");
    assert!(!yaks(&t, &["list"]).contains("only the shed"));
    let out = yaks(&t, &["--shed", "feat", "list"]);
    assert!(out.contains("only the shed has this"), "{out}");
    assert!(out.contains("in both"), "{out}");
}

#[test]
fn shed_after_the_subcommand_works_too() {
    let (t, wt) = repo_with_shed("after");
    let out = yaks(&t, &["list", "--shed", "feat"]);
    assert!(out.contains("only the shed has this"), "{out}");
    // The shed's path picks it like `yaks changes` does.
    let out = yaks(&t, &["list", &format!("--shed={}", wt.display())]);
    assert!(out.contains("only the shed has this"), "{out}");
}

#[test]
fn shed_main_goes_back_to_the_primary_and_dash_c_applies_first() {
    let (t, wt) = repo_with_shed("main");
    let out = yaks(&wt, &["--shed", "main", "list"]);
    assert!(!out.contains("only the shed"), "{out}");
    // `-C` first, then `--shed` relative to that checkout.
    let elsewhere = tmp("main-elsewhere");
    let out = yaks(
        &elsewhere,
        &["-C", wt.to_str().unwrap(), "--shed", "main", "list"],
    );
    assert!(
        out.contains("in both") && !out.contains("only the shed"),
        "{out}"
    );
    let out = yaks(
        &elsewhere,
        &["-C", t.to_str().unwrap(), "--shed", "feat", "list"],
    );
    assert!(out.contains("only the shed has this"), "{out}");
}

#[test]
fn shed_is_full_access_a_write_lands_in_that_shed() {
    let (t, wt) = repo_with_shed("write");
    yaks(&t, &["--shed", "feat", "create", "made via shed flag"]);
    assert!(yaks(&wt, &["list"]).contains("made via shed flag"));
    assert!(!yaks(&t, &["list"]).contains("made via shed flag"));
}

#[test]
fn an_unknown_shed_exits_1_listing_the_candidates() {
    let (t, wt) = repo_with_shed("unknown");
    let out = run(&t, None, &["--shed", "nope", "list"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(out.stdout.is_empty());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.starts_with("error: no shed matches `nope`"), "{err}");
    assert!(
        err.contains("feat") || err.contains(wt.to_str().unwrap()),
        "{err}"
    );
    assert!(err.contains("the sheds are:"), "{err}");
}

#[test]
fn two_shed_flags_are_an_error() {
    let (t, _wt) = repo_with_shed("two");
    let out = run(&t, None, &["--shed", "feat", "--shed", "main", "list"]);
    assert!(!out.status.success(), "{out:?}");
    assert!(out.stdout.is_empty());
}

/// A repo with a committed farm and one Delta-shaped clone of it (the layout
/// of `tests/shed_title.rs`).
fn delta_setup(tag: &str) -> (PathBuf, PathBuf) {
    let b = tmp(tag);
    let t = b.join("repo");
    std::fs::create_dir_all(&t).unwrap();
    git(&t, &["init", "-q"]);
    yaks(&t, &["init"]);
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

#[test]
fn shed_into_a_delta_clone_does_not_record_the_callers_thread_title() {
    let (t, co) = delta_setup("title");
    let out = run(
        &t,
        Some("Coordinator thread"),
        &["--shed", "abc123", "list"],
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(stored(&co, "yaks.shed.title"), "");
    assert_eq!(stored(&co, "yaks.shed.thread"), "");
    assert_eq!(stored(&t, "yaks.shed.title"), "");
    // Control: the same command run IN the clone does record it.
    let out = run(&co, Some("Worker thread"), &["list"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(stored(&co, "yaks.shed.title"), "Worker thread");
}

#[test]
fn a_positional_shed_argument_is_not_mistaken_for_the_flag() {
    // `changes <SHED>` and `answer <id>@<shed>` name sheds positionally; that
    // must not also hop the whole command into the shed.
    let (t, _wt) = repo_with_shed("positional");
    let out = yaks(&t, &["changes", "feat"]);
    assert!(out.contains("only the shed has this"), "{out}");
    let out = run(&t, None, &["changes", "nope"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("the sheds are:"));
}
