//! `yaks land` warns when the yak it records the landing on is shorn with no
//! change line (yaks-045c): the worker's context is still around. It never
//! fails the landing.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("yaks-landwarn-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.canonicalize().unwrap()
}

/// A HOME beside the repo, never inside it (the farm worktree lives there).
fn home(repo: &Path) -> PathBuf {
    let h = repo.with_file_name(format!(
        "{}-home",
        repo.file_name().unwrap().to_string_lossy()
    ));
    std::fs::create_dir_all(&h).unwrap();
    h
}

fn yaks(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yaks"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", home(cwd))
        .env("XDG_DATA_HOME", home(cwd).join("xdg"))
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

/// A repo with a worker commit (naming a yak) not yet on `main`; `shear`
/// moves the yak to shorn with the given extra args first.
fn scene(tag: &str, shear: &[&str]) -> (PathBuf, String, String) {
    let r = temp(tag);
    git(&r, &["init", "-q", "-b", "main", "."]);
    std::fs::write(r.join("a.txt"), "base\n").unwrap();
    git(&r, &["add", "a.txt"]);
    git(&r, &["commit", "-qm", "init"]);
    ok(&r, &["init", "--no-skills", "--herd", "t"]);
    let created = ok(&r, &["create", "a yak"]);
    let id = created
        .split([' ', ':'])
        .find(|w| w.starts_with("t-"))
        .unwrap_or_else(|| panic!("no id in {created}"))
        .to_string();
    if !shear.is_empty() {
        let mut args = vec!["shorn", id.as_str()];
        args.extend(shear);
        ok(&r, &args);
    }
    ok(&r, &["commit"]);
    git(&r, &["checkout", "-q", "-b", "worker"]);
    std::fs::write(r.join("b.txt"), "worker\n").unwrap();
    git(&r, &["add", "b.txt"]);
    git(&r, &["commit", "-qm", &format!("{id}: worker edit")]);
    let worker = git(&r, &["rev-parse", "HEAD"]);
    git(&r, &["checkout", "-q", "main"]);
    (r, id, worker)
}

fn warning(id: &str) -> String {
    format!(
        "warning: {id} is shorn with no change line; add one: yaks update {id} --note \"<line>\" --meta kind=change (or --no-change on shear)"
    )
}

#[test]
fn landing_a_shorn_yak_with_no_change_line_warns_and_still_lands() {
    let (r, id, worker) = scene("warn", &["--no-change"]);
    let (r2, id2, worker2) = scene("warn2", &[]);
    ok(&r2, &["shorn", &id2]);
    let out = ok(&r2, &["land", &worker2]);
    assert!(
        out.contains(&format!("recorded the landing on {id2}")),
        "{out}"
    );
    assert!(out.contains(&warning(&id2)), "{out}");
    assert!(r2.join("b.txt").exists(), "the landing happened");

    // A dry run says the same, and writes nothing.
    let (r3, id3, worker3) = scene("warn3", &[]);
    ok(&r3, &["shorn", &id3]);
    let out = ok(&r3, &["land", &worker3, "--dry-run"]);
    assert!(out.contains(&warning(&id3)), "{out}");

    // A yak with a change line or `--no-change` gets no warning.
    let out = ok(&r, &["land", &worker]);
    assert!(
        out.contains(&format!("recorded the landing on {id}")),
        "{out}"
    );
    assert!(!out.contains("no change line"), "{out}");
    let (r4, id4, worker4) = scene("warn4", &["--change", "Adds a thing."]);
    let out = ok(&r4, &["land", &worker4]);
    assert!(
        out.contains(&format!("recorded the landing on {id4}")),
        "{out}"
    );
    assert!(!out.contains("no change line"), "{out}");

    // A yak that is not shorn is not warned about.
    let (r5, id5, worker5) = scene("warn5", &[]);
    let out = ok(&r5, &["land", &worker5]);
    assert!(
        out.contains(&format!("recorded the landing on {id5}")),
        "{out}"
    );
    assert!(!out.contains("no change line"), "{out}");
    for d in [r, r2, r3, r4, r5] {
        let _ = std::fs::remove_dir_all(home(&d));
        let _ = std::fs::remove_dir_all(d);
    }
}
