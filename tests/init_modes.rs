//! `yaks init` end to end: the binary run in temp git repos with a temp
//! `$HOME` and `XDG_DATA_HOME`, so nothing outside the temp dirs is written.
//! Every farm is a farm branch (yaks-a99f): `init` starts one, or with
//! `--farm` joins another repository's.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("yaks-init-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    // Canonical, so paths printed by yaks compare equal on macOS (/var -> /private/var).
    d.canonicalize().unwrap()
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
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

// -- own farm -----------------------------------------------------------------

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
fn init_creates_an_orphan_farm_worktree_at_farm_home() {
    let r = repo("branch-fresh");
    let out = okx(&r, &["init", "--skills", "none"]);
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
fn init_rerun_is_a_no_op() {
    let r = repo("branch-rerun");
    okx(&r, &["init", "--skills", "none"]);
    let wt = worktree_of(&r, "yaks").unwrap();
    okx(&r, &["create", "--title", "keep me"]);
    let before = git(&r, &["worktree", "list", "--porcelain"]);
    let out = okx(&r, &["init", "--skills", "none"]);
    assert!(out.contains("Nothing to change"), "{out}");
    assert!(out.contains("already"), "{out}");
    assert_eq!(git(&r, &["worktree", "list", "--porcelain"]), before);
    assert_eq!(worktree_of(&r, "yaks"), Some(wt));
    assert!(okx(&r, &["list"]).contains("keep me"));
    // A herd the farm does not have is a conflict, not a rewrite.
    assert!(
        errx(&r, &["init", "--skills", "none", "--herd", "other"]).contains("different settings")
    );
}

#[test]
fn init_records_a_chosen_branch_name() {
    let r = repo("branch-named");
    okx(
        &r,
        &["init", "--branch", "personal-joel", "--skills", "none"],
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
    let out = okx(&r, &["init", "--skills", "none"]);
    assert!(out.contains("Nothing to change"), "{out}");
    // The default name records nothing.
    let d = repo("branch-default-name");
    okx(&d, &["init", "--skills", "none"]);
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
fn init_with_an_existing_origin_farm_only_adds_its_worktree() {
    let a = repo("branch-origin-a");
    okx(&a, &["init", "--skills", "none"]);
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
    let out = okx(&b, &["init", "--skills", "none"]);
    let wb = worktree_of(&b, "yaks").expect("worktree added for the existing branch");
    assert!(wb.join(".yaks/config.yaml").is_file(), "{out}");
    assert!(okx(&b, &["list"]).contains("from origin"));
    // Only the worktree was added: the history is the pushed commit, no more.
    assert_eq!(git(&wb, &["rev-list", "--count", "HEAD"]).trim(), "1");
    assert_eq!(status(&b), "");
    assert!(okx(&b, &["init", "--skills", "none"]).contains("Nothing to change"));
}

#[test]
fn init_ignores_an_in_tree_dot_yaks() {
    // A leftover committed `.yaks/` is never read: init starts the branch farm.
    let r = repo("intree");
    std::fs::create_dir_all(r.join(".yaks")).unwrap();
    std::fs::write(r.join(".yaks/config.yaml"), "herd: old\n").unwrap();
    okx(&r, &["init", "--skills", "none"]);
    let wt = worktree_of(&r, "yaks").expect("the farm branch has a worktree");
    assert_eq!(
        okx(&r, &["path"]).trim(),
        wt.join(".yaks").display().to_string()
    );
}

#[test]
fn init_refuses_a_shared_clone_and_changes_nothing() {
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
    let e = errx(&clone, &["init", "--skills", "none"]);
    assert!(e.contains("borrows another repository's objects"), "{e}");
    assert!(!data(&clone).exists());
    assert!(worktree_of(&r, "yaks").is_none());
}

#[test]
fn init_flag_misuse_is_refused_before_any_change() {
    let r = repo("branch-flags");
    // The retired flags are gone, not ignored.
    for gone in [&["init", "--mode", "team"][..], &["init", "--path", "x"]] {
        assert!(errx(&r, gone).contains("unexpected argument"), "{gone:?}");
    }
    let e = errx(&r, &["init", "--branch", "bad name", "--skills", "none"]);
    assert!(e.contains("not a valid branch name"), "{e}");
    // The code branch is not a farm.
    let code = git(&r, &["symbolic-ref", "--short", "HEAD"]);
    let e = errx(&r, &["init", "--branch", code.trim(), "--skills", "none"]);
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
fn init_skills_go_in_the_code_checkout_not_the_farm() {
    let r = repo("branch-skills");
    let out = okx(&r, &["init", "--skills", "default"]);
    assert!(r.join(".agents/skills/yaks/SKILL.md").is_file(), "{out}");
    let wt = worktree_of(&r, "yaks").unwrap();
    assert!(!wt.join(".agents").exists());
    assert_eq!(
        exclude(&r).lines().filter(|l| l.starts_with('/')).count(),
        0
    );
    assert!(okx(&r, &["init", "--skills", "default"]).contains("Nothing to change"));
}

#[test]
fn init_outside_git_is_a_clear_error() {
    let d = temp("nogit");
    let e = errx(&d, &["init"]);
    assert!(e.contains("needs a git repository"), "{e}");
    assert!(!d.join(".yaks").exists());
}

#[test]
fn a_plain_init_installs_the_default_skills() {
    let r = repo("default-skills");
    okx(&r, &["init"]);
    assert!(r.join(".agents/skills/yaks/SKILL.md").is_file());
    assert!(!r.join(".agents/skills/yaks-coordinating").exists());
    okx(&r, &["init", "--skills", "coordination"]);
    assert!(
        r.join(".agents/skills/yaks-coordinating/SKILL.md")
            .is_file()
    );
}

// -- --farm: share another repository's farm --------------------------------

#[test]
fn farm_points_this_repo_at_another_repos_farm_branch() {
    let owner = repo("share-owner");
    okx(&owner, &["init", "--skills", "none", "--herd", "api"]);
    okx(&owner, &["create", "--title", "owner yak"]);
    let owner_farm = okx(&owner, &["path"]).trim().to_string();

    let web = repo("share-web");
    let out = okx(
        &web,
        &[
            "init",
            "--farm",
            owner.to_str().unwrap(),
            "--herd",
            "web",
            "--skills",
            "none",
        ],
    );
    assert!(out.contains("set yaks.farm"), "{out}");
    assert!(out.contains("set yaks.herd = web"), "{out}");
    assert!(out.contains("declared herd web"), "{out}");
    // Nothing in web's tree, no farm branch of its own; the setting is git config.
    assert_eq!(status(&web), "");
    assert!(worktree_of(&web, "yaks").is_none());
    assert_eq!(
        git(&web, &["config", "--get", "yaks.farm"]).trim(),
        owner.display().to_string()
    );
    // Discovery lands on the owner's farm; create routes into web's herd.
    assert_eq!(okx(&web, &["path"]).trim(), owner_farm);
    let made = okx(&web, &["create", "--title", "web yak"]);
    assert!(made.contains("Created web-"), "{made}");
    assert!(okx(&owner, &["list"]).contains("web yak"));
    // A re-run is a no-op; a different checkout or herd is refused.
    let again = okx(
        &web,
        &[
            "init",
            "--farm",
            owner.to_str().unwrap(),
            "--herd",
            "web",
            "--skills",
            "none",
        ],
    );
    assert!(again.contains("Nothing to change"), "{again}");
    let other = repo("share-other");
    okx(&other, &["init", "--skills", "none"]);
    let e = errx(
        &web,
        &[
            "init",
            "--farm",
            other.to_str().unwrap(),
            "--skills",
            "none",
        ],
    );
    assert!(e.contains("will not repoint"), "{e}");
    let e = errx(
        &web,
        &[
            "init",
            "--farm",
            owner.to_str().unwrap(),
            "--herd",
            "ops",
            "--skills",
            "none",
        ],
    );
    assert!(e.contains("already web"), "{e}");
    // A plain init in a repo that shares a farm does not start a second one.
    let e = errx(&web, &["init", "--skills", "none"]);
    assert!(e.contains("uses the farm of"), "{e}");
}

#[test]
fn farm_works_from_a_git_worktree_and_a_shared_clone_of_the_sharing_repo() {
    let owner = repo("share2-owner");
    okx(&owner, &["init", "--skills", "none"]);
    let owner_farm = okx(&owner, &["path"]).trim().to_string();
    let web = repo("share2-web");
    okx(
        &web,
        &[
            "init",
            "--farm",
            owner.to_str().unwrap(),
            "--skills",
            "none",
        ],
    );
    let wt = web.with_file_name("share2-web-wt");
    git(
        &web,
        &["worktree", "add", "-q", "-b", "topic", wt.to_str().unwrap()],
    );
    assert_eq!(okx(&wt, &["path"]).trim(), owner_farm);
    let clone = web.with_file_name("share2-web-clone");
    git(
        web.parent().unwrap(),
        &[
            "clone",
            "-q",
            "--shared",
            web.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    assert_eq!(okx(&clone, &["path"]).trim(), owner_farm);
}

#[test]
fn farm_refuses_a_repo_without_a_farm_or_this_same_repo_and_changes_nothing() {
    let owner = repo("share3-owner");
    let web = repo("share3-web");
    let e = errx(
        &web,
        &[
            "init",
            "--farm",
            owner.to_str().unwrap(),
            "--skills",
            "none",
        ],
    );
    assert!(e.contains("has no farm") && e.contains("yaks init"), "{e}");
    let e = errx(
        &web,
        &["init", "--farm", "/nowhere/at/all", "--skills", "none"],
    );
    assert!(e.contains("not a git checkout"), "{e}");
    okx(&web, &["init", "--skills", "none"]);
    let e = errx(
        &web,
        &["init", "--farm", web.to_str().unwrap(), "--skills", "none"],
    );
    assert!(e.contains("this same repository"), "{e}");
    let e = errx(
        &web,
        &[
            "init",
            "--farm",
            owner.to_str().unwrap(),
            "--branch",
            "x",
            "--skills",
            "none",
        ],
    );
    assert!(e.contains("does not go with --farm"), "{e}");
    let o = Command::new("git")
        .args(["config", "--get", "yaks.farm"])
        .current_dir(&web)
        .output()
        .unwrap();
    assert!(!o.status.success(), "a refused run records nothing");
}
