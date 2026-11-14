//! Sources (yaks-2c42, design yaks-aae6): snapshots of material from outside
//! the farm, written once and cited by pages and yaks.
//!
//! A source is a directory at the farm root, `sources/<id>-<slug>/`: `<id>`
//! is `<YYYY-MM-DD>-<first 8 hex of its sha256>` and is the source's
//! identity; `<slug>` is a readable suffix from its title (yaks-7729), so a
//! tree listing says what each one is. Until `yaks migrate` renames them, a
//! directory may be just `sources/<id>/`: every lookup by id accepts both
//! (see [`dir_for`]), and nothing but the id is ever compared.
//!
//! - the original file (or, for a directory, its files under `files/`);
//! - `extracted.md`, its text: a markdown or plain-text original is copied,
//!   and for anything else (HTML, a PDF) the saver's own conversion is passed
//!   as `Request::text`. yaks extracts nothing itself: fetching and
//!   conversion belong to an agent or Hearth, which hand yaks the result;
//! - `source.md`: frontmatter saying what it is and where and when it came
//!   from (Hearth's format, so a Hearth repo's `raw/sources/` migrates as is).
//!
//! Saving the same content again (same sha256, on any day, whatever the
//! title) returns the existing source: dedup reads the `sha256:` in each
//! `source.md`, never a directory name, so a slug cannot hide a duplicate. `yaks promote` turns a yak's attachment into a source or
//! a page and rewrites the yak's link to it.

use crate::fields::{Fields, Value};
use crate::store;
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// The directory of sources at the farm root.
pub const SOURCES_DIR: &str = "sources";

/// Biggest single file saved (larger is refused: keep it outside, or in LFS
/// by hand).
pub const MAX_FILE: u64 = 20 * 1024 * 1024;
/// A directory source: at most this many files and bytes in all.
pub const MAX_DIR_FILES: usize = 200;
pub const MAX_DIR_BYTES: u64 = 5 * 1024 * 1024;

/// What to save and how to describe it.
#[derive(Debug, Default, Clone)]
pub struct Request {
    /// A file or directory on this machine.
    pub path: PathBuf,
    pub title: Option<String>,
    /// The web page this file is a copy of (the resource then is the URL;
    /// the file is recorded as `local_copy`).
    pub url: Option<String>,
    /// `sources/<id>/source.md` of the source this one is a part of (a
    /// screenshot of a page).
    pub part_of: Option<String>,
    /// A file on this machine holding the text of `path` as the saver
    /// converted it (HTML to markdown, a PDF to text): it becomes
    /// `extracted.md`. Not for a directory.
    pub text: Option<PathBuf>,
    /// Who saved it (`generated.by`).
    pub actor: String,
    /// Where it really came from, when `path` is only a copy of it (a yak's
    /// attachment being promoted: `<id>/<file>`). Recorded as the resource
    /// instead of `path`.
    pub origin: Option<String>,
    /// Update in place: when a source already names `url` as its resource,
    /// replace its content (same directory and id) instead of saving a second
    /// source. Needs `url`. Saves a new source when none has that URL.
    pub refresh: bool,
    /// Keep one file, `extracted.md`: for a markdown or plain-text file that
    /// is already the text to read, so no second copy is saved as "the
    /// original". Not for a directory, a non-text file, or with `text`.
    pub text_only: bool,
}

/// The result of a save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Saved {
    /// A new source at this farm-relative directory.
    New(String),
    /// The same content was already saved here.
    Already(String),
    /// The source at this directory had its content replaced (`refresh`).
    Refreshed(String),
}

impl Saved {
    pub fn dir(&self) -> &str {
        match self {
            Saved::New(d) | Saved::Already(d) | Saved::Refreshed(d) => d,
        }
    }
}

/// The sha256 of one file, as lowercase hex.
fn sha_hex(data: &[u8]) -> String {
    let d = Sha256::digest(data);
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// The length of a source id, `YYYY-MM-DD-` plus 8 hex.
const ID_LEN: usize = 19;
/// The longest slug (characters), cut on a word.
const SLUG_MAX: usize = 48;

/// Whether `s` is exactly a source id: `<YYYY-MM-DD>-<8 lowercase hex>`.
pub fn is_id(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == ID_LEN
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[7] == b'-'
        && b[8..10].iter().all(u8::is_ascii_digit)
        && b[10] == b'-'
        && b[11..]
            .iter()
            .all(|c| matches!(c, b'0'..=b'9' | b'a'..=b'f'))
}

/// The id a source directory name carries: its first 19 characters when they
/// are an id and the name ends there or goes on with `-<slug>`, else the
/// whole name (a directory that was never a yaks source id, kept as it is).
pub fn id_of(dir_name: &str) -> &str {
    match dir_name.get(..ID_LEN) {
        Some(id) if is_id(id) && matches!(dir_name.as_bytes().get(ID_LEN), None | Some(b'-')) => id,
        _ => dir_name,
    }
}

/// The readable part of a source's directory name, from its title: the
/// GitHub-style slug ([`crate::links::slug`]) cut to ASCII letters, digits
/// and single hyphens, at most [`SLUG_MAX`] characters cut on a word
/// (a single longer word is cut where it is), `untitled` when nothing is
/// left. Never has a `/` or a leading dot.
pub fn title_slug(title: &str) -> String {
    let mut out = String::new();
    for c in crate::links::slug(title).chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if matches!(c, '-' | '_') && !out.ends_with('-') {
            out.push('-');
        }
    }
    let mut out = out.trim_matches('-').to_string();
    if out.len() > SLUG_MAX {
        let cut = out[..=SLUG_MAX]
            .rfind('-')
            .filter(|&i| i > SLUG_MAX / 2)
            .unwrap_or(SLUG_MAX);
        out.truncate(cut);
        out = out.trim_end_matches('-').to_string();
    }
    if out.is_empty() {
        out = "untitled".into();
    }
    out
}

/// A new source's directory under the farm: `sources/<id>-<slug>`.
fn slugged_dir(id: &str, title: &str) -> String {
    format!("{SOURCES_DIR}/{id}-{}", title_slug(title))
}

/// The names of every directory under `sources/` that carries this id
/// (`<id>` itself and `<id>-<slug>`), sorted. More than one means two
/// branches saved the same source under different titles.
fn names_with_id(root: &Path, id: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root.join(SOURCES_DIR))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| id_of(n) == id)
        .collect();
    names.sort();
    names
}

/// Every farm-relative source directory (`sources/<name>`, with a
/// `source.md`) that `id` names: the directory called exactly `id` when there
/// is one (a legacy, unslugged name, or any name given in full), else, when
/// `id` is a source id, every `sources/<id>-<slug>`.
fn dirs_named(root: &Path, id: &str) -> Vec<String> {
    let is_source = |name: &str| {
        root.join(SOURCES_DIR)
            .join(name)
            .join("source.md")
            .is_file()
    };
    if id.is_empty() || id.contains('/') || id.starts_with('.') {
        return Vec::new();
    }
    if is_source(id) {
        return vec![format!("{SOURCES_DIR}/{id}")];
    }
    if !is_id(id) {
        return Vec::new();
    }
    names_with_id(root, id)
        .into_iter()
        .filter(|n| is_source(n))
        .map(|n| format!("{SOURCES_DIR}/{n}"))
        .collect()
}

/// The farm-relative directory (`sources/<name>`) of the source `id` names,
/// where `id` is a directory name as written (`<id>-<slug>`, or a legacy
/// `<id>`) or the bare id `<date>-<sha8>` of a slugged one. A wrong slug
/// does not resolve: a link the web would not follow is not one yaks follows.
pub fn dir_for(root: &Path, id: &str) -> Option<String> {
    dirs_named(root, id).into_iter().next()
}

/// The directories whose id more than one carries (`sources/<id>` beside
/// `sources/<id>-<slug>`, or two slugs): the same source saved twice, e.g. on
/// two branches. A migration to slugs must not guess which to keep.
#[allow(dead_code)] // the schema-9 migration (yaks-acc0) is their caller
pub fn duplicate_ids(root: &Path) -> Vec<Vec<String>> {
    let mut by_id: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for e in std::fs::read_dir(root.join(SOURCES_DIR))
        .into_iter()
        .flatten()
        .flatten()
    {
        if let Some(n) = e.file_name().to_str().filter(|n| is_id(id_of(n))) {
            by_id
                .entry(id_of(n).to_string())
                .or_default()
                .push(format!("{SOURCES_DIR}/{n}"));
        }
    }
    by_id
        .into_values()
        .filter(|v| v.len() > 1)
        .map(|mut v| {
            v.sort();
            v
        })
        .collect()
}

/// The renames that give every unslugged source directory its slug, as
/// `(old, new)` farm-relative directories (`sources/<id>`,
/// `sources/<id>-<slug>`), sorted. Pure: it reads `sources/` and each
/// `source.md`'s title and changes nothing; the migration applies the moves
/// (`git mv`). Idempotent: a directory that has a slug, and any name that is
/// not an id, yields nothing. A directory sharing its id with another
/// ([`duplicate_ids`]) is left out for a person to settle. Two different
/// sources cannot get one new name, since their ids differ.
#[allow(dead_code)] // the schema-9 migration (yaks-acc0) is their caller
pub fn plan_slugs(root: &Path) -> Vec<(String, String)> {
    let dup: std::collections::BTreeSet<String> =
        duplicate_ids(root).into_iter().flatten().collect();
    let mut out: Vec<(String, String)> = list(root)
        .into_iter()
        .filter(|s| {
            let name = s.dir.strip_prefix("sources/").unwrap_or(&s.dir);
            is_id(name) && !dup.contains(&s.dir)
        })
        .map(|s| {
            // The `title:` field itself: `Source::title` falls back to the
            // file name (`source`), which says nothing.
            let title = s
                .fields
                .get("title")
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            let new = slugged_dir(s.id(), &title);
            (s.dir, new)
        })
        .collect();
    out.sort();
    out
}

/// A reference to a source (a link destination such as
/// `/sources/<id>/source.md#top`, or a `sources:` entry) with a renamed
/// directory pointed at its new name, from a plan of [`plan_slugs`]. `None`
/// when `r` names no directory in the plan, and for a pinned reference
/// (`...@<rev>`): it names the path as it was at that commit.
#[allow(dead_code)] // the schema-9 migration (yaks-acc0) is their caller
pub fn rewrite_ref(r: &str, plan: &[(String, String)]) -> Option<String> {
    let at = r.find("sources/")?;
    let (lead, rest) = r.split_at(at);
    // `resources/`, `my-sources/`: other words. A prefix is only `/`, `./`, `../`.
    if !lead.is_empty() && !lead.chars().all(|c| matches!(c, '.' | '/')) {
        return None;
    }
    let after = &rest["sources/".len()..];
    let end = after.find(['/', '#', '@']).unwrap_or(after.len());
    let (name, tail) = after.split_at(end);
    if tail.split('#').next().is_some_and(|p| p.contains('@')) {
        return None;
    }
    let (_, new) = plan
        .iter()
        .find(|(old, _)| old == &format!("sources/{name}"))?;
    Some(format!("{lead}{new}{tail}"))
}

/// A directory's files (relative path, bytes), sorted, skipping `.git`,
/// dot files and common build and dependency directories.
fn dir_files(dir: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    const SKIP: &[&str] = &[
        ".git",
        "node_modules",
        "target",
        "__pycache__",
        ".venv",
        "venv",
        "dist",
        "build",
    ];
    let mut out = Vec::new();
    let mut stack = vec![(String::new(), dir.to_path_buf())];
    while let Some((rel, d)) = stack.pop() {
        for e in std::fs::read_dir(&d)
            .with_context(|| format!("reading {}", d.display()))?
            .flatten()
        {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || SKIP.contains(&name.as_str()) {
                continue;
            }
            let r = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            let p = e.path();
            if p.is_dir() {
                stack.push((r, p));
            } else if p.is_file() {
                out.push((r, std::fs::read(&p)?));
                if out.len() > MAX_DIR_FILES {
                    bail!(
                        "{} holds more than {MAX_DIR_FILES} files; save a smaller directory",
                        dir.display()
                    );
                }
            }
        }
    }
    out.sort();
    let total: u64 = out.iter().map(|(_, d)| d.len() as u64).sum();
    if total > MAX_DIR_BYTES {
        bail!(
            "{} holds {total} bytes, over the {MAX_DIR_BYTES}-byte limit for a directory source",
            dir.display()
        );
    }
    Ok(out)
}

/// The sha256 of a directory's files: each name and contents in order (name,
/// NUL, length, NUL, data), Hearth's rule, so a renamed file is a different
/// snapshot.
fn dir_sha(files: &[(String, Vec<u8>)]) -> String {
    let mut h = Sha256::new();
    for (name, data) in files {
        h.update(format!("{name}\0{}\0", data.len()).as_bytes());
        h.update(data);
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Text that is its own extraction. HTML is not: its markup is not what you
/// read, so it is converted by the saver and passed as `Request::text`.
fn is_text_name(name: &str) -> bool {
    let ext = name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(
        ext.as_str(),
        "md" | "markdown"
            | "txt"
            | "text"
            | "csv"
            | "tsv"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "rs"
            | "go"
            | "py"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "sh"
            | "sql"
            | "xml"
            | "log"
    )
}

fn media_type(name: &str) -> &'static str {
    let ext = name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "md" | "markdown" => "text/markdown",
        "txt" | "text" | "log" => "text/plain",
        "csv" => "text/csv",
        "json" => "application/json",
        "html" | "htm" => "text/html",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ if is_text_name(name) => "text/plain",
        _ => "application/octet-stream",
    }
}

/// A file name that is safe in the farm, keeping its extension; never
/// `source.md` or `extracted.md` (the notes beside it).
fn safe_name(name: &str) -> String {
    // The base: runs of anything but `[A-Za-z0-9._-]` become one `-`, then
    // leading and trailing `-`/`.` go. The extension: lowercase alphanumerics.
    let (base, ext) = match name.rsplit_once('.') {
        Some((b, e)) if !b.trim_matches('.').is_empty() => (b, e),
        _ => (name, ""),
    };
    let mut b = String::new();
    for c in base.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
            b.push(c);
        } else if !b.ends_with('-') {
            b.push('-');
        }
    }
    let mut b: String = b.trim_matches(['-', '.']).chars().take(80).collect();
    if b.is_empty() {
        b = "file".into();
    }
    let ext: String = ext
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .take(10)
        .collect();
    let n = if ext.is_empty() {
        b
    } else {
        format!("{b}.{ext}")
    };
    match n.as_str() {
        "source.md" | "extracted.md" => format!("original-{n}"),
        _ => n,
    }
}

/// The farm-relative directory of an already-saved source with this sha256,
/// on any day. Its `source.md` says: a refreshed source keeps its id, so the
/// id's `-<sha8>` suffix is only the hash it was first saved with.
pub fn find(root: &Path, sha: &str) -> Option<String> {
    let dir = root.join(SOURCES_DIR);
    let want = format!("\nsha256: {sha}\n");
    let mut hits: Vec<String> = std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| {
            std::fs::read_to_string(dir.join(n).join("source.md")).is_ok_and(|t| t.contains(&want))
        })
        .collect();
    hits.sort();
    hits.into_iter()
        .next()
        .map(|n| format!("{SOURCES_DIR}/{n}"))
}

pub(crate) fn host() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .filter(|h| !h.is_empty())
        .or_else(|| {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

fn mtime(p: &Path) -> String {
    std::fs::metadata(p)
        .and_then(|m| m.modified())
        .map(|t| {
            chrono::DateTime::<chrono::Utc>::from(t)
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
        })
        .unwrap_or_default()
}

/// Whether `abs` is under a scratch directory (the system temp dir, `/tmp`,
/// `/var/tmp`): such a path says nothing about where the material came from.
fn in_scratch(abs: &Path) -> bool {
    [std::env::temp_dir(), "/tmp".into(), "/var/tmp".into()]
        .iter()
        .filter_map(|d| std::fs::canonicalize(d).ok())
        .any(|d| abs.starts_with(d))
}

/// Save `req.path` as a source of the farm at `root` (the caller holds the
/// farm lock). A path inside the farm is refused: cite it directly.
pub fn add(root: &Path, req: &Request) -> Result<Saved> {
    add_on(
        root,
        req,
        &chrono::Utc::now().format("%Y-%m-%d").to_string(),
    )
}

/// [`add`] as of the date `today` (`YYYY-MM-DD`), the day in a new source's id.
fn add_on(root: &Path, req: &Request, today: &str) -> Result<Saved> {
    if req.refresh && req.url.is_none() {
        bail!("--refresh needs --url: the URL is what names the source to update");
    }
    let abs = std::fs::canonicalize(&req.path)
        .with_context(|| format!("{}: no such file or directory", req.path.display()))?;
    if let Ok(farm) = std::fs::canonicalize(root) {
        if abs.starts_with(&farm) {
            bail!(
                "{} is already in the farm; link it directly instead of saving a copy",
                req.path.display()
            );
        }
    }
    // Kept as the path of the directory that exists, so a part's link keeps
    // working whichever name (`<id>` or `<id>-<slug>`) it was given by.
    let part_of = match &req.part_of {
        None => None,
        Some(p) => {
            let p = crate::doks::normalize(p);
            let found = crate::locate::farm_file_in(root, &p).into_owned();
            if !root.join(&found).is_file() {
                bail!(
                    "--part-of {p}: no such file in the farm (give a source's sources/<id>/source.md)"
                );
            }
            Some(found)
        }
    };
    let is_dir = abs.is_dir();
    if req.text_only {
        if is_dir {
            bail!("--text-only is for a single file; a directory keeps its layout under files/");
        }
        if req.text.is_some() {
            bail!("--text-only and --text together: the file is its own text, or give --text");
        }
    }
    let supplied = match &req.text {
        None => None,
        Some(_) if is_dir => {
            bail!("--text is for a file; a directory's text is joined from its own files")
        }
        Some(t) => {
            let len = std::fs::metadata(t)
                .with_context(|| format!("--text {}: no such file", t.display()))?
                .len();
            if len > MAX_FILE {
                bail!(
                    "--text {} is {len} bytes, over the {MAX_FILE}-byte limit",
                    t.display()
                );
            }
            Some(
                std::fs::read_to_string(t)
                    .with_context(|| format!("--text {}: not a UTF-8 text file", t.display()))?,
            )
        }
    };
    let name = abs
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    let (sha, files, size): (String, Vec<(String, Vec<u8>)>, u64) = if is_dir {
        let files = dir_files(&abs)?;
        let size = files.iter().map(|(_, d)| d.len() as u64).sum();
        (dir_sha(&files), files, size)
    } else {
        let len = std::fs::metadata(&abs)?.len();
        if len > MAX_FILE {
            bail!(
                "{} is {len} bytes, over the {MAX_FILE}-byte limit for a source",
                req.path.display()
            );
        }
        let data = std::fs::read(&abs)?;
        let saved_as = if req.text_only {
            if !is_text_name(&name) {
                bail!(
                    "--text-only: {name} is not a markdown or text file; save it without --text-only (and give --text if it needs converting)"
                );
            }
            if std::str::from_utf8(&data).is_err() {
                bail!("--text-only: {name} is not UTF-8 text");
            }
            "extracted.md".to_string()
        } else {
            safe_name(&name)
        };
        (sha_hex(&data), vec![(saved_as, data)], len)
    };
    // `--refresh`: the source that already names this URL, if any.
    let mut prior: Option<Source> = None;
    if let Some(url) = req.url.as_deref().filter(|_| req.refresh) {
        let mut hits: Vec<Source> = list(root)
            .into_iter()
            .filter(|s| s.resource == url)
            .collect();
        if hits.len() > 1 {
            bail!(
                "more than one source names {url} ({}): refresh needs exactly one",
                hits.iter()
                    .map(|s| s.dir.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if let Some(s) = hits.pop() {
            let same_content = s
                .fields
                .get("sha256")
                .and_then(|v| v.as_str().map(String::from))
                .as_deref()
                == Some(sha.as_str());
            // The same bytes saved under another name (`--text-only`, after a
            // save without it) is still a change: the source is rewritten.
            let same_file = is_dir
                || s.fields
                    .get("file")
                    .and_then(|v| v.as_str().map(String::from))
                    .as_deref()
                    == Some(files[0].0.as_str());
            if same_content && same_file {
                return Ok(Saved::Already(s.dir));
            }
            prior = Some(s);
        }
    }
    if prior.is_none() {
        if let Some(dir) = find(root, &sha) {
            return Ok(Saved::Already(dir));
        }
    }
    // A refresh keeps the source's directory, slug and all, whatever its
    // title is now: a rename would break every link to it.
    let title = req
        .title
        .clone()
        .or_else(|| prior.as_ref().map(|s| s.title.clone()))
        .unwrap_or_else(|| name.clone());
    let rel = match &prior {
        Some(s) => s.dir.clone(),
        None => {
            let id = format!("{today}-{}", &sha[..8]);
            // The same id on other content (a one-in-four-billion hash prefix
            // on one day, or the first bytes of a source since refreshed) is
            // not this source: writing here would overwrite it.
            if let Some(other) = names_with_id(root, &id).first() {
                bail!(
                    "{SOURCES_DIR}/{other} already has the id {id} but other content (this file's sha256 is {sha}): \
                     not saved, it would overwrite that source. Save it again on another day, or `yaks source mark` the old one and move it"
                );
            }
            slugged_dir(&id, &title)
        }
    };
    let dir = root.join(&rel);
    if prior.is_some() {
        // Everything but `source.md` (rewritten below) is the old content.
        for e in std::fs::read_dir(&dir)?.flatten() {
            if e.file_name() == "source.md" {
                continue;
            }
            let p = e.path();
            if p.is_dir() {
                std::fs::remove_dir_all(&p)?;
            } else {
                std::fs::remove_file(&p)?;
            }
        }
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    for (n, data) in &files {
        let target = if is_dir {
            dir.join("files").join(n)
        } else {
            dir.join(n)
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, data).with_context(|| format!("writing {}", target.display()))?;
    }
    // Text: the saver's conversion when given; else markdown and plain text
    // are kept as they are.
    let text: Vec<(String, String)> = files
        .iter()
        .filter(|(n, _)| is_text_name(n))
        .filter_map(|(n, d)| {
            std::str::from_utf8(d)
                .ok()
                .map(|s| (n.clone(), s.to_string()))
        })
        .collect();
    let has_text = supplied.is_some() || !text.is_empty();
    if has_text {
        let body = if let Some(t) = &supplied {
            t.clone()
        } else if is_dir {
            text.iter()
                .map(|(n, t)| format!("## {n}\n\n{t}\n"))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            text[0].1.clone()
        };
        std::fs::write(dir.join("extracted.md"), body)?;
    }

    let local = req
        .origin
        .clone()
        .unwrap_or_else(|| abs.display().to_string());
    // A refresh keeps the title and `part_of` it had unless given new ones.
    let part_of = part_of.or_else(|| {
        prior.as_ref().and_then(|s| {
            s.fields
                .get("part_of")
                .and_then(|v| v.as_str().map(String::from))
        })
    });
    let mut f = Fields::default();
    f.set("type", &Value::Plain("source".into()));
    f.set("title", &Value::Str(title));
    // Provenance never carries a scratch path: `--url` and the title say
    // where it came from, and the file's name is all a description needs.
    let desc = match (&req.url, is_dir) {
        _ if req.origin.is_some() => format!("Promoted from the yak attachment {local}"),
        (Some(u), _) => format!("Saved copy of {u}"),
        (None, true) => format!("Snapshot of the directory {name} ({} files)", files.len()),
        (None, false) => format!("Snapshot of {name}"),
    };
    f.set("description", &Value::Str(desc));
    if is_dir {
        f.set(
            "files",
            &Value::List(
                files
                    .iter()
                    .map(|(n, d)| {
                        Value::Map(vec![
                            ("file".into(), Value::Str(n.clone())),
                            ("size".into(), Value::Plain(d.len().to_string())),
                        ])
                    })
                    .collect(),
            ),
        );
    } else {
        f.set("file", &Value::Str(files[0].0.clone()));
    }
    f.set(
        "media_type",
        &Value::Plain(if is_dir {
            "inode/directory".into()
        } else {
            media_type(&name).into()
        }),
    );
    f.set("size", &Value::Plain(size.to_string()));
    f.set("sha256", &Value::Plain(sha.clone()));
    if let Some(old) = prior
        .as_ref()
        .and_then(|s| {
            s.fields
                .get("sha256")
                .and_then(|v| v.as_str().map(String::from))
        })
        .filter(|old| *old != sha)
    {
        f.set("previous_sha256", &Value::Plain(old));
    }
    if let Some(p) = &part_of {
        f.set("part_of", &Value::Str(p.clone()));
    }
    let now = store::now_iso();
    let mut entry = vec![];
    match &req.url {
        Some(u) => {
            entry.push(("id".to_string(), Value::Plain("web".into())));
            entry.push(("resource".to_string(), Value::Str(u.clone())));
        }
        None if req.origin.is_some() => {
            entry.push(("id".to_string(), Value::Plain("attachment".into())));
            entry.push(("resource".to_string(), Value::Str(local.clone())));
        }
        None => {
            let resource = if in_scratch(&abs) {
                name.clone()
            } else {
                local.clone()
            };
            entry.push(("id".to_string(), Value::Plain("file".into())));
            entry.push(("resource".to_string(), Value::Str(resource)));
        }
    }
    if req.origin.is_none() {
        entry.push(("host".to_string(), Value::Str(host())));
        entry.push(("last_modified".to_string(), Value::Plain(mtime(&abs))));
    }
    entry.push(("retrieved".to_string(), Value::Plain(now.clone())));
    f.set("sources", &Value::List(vec![Value::Map(entry)]));
    f.set(
        "generated",
        &Value::Map(vec![
            ("by".into(), Value::Str(req.actor.clone())),
            ("at".into(), Value::Plain(now)),
        ]),
    );
    let note = if !has_text {
        "yaks extracts no text from this type; only the original is saved. Convert it and save it again with `--text <file>`.\n"
            .to_string()
    } else {
        "Its text is in [extracted.md](extracted.md).\n".to_string()
    };
    let page = crate::doks::Dok {
        path: format!("{rel}/source.md"),
        fields: f,
        body: note,
        has_frontmatter: true,
    };
    store::write::atomic_write(&dir.join("source.md"), &page.render())?;
    Ok(if prior.is_some() {
        Saved::Refreshed(rel)
    } else {
        Saved::New(rel)
    })
}

/// One saved source, for listing.
#[derive(Debug, Clone)]
pub struct Source {
    /// `sources/<id>` or `sources/<id>-<slug>`: the directory as it is named.
    pub dir: String,
    pub title: String,
    pub media_type: String,
    pub resource: String,
    pub fields: Fields,
}

impl Source {
    /// The source's id, `<date>-<sha8>`, whether or not its directory has a
    /// slug.
    pub fn id(&self) -> &str {
        id_of(self.dir.strip_prefix("sources/").unwrap_or(&self.dir))
    }
}

/// Every source of the farm, sorted by directory name (so oldest first: the
/// id leads). The title is the one in `source.md`, which a refresh may have
/// changed since the directory was named.
pub fn list(root: &Path) -> Vec<Source> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(root.join(SOURCES_DIR)) else {
        return out;
    };
    for e in rd.flatten() {
        let Some(id) = e.file_name().to_str().map(String::from) else {
            continue;
        };
        let dir = format!("{SOURCES_DIR}/{id}");
        let Ok(text) = std::fs::read_to_string(root.join(&dir).join("source.md")) else {
            continue;
        };
        let p = crate::doks::Dok::parse(&format!("{dir}/source.md"), &text);
        let resource = p
            .fields
            .get("sources")
            .and_then(|v| {
                v.as_list()
                    .first()
                    .and_then(|e| e.get("resource").and_then(|r| r.as_str().map(String::from)))
            })
            .unwrap_or_default();
        out.push(Source {
            dir,
            title: p.title(),
            media_type: p.str_field("media_type").unwrap_or_default(),
            resource,
            fields: p.fields,
        });
    }
    out.sort_by(|a, b| a.dir.cmp(&b.dir));
    out
}

/// How `yaks check` should treat a source nothing cites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    /// Kept on purpose: `uncited: <why>`.
    Uncited(String),
    /// Replaced by another source: `superseded_by: sources/<id>/source.md`.
    SupersededBy(String),
    /// Remove either mark.
    Clear,
}

/// The farm-relative directory (`sources/<name>`) that `given` names: an id,
/// with or without its slug, `sources/<id>`, or `sources/<id>/source.md`,
/// with or without a leading `/`. Refuses one that is not a saved source.
fn resolve_dir(root: &Path, given: &str) -> Result<String> {
    let rel = crate::doks::normalize(given);
    let id = rel.strip_prefix("sources/").unwrap_or(&rel);
    let id = id
        .strip_suffix("/source.md")
        .unwrap_or(id)
        .trim_end_matches('/');
    if id.is_empty() || id.contains('/') || id.starts_with('.') {
        bail!(
            "{given}: not a source (give its id or `sources/<id>/source.md`; `yaks source list` shows them)"
        );
    }
    let mut found = dirs_named(root, id);
    match found.len() {
        0 => bail!("{given}: no such source (`yaks source list` shows them)"),
        1 => Ok(found.remove(0)),
        _ => bail!(
            "{given}: more than one directory has this id ({}): keep one by hand",
            found.join(", ")
        ),
    }
}

/// Set or clear the mark `yaks check` honours in a source's `source.md`
/// (the caller holds the farm lock). One mark at a time: setting one
/// replaces the other. Returns the source's directory and whether the file
/// changed.
pub fn mark(root: &Path, source: &str, how: &Mark) -> Result<(String, bool)> {
    let dir = resolve_dir(root, source)?;
    let value = match how {
        Mark::Uncited(why) => {
            let why = why.trim();
            if why.is_empty() {
                bail!("--uncited needs a reason: say why this source is kept without a citation");
            }
            Some(("uncited", Value::Str(why.to_string())))
        }
        Mark::SupersededBy(by) => {
            let by_dir =
                resolve_dir(root, by).map_err(|e| anyhow::anyhow!("--superseded-by {e}"))?;
            if by_dir == dir {
                bail!("{dir} cannot supersede itself");
            }
            Some(("superseded_by", Value::Str(format!("{by_dir}/source.md"))))
        }
        Mark::Clear => None,
    };
    let path = root.join(&dir).join("source.md");
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut page = crate::doks::Dok::parse(&format!("{dir}/source.md"), &text);
    for key in ["uncited", "superseded_by"] {
        if value.as_ref().is_none_or(|(k, _)| *k != key) {
            page.fields.remove(key);
        }
    }
    if let Some((key, v)) = &value {
        page.fields.set(key, v);
    }
    let new = page.render();
    if new == text {
        return Ok((dir, false));
    }
    store::write::atomic_write(&path, &new)?;
    Ok((dir, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("yaks-sources-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn a_file_saves_once_with_text_and_a_hearth_shaped_note() {
        let base = tmp("file");
        let (root, outside) = (base.join("farm"), base.join("notes.md"));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&outside, "# Notes\n\nSome text.\n").unwrap();
        let req = Request {
            path: outside.clone(),
            actor: "tester".into(),
            ..Default::default()
        };
        let Saved::New(dir) = add(&root, &req).unwrap() else {
            panic!("new")
        };
        // The default title is the file name, so that is the slug.
        assert!(
            dir.starts_with("sources/")
                && is_id(&dir["sources/".len().."sources/".len() + ID_LEN])
                && dir.ends_with("-notesmd"),
            "{dir}"
        );
        let d = root.join(&dir);
        assert_eq!(
            std::fs::read_to_string(d.join("notes.md")).unwrap(),
            "# Notes\n\nSome text.\n"
        );
        assert_eq!(
            std::fs::read_to_string(d.join("extracted.md")).unwrap(),
            "# Notes\n\nSome text.\n"
        );
        let note = std::fs::read_to_string(d.join("source.md")).unwrap();
        for want in [
            "type: source\n",
            "title: notes.md\n",
            "file: notes.md\n",
            "media_type: text/markdown\n",
            "\nsha256: ",
            "  - id: file\n",
            "generated: {by: tester, at: ",
        ] {
            assert!(note.contains(want), "{want}\n{note}");
        }
        assert_eq!(
            add(&root, &req).unwrap(),
            Saved::Already(dir.clone()),
            "same content, same source"
        );
        assert_eq!(list(&root).len(), 1);
        assert_eq!(list(&root)[0].title, "notes.md");
        // Inside the farm: refused.
        let inside = Request {
            path: d.join("notes.md"),
            ..Default::default()
        };
        assert!(
            add(&root, &inside)
                .unwrap_err()
                .to_string()
                .contains("already in the farm")
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_directory_keeps_its_layout_and_joins_its_text() {
        let base = tmp("dir");
        let (root, proj) = (base.join("farm"), base.join("proj"));
        std::fs::create_dir_all(proj.join("src")).unwrap();
        std::fs::create_dir_all(proj.join("node_modules/x")).unwrap();
        std::fs::write(proj.join("README.md"), "readme").unwrap();
        std::fs::write(proj.join("src/main.rs"), "fn main() {}").unwrap();
        std::fs::write(proj.join("node_modules/x/i.js"), "skip").unwrap();
        std::fs::create_dir_all(&root).unwrap();
        let Saved::New(dir) = add(
            &root,
            &Request {
                path: proj,
                url: Some("https://example.test/p".into()),
                ..Default::default()
            },
        )
        .unwrap() else {
            panic!()
        };
        let d = root.join(&dir);
        assert!(d.join("files/src/main.rs").is_file() && !d.join("files/node_modules").exists());
        let ex = std::fs::read_to_string(d.join("extracted.md")).unwrap();
        assert!(
            ex.contains("## README.md\n\nreadme") && ex.contains("## src/main.rs"),
            "{ex}"
        );
        let note = std::fs::read_to_string(d.join("source.md")).unwrap();
        assert!(
            note.contains("  - id: web\n") && note.contains("resource: https://example.test/p"),
            "{note}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    /// A scratch dir that is not under the system temp dir, so a path in it
    /// is recorded as the path it is.
    fn home_tmp(tag: &str) -> PathBuf {
        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("yaks-sources-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn an_html_original_takes_the_savers_text_as_its_extraction() {
        let base = tmp("text");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let (html, md) = (base.join("page.storage.html"), base.join("page.md"));
        std::fs::write(&html, "<p>Hello &middot; world</p>").unwrap();
        std::fs::write(&md, "Hello · world\n").unwrap();
        let Saved::New(dir) = add(
            &root,
            &Request {
                path: html.clone(),
                title: Some("The page".into()),
                url: Some("https://example.test/page".into()),
                text: Some(md),
                ..Default::default()
            },
        )
        .unwrap() else {
            panic!()
        };
        let d = root.join(&dir);
        assert_eq!(
            std::fs::read_to_string(d.join("extracted.md")).unwrap(),
            "Hello · world\n"
        );
        assert!(d.join("page.storage.html").is_file());
        let note = std::fs::read_to_string(d.join("source.md")).unwrap();
        assert!(note.contains("Its text is in [extracted.md]"), "{note}");
        assert!(
            note.contains("description: Saved copy of https://example.test/page\n"),
            "{note}"
        );
        assert!(!note.contains("local_copy"), "{note}");
        assert!(
            !note.contains(&base.display().to_string()),
            "no scratch path in provenance: {note}"
        );
        // One source for the thing: saving the same original again finds it.
        assert_eq!(
            add(
                &root,
                &Request {
                    path: html,
                    ..Default::default()
                }
            )
            .unwrap(),
            Saved::Already(dir)
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn html_alone_is_not_text_and_says_how_to_convert() {
        let base = tmp("html");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let html = base.join("p.html");
        std::fs::write(&html, "<p>x</p>").unwrap();
        let Saved::New(dir) = add(
            &root,
            &Request {
                path: html,
                ..Default::default()
            },
        )
        .unwrap() else {
            panic!()
        };
        let d = root.join(&dir);
        assert!(!d.join("extracted.md").exists());
        let note = std::fs::read_to_string(d.join("source.md")).unwrap();
        assert!(note.contains("--text"), "{note}");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn text_is_refused_for_a_directory_and_for_binary() {
        let base = tmp("textbad");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let (dir, file, bin) = (base.join("d"), base.join("f.pdf"), base.join("t.bin"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.md"), "a").unwrap();
        std::fs::write(&file, "pdf").unwrap();
        std::fs::write(&bin, [0xff, 0xfe, 0x00]).unwrap();
        let err = |path: &PathBuf, text: &PathBuf| {
            add(
                &root,
                &Request {
                    path: path.clone(),
                    text: Some(text.clone()),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .to_string()
        };
        assert!(err(&dir, &bin).contains("--text is for a file"));
        assert!(err(&file, &bin).contains("not a UTF-8 text file"));
        assert!(err(&file, &base.join("missing.md")).contains("no such file"));
        assert!(!root.join(SOURCES_DIR).exists(), "nothing saved on refusal");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_scratch_path_is_not_provenance_but_another_path_is() {
        let scratch = tmp("scratch");
        let root = scratch.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let f = scratch.join("notes.md");
        std::fs::write(&f, "x").unwrap();
        let Saved::New(dir) = add(
            &root,
            &Request {
                path: f,
                ..Default::default()
            },
        )
        .unwrap() else {
            panic!()
        };
        let note = std::fs::read_to_string(root.join(&dir).join("source.md")).unwrap();
        assert!(
            note.contains("description: Snapshot of notes.md\n"),
            "{note}"
        );
        assert!(note.contains("resource: notes.md\n"), "{note}");
        let _ = std::fs::remove_dir_all(&scratch);

        let base = home_tmp("kept");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let f = base.join("notes.md");
        std::fs::write(&f, "y").unwrap();
        let Saved::New(dir) = add(
            &root,
            &Request {
                path: f.clone(),
                ..Default::default()
            },
        )
        .unwrap() else {
            panic!()
        };
        let note = std::fs::read_to_string(root.join(&dir).join("source.md")).unwrap();
        let want = std::fs::canonicalize(&f).unwrap();
        assert!(
            note.contains(&format!("resource: {}\n", want.display())),
            "{note}"
        );
        assert!(
            note.contains("description: Snapshot of notes.md\n"),
            "{note}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    fn web(path: &Path, refresh: bool) -> Request {
        Request {
            path: path.to_path_buf(),
            title: Some("A living page".into()),
            url: Some("https://docs.example.test/d/1".into()),
            refresh,
            actor: "tester".into(),
            ..Default::default()
        }
    }

    #[test]
    fn refresh_updates_a_source_in_place_keyed_by_its_url() {
        let base = tmp("refresh");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let f = base.join("page.md");
        std::fs::write(&f, "one\n").unwrap();
        let Saved::New(dir) = add(&root, &web(&f, true)).unwrap() else {
            panic!("no source with this url yet: saved as new")
        };
        let first_sha = sha_hex(b"one\n");

        // Unchanged: nothing is written.
        let note_before = std::fs::read_to_string(root.join(&dir).join("source.md")).unwrap();
        assert_eq!(
            add(&root, &web(&f, true)).unwrap(),
            Saved::Already(dir.clone())
        );
        assert_eq!(
            std::fs::read_to_string(root.join(&dir).join("source.md")).unwrap(),
            note_before
        );

        // Changed: same directory and id, new content, old hash kept.
        std::fs::write(&f, "two\n").unwrap();
        let mut req = web(&f, true);
        req.title = None;
        assert_eq!(
            add(&root, &req).unwrap(),
            Saved::Refreshed(dir.clone()),
            "same id after a refresh"
        );
        let d = root.join(&dir);
        assert_eq!(std::fs::read_to_string(d.join("page.md")).unwrap(), "two\n");
        assert_eq!(
            std::fs::read_to_string(d.join("extracted.md")).unwrap(),
            "two\n"
        );
        let note = std::fs::read_to_string(d.join("source.md")).unwrap();
        for want in [
            "title: A living page\n".to_string(),
            format!("sha256: {}\n", sha_hex(b"two\n")),
            format!("previous_sha256: {first_sha}\n"),
            "resource: https://docs.example.test/d/1\n".to_string(),
        ] {
            assert!(note.contains(&want), "{want}\n{note}");
        }
        assert_eq!(list(&root).len(), 1, "no second source");

        // The current bytes are found by a plain add, refresh or not.
        assert_eq!(
            add(
                &root,
                &Request {
                    path: f.clone(),
                    ..Default::default()
                }
            )
            .unwrap(),
            Saved::Already(dir.clone())
        );

        // A replaced file of another name leaves no stale original behind.
        let g = base.join("renamed.md");
        std::fs::write(&g, "three\n").unwrap();
        assert!(matches!(
            add(&root, &web(&g, true)).unwrap(),
            Saved::Refreshed(_)
        ));
        assert!(!d.join("page.md").exists() && d.join("renamed.md").is_file());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn plain_add_never_refreshes_and_refresh_needs_a_url() {
        let base = tmp("norefresh");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let f = base.join("page.md");
        std::fs::write(&f, "one\n").unwrap();
        add(&root, &web(&f, false)).unwrap();
        std::fs::write(&f, "two\n").unwrap();
        assert!(matches!(
            add(&root, &web(&f, false)).unwrap(),
            Saved::New(_)
        ));
        assert_eq!(
            list(&root).len(),
            2,
            "without --refresh a change is a new source"
        );

        let mut no_url = web(&f, true);
        no_url.url = None;
        assert!(
            add(&root, &no_url)
                .unwrap_err()
                .to_string()
                .contains("--refresh needs --url")
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn refresh_refuses_an_ambiguous_url() {
        let base = tmp("ambiguous");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let f = base.join("page.md");
        for body in ["one\n", "two\n"] {
            std::fs::write(&f, body).unwrap();
            add(&root, &web(&f, false)).unwrap();
        }
        std::fs::write(&f, "three\n").unwrap();
        let err = add(&root, &web(&f, true)).unwrap_err().to_string();
        assert!(err.contains("more than one source"), "{err}");
        let _ = std::fs::remove_dir_all(&base);
    }

    fn files_in(d: &Path) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(d)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }

    #[test]
    fn text_only_keeps_the_text_once_and_nothing_else() {
        let base = tmp("textonly");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let f = base.join("My Doc.md");
        std::fs::write(&f, "# Doc\n\nbody\n").unwrap();
        let req = Request {
            text_only: true,
            ..web(&f, false)
        };
        let Saved::New(dir) = add(&root, &req).unwrap() else {
            panic!()
        };
        let d = root.join(&dir);
        assert_eq!(files_in(&d), ["extracted.md", "source.md"]);
        assert_eq!(
            std::fs::read_to_string(d.join("extracted.md")).unwrap(),
            "# Doc\n\nbody\n"
        );
        let note = std::fs::read_to_string(d.join("source.md")).unwrap();
        for want in [
            "file: extracted.md\n".to_string(),
            "media_type: text/markdown\n".to_string(),
            format!("sha256: {}\n", sha_hex(b"# Doc\n\nbody\n")),
            "size: 12\n".to_string(),
            "resource: https://docs.example.test/d/1\n".to_string(),
        ] {
            assert!(note.contains(&want), "{want}\n{note}");
        }
        assert_eq!(
            add(&root, &req).unwrap(),
            Saved::Already(dir.clone()),
            "the same text again is the same source"
        );
        // Saved the usual way, the same bytes are found as well.
        assert_eq!(
            add(
                &root,
                &Request {
                    path: f,
                    ..Default::default()
                }
            )
            .unwrap(),
            Saved::Already(dir)
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn text_only_is_refused_for_what_is_not_one_text_file() {
        let base = tmp("textonlybad");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let (dir, md, html, bin, conv) = (
            base.join("d"),
            base.join("a.md"),
            base.join("p.html"),
            base.join("b.md"),
            base.join("conv.md"),
        );
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.md"), "a").unwrap();
        std::fs::write(&md, "a").unwrap();
        std::fs::write(&html, "<p>x</p>").unwrap();
        std::fs::write(&bin, [0xff, 0xfe, 0x00]).unwrap();
        std::fs::write(&conv, "c").unwrap();
        let err = |path: &PathBuf, text: Option<PathBuf>| {
            add(
                &root,
                &Request {
                    path: path.clone(),
                    text,
                    text_only: true,
                    ..Default::default()
                },
            )
            .unwrap_err()
            .to_string()
        };
        assert!(err(&dir, None).contains("--text-only is for a single file"));
        assert!(err(&md, Some(conv)).contains("--text-only and --text"));
        assert!(err(&html, None).contains("not a markdown or text file"));
        assert!(err(&bin, None).contains("not UTF-8 text"));
        assert!(!root.join(SOURCES_DIR).exists(), "nothing saved on refusal");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn text_only_refreshes_in_place() {
        let base = tmp("textonlyrefresh");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let f = base.join("page.md");
        std::fs::write(&f, "one\n").unwrap();
        let req = Request {
            text_only: true,
            ..web(&f, true)
        };
        let Saved::New(dir) = add(&root, &req).unwrap() else {
            panic!()
        };
        std::fs::write(&f, "two\n").unwrap();
        assert_eq!(add(&root, &req).unwrap(), Saved::Refreshed(dir.clone()));
        let d = root.join(&dir);
        assert_eq!(files_in(&d), ["extracted.md", "source.md"]);
        assert_eq!(
            std::fs::read_to_string(d.join("extracted.md")).unwrap(),
            "two\n"
        );
        let note = std::fs::read_to_string(d.join("source.md")).unwrap();
        assert!(
            note.contains(&format!("previous_sha256: {}\n", sha_hex(b"one\n"))),
            "{note}"
        );
        assert_eq!(list(&root).len(), 1);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn refresh_with_text_only_collapses_an_existing_two_file_source() {
        let base = tmp("collapse");
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let f = base.join("page.md");
        std::fs::write(&f, "same\n").unwrap();
        let Saved::New(dir) = add(&root, &web(&f, false)).unwrap() else {
            panic!()
        };
        let d = root.join(&dir);
        assert_eq!(files_in(&d), ["extracted.md", "page.md", "source.md"]);
        // Same bytes, but the layout changes: that is a refresh, not "already".
        let req = Request {
            text_only: true,
            ..web(&f, true)
        };
        assert_eq!(add(&root, &req).unwrap(), Saved::Refreshed(dir.clone()));
        assert_eq!(files_in(&d), ["extracted.md", "source.md"]);
        let note = std::fs::read_to_string(d.join("source.md")).unwrap();
        assert!(note.contains("file: extracted.md\n"), "{note}");
        assert!(
            !note.contains("previous_sha256"),
            "the content did not change: {note}"
        );
        assert_eq!(add(&root, &req).unwrap(), Saved::Already(dir));
        let _ = std::fs::remove_dir_all(&base);
    }

    // ---- slugged directories and dedup (yaks-7729) ------------------------

    /// A scratch farm and a file outside it holding `body`.
    fn farm_with(tag: &str, body: &str) -> (PathBuf, PathBuf, PathBuf) {
        let base = tmp(tag);
        let root = base.join("farm");
        std::fs::create_dir_all(&root).unwrap();
        let f = base.join("a.md");
        std::fs::write(&f, body).unwrap();
        (base, root, f)
    }

    fn titled(path: &Path, title: &str) -> Request {
        Request {
            path: path.to_path_buf(),
            title: Some(title.into()),
            actor: "tester".into(),
            ..Default::default()
        }
    }

    fn source_dirs(root: &Path) -> Vec<String> {
        files_in(&root.join(SOURCES_DIR))
    }

    #[test]
    fn slugs_follow_the_title_rule() {
        assert_eq!(title_slug("Hello, World!"), "hello-world");
        assert_eq!(title_slug("  Ünïcode — café  "), "ncode-caf");
        assert_eq!(title_slug("a  b--c_d"), "a-b-c-d");
        assert_eq!(title_slug(""), "untitled");
        assert_eq!(title_slug("!!! ???"), "untitled");
        assert_eq!(title_slug("../../etc/passwd"), "etcpasswd");
        assert!(!title_slug(".hidden").starts_with('.'));
        // At most 48, cut on a word.
        let long = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu";
        let s = title_slug(long);
        assert!(s.len() <= SLUG_MAX && !s.ends_with('-'), "{s}");
        assert_eq!(s, "alpha-beta-gamma-delta-epsilon-zeta-eta-theta");
        // One long word is cut where it is.
        assert_eq!(title_slug(&"x".repeat(100)).len(), SLUG_MAX);
    }

    #[test]
    fn new_sources_are_named_id_then_slug() {
        let (base, root, f) = farm_with("named", "one\n");
        let Saved::New(dir) = add(&root, &titled(&f, "A Living Page")).unwrap() else {
            panic!()
        };
        let name = dir.strip_prefix("sources/").unwrap();
        assert!(is_id(id_of(name)), "{name}");
        assert_eq!(id_of(name), &name[..ID_LEN]);
        assert!(name.ends_with("-a-living-page"), "{name}");
        assert_eq!(
            &name[11..ID_LEN],
            &sha_hex(b"one\n")[..8],
            "the id carries sha8"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    // The dedup cases of the decision on yaks-49c5: bytes already saved
    // return the existing source, whatever the title or the day.

    #[test]
    fn dedup_same_bytes_same_title() {
        let (base, root, f) = farm_with("dedup1", "same\n");
        let Saved::New(dir) = add(&root, &titled(&f, "Same title")).unwrap() else {
            panic!()
        };
        assert_eq!(
            add(&root, &titled(&f, "Same title")).unwrap(),
            Saved::Already(dir)
        );
        assert_eq!(source_dirs(&root).len(), 1);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn dedup_same_bytes_different_title_neither_renames_nor_adds() {
        let (base, root, f) = farm_with("dedup2", "same\n");
        let Saved::New(dir) = add(&root, &titled(&f, "First title")).unwrap() else {
            panic!()
        };
        let before = std::fs::read_to_string(root.join(&dir).join("source.md")).unwrap();
        assert_eq!(
            add(&root, &titled(&f, "A completely different title")).unwrap(),
            Saved::Already(dir.clone())
        );
        assert_eq!(
            add(
                &root,
                &Request {
                    path: f.clone(),
                    ..Default::default()
                }
            )
            .unwrap(),
            Saved::Already(dir.clone()),
            "no title at all"
        );
        assert_eq!(source_dirs(&root).len(), 1, "no second directory");
        assert_eq!(
            source_dirs(&root)[0],
            dir.strip_prefix("sources/").unwrap(),
            "no rename"
        );
        assert_eq!(
            std::fs::read_to_string(root.join(&dir).join("source.md")).unwrap(),
            before,
            "source.md is untouched, so is its title"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn dedup_same_bytes_on_another_day() {
        let (base, root, f) = farm_with("dedup3", "same\n");
        let Saved::New(dir) = add_on(&root, &titled(&f, "T"), "2026-01-01").unwrap() else {
            panic!()
        };
        assert!(dir.starts_with("sources/2026-01-01-"), "{dir}");
        assert_eq!(
            add_on(&root, &titled(&f, "Another title"), "2026-03-09").unwrap(),
            Saved::Already(dir),
            "the date is when it was first saved, not part of the content"
        );
        assert_eq!(source_dirs(&root).len(), 1);
        // Different bytes on that other day are a new source.
        std::fs::write(&f, "other\n").unwrap();
        let Saved::New(new) = add_on(&root, &titled(&f, "T"), "2026-03-09").unwrap() else {
            panic!()
        };
        assert!(new.starts_with("sources/2026-03-09-"), "{new}");
        assert_eq!(source_dirs(&root).len(), 2);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn same_title_different_bytes_is_a_second_source_with_no_slug_clash() {
        // Two sources with one title differ in their id, so their directory
        // names never clash: no `-2` suffix is ever needed.
        let (base, root, f) = farm_with("sametitle", "one\n");
        let Saved::New(a) = add(&root, &titled(&f, "Shared title")).unwrap() else {
            panic!()
        };
        std::fs::write(&f, "two\n").unwrap();
        let Saved::New(b) = add(&root, &titled(&f, "Shared title")).unwrap() else {
            panic!()
        };
        assert_ne!(a, b);
        assert!(a.ends_with("-shared-title") && b.ends_with("-shared-title"));
        assert_eq!(source_dirs(&root).len(), 2);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn two_different_files_whose_sha8_collide_are_refused_not_overwritten() {
        // A 32-bit prefix collision cannot be made on demand, so stage one: a
        // saved source with the id this file would get, but another sha256.
        let (base, root, f) = farm_with("collide", "new bytes\n");
        let sha = sha_hex(b"new bytes\n");
        let id = format!("2026-02-02-{}", &sha[..8]);
        let old = root.join(SOURCES_DIR).join(format!("{id}-old-title"));
        std::fs::create_dir_all(&old).unwrap();
        let other = format!("{}{}", &sha[..8], "0".repeat(56));
        let note = format!("---\ntype: source\ntitle: Old title\nsha256: {other}\n---\n\nold\n");
        std::fs::write(old.join("source.md"), &note).unwrap();
        std::fs::write(old.join("old.md"), "old bytes\n").unwrap();

        let err = add_on(&root, &titled(&f, "New title"), "2026-02-02")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("already has the id") && err.contains(&id),
            "{err}"
        );
        assert_eq!(
            source_dirs(&root),
            [format!("{id}-old-title")],
            "nothing created"
        );
        assert_eq!(
            std::fs::read_to_string(old.join("source.md")).unwrap(),
            note
        );
        assert_eq!(
            std::fs::read_to_string(old.join("old.md")).unwrap(),
            "old bytes\n",
            "the old source is untouched"
        );
        // Nor does the unslugged spelling of the old directory go unseen.
        std::fs::rename(&old, root.join(SOURCES_DIR).join(&id)).unwrap();
        assert!(
            add_on(&root, &titled(&f, "New title"), "2026-02-02").is_err(),
            "a legacy `<id>` directory collides too"
        );
        // Another day gives another id: saved.
        assert!(matches!(
            add_on(&root, &titled(&f, "New title"), "2026-02-03").unwrap(),
            Saved::New(_)
        ));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn dedup_reads_the_sha_not_the_directory_name() {
        // A slugged directory whose id does not match its content (a refreshed
        // source) still dedups on the content it now has.
        let (base, root, f) = farm_with("dedupsha", "one\n");
        let req = Request {
            url: Some("https://example.test/u".into()),
            refresh: true,
            ..titled(&f, "Page")
        };
        let Saved::New(dir) = add(&root, &req).unwrap() else {
            panic!()
        };
        std::fs::write(&f, "two\n").unwrap();
        assert_eq!(add(&root, &req).unwrap(), Saved::Refreshed(dir.clone()));
        assert_eq!(
            add(&root, &titled(&f, "Whatever")).unwrap(),
            Saved::Already(dir)
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn refresh_keeps_the_directory_when_the_title_changes() {
        let (base, root, f) = farm_with("refreshtitle", "one\n");
        let mut req = Request {
            url: Some("https://example.test/u".into()),
            refresh: true,
            ..titled(&f, "Old title")
        };
        let Saved::New(dir) = add(&root, &req).unwrap() else {
            panic!()
        };
        assert!(dir.ends_with("-old-title"));
        std::fs::write(&f, "two\n").unwrap();
        req.title = Some("A new title".into());
        assert_eq!(add(&root, &req).unwrap(), Saved::Refreshed(dir.clone()));
        assert_eq!(source_dirs(&root).len(), 1, "no second source, no rename");
        assert_eq!(
            source_dirs(&root)[0],
            dir.strip_prefix("sources/").unwrap(),
            "the slug stays"
        );
        let all = list(&root);
        assert_eq!(
            all[0].title, "A new title",
            "source.md has the current title"
        );
        assert_eq!(all[0].id(), id_of(dir.strip_prefix("sources/").unwrap()));
        let _ = std::fs::remove_dir_all(&base);
    }

    /// A farm with one slugged source and nothing else; its id and name.
    fn slugged_farm(tag: &str) -> (PathBuf, PathBuf, String, String) {
        let (base, root, f) = farm_with(tag, "x\n");
        let Saved::New(dir) = add(&root, &titled(&f, "Readable name")).unwrap() else {
            panic!()
        };
        let name = dir.strip_prefix("sources/").unwrap().to_string();
        let id = id_of(&name).to_string();
        (base, root, id, name)
    }

    #[test]
    fn a_bare_id_finds_a_slugged_directory() {
        let (base, root, id, name) = slugged_farm("lookup1");
        assert_ne!(id, name);
        let want = format!("sources/{name}");
        assert_eq!(dir_for(&root, &id), Some(want.clone()));
        // `source mark` by the bare id, `sources/<id>` and `sources/<id>/source.md`.
        for given in [
            id.clone(),
            format!("sources/{id}"),
            format!("/sources/{id}/source.md"),
        ] {
            let (dir, _) = mark(&root, &given, &Mark::Uncited("kept".into())).unwrap();
            assert_eq!(dir, want, "{given}");
        }
        // `--superseded-by` too.
        let g = base.join("b.md");
        std::fs::write(&g, "y\n").unwrap();
        let Saved::New(newer) = add(&root, &titled(&g, "Newer")).unwrap() else {
            panic!()
        };
        mark(
            &root,
            &id,
            &Mark::SupersededBy(id_of(&newer["sources/".len()..]).into()),
        )
        .unwrap();
        let note = std::fs::read_to_string(root.join(&want).join("source.md")).unwrap();
        assert!(
            note.contains(&format!("superseded_by: {newer}/source.md")),
            "{note}"
        );
        assert!(dir_for(&root, "2026-01-01-00000000").is_none());
        assert!(mark(&root, "nope", &Mark::Clear).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn the_id_with_its_slug_finds_it_and_a_wrong_slug_does_not() {
        let (base, root, id, name) = slugged_farm("lookup2");
        let want = format!("sources/{name}");
        assert_eq!(dir_for(&root, &name), Some(want.clone()));
        for given in [
            name.clone(),
            format!("sources/{name}"),
            format!("sources/{name}/source.md"),
        ] {
            let (dir, _) = mark(&root, &given, &Mark::Clear).unwrap();
            assert_eq!(dir, want, "{given}");
        }
        assert_eq!(dir_for(&root, &format!("{id}-wrong-slug")), None);
        assert!(mark(&root, &format!("{id}-wrong-slug"), &Mark::Clear).is_err());
        // A prefix of the id is not the id.
        assert_eq!(dir_for(&root, &id[..15]), None);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_legacy_directory_still_resolves_by_its_id() {
        let (base, root, f) = farm_with("legacy", "x\n");
        let Saved::New(dir) = add(&root, &titled(&f, "T")).unwrap() else {
            panic!()
        };
        let id = id_of(&dir["sources/".len()..]).to_string();
        std::fs::rename(root.join(&dir), root.join(SOURCES_DIR).join(&id)).unwrap();
        assert_eq!(dir_for(&root, &id), Some(format!("sources/{id}")));
        assert_eq!(list(&root)[0].id(), id);
        assert!(mark(&root, &id, &Mark::Clear).is_ok());
        assert!(
            matches!(add(&root, &titled(&f, "U")).unwrap(), Saved::Already(d) if d == format!("sources/{id}"))
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn part_of_by_the_bare_id_is_stored_as_the_real_path() {
        let (base, root, id, name) = slugged_farm("partof");
        let g = base.join("shot.png");
        std::fs::write(&g, "png").unwrap();
        let Saved::New(dir) = add(
            &root,
            &Request {
                part_of: Some(format!("sources/{id}/source.md")),
                ..titled(&g, "Shot")
            },
        )
        .unwrap() else {
            panic!()
        };
        let note = std::fs::read_to_string(root.join(&dir).join("source.md")).unwrap();
        assert!(
            note.contains(&format!("part_of: sources/{name}/source.md")),
            "{note}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn farm_paths_by_the_bare_id_resolve_through_the_slug() {
        let (base, root, id, name) = slugged_farm("farmfile");
        let real = format!("sources/{name}/source.md");
        let f = |p: &str| crate::locate::farm_file_in(&root, p).into_owned();
        assert_eq!(f(&format!("sources/{id}")), real);
        assert_eq!(f(&format!("sources/{id}/source.md")), real);
        assert_eq!(
            f(&format!("sources/{id}/a.md")),
            format!("sources/{name}/a.md")
        );
        assert_eq!(f(&format!("sources/{name}")), real);
        assert_eq!(f(&format!("sources/{name}/source.md")), real);
        assert_eq!(f("sources/nope"), "sources/nope/source.md");
        assert_eq!(f("doks/x.md"), "doks/x.md");
        // The link layer: exists and peek.
        let link = |p: &str| crate::links::Link::Farm {
            path: p.to_string(),
            rev: None,
            anchor: None,
        };
        assert_eq!(
            crate::links::exists(&root, &link(&format!("sources/{id}"))),
            Some(true)
        );
        assert_eq!(
            crate::links::exists(&root, &link("sources/2026-01-01-00000000")),
            Some(false)
        );
        let peek = crate::links::peek(
            &link(&format!("sources/{id}")),
            &crate::links::Bases {
                farm: &root,
                code: None,
            },
            0,
        )
        .unwrap();
        assert!(peek.text.contains("title: Readable name"), "{}", peek.text);
        let _ = std::fs::remove_dir_all(&base);
    }

    fn legacy_source(root: &Path, id: &str, title: &str) {
        let d = root.join(SOURCES_DIR).join(id);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(
            d.join("source.md"),
            format!("---\ntype: source\ntitle: {title}\nsha256: {id}\n---\n\nx\n"),
        )
        .unwrap();
    }

    #[test]
    fn plan_slugs_names_each_unslugged_directory_and_is_idempotent() {
        let base = tmp("plan");
        let root = base.join("farm");
        legacy_source(&root, "2026-01-01-aaaaaaaa", "First: the \"big\" one");
        legacy_source(&root, "2026-01-02-bbbbbbbb", "First: the \"big\" one"); // same title
        legacy_source(&root, "2026-01-03-cccccccc", "");
        // Already slugged, and not an id at all: left alone.
        legacy_source(&root, "2026-01-04-dddddddd-has-slug", "Slugged");
        legacy_source(&root, "hearth-import", "Foreign");
        let plan = plan_slugs(&root);
        assert_eq!(
            plan,
            [
                (
                    "sources/2026-01-01-aaaaaaaa".to_string(),
                    "sources/2026-01-01-aaaaaaaa-first-the-big-one".to_string()
                ),
                (
                    "sources/2026-01-02-bbbbbbbb".to_string(),
                    "sources/2026-01-02-bbbbbbbb-first-the-big-one".to_string()
                ),
                (
                    "sources/2026-01-03-cccccccc".to_string(),
                    "sources/2026-01-03-cccccccc-untitled".to_string()
                ),
            ]
        );
        // The planned names are all distinct and each keeps its id first.
        let news: std::collections::BTreeSet<_> = plan.iter().map(|(_, n)| n).collect();
        assert_eq!(news.len(), plan.len());
        // Apply it: nothing is left to plan, and every old id still resolves.
        for (old, new) in &plan {
            std::fs::rename(root.join(old), root.join(new)).unwrap();
        }
        assert!(plan_slugs(&root).is_empty(), "idempotent");
        assert!(
            dir_for(&root, "2026-01-01-aaaaaaaa").is_some_and(|d| d.ends_with("first-the-big-one"))
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn plan_slugs_leaves_a_duplicated_id_for_a_person() {
        let base = tmp("plandup");
        let root = base.join("farm");
        legacy_source(&root, "2026-01-01-aaaaaaaa", "Old");
        legacy_source(&root, "2026-01-01-aaaaaaaa-new-name", "New");
        legacy_source(&root, "2026-01-02-bbbbbbbb", "Fine");
        assert_eq!(
            duplicate_ids(&root),
            [vec![
                "sources/2026-01-01-aaaaaaaa".to_string(),
                "sources/2026-01-01-aaaaaaaa-new-name".to_string()
            ]]
        );
        let plan = plan_slugs(&root);
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].0, "sources/2026-01-02-bbbbbbbb");
        assert!(
            mark(&root, "2026-01-01-aaaaaaaa", &Mark::Clear).is_ok(),
            "the exact name wins"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn rewrite_ref_points_an_old_id_at_its_new_path() {
        let plan = vec![(
            "sources/2026-01-01-aaaaaaaa".to_string(),
            "sources/2026-01-01-aaaaaaaa-readable".to_string(),
        )];
        let r = |s: &str| rewrite_ref(s, &plan);
        let new = "sources/2026-01-01-aaaaaaaa-readable";
        assert_eq!(
            r("sources/2026-01-01-aaaaaaaa/source.md"),
            Some(format!("{new}/source.md"))
        );
        assert_eq!(r("/sources/2026-01-01-aaaaaaaa"), Some(format!("/{new}")));
        assert_eq!(
            r("../sources/2026-01-01-aaaaaaaa/shot.png#top"),
            Some(format!("../{new}/shot.png#top"))
        );
        // Already new, another source, another word, a pinned link: untouched.
        assert_eq!(r(&format!("{new}/source.md")), None);
        assert_eq!(r("sources/2026-01-02-bbbbbbbb/source.md"), None);
        assert_eq!(r("raw/sources/2026-01-01-aaaaaaaa/source.md"), None);
        assert_eq!(r("resources/2026-01-01-aaaaaaaa/x"), None);
        assert_eq!(r("/sources/2026-01-01-aaaaaaaa/source.md@3f2a9c1"), None);
        assert_eq!(r("doks/x.md"), None);
    }

    /// Not a gate: `PLAN_SLUGS_ROOT=<a copy of a farm's root> cargo test -p yaks
    /// plan_slugs_over_a_real_farm -- --ignored --nocapture` prints the
    /// renames the schema-9 migration would make (yaks-7729), the farm's own
    /// duplicates and the new names that two sources would share.
    #[test]
    #[ignore]
    fn plan_slugs_over_a_real_farm() {
        let Ok(root) = std::env::var("PLAN_SLUGS_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let plan = plan_slugs(&root);
        for (old, new) in &plan {
            println!("{old} -> {new}");
        }
        let dups = duplicate_ids(&root);
        let mut seen = std::collections::BTreeMap::<&str, usize>::new();
        for (_, n) in &plan {
            *seen.entry(n).or_default() += 1;
        }
        let clashes: Vec<_> = seen.iter().filter(|(_, c)| **c > 1).collect();
        println!(
            "{} renames, {} duplicated ids {dups:?}, {} clashing new names {clashes:?}, {} already slugged or foreign",
            plan.len(),
            dups.len(),
            clashes.len(),
            source_dirs(&root).len() - plan.len()
        );
        assert!(plan_slugs(&root).len() == plan.len());
        assert!(clashes.is_empty());
    }

    #[test]
    fn names_are_made_safe() {
        assert_eq!(safe_name("My Shot (1).PNG"), "My-Shot-1.png");
        assert_eq!(safe_name("source.md"), "original-source.md");
        assert_eq!(safe_name("..."), "file");
    }
}
