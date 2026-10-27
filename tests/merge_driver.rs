//! The yak-file merge driver (yaks-4a53), through the real binary and a real
//! `git merge` / `git rebase` across two clones: concurrent notes plus a
//! status both sides moved never conflict, leave no markers, and `yaks doctor`
//! stays clean.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

/// The farm schema this yaks writes (`store::SCHEMA`).
const SCHEMA: u32 = 6;

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("yaks-mdriver-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}

fn git_out(dir: &Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .output()
        .unwrap()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = git_out(dir, args);
    assert!(
        out.status.success(),
        "git {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// Run yaks on the farm committed in the repo at `cwd` (`<cwd>/.yaks`). The
/// driver merges yak files on the farm branch; here each repo's tree IS that
/// branch's tree, so the farm is named with YAKS_DIR rather than discovered.
fn yaks_out(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(cwd)
        .env("YAKS_DIR", cwd.join(".yaks"))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
        .args(args)
        .output()
        .unwrap()
}

fn yaks(cwd: &Path, args: &[&str]) -> String {
    let out = yaks_out(cwd, args);
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// What `yaks sync` registers (`mergedriver::ensure_registered`): the same
/// attribute line, and the driver command for THIS build of the binary.
fn register(repo: &Path) {
    let exe = assert_cmd::cargo::cargo_bin("yaks");
    std::fs::write(repo.join(".gitattributes"), ".yaks/**/*.md merge=yaks\n").unwrap();
    git(repo, &["config", "merge.yaks.name", "yaks yak-file merge"]);
    git(
        repo,
        &[
            "config",
            "merge.yaks.driver",
            &format!("{} merge-driver %O %A %B %P", exe.display()),
        ],
    );
}

/// A repo `a` with one yak, a clone `b`, and a concurrent edit committed on
/// each: a note each, and a different status each. Returns (a, b, id, branch).
fn diverged(tag: &str) -> (PathBuf, PathBuf, String, String) {
    let root = tmp(tag);
    let a = root.join("a");
    std::fs::create_dir_all(&a).unwrap();
    git(&a, &["init", "-q"]);
    std::fs::create_dir_all(a.join(".yaks")).unwrap();
    std::fs::write(a.join(".yaks/config.yaml"), "herd: yak\n").unwrap();
    std::fs::write(a.join(".yaks/schema"), format!("{}\n", SCHEMA)).unwrap();
    let created = yaks(&a, &["create", "concurrent yak"]);
    let id = created.split([' ', ':']).nth(1).unwrap().to_string();
    git(&a, &["add", "."]);
    git(&a, &["commit", "-q", "-m", "base"]);
    let branch = git(&a, &["branch", "--show-current"]).trim().to_string();
    git(&root, &["clone", "-q", a.to_str().unwrap(), "b"]);
    let b = root.join("b");

    yaks(&a, &["update", &id, "--note", "note from a"]);
    yaks(&a, &["shave", &id]);
    git(&a, &["commit", "-qam", "a edits"]);
    yaks(&b, &["update", &id, "--note", "note from b"]);
    yaks(&b, &["shorn", &id]);
    git(&b, &["commit", "-qam", "b edits"]);
    (a, b, id, branch)
}

fn assert_merged(b: &Path, id: &str) {
    let path = String::from_utf8(yaks_out(b, &["path", id]).stdout).unwrap();
    let text = std::fs::read_to_string(path.trim()).unwrap();
    assert!(
        !text.contains("<<<<<<<") && !text.contains(">>>>>>>"),
        "{text}"
    );
    assert_eq!(text.matches("\nupdated:").count(), 1, "{text}");
    assert!(
        text.contains("note from a") && text.contains("note from b"),
        "{text}"
    );
    assert!(text.contains("merge conflict: status was"), "{text}");
    assert!(git(b, &["status", "--porcelain"]).trim().is_empty());
    let doctor = yaks_out(b, &["doctor", "--strict"]);
    assert!(
        doctor.status.success(),
        "doctor: {}{}",
        String::from_utf8_lossy(&doctor.stdout),
        String::from_utf8_lossy(&doctor.stderr)
    );
    // The yak still loads and shows both notes.
    let shown = yaks(b, &["show", id]);
    assert!(
        shown.contains("note from a") && shown.contains("note from b"),
        "{shown}"
    );
}

#[test]
fn without_the_driver_the_same_merge_conflicts() {
    let (_a, b, _id, branch) = diverged("control");
    git(&b, &["fetch", "-q", "origin"]);
    let out = git_out(&b, &["merge", &format!("origin/{branch}")]);
    assert!(!out.status.success(), "plain git merge should conflict");
}

#[test]
fn git_merge_with_the_driver_yields_a_valid_yak() {
    let (_a, b, id, branch) = diverged("merge");
    register(&b);
    git(&b, &["fetch", "-q", "origin"]);
    git(
        &b,
        &["merge", "-q", "-m", "merge", &format!("origin/{branch}")],
    );
    // .gitattributes is untracked here (sync commits it); nothing else is dirty.
    git(&b, &["add", ".gitattributes"]);
    git(&b, &["commit", "-qm", "attrs"]);
    assert_merged(&b, &id);
}

#[test]
fn git_rebase_with_the_driver_yields_a_valid_yak() {
    let (_a, b, id, branch) = diverged("rebase");
    register(&b);
    git(&b, &["fetch", "-q", "origin"]);
    git(&b, &["rebase", &format!("origin/{branch}")]);
    git(&b, &["add", ".gitattributes"]);
    git(&b, &["commit", "-qm", "attrs"]);
    assert_merged(&b, &id);
}

#[test]
fn a_non_yak_file_still_conflicts_in_git() {
    // config.yaml changed on both sides: the driver is not for it, but a file
    // matched by the rule that is not a yak must conflict, never be mangled.
    let root = tmp("nonyak");
    let a = root.join("a");
    std::fs::create_dir_all(a.join(".yaks/h/yaks")).unwrap();
    git(&a, &["init", "-q"]);
    std::fs::write(a.join(".yaks/h/yaks/notes.md"), "one\n").unwrap();
    git(&a, &["add", "."]);
    git(&a, &["commit", "-qm", "base"]);
    let branch = git(&a, &["branch", "--show-current"]).trim().to_string();
    git(&root, &["clone", "-q", a.to_str().unwrap(), "b"]);
    let b = root.join("b");
    std::fs::write(a.join(".yaks/h/yaks/notes.md"), "two\n").unwrap();
    git(&a, &["commit", "-qam", "a"]);
    std::fs::write(b.join(".yaks/h/yaks/notes.md"), "three\n").unwrap();
    git(&b, &["commit", "-qam", "b"]);
    register(&b);
    git(&b, &["fetch", "-q", "origin"]);
    let out = git_out(&b, &["merge", &format!("origin/{branch}")]);
    assert!(!out.status.success());
    let text = std::fs::read_to_string(b.join(".yaks/h/yaks/notes.md")).unwrap();
    assert_eq!(text, "three\n", "ours left untouched for git to report");
}
