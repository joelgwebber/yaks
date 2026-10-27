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
    let out = Command::new("git")
        .args(["init", "-q", "-b", "main"])
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
