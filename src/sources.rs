//! Sources (yaks-2c42, design yaks-aae6): snapshots of material from outside
//! the farm, written once and cited by pages and yaks.
//!
//! A source is a directory `sources/<id>/` at the farm root, `<id>` being
//! `<YYYY-MM-DD>-<first 8 hex of its sha256>`:
//!
//! - the original file (or, for a directory, its files under `files/`);
//! - `extracted.md`, its text, when it is text (markdown or plain text is
//!   copied; yaks extracts nothing else: fetching and conversion belong to
//!   an agent or Hearth, which hand yaks the result);
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
    /// Who saved it (`generated.by`).
    pub actor: String,
    /// Where it really came from, when `path` is only a copy of it (a yak's
    /// attachment being promoted: `<id>/<file>`). Recorded as the resource
    /// instead of `path`.
    pub origin: Option<String>,
}

/// The result of a save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Saved {
    /// A new source at this farm-relative directory.
    New(String),
    /// The same content was already saved here.
    Already(String),
}

impl Saved {
    pub fn dir(&self) -> &str {
        match self {
            Saved::New(d) | Saved::Already(d) => d,
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
            | "html"
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
/// on any day (ids end in its first 8 hex digits).
pub fn find(root: &Path, sha: &str) -> Option<String> {
    let dir = root.join(SOURCES_DIR);
    let mut hits: Vec<String> = std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| n.ends_with(&format!("-{}", &sha[..8])))
        .collect();
    hits.sort();
    hits.into_iter()
        .find(|n| {
            std::fs::read_to_string(dir.join(n).join("source.md"))
                .is_ok_and(|t| t.contains(&format!("\nsha256: {sha}\n")))
        })
        .map(|n| format!("{SOURCES_DIR}/{n}"))
}

fn host() -> String {
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

/// Save `req.path` as a source of the farm at `root` (the caller holds the
/// farm lock). A path inside the farm is refused: cite it directly.
pub fn add(root: &Path, req: &Request) -> Result<Saved> {
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
        let p = crate::pages::normalize(p);
        if !root.join(&p).is_file() {
            bail!(
                "--part-of {p}: no such file in the farm (give a source's sources/<id>/source.md)"
            );
        }
    }
    let is_dir = abs.is_dir();
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
        (sha_hex(&data), vec![(safe_name(&name), data)], len)
    };
    if let Some(dir) = find(root, &sha) {
        return Ok(Saved::Already(dir));
    }
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let id = format!("{today}-{}", &sha[..8]);
    let rel = format!("{SOURCES_DIR}/{id}");
    let dir = root.join(&rel);
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
    // Text: markdown and plain text are kept as they are.
    let text: Vec<(String, String)> = files
        .iter()
        .filter(|(n, _)| is_text_name(n))
        .filter_map(|(n, d)| {
            std::str::from_utf8(d)
                .ok()
                .map(|s| (n.clone(), s.to_string()))
        })
        .collect();
    if !text.is_empty() {
        let body = if is_dir {
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
    let title = req.title.clone().unwrap_or_else(|| name.clone());
    let mut f = Fields::default();
    f.set("type", &Value::Plain("source".into()));
    f.set("title", &Value::Str(title));
    let desc = match (&req.url, is_dir) {
        _ if req.origin.is_some() => format!("Promoted from the yak attachment {local}"),
        (Some(u), _) => format!("Saved copy of {u}, from the local file {local}"),
        (None, true) => format!(
            "Snapshot of the local directory {local} ({} files)",
            files.len()
        ),
        (None, false) => format!("Snapshot of the local file {local}"),
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
    if let Some(p) = &req.part_of {
        f.set("part_of", &Value::Str(crate::pages::normalize(p)));
    }
    let now = store::now_iso();
    let mut entry = vec![];
    match &req.url {
        Some(u) => {
            entry.push(("id".to_string(), Value::Plain("web".into())));
            entry.push(("resource".to_string(), Value::Str(u.clone())));
            entry.push(("local_copy".to_string(), Value::Str(local.clone())));
        }
        None if req.origin.is_some() => {
            entry.push(("id".to_string(), Value::Plain("attachment".into())));
            entry.push(("resource".to_string(), Value::Str(local.clone())));
        }
        None => {
            entry.push(("id".to_string(), Value::Plain("file".into())));
            entry.push(("resource".to_string(), Value::Str(local.clone())));
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
    let note = if text.is_empty() {
        "Text extraction isn't done for this type; only the original is saved.\n".to_string()
    } else {
        "Its text is in [extracted.md](extracted.md).\n".to_string()
    };
    let page = crate::pages::Page {
        path: format!("{rel}/source.md"),
        fields: f,
        body: note,
        has_frontmatter: true,
    };
    store::write::atomic_write(&dir.join("source.md"), &page.render())?;
    Ok(Saved::New(rel))
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
        let p = crate::pages::Page::parse(&format!("{dir}/source.md"), &text);
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

    #[test]
    fn names_are_made_safe() {
        assert_eq!(safe_name("My Shot (1).PNG"), "My-Shot-1.png");
        assert_eq!(safe_name("source.md"), "original-source.md");
        assert_eq!(safe_name("..."), "file");
    }
}
