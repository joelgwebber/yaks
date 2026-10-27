//! `yaks sync` (yaks-e3b2) through the real binary: a code repo, a bare
//! origin, and two machines' clones whose farms live on the farm branch.
//! First publish, concurrent edits to one yak converging from both sides with
//! no hand git, the half-syncs, a private farm, and a conflict the merge
//! driver cannot settle (left exactly as it was).

use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("yaks-sync-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

struct Machine {
    code: PathBuf,
    data: PathBuf,
}

impl Machine {
    fn yaks_out(&self, actor: &str, args: &[&str]) -> std::process::Output {
        Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.code)
            .env_remove("YAKS_DIR")
            .env("XDG_DATA_HOME", &self.data)
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", actor)
            .args(args)
            .output()
            .unwrap()
    }

    fn yaks(&self, args: &[&str]) -> String {
        let out = self.yaks_out("t", args);
        assert!(
            out.status.success(),
            "yaks {args:?}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    /// The farm worktree, wherever discovery put it.
    fn farm(&self) -> PathBuf {
        PathBuf::from(self.yaks(&["path"]).trim())
            .parent()
            .unwrap()
            .to_path_buf()
    }
}

fn identity(dir: &Path, who: &str) {
    git(dir, &["config", "user.name", who]);
    git(dir, &["config", "user.email", &format!("{who}@t")]);
}

/// Machine A: a code repo with a new farm branch (`t` herd, one yak) and an
/// empty bare origin. Returns (base, A, the yak id).
fn origin_and_a(tag: &str) -> (PathBuf, Machine, String) {
    let base = tmp(tag);
    git(&base, &["init", "-q", "--bare", "origin.git"]);
    let code = base.join("a");
    std::fs::create_dir_all(&code).unwrap();
    git(&code, &["init", "-q", "-b", "main"]);
    identity(&code, "a");
    git(&code, &["commit", "-q", "--allow-empty", "-m", "init"]);
    git(
        &code,
        &[
            "remote",
            "add",
            "origin",
            base.join("origin.git").to_str().unwrap(),
        ],
    );
    git(&code, &["push", "-q", "origin", "main"]);
    let farm = base.join("a-farm");
    git(
        &code,
        &[
            "worktree",
            "add",
            "-q",
            "--orphan",
            "-b",
            "yaks",
            farm.to_str().unwrap(),
        ],
    );
    let a = Machine {
        code,
        data: base.join("a-data"),
    };
    let init = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(&farm)
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .args(["init", "--herd", "t"])
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    let out = a.yaks(&["create", "shared", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    (base, a, v["id"].as_str().unwrap().to_string())
}

/// Machine B: a fresh clone; its first yaks command bootstraps the farm.
fn clone_b(base: &Path) -> Machine {
    git(
        base,
        &[
            "clone",
            "-q",
            base.join("origin.git").to_str().unwrap(),
            "b",
        ],
    );
    let b = Machine {
        code: base.join("b"),
        data: base.join("b-data"),
    };
    identity(&b.code, "b");
    b.yaks(&["list"]);
    b
}

fn remote_heads(base: &Path) -> String {
    git(
        base,
        &[
            "--git-dir",
            "origin.git",
            "for-each-ref",
            "--format=%(refname)",
        ],
    )
}

#[test]
fn first_sync_publishes_the_branch_with_the_driver_rule_and_sets_its_upstream() {
    let (base, a, _) = origin_and_a("publish");
    let out = a.yaks(&["sync"]);
    assert!(out.contains("committed .gitattributes"), "{out}");
    assert!(out.contains("origin has no yaks branch yet"), "{out}");
    assert!(out.contains("pushed 2 commit(s) to origin/yaks"), "{out}");
    assert!(out.contains("up to date with origin/yaks"), "{out}");
    assert!(remote_heads(&base).contains("refs/heads/yaks"));
    let farm = a.farm();
    assert_eq!(
        git(&farm, &["rev-parse", "--abbrev-ref", "yaks@{upstream}"]).trim(),
        "origin/yaks"
    );
    assert!(git(&farm, &["ls-files", ".gitattributes"]).contains(".gitattributes"));
    // A second sync has nothing to do.
    let again = a.yaks(&["sync"]);
    assert!(again.contains("nothing to commit"), "{again}");
    assert!(again.contains("nothing to push"), "{again}");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn concurrent_edits_to_one_yak_converge_from_both_machines() {
    let (base, a, id) = origin_and_a("converge");
    a.yaks(&["sync"]);
    let b = clone_b(&base);
    // Both machines edit the same yak before either syncs.
    a.yaks(&["update", &id, "--note", "from A"]);
    a.yaks(&["shave", &id]);
    b.yaks(&["update", &id, "--note", "from B"]);
    b.yaks(&["update", &id, "--add-label", "bee"]);
    a.yaks(&["sync"]);
    let out = b.yaks(&["sync"]);
    assert!(out.contains("pulled 1 commit(s)"), "{out}");
    assert!(out.contains("pushed 1 commit(s)"), "{out}");
    a.yaks(&["sync"]);
    let file = |m: &Machine| {
        std::fs::read_to_string(m.farm().join(format!(".yaks/t/yaks/{id}.md"))).unwrap()
    };
    let (fa, fb) = (file(&a), file(&b));
    assert_eq!(fa, fb, "both machines hold the same yak");
    assert!(fa.contains("from A") && fa.contains("from B"), "{fa}");
    assert_eq!(fa.matches("\nupdated:").count(), 1, "{fa}");
    assert!(
        fa.contains("status: shaving") && fa.contains("- bee"),
        "{fa}"
    );
    assert!(!fa.contains("<<<<<<<"));
    for m in [&a, &b] {
        assert!(m.yaks_out("t", &["doctor"]).status.success());
        let st = m.yaks(&["status"]);
        assert!(st.contains("up to date with origin/yaks"), "{st}");
    }
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn half_syncs_skip_their_step() {
    let (base, a, id) = origin_and_a("half");
    // --no-push: committed, nothing published.
    let out = a.yaks(&["sync", "--no-push"]);
    assert!(out.contains("push skipped (--no-push)"), "{out}");
    assert!(!remote_heads(&base).contains("refs/heads/yaks"));
    // --no-commit with a dirty farm: no commit, no pull, push what is committed.
    a.yaks(&["update", &id, "--note", "local only"]);
    let out = a.yaks(&["sync", "--no-commit"]);
    assert!(out.contains("commit skipped (--no-commit)"), "{out}");
    assert!(
        out.contains("pull skipped: the farm has uncommitted changes"),
        "{out}"
    );
    assert!(out.contains("pushed"), "{out}");
    assert!(
        !git(&a.farm(), &["status", "--porcelain"]).is_empty(),
        "the edit stays local"
    );
    // --no-pull: commits and pushes without fetching.
    let out = a.yaks(&["sync", "--no-pull"]);
    assert!(out.contains("pull skipped (--no-pull)"), "{out}");
    assert!(out.contains("pushed 1 commit(s)"), "{out}");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn a_private_farm_syncs_locally_but_never_pushes() {
    let (base, a, _) = origin_and_a("private");
    a.yaks(&["private"]);
    let out = a.yaks(&["sync"]);
    assert!(out.contains("push skipped: this farm is private"), "{out}");
    assert!(out.contains("not published"), "{out}");
    assert!(!remote_heads(&base).contains("refs/heads/yaks"));
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn a_conflict_the_driver_cannot_settle_aborts_and_changes_nothing() {
    let (base, a, _) = origin_and_a("conflict");
    a.yaks(&["sync"]);
    let b = clone_b(&base);
    // config.yaml is not a yak: the driver does not touch it.
    let edit = |m: &Machine, v: &str| {
        let p = m.farm().join(".yaks/config.yaml");
        let text = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, text.replace("default_priority: 3", v)).unwrap();
    };
    edit(&a, "default_priority: 1");
    edit(&b, "default_priority: 2");
    a.yaks(&["sync"]);
    let before = git(&b.farm(), &["rev-parse", "HEAD"]);
    let out = b.yaks_out("t", &["sync"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("config.yaml") && err.contains("aborted"),
        "{err}"
    );
    // B's own commit is kept, the rebase is gone, nothing was pushed over A.
    let farm = b.farm();
    assert!(!farm.join(".git").is_dir() || !farm.join(".git/rebase-merge").exists());
    assert!(git(&farm, &["status"]).contains("On branch yaks"));
    let after = git(&farm, &["rev-parse", "HEAD"]);
    assert_ne!(before, after, "B's edit was committed before the pull");
    assert!(
        std::fs::read_to_string(farm.join(".yaks/config.yaml"))
            .unwrap()
            .contains("default_priority: 2")
    );
    let _ = std::fs::remove_dir_all(&base);
}

/// `sync` refuses a farm that is not a farm branch (one reached through
/// YAKS_DIR): it has no branch to pull or push.
#[test]
fn a_yaks_dir_farm_is_refused() {
    let base = tmp("yaksdir");
    let farm = base.join(".yaks");
    std::fs::create_dir_all(&farm).unwrap();
    std::fs::write(farm.join("config.yaml"), "herd: t\n").unwrap();
    std::fs::write(farm.join("schema"), "6\n").unwrap();
    git(&base, &["init", "-q", "-b", "main"]);
    let out = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(&base)
        .env("YAKS_DIR", &farm)
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .args(["sync"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not on its own farm branch"));
    let _ = std::fs::remove_dir_all(&base);
}
