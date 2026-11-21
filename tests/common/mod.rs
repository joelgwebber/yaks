//! Shared setup for the integration tests: a farm is always a farm branch
//! (yaks-a99f), so a throwaway farm is a scratch git repo plus `yaks init`,
//! with the farm worktree put in a data dir beside it (never the real
//! `~/.local/share`).
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// `git init` in `dir` (created if absent).
pub fn git_init(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    // The `files` ref format whatever the user's `init.defaultrefformat`: the
    // git-spawn budgets in these tests count a files-format repository
    // (reftable is covered on its own, yaks-0a0a).
    let out = Command::new("git")
        .args([
            "-c",
            "init.defaultRefFormat=files",
            "init",
            "-q",
            "-b",
            "main",
        ])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git init: {out:?}");
}

/// The `XDG_DATA_HOME` for a test repo at `dir`: a sibling, so the farm
/// worktree lands outside the checkout.
pub fn data_home(dir: &Path) -> PathBuf {
    let name = dir.file_name().unwrap().to_string_lossy().into_owned();
    dir.with_file_name(format!("{name}-data"))
}

/// Remove a test repo and its data dir.
pub fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(data_home(dir));
}

pub struct Repo {
    pub dir: PathBuf,
    n: std::cell::Cell<u32>,
}

/// A scratch code repo with a farm (`yaks init`) beside it, for tests that
/// drive the real binary over commits, tags and yaks.
impl Repo {
    pub fn new(tag: &str) -> Repo {
        let dir = std::env::temp_dir().join(format!("yaks-release-{tag}-{}", std::process::id()));
        cleanup(&dir);
        git_init(&dir);
        let r = Repo {
            dir,
            n: Default::default(),
        };
        r.run(&["init", "--no-skills"]);
        r
    }

    pub fn cmd(&self, args: &[&str]) -> std::process::Output {
        assert_cmd::Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.dir)
            .env("XDG_DATA_HOME", data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", "tester")
            .args(args)
            .output()
            .unwrap()
    }

    pub fn run(&self, args: &[&str]) -> String {
        let out = self.cmd(args);
        assert!(
            out.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    pub fn git(&self, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
            .args(args)
            .current_dir(&self.dir)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// A commit changing a file; returns its full sha.
    pub fn commit(&self, message: &str) -> String {
        self.commit_in("f.txt", message)
    }

    pub fn commit_in(&self, file: &str, message: &str) -> String {
        let n = self.n.get() + 1;
        self.n.set(n);
        std::fs::write(self.dir.join(file), format!("{n}\n")).unwrap();
        self.git(&["add", file]);
        self.git(&["commit", "-q", "-m", message]);
        self.git(&["rev-parse", "HEAD"])
    }

    pub fn create(&self, title: &str, kind: &str, parent: Option<&str>) -> String {
        let mut args = vec!["create", title, "--type", kind, "--json"];
        if let Some(p) = parent {
            args.extend(["--parent", p]);
        }
        let v: serde_json::Value = serde_json::from_str(&self.run(&args)).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    pub fn changes(&self, args: &[&str]) -> String {
        let mut a = vec!["release", "changes"];
        a.extend(args);
        self.run(&a)
    }

    pub fn changes_json(&self, args: &[&str]) -> serde_json::Value {
        let mut a = vec!["--json"];
        a.extend(args);
        serde_json::from_str(&self.changes(&a)).unwrap()
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        cleanup(&self.dir);
    }
}
