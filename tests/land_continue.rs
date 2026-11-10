//! `yaks land --continue` / `--abort` (yaks-760d): a landing that stopped on a
//! conflicting cherry-pick is finished by yaks, which then records the landing,
//! through the real binary in temp git repos (farm = a branch) with `HOME` and
//! `XDG_DATA_HOME` in the temp dir.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("yaks-landcont-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.canonicalize().unwrap()
}

fn home() -> PathBuf {
    let h = std::env::temp_dir().join(format!("yaks-landcont-home-{}", std::process::id()));
    std::fs::create_dir_all(&h).unwrap();
    h.canonicalize().unwrap()
}

fn yaks(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yaks"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", home())
        .env("XDG_DATA_HOME", home().join("xdg"))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        // `git cherry-pick --continue` must not wait for an editor on its own.
        .env_remove("GIT_EDITOR")
        .env_remove("YAKS_DIR")
        .env_remove("YAKS_NO_BOOTSTRAP")
        .output()
        .unwrap()
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn ok(cwd: &Path, args: &[&str]) -> String {
    let o = yaks(cwd, args);
    assert!(o.status.success(), "yaks {args:?} failed:\n{}", text(&o));
    text(&o)
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(cwd)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", text(&o));
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

struct Scene {
    repo: PathBuf,
    farm: PathBuf,
    id: String,
    other: String,
    worker: String,
}

/// A repo whose `main` edited `a.txt` after the worker (who edited it too) branched
/// off, so landing the worker conflicts. The yak is named only by `--yak` (the
/// worker's message does not name it) when `named` is false.
fn conflict(tag: &str, named: bool) -> Scene {
    let r = temp(tag);
    git(&r, &["init", "-q", "-b", "main", "."]);
    std::fs::write(r.join("a.txt"), "base\n").unwrap();
    git(&r, &["add", "a.txt"]);
    git(&r, &["commit", "-qm", "init"]);
    ok(&r, &["init", "--no-skills", "--herd", "t"]);
    let farm = PathBuf::from(ok(&r, &["path"]).trim());
    let created = ok(&r, &["create", "a yak"]);
    let id = created
        .split([' ', ':'])
        .find(|w| w.starts_with("t-"))
        .unwrap_or_else(|| panic!("no id in {created}"))
        .to_string();
    ok(&r, &["commit"]);
    git(&r, &["checkout", "-q", "-b", "worker"]);
    std::fs::write(r.join("a.txt"), "worker\n").unwrap();
    let msg = if named {
        format!("{id}: worker edit")
    } else {
        "worker edit".to_string()
    };
    git(&r, &["commit", "-qam", &msg]);
    let worker = git(&r, &["rev-parse", "HEAD"]);
    git(&r, &["checkout", "-q", "main"]);
    std::fs::write(r.join("a.txt"), "main\n").unwrap();
    git(&r, &["commit", "-qam", "main edit"]);
    let other = git(&r, &["rev-parse", "HEAD"]);
    Scene {
        repo: r,
        farm,
        id,
        other,
        worker,
    }
}

fn notes(sc: &Scene) -> String {
    std::fs::read_to_string(sc.farm.join(format!("t/yaks/{}.md", sc.id))).unwrap()
}

fn resolve(sc: &Scene) {
    std::fs::write(sc.repo.join("a.txt"), "resolved\n").unwrap();
    git(&sc.repo, &["add", "a.txt"]);
}

#[test]
fn continue_finishes_the_pick_and_records_the_landing() {
    let sc = conflict("continue", true);
    let r = &sc.repo;
    let o = yaks(r, &["land", &sc.worker]);
    assert_eq!(o.status.code(), Some(5), "{}", text(&o));
    assert!(text(&o).contains("yaks land --continue"), "{}", text(&o));
    assert!(r.join(".git/CHERRY_PICK_HEAD").exists());
    assert!(!notes(&sc).contains("kind: landed"), "nothing recorded yet");

    // While it waits, a second `land` neither resets the tree nor lands.
    let again = yaks(r, &["land", &sc.worker]);
    assert_eq!(again.status.code(), Some(8), "{}", text(&again));
    assert!(r.join(".git/CHERRY_PICK_HEAD").exists());

    resolve(&sc);
    let out = ok(r, &["land", "--continue"]);
    assert!(
        out.contains(&format!("recorded the landing on {}", sc.id)),
        "{out}"
    );
    assert!(!r.join(".git/CHERRY_PICK_HEAD").exists());
    assert!(!r.join(".git/yaks-land.json").exists(), "state removed");
    assert_eq!(git(r, &["rev-parse", "HEAD^"]), sc.other);
    assert_eq!(
        std::fs::read_to_string(r.join("a.txt")).unwrap(),
        "resolved\n"
    );

    let landed = git(r, &["rev-parse", "HEAD"]);
    let body = notes(&sc);
    assert!(body.contains("kind: landed"), "{body}");
    assert!(body.contains(&format!("commit: {landed}")), "{body}");
    assert!(body.contains(&format!("source: {}", sc.worker)), "{body}");
    assert!(body.contains("branch: main"), "{body}");

    let c = ok(r, &["commits", &sc.id]);
    assert!(
        c.contains(&format!("landed {} on main", &landed[..7])),
        "{c}"
    );

    // Nothing is left to continue.
    let none = yaks(r, &["land", "--continue"]);
    assert_eq!(none.status.code(), Some(1), "{}", text(&none));
}

#[test]
fn continue_keeps_the_yak_flag_of_the_first_run() {
    let sc = conflict("yakflag", false);
    let r = &sc.repo;
    let o = yaks(r, &["land", &sc.worker, "--yak", &sc.id]);
    assert_eq!(o.status.code(), Some(5), "{}", text(&o));
    resolve(&sc);
    let out = ok(r, &["land", "--continue"]);
    assert!(
        out.contains(&format!("recorded the landing on {}", sc.id)),
        "{out}"
    );
    assert!(notes(&sc).contains("kind: landed"));
}

#[test]
fn continue_while_conflicts_remain_changes_nothing_and_can_be_retried() {
    let sc = conflict("unresolved", true);
    let r = &sc.repo;
    yaks(r, &["land", &sc.worker]);
    let o = yaks(r, &["land", "--continue"]);
    assert_eq!(o.status.code(), Some(5), "{}", text(&o));
    assert!(r.join(".git/CHERRY_PICK_HEAD").exists());
    assert!(r.join(".git/yaks-land.json").exists(), "still waiting");
    resolve(&sc);
    ok(r, &["land", "--continue"]);
    assert!(notes(&sc).contains("kind: landed"));
}

#[test]
fn a_pick_finished_with_git_is_still_recorded_by_continue() {
    let sc = conflict("gitfinished", true);
    let r = &sc.repo;
    yaks(r, &["land", &sc.worker]);
    resolve(&sc);
    let o = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(["cherry-pick", "--continue"])
        .env("GIT_EDITOR", "true")
        .current_dir(r)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", text(&o));
    let out = ok(r, &["land", "--continue"]);
    assert!(out.contains("recorded the landing"), "{out}");
    assert!(notes(&sc).contains("kind: landed"));
}

#[test]
fn abort_restores_the_tree_and_forgets_the_state() {
    let sc = conflict("abort", true);
    let r = &sc.repo;
    yaks(r, &["land", &sc.worker]);
    ok(r, &["land", "--abort"]);
    assert!(!r.join(".git/CHERRY_PICK_HEAD").exists());
    assert!(!r.join(".git/yaks-land.json").exists());
    assert_eq!(git(r, &["rev-parse", "HEAD"]), sc.other);
    assert_eq!(git(r, &["status", "--porcelain"]), "");
    assert!(!notes(&sc).contains("kind: landed"));
    assert_eq!(
        yaks(r, &["land", "--abort"]).status.code(),
        Some(1),
        "nothing left to abort"
    );
    // It can be landed again from scratch.
    let again = yaks(r, &["land", &sc.worker]);
    assert_eq!(again.status.code(), Some(5), "{}", text(&again));
}

#[test]
fn the_flags_are_exclusive_and_a_sha_is_required_otherwise() {
    let sc = conflict("flags", true);
    let r = &sc.repo;
    for args in [
        vec!["land"],
        vec!["land", "--continue", "--abort"],
        vec!["land", "--continue", &sc.worker],
        vec!["land", "--abort", "--yak", "x"],
    ] {
        assert!(!yaks(r, &args).status.success(), "{args:?}");
    }
}
