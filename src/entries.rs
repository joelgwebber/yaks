//! What a directory of the farm holds, read one way (yaks-84fe, design
//! `/doks/design/files-directories-archives.md`): the TUI's directory view,
//! `yaks peek` of a directory link and the attachments count all list
//! through here, so they agree on what is shown.
//!
//! A [`Container`] is something that holds entries: a directory, or an
//! archive (`.zip`, `.tar`, `.tar.gz`, `.tgz`, yaks-93ee) read as a read-only
//! directory through the same [`list`] and [`read`], so no caller learns about
//! archives. A path walks *through* an archive like a directory
//! (`<id>/code.zip/src/main.rs`): [`container_at`] finds the archive on a
//! farm path and says what is left inside it.
//!
//! The rules, held once: a farm-relative path never has an empty, `.`, `..`
//! or `.git` segment or a `\` ([`safe_rel`]); a listing leaves out `.git` and
//! symlinks (one could point out of the farm), and keeps every other dot
//! file, as directory sources and attached directories keep them.

use anyhow::{Result, bail};
use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Largest archive file that is browsed; a bigger one stays a plain file.
pub const ARCHIVE_MAX_BYTES: u64 = 50 << 20;

/// Most members an archive's index holds; the rest are not listed.
pub const MAX_MEMBERS: usize = 10_000;

/// Most bytes of a compressed tar read while indexing or reading a member
/// (a compression bomb stops here).
const STREAM_MAX_BYTES: u64 = 512 << 20;

/// One entry of a container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    /// A directory, or an archive that unfolds like one ([`Entry::archive`]).
    pub is_dir: bool,
    /// Bytes, for a file or an archive (0 for a directory).
    pub size: u64,
    /// A browsable archive file in a directory listing: `is_dir`, but one
    /// file to the person counting files and to the OS opener.
    pub archive: bool,
}

/// Something that holds entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Container {
    /// A directory on disk.
    Dir(PathBuf),
    /// A `.zip` file.
    Zip(PathBuf),
    /// A `.tar` file, or a gzipped one (`.tar.gz`, `.tgz`).
    Tar { path: PathBuf, gz: bool },
}

/// What a name inside a container is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Node {
    Dir,
    File(u64),
}

/// The archive format a file name says, by its extension (any case).
fn archive_kind(name: &str) -> Option<ArchiveKind> {
    let n = name.to_ascii_lowercase();
    if n.ends_with(".zip") {
        Some(ArchiveKind::Zip)
    } else if n.ends_with(".tar.gz") || n.ends_with(".tgz") {
        Some(ArchiveKind::Tar { gz: true })
    } else if n.ends_with(".tar") {
        Some(ArchiveKind::Tar { gz: false })
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchiveKind {
    Zip,
    Tar { gz: bool },
}

/// Whether `name` has an extension of an archive format this module browses.
pub fn is_browsable_name(name: &str) -> bool {
    archive_kind(name).is_some()
}

impl ArchiveKind {
    fn container(self, path: PathBuf) -> Container {
        match self {
            ArchiveKind::Zip => Container::Zip(path),
            ArchiveKind::Tar { gz } => Container::Tar { path, gz },
        }
    }
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

/// The archive on path `rel` below `base`, and the path left inside it
/// (`""` for its top). `None` when no segment is an archive file (a
/// supported format, under [`ARCHIVE_MAX_BYTES`], that opens), or the walk
/// meets a missing place or a symlink first.
fn through_archive(base: &Path, rel: &str) -> Option<(Container, String)> {
    let segs: Vec<&str> = rel.split('/').filter(|s| !s.is_empty()).collect();
    let mut cur = base.to_path_buf();
    for (i, seg) in segs.iter().enumerate() {
        cur.push(seg);
        let m = std::fs::symlink_metadata(&cur).ok()?;
        if m.is_dir() {
            continue;
        }
        if !m.is_file() || m.len() > ARCHIVE_MAX_BYTES {
            return None;
        }
        let c = archive_kind(seg)?.container(cur);
        index(&c)?;
        return Some((c, segs[i + 1..].join("/")));
    }
    None
}

/// The container at farm path `rel` under `root` and the path inside it,
/// when it is a safe path that names one: a directory (inner `""`), or an
/// archive on the path (`a.zip`, or `a.zip/src/x.rs`: the inner path may
/// name a file or a directory of it, [`stat`] says which).
pub fn container_at(root: &Path, rel: &str) -> Option<(Container, String)> {
    if !safe_rel(rel) {
        return None;
    }
    let rel = rel.trim_end_matches('/');
    if let Some(found) = through_archive(root, rel) {
        return Some(found);
    }
    let p = root.join(rel);
    // `symlink_metadata`: a symlink to a directory is not one of ours.
    std::fs::symlink_metadata(&p)
        .is_ok_and(|m| m.is_dir())
        .then_some((Container::Dir(p), String::new()))
}

/// [`container_at`] when `rel` names a directory: a real one, an archive's
/// top, or a directory inside an archive. A file inside an archive is not
/// one.
pub fn dir_at(root: &Path, rel: &str) -> Option<(Container, String)> {
    let (c, inner) = container_at(root, rel)?;
    (stat(&c, &inner) == Some(Node::Dir)).then_some((c, inner))
}

/// What `inner` is in `c`: a directory (an archive's top included), a file
/// with its size, or nothing. A symlink is nothing.
pub fn stat(c: &Container, inner: &str) -> Option<Node> {
    match c {
        Container::Dir(d) => {
            if let Some((c, i)) = through_archive(d, inner) {
                return stat(&c, &i);
            }
            let m = std::fs::symlink_metadata(d.join(inner)).ok()?;
            if m.is_dir() {
                Some(Node::Dir)
            } else if m.is_file() {
                Some(Node::File(m.len()))
            } else {
                None
            }
        }
        _ => {
            let idx = index(c)?;
            let inner = inner.trim_end_matches('/');
            if inner.is_empty() {
                return Some(Node::Dir);
            }
            let (parent, name) = inner.rsplit_once('/').unwrap_or(("", inner));
            idx.dirs.get(parent)?.get(name).map(|m| m.node())
        }
    }
}

/// Whether the name is left out of every listing.
fn hidden(name: &str) -> bool {
    name == ".git"
}

/// The entries of `c` at `inner` (a path below it, `""` for its top; it may
/// pass through an archive): directories first, then files, each sorted by
/// name (case-insensitive, then exactly). Symlinks and `.git` are left out. An
/// unreadable place lists nothing.
pub fn list(c: &Container, inner: &str) -> Vec<Entry> {
    let mut out = match c {
        Container::Dir(d) => {
            if let Some((c, i)) = through_archive(d, inner) {
                return list(&c, &i);
            }
            list_dir(d, inner)
        }
        _ => list_archive(c, inner),
    };
    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

fn list_dir(dir: &Path, inner: &str) -> Vec<Entry> {
    let dir = if inner.is_empty() {
        dir.to_path_buf()
    } else {
        dir.join(inner)
    };
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    rd.flatten()
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
            // The name, the size and the first bytes say it is an archive:
            // indexing each one to list a directory would read them all.
            let archive = t.is_file()
                && size <= ARCHIVE_MAX_BYTES
                && archive_kind(&name).is_some_and(|k| sniff(&e.path(), k));
            Some(Entry {
                name,
                is_dir: t.is_dir() || archive,
                size,
                archive,
            })
        })
        .collect()
}

/// Whether the file's first bytes are those of archive format `k` (a cheap
/// check that a `.tgz` of text is not offered as one; opening it for real
/// is [`index`]'s job).
fn sniff(path: &Path, k: ArchiveKind) -> bool {
    let mut head = [0u8; 262];
    let n = std::fs::File::open(path)
        .and_then(|mut f| f.read(&mut head))
        .unwrap_or(0);
    match k {
        ArchiveKind::Zip => head[..n].starts_with(b"PK"),
        ArchiveKind::Tar { gz: true } => head[..n].starts_with(&[0x1f, 0x8b]),
        // A tar's `ustar` magic is at 257; an old v7 tar has none, so a tar
        // is judged by the real index.
        ArchiveKind::Tar { gz: false } => {
            n >= 257
                && index(&ArchiveKind::Tar { gz: false }.container(path.to_path_buf())).is_some()
        }
    }
}

fn list_archive(c: &Container, inner: &str) -> Vec<Entry> {
    let Some(idx) = index(c) else {
        return Vec::new();
    };
    let Some(children) = idx.dirs.get(inner.trim_end_matches('/')) else {
        return Vec::new();
    };
    children
        .iter()
        .map(|(name, m)| Entry {
            name: name.clone(),
            is_dir: m.node() == Node::Dir,
            size: match m.node() {
                Node::File(n) => n,
                Node::Dir => 0,
            },
            archive: false,
        })
        .collect()
}

/// The bytes of file `inner` of `c`, at most `max`: a bigger file is
/// refused (by its declared size, and by what is actually read, through
/// `take(max + 1)`, so a lying or bomb-like member costs a bounded read).
pub fn read(c: &Container, inner: &str, max: u64) -> Result<Vec<u8>> {
    let too_big = || anyhow::anyhow!("{inner}: too big to read here (over {max} bytes)");
    let bounded = |r: &mut dyn Read| -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        r.take(max + 1).read_to_end(&mut buf)?;
        if buf.len() as u64 > max {
            return Err(too_big());
        }
        Ok(buf)
    };
    match c {
        Container::Dir(d) => {
            if let Some((c, i)) = through_archive(d, inner) {
                return read(&c, &i, max);
            }
            if !safe_rel(inner) {
                bail!("{inner}: not a path of this farm");
            }
            let p = d.join(inner);
            if !std::fs::symlink_metadata(&p).is_ok_and(|m| m.is_file()) {
                bail!("{inner}: no such file");
            }
            bounded(&mut std::fs::File::open(&p)?)
        }
        Container::Zip(path) => {
            let Some(Node::File(size)) = stat(c, inner) else {
                bail!("{inner}: no such file in the archive");
            };
            if size > max {
                return Err(too_big());
            }
            let ord = member_ord(c, inner).ok_or_else(|| anyhow::anyhow!("{inner}: not found"))?;
            let f = std::io::BufReader::new(std::fs::File::open(path)?);
            let mut z = zip::ZipArchive::new(f)?;
            let mut m = z.by_index(ord)?;
            bounded(&mut m)
        }
        Container::Tar { path, gz } => {
            let Some(Node::File(size)) = stat(c, inner) else {
                bail!("{inner}: no such file in the archive");
            };
            if size > max {
                return Err(too_big());
            }
            let ord = member_ord(c, inner).ok_or_else(|| anyhow::anyhow!("{inner}: not found"))?;
            let reader = tar_reader(path, *gz)?;
            let mut ar = tar::Archive::new(reader);
            for (i, e) in ar.entries()?.enumerate() {
                let mut e = e?;
                if i == ord {
                    return bounded(&mut e);
                }
            }
            bail!("{inner}: not found in the archive")
        }
    }
}

/// The text of the top of `c` at `inner`: what `yaks peek` and the TUI's
/// peek pane show for a directory link.
pub fn listing(c: &Container, inner: &str) -> String {
    let entries = list(c, inner);
    if entries.is_empty() {
        return "(empty directory)\n".into();
    }
    entries
        .iter()
        .map(|e| {
            if e.is_dir && !e.archive {
                format!("{}/\n", e.name)
            } else {
                format!("{}\n", e.name)
            }
        })
        .collect()
}

/// Files in `c` at `inner`, all the way down, counted to `cap` (so a huge
/// tree costs a bounded walk), by the listing's rules. A browsable archive
/// file counts as one file; its members are not counted.
pub fn count_files(c: &Container, inner: &str, cap: usize) -> usize {
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
            if e.is_dir && !e.archive {
                walk(c, &path, cap, n);
            } else {
                *n += 1;
            }
        }
    }
    let mut n = 0;
    walk(c, inner, cap, &mut n);
    n
}

// ------------------------------------------------------------ archive index

/// One name of an archive: a directory, or a file with its size and its
/// ordinal among the archive's entries (how a read finds it again).
#[derive(Debug, Clone, Copy)]
enum Member {
    Dir,
    File { size: u64, ord: usize },
}

impl Member {
    fn node(&self) -> Node {
        match self {
            Member::Dir => Node::Dir,
            Member::File { size, .. } => Node::File(*size),
        }
    }
}

/// An archive's members as directories: `""` is the top, every other key a
/// directory's path; each holds its children by name. Directories the
/// archive never lists are made up from the paths below them.
#[derive(Debug, Default)]
struct Index {
    dirs: HashMap<String, BTreeMap<String, Member>>,
}

impl Index {
    fn add(&mut self, path: &str, m: Member) {
        let mut parent = String::new();
        let mut segs = path.split('/').peekable();
        while let Some(seg) = segs.next() {
            let child = if parent.is_empty() {
                seg.to_string()
            } else {
                format!("{parent}/{seg}")
            };
            let dir = self.dirs.entry(parent).or_default();
            if segs.peek().is_some() {
                // An ancestor: a directory, even where a file of that name came first.
                dir.insert(seg.to_string(), Member::Dir);
            } else if !matches!((dir.get(seg), m), (Some(Member::Dir), Member::File { .. })) {
                dir.insert(seg.to_string(), m);
            }
            parent = child;
        }
        if matches!(m, Member::Dir) {
            self.dirs.entry(path.to_string()).or_default();
        }
    }
}

/// A member's name as a farm-style path: no leading `./`, no trailing `/`.
/// `None` for a name that is skipped (see the module's archive rules).
fn clean_member(raw: &str) -> Option<String> {
    let mut s = raw;
    while let Some(r) = s.strip_prefix("./") {
        s = r;
    }
    let s = s.trim_end_matches('/');
    if s.is_empty() || s.starts_with('/') || s.contains('\\') {
        return None;
    }
    let bad = |seg: &str| {
        matches!(seg, "" | "." | ".." | ".git")
            // A Windows drive (`C:`) would climb out of a temp copy's directory.
            || (seg.len() >= 2 && seg.as_bytes()[1] == b':' && seg.as_bytes()[0].is_ascii_alphabetic())
    };
    s.split('/').all(|seg| !bad(seg)).then(|| s.to_string())
}

fn tar_reader(path: &Path, gz: bool) -> Result<Box<dyn Read>> {
    let f = std::io::BufReader::new(std::fs::File::open(path)?);
    Ok(if gz {
        Box::new(flate2::read::GzDecoder::new(f).take(STREAM_MAX_BYTES))
    } else {
        Box::new(f.take(STREAM_MAX_BYTES))
    })
}

fn build_zip(path: &Path) -> Option<Index> {
    let f = std::io::BufReader::new(std::fs::File::open(path).ok()?);
    let mut z = zip::ZipArchive::new(f).ok()?;
    let mut idx = Index::default();
    idx.dirs.entry(String::new()).or_default();
    for i in 0..z.len().min(MAX_MEMBERS) {
        let Ok(e) = z.by_index_raw(i) else { continue };
        let Some(name) = clean_member(e.name()) else {
            continue;
        };
        if e.is_symlink() {
            continue;
        }
        let m = if e.is_dir() {
            Member::Dir
        } else {
            Member::File {
                size: e.size(),
                ord: i,
            }
        };
        idx.add(&name, m);
    }
    Some(idx)
}

fn build_tar(path: &Path, gz: bool) -> Option<Index> {
    let mut ar = tar::Archive::new(tar_reader(path, gz).ok()?);
    let mut idx = Index::default();
    idx.dirs.entry(String::new()).or_default();
    let mut seen = 0;
    let entries = ar.entries().ok()?;
    for (i, e) in entries.enumerate() {
        // A broken stream ends the listing; one that never gave a member is
        // not a tar at all.
        let Ok(e) = e else {
            if seen == 0 {
                return None;
            }
            break;
        };
        if i >= MAX_MEMBERS {
            break;
        }
        seen += 1;
        let t = e.header().entry_type();
        let m = if t.is_dir() {
            Member::Dir
        } else if t.is_file() {
            Member::File {
                size: e.size(),
                ord: i,
            }
        } else {
            continue; // symlinks, hard links, devices, ...
        };
        let raw = e.path_bytes();
        let Ok(raw) = std::str::from_utf8(&raw) else {
            continue;
        };
        if let Some(name) = clean_member(raw) {
            idx.add(&name, m);
        }
    }
    Some(idx)
}

/// An archive's index, built once per (path, mtime, size) for the life of
/// the process (a tar has no index: building one reads the whole stream).
/// `None` for a file that is not a readable archive.
fn index(c: &Container) -> Option<Arc<Index>> {
    type Cache = HashMap<PathBuf, (u128, u64, Option<Arc<Index>>)>;
    static CACHE: Mutex<Option<Cache>> = Mutex::new(None);
    let path = match c {
        Container::Zip(p) | Container::Tar { path: p, .. } => p,
        Container::Dir(_) => return None,
    };
    let m = std::fs::metadata(path).ok()?;
    let mtime = m
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    if let Ok(g) = CACHE.lock() {
        if let Some((t, n, hit)) = g.as_ref().and_then(|c| c.get(path)) {
            if *t == mtime && *n == m.len() {
                return hit.clone();
            }
        }
    }
    let built = match c {
        Container::Zip(p) => build_zip(p),
        Container::Tar { path, gz } => build_tar(path, *gz),
        Container::Dir(_) => None,
    }
    .map(Arc::new);
    if let Ok(mut g) = CACHE.lock() {
        g.get_or_insert_with(Default::default)
            .insert(path.clone(), (mtime, m.len(), built.clone()));
    }
    built
}

/// The ordinal of file `inner` among the archive's entries.
fn member_ord(c: &Container, inner: &str) -> Option<usize> {
    let idx = index(c)?;
    let inner = inner.trim_end_matches('/');
    let (parent, name) = inner.rsplit_once('/').unwrap_or(("", inner));
    match idx.dirs.get(parent)?.get(name)? {
        Member::File { ord, .. } => Some(*ord),
        Member::Dir => None,
    }
}

/// Real archives for tests, written with the crates that read them.
#[cfg(test)]
pub(crate) mod fixtures {
    use std::io::Write;
    use std::path::Path;

    /// A `.zip` of `(name, data)` members, names exactly as given (a name
    /// ending in `/` is a directory entry; `../x` and `/x` are written
    /// as they are, which is how a hostile archive looks).
    pub(crate) fn zip(path: &Path, members: &[(&str, &[u8])]) {
        let mut z = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
        let o = zip::write::SimpleFileOptions::default();
        for (name, data) in members {
            if name.ends_with('/') {
                z.add_directory(name.trim_end_matches('/'), o).unwrap();
            } else {
                z.start_file(*name, o).unwrap();
                z.write_all(data).unwrap();
            }
        }
        z.finish().unwrap();
    }

    fn tar_bytes(members: &[(&str, &[u8])], symlinks: &[(&str, &str)]) -> Vec<u8> {
        let mut b = tar::Builder::new(Vec::new());
        let mut put = |name: &str, kind: tar::EntryType, data: &[u8], link: Option<&str>| {
            let mut h = tar::Header::new_gnu();
            // Written as bytes: `set_path` would refuse `..` and `/x`.
            h.as_old_mut().name[..name.len()].copy_from_slice(name.as_bytes());
            h.set_entry_type(kind);
            h.set_size(data.len() as u64);
            h.set_mode(0o644);
            if let Some(l) = link {
                h.set_link_name(l).unwrap();
            }
            h.set_cksum();
            b.append(&h, data).unwrap();
        };
        for (name, data) in members {
            put(name, tar::EntryType::Regular, data, None);
        }
        for (name, to) in symlinks {
            put(name, tar::EntryType::Symlink, b"", Some(to));
        }
        b.into_inner().unwrap()
    }

    /// A `.tar` (plain), with `symlinks` as `(name, target)` entries too.
    pub(crate) fn tar(path: &Path, members: &[(&str, &[u8])], symlinks: &[(&str, &str)]) {
        std::fs::write(path, tar_bytes(members, symlinks)).unwrap();
    }

    /// A `.tar.gz` / `.tgz`.
    pub(crate) fn tgz(path: &Path, members: &[(&str, &[u8])], symlinks: &[(&str, &str)]) {
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        e.write_all(&tar_bytes(members, symlinks)).unwrap();
        std::fs::write(path, e.finish().unwrap()).unwrap();
    }
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
        let (c, inner) = container_at(root, "d/").unwrap();
        assert_eq!(inner, "");
        let names: Vec<String> = list(&c, "").into_iter().map(|e| e.name).collect();
        assert_eq!(names, ["src", ".gitignore", "A.txt", "b.txt"]);
        assert_eq!(count_files(&c, "", 999), 4);
        assert_eq!(count_files(&c, "", 2), 2);
        assert_eq!(listing(&c, ""), "src/\n.gitignore\nA.txt\nb.txt\n");
        assert_eq!(container_at(root, "d/b.txt"), None);
        assert_eq!(container_at(root, "d/../d"), None);
        #[cfg(unix)]
        assert_eq!(container_at(root, "d/etc"), None);
        let _ = std::fs::remove_dir_all(root);
    }

    const MEMBERS: &[(&str, &[u8])] = &[
        ("README.md", b"# hi\n"),
        ("src/main.rs", b"fn main() {}\nlet x = 2;\n"),
        ("src/deep/notes.txt", b"n"),
        ("../evil.txt", b"evil"),
        ("/abs.txt", b"abs"),
        ("a//b.txt", b"empty segment"),
        (".git/config", b"git"),
        ("ok/.git/HEAD", b"git"),
    ];

    fn names(c: &Container, inner: &str) -> Vec<String> {
        list(c, inner).into_iter().map(|e| e.name).collect()
    }

    /// The same checks for every format: list at the top and nested, read a
    /// member, and a hostile member is neither listed nor readable.
    fn check_archive(root: &Path, rel: &str) {
        let (c, inner) = container_at(root, rel).expect(rel);
        assert_eq!(inner, "", "{rel}");
        assert_eq!(names(&c, ""), ["src", "README.md"], "{rel}");
        assert_eq!(names(&c, "src"), ["deep", "main.rs"], "{rel}");
        assert_eq!(names(&c, "src/deep"), ["notes.txt"], "{rel}");
        assert_eq!(
            read(&c, "src/main.rs", 1 << 20).unwrap(),
            b"fn main() {}\nlet x = 2;\n"
        );
        for bad in [
            "../evil.txt",
            "/abs.txt",
            "abs.txt",
            "evil.txt",
            ".git/config",
            "a/b.txt",
        ] {
            assert!(read(&c, bad, 1 << 20).is_err(), "{rel}: {bad}");
            assert_eq!(stat(&c, bad), None, "{rel}: {bad}");
        }
        // Over the cap: refused, by size.
        assert!(read(&c, "src/main.rs", 5).is_err());
        assert_eq!(read(&c, "src/main.rs", 24).unwrap().len(), 24);
        assert!(read(&c, "src/main.rs", 23).is_err());
        assert_eq!(stat(&c, "src"), Some(Node::Dir));
        assert_eq!(stat(&c, "src/main.rs"), Some(Node::File(24)));
        // The path walks through the archive from the farm root.
        let (c2, inner) = container_at(root, &format!("{rel}/src/deep")).unwrap();
        assert_eq!((c2.clone(), inner.as_str()), (c, "src/deep"));
        let (_, inner) = container_at(root, &format!("{rel}/src/main.rs")).unwrap();
        assert_eq!(inner, "src/main.rs");
    }

    #[test]
    fn zip_tar_and_tgz_list_read_and_refuse_hostile_members() {
        let t = tempfile_dir();
        let root = t.as_path();
        fixtures::zip(&root.join("a.zip"), MEMBERS);
        fixtures::tar(&root.join("a.tar"), MEMBERS, &[("link", "/etc/passwd")]);
        fixtures::tgz(&root.join("a.tar.gz"), MEMBERS, &[("link", "/etc/passwd")]);
        fixtures::tgz(&root.join("a.TGZ"), MEMBERS, &[]);
        for rel in ["a.zip", "a.tar", "a.tar.gz", "a.TGZ"] {
            check_archive(root, rel);
        }
        // A symlink member is neither listed nor readable.
        let (c, _) = container_at(root, "a.tar.gz").unwrap();
        assert_eq!(stat(&c, "link"), None);
        assert!(read(&c, "link", 99).is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_directory_lists_an_archive_as_one_file_that_unfolds() {
        let t = tempfile_dir();
        let root = t.as_path();
        put(root, "d/b.txt");
        fixtures::zip(&root.join("d/a.zip"), &[("x.rs", b"x")]);
        std::fs::write(root.join("d/fake.tgz"), "not a tarball").unwrap();
        std::fs::write(root.join("d/old.7z"), "7z").unwrap();
        let (c, _) = container_at(root, "d/").unwrap();
        let es = list(&c, "");
        let a = es.iter().find(|e| e.name == "a.zip").unwrap();
        assert!(a.is_dir && a.archive && a.size > 0);
        for plain in ["fake.tgz", "old.7z", "b.txt"] {
            let e = es.iter().find(|e| e.name == plain).unwrap();
            assert!(!e.is_dir && !e.archive, "{plain}");
        }
        // One file to the count and the peek listing; its members are not counted.
        assert_eq!(count_files(&c, "", 99), 4);
        assert!(listing(&c, "").contains("a.zip\n"));
        // A fake archive is not a container at all.
        assert_eq!(container_at(root, "d/fake.tgz"), None);
        let (z, inner) = container_at(root, "d/a.zip").unwrap();
        assert_eq!(listing(&z, &inner), "x.rs\n");
        // A listing through the directory reaches in, too.
        assert_eq!(names(&c, "a.zip"), ["x.rs"]);
        assert_eq!(read(&c, "a.zip/x.rs", 9).unwrap(), b"x");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn an_archive_over_the_size_cap_stays_a_plain_file() {
        let t = tempfile_dir();
        let root = t.as_path();
        fixtures::zip(&root.join("a.zip"), &[("x.rs", b"x")]);
        let f = std::fs::OpenOptions::new()
            .write(true)
            .open(root.join("a.zip"))
            .unwrap();
        f.set_len(ARCHIVE_MAX_BYTES + 1).unwrap();
        assert_eq!(container_at(root, "a.zip"), None);
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
