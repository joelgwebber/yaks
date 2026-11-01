//! A Delta cloud runner's layout (yaks-b46b): a BARE managed host with no
//! branches, and clones that borrow its objects (`objects/info/alternates`).
//! The first yaks command in a clone must create the farm worktree in the
//! host (fetching the farm branch into it), and a second clone must find it.

mod common;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn git(dir: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn yaks(cwd: &Path, data: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yaks"))
        .current_dir(cwd)
        .env("XDG_DATA_HOME", data)
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .env_remove("YAKS_DIR")
        .args(args)
        .output()
        .unwrap()
}

fn ok(cwd: &Path, data: &Path, args: &[&str]) -> String {
    let o = yaks(cwd, data, args);
    assert!(
        o.status.success(),
        "yaks {args:?}: {}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

/// A clone of `host` that borrows its objects, with `origin` pointing at the
/// real upstream (as a Delta clone's does) and its refs fetched.
fn clone_of(host: &Path, origin: &Path, at: &Path) {
    git(
        host.parent().unwrap(),
        &[
            "clone",
            "-q",
            "--shared",
            host.to_str().unwrap(),
            at.to_str().unwrap(),
        ],
    );
    git(
        at,
        &["remote", "set-url", "origin", origin.to_str().unwrap()],
    );
    git(at, &["fetch", "-q", "origin"]);
}

#[test]
fn a_clone_of_a_bare_host_creates_the_farm_worktree_in_the_host() {
    let base = std::env::temp_dir().join(format!("yaks-barehost-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let base: PathBuf = base.canonicalize().unwrap();
    let data = base.join("data");

    // Upstream: a repo with `main` and a published farm branch.
    let seed = base.join("seed");
    common::git_init(&seed);
    std::fs::write(seed.join("README.md"), "x\n").unwrap();
    git(&seed, &["add", "README.md"]);
    git(&seed, &["commit", "-qm", "init"]);
    let seed_data = base.join("seed-data");
    ok(&seed, &seed_data, &["init", "--no-skills", "--herd", "t"]);
    let id = ok(&seed, &seed_data, &["create", "from upstream", "--json"]);
    let id = serde_json::from_str::<serde_json::Value>(&id).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    ok(&seed, &seed_data, &["commit"]);
    let origin = base.join("origin.git");
    git(&base, &["init", "-q", "--bare", origin.to_str().unwrap()]);
    git(
        &seed,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    );
    git(&seed, &["push", "-q", "origin", "main", "yaks"]);

    // The managed host: bare, `origin` set, and no branches at all.
    let host = base.join("host.git");
    git(&base, &["init", "-q", "--bare", host.to_str().unwrap()]);
    git(
        &host,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    );
    git(
        &host,
        &["fetch", "-q", "origin", "+refs/heads/main:refs/heads/main"],
    );

    let a = base.join("a");
    let b = base.join("b");
    clone_of(&host, &origin, &a);
    clone_of(&host, &origin, &b);
    assert!(
        std::fs::read_to_string(a.join(".git/objects/info/alternates")).is_ok(),
        "the clone borrows the host's objects"
    );

    // First command in clone a: the farm worktree appears, in the host.
    let shown = ok(&a, &data, &["show", &id]);
    assert!(shown.contains("from upstream"), "{shown}");
    let listed = git(&host, &["worktree", "list", "--porcelain"]);
    assert!(listed.contains("branch refs/heads/yaks"), "{listed}");
    let farm_a = ok(&a, &data, &["path"]);

    // Clone b finds the same farm, and a write in a is visible from b.
    assert_eq!(ok(&b, &data, &["path"]), farm_a);
    ok(&a, &data, &["update", &id, "--note", "written in a"]);
    assert!(ok(&b, &data, &["show", &id]).contains("written in a"));

    // The farm branch tracks origin, so sync can pull and push.
    assert_eq!(git(&host, &["config", "branch.yaks.remote"]), "origin");
    ok(&a, &data, &["sync"]);
    assert!(git(&origin, &["log", "--oneline", "yaks"]).contains(&id));

    let _ = std::fs::remove_dir_all(&base);
}
