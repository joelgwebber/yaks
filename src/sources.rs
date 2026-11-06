//! Sources (yaks-2c42, design yaks-aae6): snapshots of material from outside
//! the farm, written once and cited by pages and yaks.
//!
//! A source is a directory `sources/<id>/` at the farm root, `<id>` being
//! `<YYYY-MM-DD>-<first 8 hex of its sha256>`:
//!
//! - the original file (or, for a directory, its files under `files/`);
//! - `extracted.md`, its text: a markdown or plain-text original is copied,
//!   and for anything else (HTML, a PDF) the saver's own conversion is passed
//!   as `Request::text`. yaks extracts nothing itself: fetching and
//!   conversion belong to an agent or Hearth, which hand yaks the result;
//! - `source.md`: frontmatter saying what it is and where and when it came
//!   from (Hearth's format, so a Hearth repo's `raw/sources/` migrates as is).
//!
//! Saving the same content again (same sha256, on any day) returns the
//! existing source. `yaks promote` turns a yak's attachment into a source or
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
    if let Some(p) = &req.part_of {
        let p = crate::doks::normalize(p);
        if !root.join(&p).is_file() {
            bail!(
                "--part-of {p}: no such file in the farm (give a source's sources/<id>/source.md)"
            );
        }
    }
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
    let rel = match &prior {
        Some(s) => s.dir.clone(),
        None => {
            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
            format!("{SOURCES_DIR}/{today}-{}", &sha[..8])
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
    let title = req
        .title
        .clone()
        .or_else(|| prior.as_ref().map(|s| s.title.clone()))
        .unwrap_or_else(|| name.clone());
    let part_of = req.part_of.clone().or_else(|| {
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
        f.set("part_of", &Value::Str(crate::doks::normalize(p)));
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
    /// `sources/<id>`.
    pub dir: String,
    pub title: String,
    pub media_type: String,
    pub resource: String,
    pub fields: Fields,
}

/// Every source of the farm, sorted by id (oldest first).
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

/// The farm-relative directory (`sources/<id>`) that `given` names: a bare
/// id, `sources/<id>`, or `sources/<id>/source.md`, with or without a
/// leading `/`. Refuses one that is not a saved source.
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
    let dir = format!("{SOURCES_DIR}/{id}");
    if !root.join(&dir).join("source.md").is_file() {
        bail!("{given}: no such source (`yaks source list` shows them)");
    }
    Ok(dir)
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
        assert!(
            dir.starts_with("sources/") && dir.len() == "sources/".len() + 19,
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

    #[test]
    fn names_are_made_safe() {
        assert_eq!(safe_name("My Shot (1).PNG"), "My-Shot-1.png");
        assert_eq!(safe_name("source.md"), "original-source.md");
        assert_eq!(safe_name("..."), "file");
    }
}
