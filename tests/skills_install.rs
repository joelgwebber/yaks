//! `yaks skills install` end to end: the binary run in a temp git repo with a
//! temp `$HOME`, so nothing outside the temp dir is ever written.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("yaks-skills-cli-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn yaks(cwd: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yaks"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("YAKS_SKILLS_AUTOSYNC", "0")
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

#[test]
fn installs_project_local_by_default_and_opt_in_adds_the_coordination_group() {
    let root = temp("project");
    let home = temp("project-home");
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join("sub")).unwrap();

    let out = yaks(&root.join("sub"), &home, &["skills", "install"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("project-local"), "{}", text(&out));
    assert!(root.join(".agents/skills/yaks/SKILL.md").is_file());
    assert!(root.join(".agents/skills/yaks-tracker/SKILL.md").is_file());
    assert!(!root.join(".agents/skills/yaks-working").exists());
    assert!(
        !home.join(".agents").exists(),
        "default must not touch $HOME"
    );

    let out = yaks(
        &root,
        &home,
        &["skills", "install", "--with", "coordination"],
    );
    assert!(out.status.success(), "{}", text(&out));
    assert!(root.join(".agents/skills/yaks-working/SKILL.md").is_file());
    assert!(
        root.join(".agents/skills/yaks-coordinating-delta/land.sh")
            .is_file()
    );

    let out = yaks(&root, &home, &["skills", "status"]);
    let t = text(&out);
    assert!(out.status.success(), "{t}");
    assert!(
        t.contains("yaks-coordinating-delta") && !t.contains("stale"),
        "{t}"
    );

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn user_flag_and_no_git_repo_use_the_home_directory() {
    let root = temp("user");
    let home = temp("user-home");
    std::fs::create_dir_all(root.join(".git")).unwrap();

    let out = yaks(&root, &home, &["skills", "install", "--user"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(home.join(".agents/skills/yaks/SKILL.md").is_file());
    assert!(!root.join(".agents").exists());

    // Outside any git repo, no flag: the user dir as before, and it says so.
    let plain = temp("nogit");
    let home2 = temp("nogit-home");
    let out = yaks(&plain, &home2, &["skills", "install"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("not inside a git repo"),
        "{}",
        text(&out)
    );
    assert!(home2.join(".agents/skills/yaks/SKILL.md").is_file());

    // --dir wins over the repo default.
    let dir = temp("dir");
    let out = yaks(
        &root,
        &home,
        &["skills", "install", "--dir", dir.to_str().unwrap()],
    );
    assert!(out.status.success(), "{}", text(&out));
    assert!(dir.join("yaks-tracker/SKILL.md").is_file());

    for d in [root, home, plain, home2, dir] {
        let _ = std::fs::remove_dir_all(d);
    }
}

#[test]
fn a_yaks_checkout_refuses_the_default_and_names_the_way_out() {
    let root = temp("srctree");
    let home = temp("srctree-home");
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join(".agents/skills")).unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"yaks\"\n").unwrap();

    let out = yaks(&root, &home, &["skills", "install", "--force"]);
    assert!(!out.status.success());
    let t = text(&out);
    assert!(t.contains("--user") && t.contains("--dir"), "{t}");
    assert!(!root.join(".agents/skills/yaks").exists());

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&home);
}
