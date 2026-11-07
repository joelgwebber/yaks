//! Checkout weight (yaks-2b68, design yaks-9756): what a farm costs to clone,
//! and Git LFS for its binary originals.
//!
//! A plain `git clone` of a code repository fetches every branch, the farm
//! branch included, so every code clone pays for the farm's whole history:
//! each source, attachment and per-change commit. [`report`] measures that
//! (`yaks weight`); [`enable_lfs`] (`yaks lfs`) routes binary originals under
//! `.yaks/` to Git LFS on the farm branch, so a clone fetches small pointers
//! and the bytes come down only where the farm is checked out.

use anyhow::{Context, Result, bail};
use serde_json::{Value as Json, json};
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if !out.status.success() {
        bail!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// The farm worktree (git top-level) and its branch; refuses a farm that is
/// not on its own branch.
fn farm_top(root: &Path) -> Result<(PathBuf, String)> {
    let Some(branch) = crate::store::farm_branch(root) else {
        bail!(
            "{} is not on its own farm branch, so it has no git weight to measure \
             (start one with `yaks init`)",
            root.display()
        );
    };
    let top = root
        .parent()
        .context("the farm directory has no parent")?
        .to_path_buf();
    Ok((top, branch))
}

/// One kind of file in the farm's current tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bucket {
    pub files: usize,
    pub bytes: u64,
}

/// One blob in the farm branch's history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blob {
    pub path: String,
    pub bytes: u64,
}

/// What the farm branch weighs.
#[derive(Debug, Clone)]
pub struct Report {
    pub branch: String,
    pub commits: usize,
    /// Bytes on disk (compressed) of every object reachable from the farm
    /// branch, including any history it shares with code branches.
    pub history_bytes: u64,
    /// Of those, the bytes no other branch reaches: what a clone of the code
    /// repository fetches for the farm.
    pub farm_only_bytes: u64,
    /// The current tree, by kind: yaks, attachments, doks, sources, other.
    pub tree: Vec<(&'static str, Bucket)>,
    /// The biggest blobs only the farm branch holds, largest first.
    pub largest: Vec<Blob>,
    /// Whether the farm branch's `.gitattributes` routes files to LFS.
    pub lfs: bool,
    /// Bytes of binary files in the current tree stored as plain git blobs
    /// (not LFS pointers): what `yaks lfs` would convert.
    pub binary_in_git: u64,
}

/// The kind of a farm-tree path (relative to the farm branch's top, so
/// starting `.yaks/`).
fn kind_of(path: &str) -> &'static str {
    let Some(rel) = path.strip_prefix(".yaks/") else {
        return "other";
    };
    let parts: Vec<&str> = rel.split('/').collect();
    if parts.first() == Some(&crate::sources::SOURCES_DIR) {
        return "sources";
    }
    if parts.contains(&crate::doks::DOKS_DIR) {
        return "doks";
    }
    if parts.len() >= 3 && parts[1] == "yaks" {
        return if parts.len() == 3 {
            "yaks"
        } else {
            "attachments"
        };
    }
    "other"
}

/// File extensions of binary originals: routed to LFS by `yaks lfs`, and
/// counted as binary by the report.
pub const BINARY_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "heic", "bmp", "tif", "tiff", "ico", "psd", "pdf", "doc",
    "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "key", "pages", "numbers", "epub",
    "mp3", "m4a", "wav", "flac", "ogg", "mp4", "m4v", "mov", "webm", "mkv", "zip", "gz", "tgz",
    "tar", "bz2", "xz", "zst", "7z", "rar",
];

fn is_binary(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, e)| BINARY_EXTS.iter().any(|b| b.eq_ignore_ascii_case(e)))
}

/// How many largest blobs the report lists.
const LARGEST: usize = 10;

/// Measure the farm at `root` (a farm branch).
pub fn report(root: &Path) -> Result<Report> {
    let (top, branch) = farm_top(root)?;
    let commits = git(&top, &["rev-list", "--count", &branch])?
        .trim()
        .parse()
        .unwrap_or(0);
    let usage = |extra: &[&str]| -> Result<u64> {
        let mut args = vec!["rev-list", "--objects", "--disk-usage", branch.as_str()];
        args.extend_from_slice(extra);
        Ok(git(&top, &args)?.trim().parse().unwrap_or(0))
    };
    let history_bytes = usage(&[])?;
    let not_branch = format!("--exclude={branch}");
    let not_remote = format!("--exclude=*/{branch}");
    // Everything but the farm branch: what a code checkout has anyway.
    let others: [&str; 5] = ["--not", &not_branch, "--branches", &not_remote, "--remotes"];
    let farm_only_bytes = usage(&others)?;

    // The current tree: `<mode> <type> <sha> <size>\t<path>`.
    let mut tree: Vec<(&'static str, Bucket)> = ["yaks", "attachments", "doks", "sources", "other"]
        .into_iter()
        .map(|k| (k, Bucket::default()))
        .collect();
    let mut binary_in_git = 0u64;
    for line in git(&top, &["ls-tree", "-r", "-l", &branch])?.lines() {
        let Some((meta, path)) = line.split_once('\t') else {
            continue;
        };
        let size: u64 = meta
            .split_whitespace()
            .nth(3)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let k = kind_of(path);
        if let Some((_, b)) = tree.iter_mut().find(|(n, _)| *n == k) {
            b.files += 1;
            b.bytes += size;
        }
        // An LFS pointer is a small text blob; anything bigger is the bytes.
        if is_binary(path) && size > 200 {
            binary_in_git += size;
        }
    }

    // Largest farm-only blobs in history (a farm branch that once shared
    // history with code would otherwise list code files): every object with
    // the path it was first seen at, sized by cat-file.
    let mut args = vec!["rev-list", "--objects", branch.as_str()];
    args.extend_from_slice(&others);
    let objects = git(&top, &args)?;
    let mut child = Command::new("git")
        .arg("-C")
        .arg(&top)
        .args([
            "cat-file",
            "--batch-check=%(objecttype) %(objectsize) %(rest)",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("running git cat-file")?;
    {
        use std::io::Write;
        let mut stdin = child.stdin.take().context("git cat-file stdin")?;
        let feed = objects.clone();
        std::thread::spawn(move || {
            let _ = stdin.write_all(feed.as_bytes());
        });
    }
    let out = child.wait_with_output().context("reading git cat-file")?;
    let mut largest: Vec<Blob> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            let mut it = l.splitn(3, ' ');
            let (kind, size, path) = (it.next()?, it.next()?, it.next().unwrap_or(""));
            (kind == "blob").then(|| Blob {
                path: path.to_string(),
                bytes: size.parse().unwrap_or(0),
            })
        })
        .collect();
    largest.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));
    largest.truncate(LARGEST);

    let attrs = git(&top, &["show", &format!("{branch}:.gitattributes")]).unwrap_or_default();
    Ok(Report {
        branch,
        commits,
        history_bytes,
        farm_only_bytes,
        tree,
        largest,
        lfs: attrs.contains("filter=lfs"),
        binary_in_git,
    })
}

/// `1234567` -> `1.2 MiB`.
pub fn human(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= 1024.0 * 1024.0 {
        format!("{:.1} MiB", b / (1024.0 * 1024.0))
    } else if b >= 1024.0 {
        format!("{:.1} KiB", b / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

impl Report {
    pub fn to_json(&self) -> Json {
        let tree: serde_json::Map<String, Json> = self
            .tree
            .iter()
            .map(|(k, b)| (k.to_string(), json!({"files": b.files, "bytes": b.bytes})))
            .collect();
        json!({
            "branch": self.branch,
            "commits": self.commits,
            "history_bytes": self.history_bytes,
            "farm_only_bytes": self.farm_only_bytes,
            "tree": tree,
            "largest": self.largest.iter().map(|b| json!({"path": b.path, "bytes": b.bytes})).collect::<Vec<_>>(),
            "lfs": self.lfs,
            "binary_in_git": self.binary_in_git,
        })
    }

    pub fn render(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "farm branch {}: {} commits; a clone of this repository fetches {} for it \
             (objects no other branch has)\n",
            self.branch,
            self.commits,
            human(self.farm_only_bytes)
        ));
        if self.history_bytes > self.farm_only_bytes * 2 {
            s.push_str(&format!(
                "(its history reaches {} in all: it shares commits with code branches)\n",
                human(self.history_bytes)
            ));
        }
        s.push_str("\ncurrent tree:\n");
        for (k, b) in &self.tree {
            if b.files > 0 {
                s.push_str(&format!(
                    "  {k:<12} {:>5} files  {:>10}\n",
                    b.files,
                    human(b.bytes)
                ));
            }
        }
        if !self.largest.is_empty() {
            s.push_str("\nlargest blobs only the farm branch holds:\n");
            for b in &self.largest {
                s.push_str(&format!("  {:>10}  {}\n", human(b.bytes), b.path));
            }
        }
        s.push('\n');
        if self.lfs {
            s.push_str("lfs: on (binary originals under .yaks/ are stored in Git LFS)\n");
        } else {
            s.push_str(&format!(
                "lfs: off; {} of binary files in the tree are plain git blobs{}\n",
                human(self.binary_in_git),
                if self.binary_in_git > 0 {
                    " (`yaks lfs` moves them to Git LFS)"
                } else {
                    ""
                }
            ));
        }
        s
    }
}

/// The marker comment above the rules [`enable_lfs`] writes.
const LFS_MARKER: &str = "# yaks lfs: binary originals under .yaks/ are stored in Git LFS";

/// A case-insensitive glob for one extension: `png` -> `[pP][nN][gG]`.
fn ci_glob(ext: &str) -> String {
    ext.chars()
        .map(|c| {
            if c.is_ascii_alphabetic() {
                format!("[{}{}]", c.to_ascii_lowercase(), c.to_ascii_uppercase())
            } else {
                c.to_string()
            }
        })
        .collect()
}

/// The `.gitattributes` lines routing binary originals under `.yaks/` to LFS.
pub fn lfs_rules() -> Vec<String> {
    let mut out = vec![LFS_MARKER.to_string()];
    out.extend(BINARY_EXTS.iter().map(|e| {
        format!(
            ".yaks/**/*.{} filter=lfs diff=lfs merge=lfs -text",
            ci_glob(e)
        )
    }));
    out
}

/// What [`enable_lfs`] did.
#[derive(Debug, PartialEq, Eq)]
pub enum LfsOutcome {
    /// The rules were written and committed, with the binaries already in
    /// the tree converted to pointers.
    Enabled { commit: String, converted: usize },
    /// They were already in place.
    Already,
}

/// Route binary originals under `.yaks/` to Git LFS on the farm branch:
/// write the rules into the branch's `.gitattributes` (keeping its other
/// lines) and install LFS's hooks for this repository (`git lfs install
/// --local`), so a push uploads the bytes. One commit holds the rules and
/// the binaries already in the tree, converted to pointers; older history
/// keeps their plain blobs. The commit holds only those paths (anything else
/// staged stays staged). Refuses when `git lfs` is not installed.
pub fn enable_lfs(root: &Path) -> Result<LfsOutcome> {
    let (top, _) = farm_top(root)?;
    if git(&top, &["lfs", "version"]).is_err() {
        bail!(
            "git-lfs is not installed (`git lfs version` failed): install it (https://git-lfs.com), \
             then run `yaks lfs` again"
        );
    }
    // Hooks first: a farm that commits LFS pointers it cannot push is worse
    // than one without LFS.
    git(&top, &["lfs", "install", "--local"]).context(
        "installing Git LFS's hooks for this repository (`git lfs install --local`); \
         an existing hook may need `git lfs update --manual`",
    )?;
    let attrs = top.join(".gitattributes");
    let text = match std::fs::read_to_string(&attrs) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("reading {}", attrs.display())),
    };
    let have: Vec<&str> = text.lines().map(str::trim).collect();
    let missing: Vec<String> = lfs_rules()
        .into_iter()
        .filter(|r| !have.contains(&r.as_str()))
        .collect();
    if missing.is_empty() {
        return Ok(LfsOutcome::Already);
    }
    let mut new = text.clone();
    if !new.is_empty() && !new.ends_with('\n') {
        new.push('\n');
    }
    if !new.is_empty() {
        new.push('\n');
    }
    for r in &missing {
        new.push_str(r);
        new.push('\n');
    }
    crate::store::write::atomic_write(&attrs, &new)?;
    git(&top, &["add", "--", ".gitattributes"])?;
    // Binaries already in the tree would otherwise show as modified (git now
    // cleans them through LFS): convert them in the same commit. History
    // keeps their old blobs.
    let committed: Vec<String> = git(&top, &["ls-files", "--", ".yaks"])?
        .lines()
        .filter(|p| is_binary(p))
        .map(String::from)
        .collect();
    if !committed.is_empty() {
        let mut a = vec!["add", "--renormalize", "--"];
        a.extend(committed.iter().map(String::as_str));
        git(&top, &a)?;
    }
    let mut args: Vec<String> = Vec::new();
    for (key, value) in [("user.name", "yaks"), ("user.email", "yaks@localhost")] {
        if git(&top, &["config", "--get", key]).is_err() {
            args.push("-c".into());
            args.push(format!("{key}={value}"));
        }
    }
    args.extend(
        [
            "commit",
            "-q",
            "--only",
            "-m",
            "yaks: store binary originals in Git LFS",
            "--",
            ".gitattributes",
        ]
        .map(String::from),
    );
    args.extend(committed.iter().cloned());
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    git(&top, &a)?;
    let sha = git(&top, &["rev-parse", "--short", "HEAD"])?
        .trim()
        .to_string();
    Ok(LfsOutcome::Enabled {
        commit: sha,
        converted: committed.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_follow_the_farm_layout() {
        assert_eq!(kind_of(".yaks/yak/yaks/yak-1.md"), "yaks");
        assert_eq!(kind_of(".yaks/yak/yaks/yak-1/shot.png"), "attachments");
        assert_eq!(kind_of(".yaks/doks/topics/x.md"), "doks");
        assert_eq!(kind_of(".yaks/yak/doks/x.md"), "doks");
        // A dok's attachment directory (yaks-db66) is counted with its doks;
        // a session note's, like the notes themselves, under `other`.
        assert_eq!(kind_of(".yaks/doks/topics/x/shot.png"), "doks");
        assert_eq!(kind_of(".yaks/yak/doks/x/shot.png"), "doks");
        assert_eq!(kind_of(".yaks/sessions/2026/11/n/shot.png"), "other");
        assert_eq!(
            kind_of(".yaks/sources/2026-01-01-abcd1234/a.pdf"),
            "sources"
        );
        assert_eq!(kind_of(".yaks/config.yaml"), "other");
        assert_eq!(kind_of(".gitattributes"), "other");
    }

    #[test]
    fn rules_are_case_insensitive_and_scoped_to_the_farm() {
        let r = lfs_rules();
        assert_eq!(r[0], LFS_MARKER);
        assert!(
            r.contains(&".yaks/**/*.[pP][nN][gG] filter=lfs diff=lfs merge=lfs -text".to_string())
        );
        assert!(r.contains(&".yaks/**/*.7[zZ] filter=lfs diff=lfs merge=lfs -text".to_string()));
        assert!(
            !r.iter()
                .any(|l| l.contains("*.md") || l.contains("[sS][vV][gG]"))
        );
    }
}
