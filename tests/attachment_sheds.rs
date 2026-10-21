//! `yaks attachment <id> [<file>]` (yaks-31e2), driven through the real binary
//! against a real git repository with linked worktrees (sheds): an attachment
//! a worker added in its shed is found from the primary checkout, this
//! checkout's own file wins over a shed's copy, and a missing one exits 1.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

struct Repo {
    dir: PathBuf,
    sheds: Vec<PathBuf>,
}

fn git(dir: &Path, args: &[&str]) {
    let out = Proc::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run(dir: &Path, actor: &str, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(dir)
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", actor)
        .env_remove("YAKS_DIR")
        .args(args)
        .output()
        .unwrap()
}

fn yaks(dir: &Path, actor: &str, args: &[&str]) -> String {
    let out = run(dir, actor, args);
    assert!(
        out.status.success(),
        "{args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

impl Repo {
    /// A repo with one committed yak (`alpha`) and `names.len()` worktree
    /// sheds forked from that commit.
    fn new(tag: &str, names: &[&str]) -> (Repo, String) {
        let base = std::env::temp_dir().join(format!(
            "yaks-attachment-sheds-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("repo");
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        yaks(&dir, "main-actor", &["init"]);
        let v: Value =
            serde_json::from_str(&yaks(&dir, "main-actor", &["create", "alpha", "--json"]))
                .unwrap();
        let id = v["id"].as_str().unwrap().to_string();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "init"]);
        let sheds = names
            .iter()
            .map(|n| {
                let p = base.join(n);
                git(
                    &dir,
                    &["worktree", "add", "-q", "-b", n, p.to_str().unwrap()],
                );
                p
            })
            .collect();
        (Repo { dir, sheds }, id)
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

#[test]
fn an_attachment_added_in_a_shed_is_found_from_the_primary() {
    let (r, id) = Repo::new("att", &["worker"]);
    let worker = &r.sheds[0];
    let log = worker.join("build-log.txt");
    std::fs::write(&log, "ok\n").unwrap();
    yaks(worker, "worker", &["attach", &id, log.to_str().unwrap()]);
    // Not here: the primary's farm has no such file.
    let herd = id.split_once('-').unwrap().0;
    assert!(
        !r.dir
            .join(".yaks")
            .join(herd)
            .join("yaks")
            .join(&id)
            .join("build-log.txt")
            .exists()
    );

    let path = yaks(&r.dir, "main-actor", &["attachment", &id, "build-log.txt"]);
    let path = PathBuf::from(path.trim());
    // Canonical on both sides: on macOS the temp dir is a /var -> /private/var link.
    let canon = |p: &std::path::Path| p.canonicalize().unwrap();
    assert!(
        canon(&path).starts_with(canon(worker)),
        "{}",
        path.display()
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "ok\n");

    let listed = yaks(&r.dir, "main-actor", &["attachment", &id]);
    assert!(listed.starts_with("worker  "), "{listed}");
    assert!(listed.trim_end().ends_with("build-log.txt"), "{listed}");

    let v: Value =
        serde_json::from_str(&yaks(&r.dir, "main-actor", &["attachment", &id, "--json"])).unwrap();
    assert_eq!(v[0]["name"], "build-log.txt");
    assert_eq!(v[0]["shed"], "worker");

    // This checkout's own copy wins, and is marked `.`.
    let mine = r.dir.join("build-log.txt");
    std::fs::write(&mine, "mine\n").unwrap();
    yaks(
        &r.dir,
        "main-actor",
        &["attach", &id, mine.to_str().unwrap()],
    );
    let listed = yaks(&r.dir, "main-actor", &["attachment", &id]);
    assert!(listed.starts_with(".  "), "{listed}");
    assert_eq!(listed.lines().count(), 1, "{listed}");

    let missing = run(&r.dir, "main-actor", &["attachment", &id, "nope.txt"]);
    assert_eq!(missing.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("no attachment nope.txt"));
}
