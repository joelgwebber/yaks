//! `yaks init --mode/--skills/--path` end to end: the binary run in temp git
//! repos with a temp `$HOME`, so nothing outside the temp dirs is ever written.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("yaks-init-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    // Canonical, so paths printed by yaks compare equal on macOS (/var -> /private/var).
    d.canonicalize().unwrap()
}

fn yaks(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yaks"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", cwd.join("__home"))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env_remove("YAKS_DIR")
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

fn err(cwd: &Path, args: &[&str]) -> String {
    let o = yaks(cwd, args);
    assert!(
        !o.status.success(),
        "yaks {args:?} should fail:\n{}",
        text(&o)
    );
    text(&o)
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", text(&o));
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// A temp git repo with one commit (so `git status` is meaningful).
fn repo(tag: &str) -> PathBuf {
    let r = temp(tag);
    git(&r, &["init", "-q", "."]);
    std::fs::write(r.join("README.md"), "x\n").unwrap();
    git(&r, &["add", "README.md"]);
    git(&r, &["commit", "-qm", "init"]);
    r
}

fn status(r: &Path) -> String {
    git(r, &["status", "--short"])
}

fn exclude(r: &Path) -> String {
    std::fs::read_to_string(r.join(".git/info/exclude")).unwrap_or_default()
}

#[test]
fn plain_init_is_unchanged_and_hints_at_skills() {
    let r = repo("plain");
    let out = ok(&r, &["init"]);
    assert!(out.contains("Initialized empty yaks farm in"), "{out}");
    assert!(out.contains("yaks init --skills default"), "{out}");
    assert!(out.contains("yaks skills install"), "{out}");
    assert!(r.join(".yaks/hairy").is_dir());
    assert!(!r.join(".agents").exists(), "plain init installs no skills");
    assert_eq!(
        exclude(&r).lines().filter(|l| l.starts_with('/')).count(),
        0
    );
    // Still refuses to clobber, exactly as before.
    assert!(err(&r, &["init"]).contains("already exists"));
}

#[test]
fn plain_init_outside_git_points_at_the_user_install() {
    let d = temp("plain-nogit");
    let out = ok(&d, &["init"]);
    assert!(out.contains("not a git repo"), "{out}");
    assert!(!out.contains("init --skills"), "{out}");
}

#[test]
fn team_mode_creates_the_farm_and_skills_as_ordinary_project_files() {
    let r = repo("team");
    let out = ok(&r, &["init", "--mode", "team", "--skills", "default"]);
    assert!(out.contains("ordinary project files"), "{out}");
    assert!(r.join(".yaks/hairy").is_dir());
    assert!(r.join(".agents/skills/yaks/SKILL.md").is_file());
    assert!(r.join(".agents/skills/yaks-tracker/SKILL.md").is_file());
    let st = status(&r);
    assert!(st.contains(".yaks/"), "team farm is visible to git:\n{st}");
    assert!(
        st.contains(".agents/"),
        "team skills are visible to git:\n{st}"
    );
    assert!(
        !exclude(&r).contains("/.yaks"),
        "team mode excludes nothing"
    );
    assert!(!out.contains("YAKS_DIR"), "{out}");
}

#[test]
fn private_mode_keeps_farm_and_skills_out_of_git() {
    let r = repo("private");
    let out = ok(&r, &["init", "--mode", "private", "--skills", "default"]);
    assert!(r.join(".yaks/hairy").is_dir());
    assert!(r.join(".agents/skills/yaks/SKILL.md").is_file());
    assert_eq!(status(&r), "", "nothing of init may show in git status");
    let ex = exclude(&r);
    for line in [
        "/.yaks",
        "/.agents/skills/yaks",
        "/.agents/skills/yaks-tracker",
    ] {
        assert!(ex.lines().any(|l| l == line), "{line} missing from:\n{ex}");
    }
    assert!(!r.join(".gitignore").exists(), "never .gitignore");
    assert!(out.contains("YAKS_DIR="), "{out}");
    assert!(ok(&r, &["list"]).contains("No tasks"));
}

#[test]
fn mode_alone_installs_the_default_skills_and_skills_none_does_not() {
    let r = repo("mode-only");
    ok(&r, &["init", "--mode", "private"]);
    assert!(r.join(".agents/skills/yaks/SKILL.md").is_file());
    let r2 = repo("skills-none");
    let out = ok(&r2, &["init", "--mode", "private", "--skills", "none"]);
    assert!(out.contains("skills: none requested"), "{out}");
    assert!(!r2.join(".agents").exists());
}

#[test]
fn coordination_adds_the_group_and_excludes_each_directory_in_private_mode() {
    let r = repo("coord");
    ok(
        &r,
        &["init", "--mode", "private", "--skills", "coordination"],
    );
    assert!(
        r.join(".agents/skills/yaks-coordinating-delta/land.sh")
            .is_file()
    );
    assert_eq!(status(&r), "");
    assert!(exclude(&r).contains("/.agents/skills/yaks-coordinating-delta"));
}

#[test]
fn pointer_mode_creates_the_farm_at_the_path_and_discovery_follows_it() {
    let r = repo("pointer");
    let farm_dir = temp("pointer-farm");
    let farm_arg = farm_dir.to_str().unwrap();
    let out = ok(
        &r,
        &[
            "init", "--mode", "pointer", "--path", farm_arg, "--herd", "web", "--skills", "default",
        ],
    );
    assert!(
        farm_dir.join(".yaks/hairy").is_dir(),
        "farm created at the path"
    );
    let pointer = std::fs::read_to_string(r.join(".yaks")).unwrap();
    assert_eq!(pointer, format!("path: {farm_arg}\nherd: web\n"));
    assert_eq!(status(&r), "", "pointer and skills stay out of git");
    assert!(exclude(&r).lines().any(|l| l == "/.yaks"));
    assert!(
        out.contains(&format!("YAKS_DIR={}", farm_dir.join(".yaks").display())),
        "{out}"
    );

    // `yaks list` works from the repo root and from a subdirectory; create
    // routes into the pointer's herd.
    ok(&r, &["list"]);
    std::fs::create_dir_all(r.join("a/b")).unwrap();
    let made = ok(&r.join("a/b"), &["create", "--title", "hello"]);
    assert!(made.contains("web-"), "{made}");
    assert!(ok(&r.join("a/b"), &["list"]).contains("hello"));
    assert!(ok(&r, &["list"]).contains("hello"));
}

#[test]
fn pointer_mode_reuses_an_existing_farm_and_declares_the_herd() {
    let r = repo("pointer-existing");
    let farm_dir = temp("pointer-existing-farm");
    ok(&farm_dir, &["init"]);
    let before = std::fs::read_to_string(farm_dir.join(".yaks/config.yaml")).unwrap();
    let out = ok(
        &r,
        &[
            "init",
            "--mode",
            "pointer",
            "--path",
            farm_dir.to_str().unwrap(),
            "--herd",
            "web",
            "--skills",
            "none",
        ],
    );
    assert!(out.contains("already exists, unchanged"), "{out}");
    let after = std::fs::read_to_string(farm_dir.join(".yaks/config.yaml")).unwrap();
    assert!(after.starts_with(&before.trim_end().to_string()), "{after}");
    assert!(after.contains("  web:"), "herd declared:\n{after}");
    ok(&r, &["list"]);
}

#[test]
fn a_second_identical_run_changes_nothing_and_says_so() {
    for (tag, extra) in [
        ("idem-team", vec!["team"]),
        ("idem-private", vec!["private"]),
    ] {
        let r = repo(tag);
        let args = ["init", "--mode", extra[0], "--skills", "default"];
        ok(&r, &args);
        let ex = exclude(&r);
        let st = status(&r);
        let skill = std::fs::read(r.join(".agents/skills/yaks/SKILL.md")).unwrap();
        let out = ok(&r, &args);
        assert!(out.contains("Nothing to change"), "{out}");
        assert!(!out.contains("installed "), "{out}");
        assert_eq!(exclude(&r), ex);
        assert_eq!(status(&r), st);
        assert_eq!(
            std::fs::read(r.join(".agents/skills/yaks/SKILL.md")).unwrap(),
            skill
        );
    }
    // Pointer mode too.
    let r = repo("idem-pointer");
    let farm_dir = temp("idem-pointer-farm");
    let args = [
        "init",
        "--mode",
        "pointer",
        "--path",
        farm_dir.to_str().unwrap(),
    ];
    ok(&r, &args);
    let pointer = std::fs::read_to_string(r.join(".yaks")).unwrap();
    let ex = exclude(&r);
    assert!(ok(&r, &args).contains("Nothing to change"));
    assert_eq!(std::fs::read_to_string(r.join(".yaks")).unwrap(), pointer);
    assert_eq!(exclude(&r), ex);
}

#[test]
fn adding_skills_to_an_existing_team_farm_leaves_the_farm_alone() {
    let r = repo("add-skills");
    ok(&r, &["init", "--type", "feature"]);
    let config = std::fs::read(r.join(".yaks/config.yaml")).unwrap();
    let out = ok(&r, &["init", "--skills", "default"]);
    assert!(out.contains("already exists, unchanged"), "{out}");
    assert!(out.contains("installed yaks "), "{out}");
    assert!(r.join(".agents/skills/yaks/SKILL.md").is_file());
    assert_eq!(std::fs::read(r.join(".yaks/config.yaml")).unwrap(), config);
    assert!(!out.contains("Nothing to change"));
    // More flags later add only what is missing: the coordination group.
    let more = ok(&r, &["init", "--skills", "coordination"]);
    assert!(more.contains("installed yaks-coordinating "), "{more}");
    assert!(more.contains("ok yaks is already current"), "{more}");
}

#[test]
fn skills_alone_keeps_an_existing_private_farm_private() {
    let r = repo("skills-keep-private");
    ok(&r, &["init", "--mode", "private", "--skills", "none"]);
    ok(&r, &["init", "--skills", "default"]);
    assert_eq!(status(&r), "", "skills joined the private farm's excludes");
}

#[test]
fn an_edited_skill_is_never_overwritten() {
    let r = repo("edited");
    ok(&r, &["init", "--mode", "team", "--skills", "default"]);
    let path = r.join(".agents/skills/yaks/SKILL.md");
    let mut edited = std::fs::read_to_string(&path).unwrap();
    edited.push_str("\nmy local edit\n");
    std::fs::write(&path, &edited).unwrap();
    let out = ok(&r, &["init", "--mode", "team", "--skills", "default"]);
    assert!(out.contains("skip yaks [modified]"), "{out}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), edited);
}

#[test]
fn a_different_mode_on_an_existing_farm_is_an_error_naming_the_change() {
    let team = repo("conv-team");
    ok(&team, &["init"]);
    let e = err(&team, &["init", "--mode", "private"]);
    assert!(e.contains("team farm") && e.contains("info/exclude"), "{e}");
    assert!(!exclude(&team).contains("/.yaks"), "nothing converted");

    let private = repo("conv-private");
    ok(&private, &["init", "--mode", "private", "--skills", "none"]);
    let e = err(&private, &["init", "--mode", "team"]);
    assert!(
        e.contains("private farm") && e.contains("remove that line"),
        "{e}"
    );
    assert!(exclude(&private).contains("/.yaks"));
    let farm = temp("conv-farm");
    let e = err(
        &private,
        &[
            "init",
            "--mode",
            "pointer",
            "--path",
            farm.to_str().unwrap(),
        ],
    );
    assert!(e.contains("move it"), "{e}");
    assert!(!farm.join(".yaks").exists());

    let ptr = repo("conv-pointer");
    ok(
        &ptr,
        &[
            "init",
            "--mode",
            "pointer",
            "--path",
            farm.to_str().unwrap(),
            "--skills",
            "none",
        ],
    );
    let e = err(&ptr, &["init", "--mode", "team"]);
    assert!(e.contains("pointer file"), "{e}");
    let other = temp("conv-other");
    let e = err(
        &ptr,
        &[
            "init",
            "--mode",
            "pointer",
            "--path",
            other.to_str().unwrap(),
        ],
    );
    assert!(e.contains("already points elsewhere"), "{e}");
    assert!(!other.join(".yaks").exists() || e.contains("edit or remove"));
}

#[test]
fn conflicting_settings_on_an_existing_farm_are_refused() {
    let r = repo("settings");
    ok(&r, &["init"]);
    let e = err(&r, &["init", "--skills", "none", "--priority", "1"]);
    assert!(
        e.contains("--priority 1") && e.contains("config.yaml"),
        "{e}"
    );
}

#[test]
fn private_and_pointer_outside_git_are_clear_errors() {
    let d = temp("nogit");
    let e = err(&d, &["init", "--mode", "private"]);
    assert!(e.contains("needs a git repository"), "{e}");
    let farm = temp("nogit-farm");
    let e = err(
        &d,
        &[
            "init",
            "--mode",
            "pointer",
            "--path",
            farm.to_str().unwrap(),
        ],
    );
    assert!(e.contains("needs a git repository"), "{e}");
    let e = err(&d, &["init", "--skills", "default"]);
    assert!(
        e.contains("needs a git repository") && e.contains("yaks skills install"),
        "{e}"
    );
    assert!(!d.join(".yaks").exists(), "a refused init writes nothing");
    // A team farm with no skills needs no git.
    ok(&d, &["init", "--mode", "team", "--skills", "none"]);
    assert!(d.join(".yaks/hairy").is_dir());
}

#[test]
fn path_and_mode_pointer_go_together() {
    let r = repo("path-flags");
    assert!(err(&r, &["init", "--mode", "pointer"]).contains("--path"));
    assert!(err(&r, &["init", "--mode", "private", "--path", "x"]).contains("--mode pointer"));
    assert!(err(&r, &["init", "--path", "x"]).contains("--mode pointer"));
    assert!(!r.join(".yaks").exists());
}

#[test]
fn exclude_edits_keep_other_lines_and_a_missing_final_newline() {
    let r = repo("exclude-keep");
    std::fs::write(r.join(".git/info/exclude"), "# mine\n*.log").unwrap();
    ok(&r, &["init", "--mode", "private", "--skills", "none"]);
    assert_eq!(exclude(&r), "# mine\n*.log\n/.yaks\n");
    // And a repo whose info/ directory is missing gets it created.
    let r2 = repo("exclude-create");
    std::fs::remove_dir_all(r2.join(".git/info")).unwrap();
    ok(&r2, &["init", "--mode", "private", "--skills", "none"]);
    assert_eq!(exclude(&r2), "/.yaks\n");
}

#[test]
fn a_farm_in_a_subdirectory_is_excluded_at_its_own_path() {
    let r = repo("subdir");
    std::fs::create_dir_all(r.join("sub")).unwrap();
    ok(
        &r.join("sub"),
        &["init", "--mode", "private", "--skills", "none"],
    );
    assert!(
        exclude(&r).lines().any(|l| l == "/sub/.yaks"),
        "{}",
        exclude(&r)
    );
    assert_eq!(status(&r), "");
}

#[test]
fn a_linked_worktree_writes_the_shared_exclude() {
    let r = repo("linked");
    let wt = temp("linked-wt");
    std::fs::remove_dir_all(&wt).unwrap();
    git(
        &r,
        &[
            "worktree",
            "add",
            "-q",
            wt.to_str().unwrap(),
            "-b",
            "feature",
        ],
    );
    ok(&wt, &["init", "--mode", "private", "--skills", "none"]);
    assert_eq!(status(&wt), "");
    assert!(exclude(&r).lines().any(|l| l == "/.yaks"));
}
