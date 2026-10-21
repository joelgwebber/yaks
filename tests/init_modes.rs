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
    assert!(r.join(".yaks/schema").is_file());
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
    assert!(r.join(".yaks/schema").is_file());
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
    assert!(r.join(".yaks/schema").is_file());
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
        r.join(".agents/skills/yaks-coordinating-delta/SKILL.md")
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
        farm_dir.join(".yaks/schema").is_file(),
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
    assert!(d.join(".yaks/schema").is_file());
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

// -- --mode branch ------------------------------------------------------------

/// The data dir a branch-mode test points `XDG_DATA_HOME` at: a sibling of the
/// repo, so the farm worktree lands outside every checkout and the real data dir
/// is never touched.
fn data(r: &Path) -> PathBuf {
    let name = r.file_name().unwrap().to_string_lossy().into_owned();
    r.parent().unwrap().join(format!("{name}-xdg"))
}

fn yaks_x(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yaks"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", cwd.join("__home"))
        .env("XDG_DATA_HOME", data(cwd))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env_remove("YAKS_DIR")
        .env_remove("YAKS_NO_BOOTSTRAP")
        .output()
        .unwrap()
}

fn okx(cwd: &Path, args: &[&str]) -> String {
    let o = yaks_x(cwd, args);
    assert!(o.status.success(), "yaks {args:?} failed:\n{}", text(&o));
    text(&o)
}

fn errx(cwd: &Path, args: &[&str]) -> String {
    let o = yaks_x(cwd, args);
    assert!(
        !o.status.success(),
        "yaks {args:?} should fail:\n{}",
        text(&o)
    );
    text(&o)
}

/// The worktree that has `branch` checked out (unborn included), per git.
fn worktree_of(r: &Path, branch: &str) -> Option<PathBuf> {
    let list = git(r, &["worktree", "list", "--porcelain"]);
    let want = format!("branch refs/heads/{branch}");
    list.split("\n\n").find_map(|block| {
        block.lines().any(|l| l == want).then(|| {
            let p = block.lines().next()?.strip_prefix("worktree ")?;
            Path::new(p).canonicalize().ok()
        })?
    })
}

fn entries(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn branch_mode_creates_an_orphan_farm_worktree_at_farm_home() {
    let r = repo("branch-fresh");
    let out = okx(&r, &["init", "--mode", "branch", "--skills", "none"]);
    let wt = worktree_of(&r, "yaks").expect("the farm branch has a worktree");
    assert!(
        wt.starts_with(data(&r).canonicalize().unwrap().join("yaks/farms")),
        "worktree at farm_home, got {}\n{out}",
        wt.display()
    );
    assert!(out.contains(&wt.display().to_string()), "{out}");
    // Orphan: the worktree holds only the farm, and the branch has no commits,
    // so it shares no history with the code branch. Nothing is committed.
    assert_eq!(entries(&wt), vec![".git", ".yaks"]);
    assert!(wt.join(".yaks/config.yaml").is_file());
    let o = Command::new("git")
        .args([
            "-C",
            wt.to_str().unwrap(),
            "rev-parse",
            "--verify",
            "-q",
            "HEAD",
        ])
        .output()
        .unwrap();
    assert!(!o.status.success(), "the new farm branch has no commit yet");
    assert!(
        out.contains("yaks sync") && out.contains("yaks private"),
        "{out}"
    );
    // The code checkout is untouched, and discovery finds the farm from it.
    assert_eq!(status(&r), "");
    assert!(!r.join(".yaks").exists());
    let path = okx(&r, &["path"]);
    assert_eq!(path.trim(), wt.join(".yaks").display().to_string());
    okx(&r, &["create", "--title", "first yak"]);
    assert!(okx(&r, &["list"]).contains("first yak"));
    assert_eq!(status(&r), "");
}

#[test]
fn branch_mode_rerun_is_a_no_op() {
    let r = repo("branch-rerun");
    okx(&r, &["init", "--mode", "branch", "--skills", "none"]);
    let wt = worktree_of(&r, "yaks").unwrap();
    okx(&r, &["create", "--title", "keep me"]);
    let before = git(&r, &["worktree", "list", "--porcelain"]);
    let out = okx(&r, &["init", "--mode", "branch", "--skills", "none"]);
    assert!(out.contains("Nothing to change"), "{out}");
    assert!(out.contains("already"), "{out}");
    assert_eq!(git(&r, &["worktree", "list", "--porcelain"]), before);
    assert_eq!(worktree_of(&r, "yaks"), Some(wt));
    assert!(okx(&r, &["list"]).contains("keep me"));
    // A herd the farm does not have is a conflict, not a rewrite.
    assert!(
        errx(
            &r,
            &[
                "init", "--mode", "branch", "--skills", "none", "--herd", "other"
            ]
        )
        .contains("different settings")
    );
}

#[test]
fn branch_mode_records_a_chosen_branch_name() {
    let r = repo("branch-named");
    okx(
        &r,
        &[
            "init",
            "--mode",
            "branch",
            "--branch",
            "personal-joel",
            "--skills",
            "none",
        ],
    );
    assert_eq!(
        git(&r, &["config", "--get", "yaks.branch"]).trim(),
        "personal-joel"
    );
    let wt = worktree_of(&r, "personal-joel").expect("worktree on the chosen branch");
    assert!(worktree_of(&r, "yaks").is_none());
    assert_eq!(
        okx(&r, &["path"]).trim(),
        wt.join(".yaks").display().to_string()
    );
    // Re-running without --branch uses the recorded name: a no-op.
    let out = okx(&r, &["init", "--mode", "branch", "--skills", "none"]);
    assert!(out.contains("Nothing to change"), "{out}");
    // The default name records nothing.
    let d = repo("branch-default-name");
    okx(&d, &["init", "--mode", "branch", "--skills", "none"]);
    let o = Command::new("git")
        .args(["config", "--get", "yaks.branch"])
        .current_dir(&d)
        .output()
        .unwrap();
    assert!(
        !o.status.success(),
        "the default name is not written to config"
    );
}

#[test]
fn branch_mode_with_an_existing_origin_farm_only_adds_its_worktree() {
    let a = repo("branch-origin-a");
    okx(&a, &["init", "--mode", "branch", "--skills", "none"]);
    okx(&a, &["create", "--title", "from origin"]);
    let wa = worktree_of(&a, "yaks").unwrap();
    git(&wa, &["add", "-A"]);
    git(&wa, &["commit", "-qm", "the farm"]);
    let origin = temp("branch-origin-bare");
    git(&origin, &["init", "-q", "--bare", "."]);
    git(&a, &["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&a, &["push", "-q", "origin", "--all"]);
    let host = temp("branch-origin-b");
    let b = host.join("clone");
    git(
        &host,
        &["clone", "-q", origin.to_str().unwrap(), b.to_str().unwrap()],
    );
    assert!(worktree_of(&b, "yaks").is_none());
    let out = okx(&b, &["init", "--mode", "branch", "--skills", "none"]);
    let wb = worktree_of(&b, "yaks").expect("worktree added for the existing branch");
    assert!(wb.join(".yaks/config.yaml").is_file(), "{out}");
    assert!(okx(&b, &["list"]).contains("from origin"));
    // Only the worktree was added: the history is the pushed commit, no more.
    assert_eq!(git(&wb, &["rev-list", "--count", "HEAD"]).trim(), "1");
    assert_eq!(status(&b), "");
    assert!(
        okx(&b, &["init", "--mode", "branch", "--skills", "none"]).contains("Nothing to change")
    );
}

#[test]
fn branch_mode_refuses_an_in_tree_farm_and_changes_nothing() {
    let r = repo("branch-intree");
    ok(&r, &["init"]);
    let e = errx(&r, &["init", "--mode", "branch", "--skills", "none"]);
    assert!(e.contains("migrate --to-branch"), "{e}");
    assert!(worktree_of(&r, "yaks").is_none());
    assert!(!data(&r).exists(), "nothing created in the data dir");
}

#[test]
fn branch_mode_refuses_a_shared_clone_and_changes_nothing() {
    let r = repo("branch-shared");
    let b = temp("branch-shared-clone");
    let clone = b.join("clone");
    git(
        &b,
        &[
            "clone",
            "-q",
            "--shared",
            r.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let e = errx(&clone, &["init", "--mode", "branch", "--skills", "none"]);
    assert!(e.contains("borrows another repository's objects"), "{e}");
    assert!(!data(&clone).exists());
    assert!(worktree_of(&r, "yaks").is_none());
}

#[test]
fn branch_mode_flag_misuse_is_refused_before_any_change() {
    let r = repo("branch-flags");
    let e = errx(&r, &["init", "--mode", "branch", "--path", "x"]);
    assert!(e.contains("farm home") || e.contains("farm_home"), "{e}");
    let e = errx(&r, &["init", "--branch", "other", "--skills", "none"]);
    assert!(
        e.contains("--branch only applies to `--mode branch`"),
        "{e}"
    );
    let e = errx(
        &r,
        &[
            "init", "--mode", "branch", "--branch", "bad name", "--skills", "none",
        ],
    );
    assert!(e.contains("not a valid branch name"), "{e}");
    // The code branch is not a farm.
    let code = git(&r, &["symbolic-ref", "--short", "HEAD"]);
    let e = errx(
        &r,
        &[
            "init",
            "--mode",
            "branch",
            "--branch",
            code.trim(),
            "--skills",
            "none",
        ],
    );
    assert!(e.contains("the branch you are on"), "{e}");
    assert!(!data(&r).exists());
    let o = Command::new("git")
        .args(["config", "--get", "yaks.branch"])
        .current_dir(&r)
        .output()
        .unwrap();
    assert!(!o.status.success(), "a refused run records no branch name");
}

#[test]
fn branch_mode_skills_go_in_the_code_checkout_not_the_farm() {
    let r = repo("branch-skills");
    let out = okx(&r, &["init", "--mode", "branch", "--skills", "default"]);
    assert!(r.join(".agents/skills/yaks/SKILL.md").is_file(), "{out}");
    let wt = worktree_of(&r, "yaks").unwrap();
    assert!(!wt.join(".agents").exists());
    assert_eq!(
        exclude(&r).lines().filter(|l| l.starts_with('/')).count(),
        0
    );
    assert!(
        okx(&r, &["init", "--mode", "branch", "--skills", "default"]).contains("Nothing to change")
    );
}
