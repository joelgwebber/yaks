//! The farm's local activity log (yaks-9eaa): which checkout (shed) wrote
//! which yak, and when. What `yaks sheds` and the TUI's Sheds tab show about
//! a shed's work comes from here, joined to discovery by checkout path.
//!
//! One file per checkout, `<farm>/.activity/<hash>.log`, where `<hash>` is a
//! stable hash of the checkout's canonical top-level path. Its first line is
//! `# <path>` (for a human reading it); every other line is `<ts>\t<id>`, one
//! per write of a yak file ([`crate::store::write::save`] calls [`record`]),
//! oldest first.
//!
//! It is per-machine metadata, never committed and never merged: a checkout
//! path means nothing on another machine. The directory ignores itself (it
//! holds a `.gitignore` of `*`), so `yaks status`/`yaks commit` and git never
//! see it, with no edit to the farm's own `.gitignore`. Best effort: a write
//! that cannot be logged is still a write, and a missing or unreadable log is
//! an empty one. Who wrote a note stays the note's actor, in the yak file.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// The log directory under the farm root.
pub const DIR: &str = ".activity";

/// One logged write: when, and which yak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub ts: String,
    pub id: String,
}

/// The log file of `checkout` (a canonical top-level path) in the farm at
/// `root`.
pub fn log_path(root: &Path, checkout: &Path) -> PathBuf {
    root.join(DIR).join(format!(
        "{:016x}.log",
        crate::store::stable_hash(&checkout.to_string_lossy())
    ))
}

/// The checkout this process writes from: the canonical git top-level of the
/// cwd, asked once. `None` outside a git repository (or with
/// `YAKS_NO_ACTIVITY` set), and then nothing is logged.
fn this_checkout() -> Option<&'static Path> {
    static HERE: OnceLock<Option<PathBuf>> = OnceLock::new();
    HERE.get_or_init(|| {
        if std::env::var_os("YAKS_NO_ACTIVITY").is_some_and(|v| !v.is_empty()) {
            return None;
        }
        let cwd = std::env::current_dir().ok()?;
        let out = Command::new("git")
            .arg("-C")
            .arg(&cwd)
            .args(["rev-parse", "--show-toplevel"])
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let top = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!top.is_empty()).then(|| fs::canonicalize(&top).unwrap_or_else(|_| PathBuf::from(top)))
    })
    .as_deref()
}

/// Log a write of yak `id` in the farm at `root`, from this process's
/// checkout. Best effort and silent.
pub fn record(root: &Path, id: &str) {
    if let Some(here) = this_checkout() {
        let _ = record_for(root, here, id, &crate::store::now_iso());
    }
}

/// [`record`] for an explicit checkout and timestamp (the testable core).
pub fn record_for(root: &Path, checkout: &Path, id: &str, ts: &str) -> std::io::Result<()> {
    let dir = root.join(DIR);
    if !dir.is_dir() {
        fs::create_dir_all(&dir)?;
    }
    let ignore = dir.join(".gitignore");
    if !ignore.exists() {
        fs::write(&ignore, "*\n")?;
    }
    let path = log_path(root, checkout);
    let new = !path.exists();
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let mut line = String::new();
    if new {
        line.push_str(&format!("# {}\n", checkout.display()));
    }
    line.push_str(&format!("{ts}\t{id}\n"));
    // One write per line, so concurrent appenders never interleave a line.
    f.write_all(line.as_bytes())
}

/// Every write `checkout` logged in the farm at `root`, oldest first. Empty
/// when there is no log.
pub fn read(root: &Path, checkout: &Path) -> Vec<Entry> {
    let Ok(text) = fs::read_to_string(log_path(root, checkout)) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let (ts, id) = l.split_once('\t')?;
            let (ts, id) = (ts.trim(), id.trim());
            (!ts.is_empty() && !id.is_empty()).then(|| Entry {
                ts: ts.to_string(),
                id: id.to_string(),
            })
        })
        .collect()
}

/// The distinct yaks in `entries`, each with its newest write, newest first.
pub fn latest_per_yak(entries: &[Entry]) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    for e in entries.iter().rev() {
        if !out.iter().any(|o| o.id == e.id) {
            out.push(e.clone());
        }
    }
    out.sort_by(|a, b| b.ts.cmp(&a.ts));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("yaks-activity-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn a_log_records_per_checkout_and_reads_back_in_order() {
        let root = tmp("order");
        let (a, b) = (Path::new("/src/a"), Path::new("/src/b"));
        record_for(&root, a, "y-0001", "2026-01-01T00:00:01Z").unwrap();
        record_for(&root, b, "y-0002", "2026-01-01T00:00:02Z").unwrap();
        record_for(&root, a, "y-0003", "2026-01-01T00:00:03Z").unwrap();
        let ids = |c| read(&root, c).into_iter().map(|e| e.id).collect::<Vec<_>>();
        assert_eq!(ids(a), ["y-0001", "y-0003"]);
        assert_eq!(ids(b), ["y-0002"]);
        assert!(read(&root, Path::new("/src/c")).is_empty());
        let text = fs::read_to_string(log_path(&root, a)).unwrap();
        assert!(text.starts_with("# /src/a\n"), "{text}");
        assert_eq!(
            fs::read_to_string(root.join(DIR).join(".gitignore")).unwrap(),
            "*\n"
        );
    }

    #[test]
    fn latest_per_yak_dedupes_and_sorts_newest_first() {
        let e = |ts: &str, id: &str| Entry {
            ts: ts.into(),
            id: id.into(),
        };
        let got = latest_per_yak(&[
            e("2026-01-01T00:00:01Z", "a"),
            e("2026-01-01T00:00:02Z", "b"),
            e("2026-01-01T00:00:03Z", "a"),
        ]);
        assert_eq!(
            got,
            [
                e("2026-01-01T00:00:03Z", "a"),
                e("2026-01-01T00:00:02Z", "b")
            ]
        );
    }
}
