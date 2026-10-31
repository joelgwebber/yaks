//! `yaks commits` and the landed-sha record (yaks-7a11, yaks-735a), through the
//! real binary in temp git repos whose farm is a branch, with `HOME` and
//! `XDG_DATA_HOME` in the temp dir so nothing outside it is touched.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("yaks-commits-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.canonicalize().unwrap()
}

fn yaks(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yaks"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", cwd.join("__home"))
        .env("XDG_DATA_HOME", cwd.join("__xdg"))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
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

/// A code repo on `main` with a branch farm holding one yak; returns
/// (repo, farm worktree, yak id).
fn branch_farm(tag: &str) -> (PathBuf, PathBuf, String) {
    let r = temp(tag);
    git(&r, &["init", "-q", "-b", "main", "."]);
    std::fs::write(r.join("README.md"), "x\n").unwrap();
    git(&r, &["add", "README.md"]);
    git(&r, &["commit", "-qm", "init"]);
    ok(
        &r,
        &[
            "init", "--mode", "branch", "--skills", "none", "--herd", "t",
        ],
    );
    let farm = PathBuf::from(ok(&r, &["path"]).trim());
    let created = ok(&r, &["create", "a yak"]);
    let id = created
        .split(|c: char| c == ' ' || c == ':')
        .find(|w| w.starts_with("t-"))
        .unwrap_or_else(|| panic!("no id in {created}"))
        .to_string();
    ok(&r, &["commit"]);
    (r, farm, id)
}

fn code_commit(r: &Path, file: &str, msg: &str) -> String {
    std::fs::write(r.join(file), "y\n").unwrap();
    git(r, &["add", file]);
    git(r, &["commit", "-qm", msg]);
    git(r, &["rev-parse", "HEAD"])
}

#[test]
fn commits_greps_the_code_repo_not_the_farm_branch() {
    let (r, farm, id) = branch_farm("grep");
    let on_main = code_commit(&r, "a.txt", &format!("{id}: first change"));
    git(&r, &["checkout", "-q", "-b", "side"]);
    let on_side = code_commit(&r, "b.txt", &format!("{id}: side change"));
    git(&r, &["checkout", "-q", "main"]);
    code_commit(&r, "c.txt", "unrelated");

    let out = ok(&r, &["commits", &id]);
    let (naming, farm_part) = out.split_once("Farm history").expect(&out);
    assert!(naming.contains(&on_main[..7]), "{out}");
    assert!(
        naming.contains(&on_side[..7]),
        "all branches searched:\n{out}"
    );
    assert!(
        !naming.contains("yaks: created"),
        "farm commits are not code:\n{out}"
    );
    assert!(!naming.contains("unrelated"), "{out}");
    assert!(
        farm_part.contains("yaks: created"),
        "the yak file's own history, labelled as farm history:\n{out}"
    );
    // Newest first: the side-branch commit came after the main one.
    assert!(
        naming.find(&on_side[..7]).unwrap() < naming.find(&on_main[..7]).unwrap(),
        "{out}"
    );

    // Run from a subdirectory, and with -C from elsewhere: the same code repo.
    std::fs::create_dir_all(r.join("sub")).unwrap();
    assert!(ok(&r.join("sub"), &["commits", &id]).contains(&on_main[..7]));
    let elsewhere = temp("grep-elsewhere");
    let via_c = ok(&elsewhere, &["-C", r.to_str().unwrap(), "commits", &id]);
    assert!(via_c.contains(&on_main[..7]), "{via_c}");

    // From the farm's own worktree there is no code to search: say so.
    let from_farm = ok(&farm, &["commits", &id]);
    assert!(
        from_farm.contains("not run from a code checkout"),
        "{from_farm}"
    );
    assert!(!from_farm.contains(&on_main[..7]), "{from_farm}");
}
