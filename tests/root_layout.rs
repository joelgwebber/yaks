//! A farm branch whose ROOT is the farm root (no `.yaks/` directory;
//! yaks-6ffd), through the real binary: per-change commits, `status`,
//! `commit`, `sync` with a dirty farm, and `weight` all see it as a farm
//! branch, with the per-yak verbs. `yaks init` still writes `.yaks/`, so the
//! branch is set up by hand.

mod common;
use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A code repo plus a `yaks` branch checked out beside it with the farm at its
/// root. Returns `(code repo, farm worktree)`.
fn repo(tag: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("yaks-rootlayout-{tag}-{}", std::process::id()));
    common::cleanup(&dir);
    common::git_init(&dir);
    std::fs::write(dir.join("a.txt"), "a\n").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "code"]);
    let wt = common::data_home(&dir).join("farm");
    git(
        &dir,
        &[
            "worktree",
            "add",
            "-q",
            "--orphan",
            "-b",
            "yaks",
            wt.to_str().unwrap(),
        ],
    );
    std::fs::write(wt.join("config.yaml"), "herd: yak\ndefault_type: chore\n").unwrap();
    std::fs::write(wt.join("schema"), "8\n").unwrap();
    std::fs::write(wt.join(".gitignore"), ".lock\n").unwrap();
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-q", "-m", "yaks: new farm"]);
    let wt = wt.canonicalize().unwrap();
    (dir, wt)
}

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(dir)
        .env("XDG_DATA_HOME", common::data_home(dir))
        .env_remove("YAKS_DIR")
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .args(args)
        .output()
        .unwrap()
}

fn yaks(dir: &Path, args: &[&str]) -> String {
    let out = run(dir, args);
    assert!(
        out.status.success(),
        "{args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn every_change_is_committed_on_the_farm_branch_with_its_verb() {
    let (dir, wt) = repo("autocommit");
    let created = yaks(&dir, &["create", "first"]);
    let id = created.split([' ', ':']).nth(1).unwrap().to_string();
    yaks(&dir, &["shave", &id]);
    yaks(&dir, &["update", &id, "--note", "n"]);
    yaks(&dir, &["shorn", &id]);
    let log = git(&wt, &["log", "--format=%s"]);
    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        [
            format!("yaks: shorn {id}"),
            format!("yaks: updated {id}"),
            format!("yaks: shaving {id}"),
            format!("yaks: created {id}"),
            "yaks: new farm".to_string(),
        ]
    );
    // The yak file is where the root layout puts it, with no `.yaks/`.
    assert!(wt.join("yak/yaks").join(format!("{id}.md")).is_file());
    assert!(!wt.join(".yaks").exists());
    assert_eq!(
        yaks(&dir, &["status"]),
        "farm clean: nothing to commit\n\
         farm branch yaks: not published (no upstream); `yaks sync` publishes it\n"
    );
    common::cleanup(&dir);
}

#[test]
fn status_and_commit_name_each_yak_and_sync_sees_a_dirty_farm() {
    let (dir, wt) = repo("dirty");
    let origin = common::data_home(&dir).join("origin.git");
    git(&dir, &["init", "-q", "--bare", origin.to_str().unwrap()]);
    git(&wt, &["remote", "add", "origin", origin.to_str().unwrap()]);
    yaks(&dir, &["sync"]);

    std::fs::create_dir_all(wt.join("yak/yaks")).unwrap();
    std::fs::write(
        wt.join("yak/yaks/yak-ffff.md"),
        "---\nid: yak-ffff\ntitle: by hand\nstatus: hairy\n---\n",
    )
    .unwrap();
    let status = yaks(&dir, &["status"]);
    assert!(status.contains("yak-ffff  created"), "{status}");
    assert!(
        status.contains("as: yaks: created yak-ffff"),
        "per-yak message, not `farm config`: {status}"
    );
    // A dirty farm is seen as dirty: the pull is skipped, not attempted (git
    // would refuse to rebase over it).
    let sync = yaks(&dir, &["sync", "--no-commit"]);
    assert!(
        sync.contains("pull skipped: the farm has uncommitted changes"),
        "{sync}"
    );
    // `weight` buckets the yak file as a yak, not as `other`.
    yaks(&dir, &["commit"]);
    let weight = yaks(&dir, &["weight"]);
    assert!(weight.contains("yaks             1 file"), "{weight}");
    common::cleanup(&dir);
}
