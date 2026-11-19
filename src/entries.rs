//! What a directory of the farm holds, read one way (yaks-84fe, design
//! `/doks/design/files-directories-archives.md`): the TUI's directory view,
//! `yaks peek` of a directory link and the attachments count all list
//! through here, so they agree on what is shown.
//!
//! A [`Container`] is something that holds entries. Today that is a
//! directory; an archive is the next kind (yaks-93ee), read through the same
//! [`list`], so no caller learns about archives.
//!
//! The rules, held once: a farm-relative path never has an empty, `.`, `..`
//! or `.git` segment or a `\` ([`safe_rel`]); a listing leaves out `.git` and
//! symlinks (one could point out of the farm), and keeps every other dot
//! file, as directory sources and attached directories keep them.

use std::path::{Path, PathBuf};

/// One entry of a container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    /// Bytes, for a file (0 for a directory).
    pub size: u64,
}

/// Something that holds entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Container {
    /// A directory on disk.
    Dir(PathBuf),
}

/// Whether `rel`, a farm-relative path (no leading `/`; a trailing `/` is
/// allowed), is one a link may name: no empty, `.`, `..` or `.git` segment
/// and no `\`, so it cannot climb out of the farm or into git.
pub fn safe_rel(rel: &str) -> bool {
    let rel = rel.strip_suffix('/').unwrap_or(rel);
    !rel.is_empty()
        && !rel.contains('\\')
        && rel
            .split('/')
            .all(|c| !matches!(c, "" | "." | ".." | ".git"))
}

/// The container at farm path `rel` under `root`, when it is a safe path
/// that names one (a directory, today).
pub fn container_at(root: &Path, rel: &str) -> Option<Container> {
    if !safe_rel(rel) {
        return None;
    }
    let p = root.join(rel.trim_end_matches('/'));
    // `symlink_metadata`: a symlink to a directory is not one of ours.
    std::fs::symlink_metadata(&p)
        .is_ok_and(|m| m.is_dir())
        .then_some(Container::Dir(p))
}

/// Whether `name` is left out of every listing.
fn hidden(name: &str) -> bool {
    name == ".git"
}

/// The entries of `c` at `inner` (a path below it, `""` for its top):
/// directories first, then files, each sorted by name (case-insensitive,
/// then exactly). Symlinks and `.git` are left out. An unreadable place
/// lists nothing.
pub fn list(c: &Container, inner: &str) -> Vec<Entry> {
    let Container::Dir(dir) = c;
    let dir = if inner.is_empty() {
        dir.clone()
    } else {
        dir.join(inner)
    };
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<Entry> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if hidden(&name) {
                return None;
            }
            let t = e.file_type().ok()?;
            if t.is_symlink() || !(t.is_dir() || t.is_file()) {
                return None;
            }
            let size = if t.is_file() {
                e.metadata().map_or(0, |m| m.len())
            } else {
                0
            };
            Some(Entry {
                name,
                is_dir: t.is_dir(),
                size,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

/// Files in `c`, all the way down, counted to `cap` (so a huge tree costs a
/// bounded walk), by the listing's rules.
pub fn count_files(c: &Container, cap: usize) -> usize {
    fn walk(c: &Container, inner: &str, cap: usize, n: &mut usize) {
        for e in list(c, inner) {
            if *n >= cap {
                return;
            }
            let path = if inner.is_empty() {
                e.name
            } else {
                format!("{inner}/{}", e.name)
            };
            if e.is_dir {
                walk(c, &path, cap, n);
            } else {
                *n += 1;
            }
        }
    }
    let mut n = 0;
    walk(c, "", cap, &mut n);
    n
}

/// The top level of `c` as text, one entry a line, a directory as `name/`:
/// what `yaks peek` and the TUI's peek pane show for a directory link.
pub fn listing(c: &Container) -> String {
    let entries = list(c, "");
    if entries.is_empty() {
        return "(empty directory)\n".into();
    }
    entries
        .iter()
        .map(|e| {
            if e.is_dir {
                format!("{}/\n", e.name)
            } else {
                format!("{}\n", e.name)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(root: &Path, rel: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, "x").unwrap();
    }

    #[test]
    fn safe_paths_never_climb_out_or_into_git() {
        for ok in ["doks/x", "doks/x/", "a/.github/ci.yml"] {
            assert!(safe_rel(ok), "{ok}");
        }
        for bad in [
            "", "/", "a//b", "a/./b", "a/../b", "..", ".git", "a/.git/x", "a\\b",
        ] {
            assert!(!safe_rel(bad), "{bad}");
        }
    }

    #[test]
    fn a_listing_is_dirs_first_and_leaves_out_git_and_symlinks() {
        let t = tempfile_dir();
        let root = t.as_path();
        put(root, "d/b.txt");
        put(root, "d/A.txt");
        put(root, "d/.gitignore");
        put(root, "d/src/x.rs");
        put(root, "d/.git/HEAD");
        #[cfg(unix)]
        std::os::unix::fs::symlink("/etc", root.join("d/etc")).unwrap();
        let c = container_at(root, "d/").unwrap();
        let names: Vec<String> = list(&c, "").into_iter().map(|e| e.name).collect();
        assert_eq!(names, ["src", ".gitignore", "A.txt", "b.txt"]);
        assert_eq!(count_files(&c, 999), 4);
        assert_eq!(count_files(&c, 2), 2);
        assert_eq!(listing(&c), "src/\n.gitignore\nA.txt\nb.txt\n");
        assert_eq!(container_at(root, "d/b.txt"), None);
        assert_eq!(container_at(root, "d/../d"), None);
        #[cfg(unix)]
        assert_eq!(container_at(root, "d/etc"), None);
        let _ = std::fs::remove_dir_all(root);
    }

    fn tempfile_dir() -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let d = std::env::temp_dir().join(format!(
            "yaks-entries-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }
}
