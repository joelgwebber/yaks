//! The stale project-local skills notice end to end: the binary run in a real
//! temp git repo with a temp `$HOME`, so nothing outside the temp dir is
//! written. Project-local skills are never auto-written (yaks-3859); an
//! ordinary command tells you, once per yaks version per checkout.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("yaks-notice-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.canonicalize().unwrap()
}

/// Run yaks with `YAKS_SKILLS_AUTOSYNC` set explicitly (`on` = "1").
fn yaks_env(cwd: &Path, autosync: &str, args: &[&str]) -> Output {
    // One temp HOME per repo, wherever in it the command runs.
    let top = cwd.ancestors().find(|d| d.join(".git").exists());
    Command::new(env!("CARGO_BIN_EXE_yaks"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", top.unwrap_or(cwd).join("__home"))
        .env("YAKS_SKILLS_AUTOSYNC", autosync)
        .env_remove("YAKS_DIR")
        .output()
        .unwrap()
}

fn yaks(cwd: &Path, args: &[&str]) -> Output {
    yaks_env(cwd, "1", args)
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", err(&o));
    out(&o)
}

/// FNV-1a as `skills::digest` computes it (a copy: the binary's is private).
fn digest(s: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    format!("{h:016x}")
}

/// Simulate an install from an older yaks of older content: strip the stamp
/// off the installed SKILL.md, change the content, and re-stamp it at 0.0.0
/// with the digest of what remains. That is the `stale` state.
fn make_stale(skill_md: &Path) {
    let installed = std::fs::read_to_string(skill_md).unwrap();
    let mut body: String = installed
        .lines()
        .filter(|l| *l != "metadata:" && !l.trim_start().starts_with("yaks-"))
        .map(|l| format!("{l}\n"))
        .collect();
    body.push_str("<!-- written by an older yaks -->\n");
    let block = format!(
        "metadata:\n  yaks-version: \"0.0.0\"\n  yaks-digest: \"{}\"\n",
        digest(&body)
    );
    // The stamp goes last in the frontmatter, before the closing `---`.
    let close = body[4..].find("\n---\n").unwrap() + 4 + 1;
    body.insert_str(close, &block);
    std::fs::write(skill_md, body).unwrap();
}

/// A git repo with a farm and the default skills installed project-local.
fn repo(tag: &str) -> PathBuf {
    let root = temp(tag);
    git(&root, &["init", "-q"]);
    let o = yaks_env(&root, "0", &["init", "--skills", "none"]);
    assert!(o.status.success(), "{}{}", out(&o), err(&o));
    let o = yaks_env(&root, "0", &["skills", "install"]);
    assert!(o.status.success(), "{}{}", out(&o), err(&o));
    // Let the user-level auto-sync settle in the temp HOME, so its own notes
    // don't mix into what these tests assert about stderr.
    let o = yaks(&root, &["list"]);
    assert!(o.status.success(), "{}{}", out(&o), err(&o));
    root
}

const NOTICE_TAIL: &str = "run `yaks skills install` to update them";

#[test]
fn a_stale_install_is_announced_once_then_install_fixes_it() {
    let root = repo("once");
    let skill = root.join(".agents/skills/yaks/SKILL.md");
    assert_eq!(err(&yaks(&root, &["list"])), "", "current: silent");
    make_stale(&skill);
    let porcelain = git(&root, &["status", "--porcelain"]);
    let mtime = std::fs::metadata(&skill).unwrap().modified().unwrap();

    let first = yaks(&root, &["list"]);
    assert!(first.status.success());
    let e = err(&first);
    assert_eq!(e.lines().count(), 1, "exactly one stderr line: {e:?}");
    assert!(
        e.starts_with("note: the skills in .agents/skills are from yaks 0.0.0, this is ")
            && e.trim_end().ends_with(NOTICE_TAIL),
        "{e:?}"
    );
    assert_eq!(err(&yaks(&root, &["list"])), "", "second command: silent");
    // From a subdirectory too: the marker is per checkout, not per cwd.
    std::fs::create_dir_all(root.join("sub")).unwrap();
    assert_eq!(err(&yaks(&root.join("sub"), &["list"])), "");

    // The marker is in the git dir, so the working tree is exactly as it was.
    let marker = root.join(".git/yaks-skills-notified");
    assert!(marker.is_file());
    assert_eq!(git(&root, &["status", "--porcelain"]), porcelain);
    assert_eq!(
        std::fs::metadata(&skill).unwrap().modified().unwrap(),
        mtime
    );

    // A different yaks version later: told again.
    std::fs::write(&marker, "0.0.0\n").unwrap();
    assert_eq!(err(&yaks(&root, &["list"])).lines().count(), 1);
    assert_eq!(err(&yaks(&root, &["list"])), "");

    // `status` and `doctor` say the same thing in the same words.
    let status = out(&yaks(&root, &["skills", "status"]));
    assert!(
        status.contains("stale") && status.contains(NOTICE_TAIL),
        "{status}"
    );
    let doctor = out(&yaks(&root, &["doctor"]));
    assert!(
        doctor.contains("the skills in .agents/skills are from yaks 0.0.0")
            && doctor.contains(NOTICE_TAIL),
        "{doctor}"
    );

    // The command the notice names makes it current, and quiet.
    let o = yaks(&root, &["skills", "install"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&yaks(&root, &["skills", "status"])).contains("current"));
    std::fs::remove_file(&marker).unwrap();
    assert_eq!(err(&yaks(&root, &["list"])), "", "after install: silent");
    assert!(!marker.exists(), "nothing to announce, nothing remembered");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_notice_is_silent_for_json_the_switch_and_the_skills_command() {
    let root = repo("quiet");
    make_stale(&root.join(".agents/skills/yaks/SKILL.md"));
    let marker = root.join(".git/yaks-skills-notified");

    for off in ["0", "false", "never"] {
        let o = yaks_env(&root, off, &["list"]);
        assert_eq!(err(&o), "", "YAKS_SKILLS_AUTOSYNC={off}");
    }
    assert_eq!(err(&yaks(&root, &["list", "--json"])), "", "--json");
    // `skills` prints its own status; it never adds the notice line.
    let o = yaks(&root, &["skills", "status"]);
    assert!(!err(&o).contains("note:"), "{}", err(&o));
    assert!(!marker.exists(), "none of those used up the notice");

    // And the notice is still there for the next ordinary command.
    assert_eq!(err(&yaks(&root, &["list"])).lines().count(), 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_opt_in_stale_skill_names_with_coordination() {
    let root = temp("optin");
    git(&root, &["init", "-q"]);
    for args in [
        &["init", "--skills", "none"][..],
        &["skills", "install", "--with", "coordination"],
    ] {
        let o = yaks_env(&root, "0", args);
        assert!(o.status.success(), "{}{}", out(&o), err(&o));
    }
    yaks(&root, &["list"]); // settle the user-level sync
    make_stale(&root.join(".agents/skills/yaks-working/SKILL.md"));
    let e = err(&yaks(&root, &["list"]));
    assert!(
        e.trim_end()
            .ends_with("run `yaks skills install --with coordination` to update them"),
        "{e:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
