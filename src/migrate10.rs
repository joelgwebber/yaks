//! Schema 10 (yaks-e83f): a source becomes one file plus, when there is
//! something to keep beside its text, a sibling directory.
//!
//! Before: `sources/<name>/source.md` (provenance and a boilerplate body),
//! `sources/<name>/extracted.md` (the text), the original beside them, and a
//! directory source's tree under `sources/<name>/files/`.
//!
//! After: `sources/<name>.md`, whose frontmatter is the provenance and whose
//! body is the text. `sources/<name>/` holds the original only when it is not
//! the text, and a directory source's tree directly (no `files/` level).
//!
//! References follow: `sources/<name>/source.md` and `.../extracted.md` name
//! `sources/<name>.md`, and `.../files/<path>` names `sources/<name>/<path>`.
//! A reference to a kept original (`sources/<name>/shot.png`) is unchanged,
//! because the directory keeps its name. No alias for the old files stays in
//! the resolver.
//!
//! [`plan`] reads only; [`apply`] writes. `migrate::run` calls them before it
//! writes the `schema` marker, so an interrupted run is simply run again (a
//! source already converted is no longer in the old layout, so it is skipped).

use crate::fields::Value;
use crate::sources::source_file;
use crate::store;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One source to convert.
#[derive(Debug, Clone)]
pub struct Convert {
    /// `sources/<name>`: the old directory, and the new stem.
    pub dir: String,
    /// A directory source (its tree was under `files/`).
    pub is_dir: bool,
    /// The new file's text.
    pub text: String,
    /// Files to delete from the directory: `source.md`, `extracted.md`, and
    /// an original that is the text itself.
    pub remove: Vec<String>,
}

/// What converting a farm's sources does, or would.
#[derive(Debug, Default)]
pub struct Plan {
    pub converts: Vec<Convert>,
    /// Old name (and its bare id) to the source's facts, for references.
    refs: HashMap<String, Ref>,
}

#[derive(Debug, Clone)]
struct Ref {
    dir: String,
    is_dir: bool,
    /// For a directory source: the slug of a joined-text heading to that
    /// file's path in the tree, so `extracted.md#docs-amd` keeps naming
    /// `docs/a.md`.
    headings: HashMap<String, String>,
}

/// What [`apply`] did, or a dry run would.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    /// Sources converted to the one-file layout.
    pub sources: usize,
    /// Of those, directory sources whose tree moved up out of `files/`.
    pub trees: usize,
    /// Redundant copies removed (an original identical to its text).
    pub duplicates: usize,
    /// Reference edits, and the files that held one.
    pub links: usize,
    pub link_files: usize,
}

impl Report {
    pub fn is_empty(&self) -> bool {
        self.sources == 0 && self.links == 0
    }
}

/// The body lines `source add` wrote before schema 10. They say nothing once
/// the text is the body.
fn boilerplate(line: &str) -> bool {
    let l = line.trim();
    l.is_empty()
        || l == "Its text is in [extracted.md](extracted.md)."
        || l.starts_with("yaks extracts no text from this type")
        || (l.starts_with("Skipped ") && l.contains(": `"))
}

fn is_prose_name(name: &str) -> bool {
    let ext = name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(ext.as_str(), "md" | "markdown" | "txt" | "text")
}

fn count_files(dir: &Path) -> usize {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return 0;
    };
    rd.flatten()
        .map(|e| {
            if e.path().is_dir() {
                count_files(&e.path())
            } else {
                1
            }
        })
        .sum()
}

/// Plan the conversion of every source still in the old layout under `root`.
pub fn plan(root: &Path) -> Result<Plan> {
    let mut out = Plan::default();
    for s in crate::sources::legacy_list(root) {
        let dir_abs = root.join(&s.dir);
        let name = s.dir.trim_start_matches("sources/").to_string();
        let record = std::fs::read_to_string(dir_abs.join("source.md"))
            .with_context(|| format!("reading {}/source.md", s.dir))?;
        let old = crate::doks::Dok::parse(&format!("{}/source.md", s.dir), &record);
        let extracted = std::fs::read_to_string(dir_abs.join("extracted.md")).ok();
        let is_dir = old.str_field("media_type").as_deref() == Some("inode/directory")
            || dir_abs.join("files").is_dir();
        let mut fields = old.fields.clone();
        let mut remove = vec!["source.md".to_string()];
        if extracted.is_some() {
            remove.push("extracted.md".into());
        }
        let mut headings = HashMap::new();
        let mut body = if is_dir {
            let listed = fields.get("files").map(|v| v.as_list().len()).unwrap_or(0);
            let count = if listed > 0 {
                listed
            } else {
                count_files(&dir_abs.join("files"))
            };
            fields.remove("files");
            fields.set("file_count", &Value::Plain(count.to_string()));
            if let Some(x) = &extracted {
                for line in x.lines() {
                    if let Some(path) = line.strip_prefix("## ") {
                        headings.insert(crate::links::slug(path), path.trim().to_string());
                    }
                }
            }
            let enc = crate::links::percent_encode_path(&name);
            format!("The saved tree: [{name}/]({enc}/).\n")
        } else {
            let file = old.str_field("file");
            // The original is the text when it is markdown or plain text and
            // `extracted.md` holds the same bytes, or when the source was
            // saved text-only (`file: extracted.md`).
            let redundant = match (&file, &extracted) {
                (Some(f), Some(_)) if f == "extracted.md" => true,
                (Some(f), Some(x)) if is_prose_name(f) => {
                    std::fs::read(dir_abs.join(f)).is_ok_and(|b| b == x.as_bytes())
                }
                _ => false,
            };
            if redundant {
                if let Some(f) = file.as_ref().filter(|f| *f != "extracted.md") {
                    remove.push(f.clone());
                }
                fields.remove("file");
            }
            match &extracted {
                Some(x) => x.clone(),
                None => {
                    let f = file.unwrap_or_default();
                    format!(
                        "yaks extracts no text from this type; the original is [{f}]({}/{}). \
                         Convert it and save it again with `--text <file>`.\n",
                        crate::links::percent_encode_path(&name),
                        crate::links::percent_encode_path(&f)
                    )
                }
            }
        };
        // A person's own words in the old record (anything but the lines yaks
        // wrote) are kept, after the text.
        let notes: Vec<&str> = old.body.lines().filter(|l| !boilerplate(l)).collect();
        if !notes.is_empty() {
            if !body.ends_with('\n') {
                body.push('\n');
            }
            body.push_str("\n## Notes kept with the source\n\n");
            body.push_str(&notes.join("\n"));
            body.push('\n');
        }
        let r = Ref {
            dir: s.dir.clone(),
            is_dir,
            headings,
        };
        let id = crate::sources::id_of(&name).to_string();
        if id != name {
            out.refs.entry(id).or_insert_with(|| r.clone());
        }
        out.refs.insert(name, r);
        let text = crate::doks::Dok {
            path: source_file(&s.dir),
            fields,
            body,
            has_frontmatter: true,
        }
        .render();
        out.converts.push(Convert {
            dir: s.dir.clone(),
            is_dir,
            text,
            remove,
        });
    }
    // References inside the new files themselves (`part_of:`,
    // `superseded_by:`) point at the new names too.
    let refs = out.refs.clone();
    for c in &mut out.converts {
        c.text = rewrite_front(&c.text, &refs).0;
    }
    Ok(out)
}

/// One reference to a source as written (`/sources/<name>/source.md#x`, with
/// any leading `/` or `./`), pointed at its schema-10 place. `None` when it
/// names no source being converted, needs no change, or is pinned to a
/// revision (`@<rev>`), which names the file as it was then.
fn rewrite_ref(r: &str, refs: &HashMap<String, Ref>) -> Option<String> {
    let at = r.find("sources/")?;
    let (lead, rest) = r.split_at(at);
    if !lead.chars().all(|c| matches!(c, '.' | '/')) {
        return None;
    }
    let after = &rest["sources/".len()..];
    let (name, tail) = after.split_once('/')?;
    let (path, frag) = match tail.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (tail, None),
    };
    if path.contains('@') {
        return None;
    }
    let src = refs.get(name)?;
    let file = source_file(&src.dir);
    let anchor = frag.map(|f| format!("#{f}")).unwrap_or_default();
    let new = match path {
        "source.md" => format!("{file}{anchor}"),
        "extracted.md" if src.is_dir => match frag.and_then(|f| src.headings.get(f)) {
            Some(p) => format!("{}/{}", src.dir, crate::links::percent_encode_path(p)),
            None => format!("{}/", src.dir),
        },
        "extracted.md" => format!("{file}{anchor}"),
        p if src.is_dir && (p == "files" || p.starts_with("files/")) => {
            let inner = p.trim_start_matches("files").trim_start_matches('/');
            format!("{}/{inner}{anchor}", src.dir)
        }
        _ => return None,
    };
    Some(format!("{lead}{new}"))
}

/// [`rewrite_ref`] over the frontmatter of `text`: entries of a `sources:`
/// list, `part_of:`, `superseded_by:` (plain values, not markdown links). Each
/// `sources/<name>/<file>` token at a word edge is replaced where it stands.
fn rewrite_front(text: &str, refs: &HashMap<String, Ref>) -> (String, usize) {
    let Some((front, _)) = store::split_frontmatter(text) else {
        return (text.to_string(), 0);
    };
    let end = text.find(front).map_or(0, |i| i + front.len());
    let head = &text[..end];
    let mut out = String::with_capacity(text.len());
    let mut n = 0;
    let mut i = 0;
    while let Some(k) = head[i..].find("sources/") {
        let start = i + k;
        let at_edge = head[..start]
            .chars()
            .next_back()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '-'));
        let len = head[start..]
            .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ',' | ']' | ')' | '}'))
            .unwrap_or(head.len() - start);
        let token = &head[start..start + len];
        out.push_str(&head[i..start]);
        match rewrite_ref(token, refs).filter(|_| at_edge) {
            Some(new) => {
                out.push_str(&new);
                n += 1;
            }
            None => out.push_str(token),
        }
        i = start + len;
    }
    out.push_str(&head[i..]);
    out.push_str(&text[end..]);
    (out, n)
}

/// A file's text with every reference to a converted source rewritten: its
/// frontmatter values, then its markdown links (code spans and fences are
/// left alone).
fn rewrite_file(text: &str, refs: &HashMap<String, Ref>) -> (String, usize) {
    let (t, a) = rewrite_front(text, refs);
    let (t, b) = crate::links::rewrite(&t, |dest| rewrite_ref(dest, refs));
    (t, a + b)
}

/// Every markdown file that may refer to a source: yak files, their markdown
/// attachments, doks (farm-wide and per herd), schemas and session notes.
/// Sources are snapshots: only their own frontmatter is rewritten, in
/// [`plan`].
fn referring_files(root: &Path) -> Vec<PathBuf> {
    let mut out = store::yak_files(root);
    for id in store::all_ids(root) {
        for a in crate::farm::list_attachments(root, &id) {
            if crate::links::is_markdown_name(&a.name) {
                out.push(a.path);
            }
        }
    }
    let mut tops = vec![
        root.join("doks"),
        root.join("schemas"),
        root.join("sessions"),
    ];
    if let Ok(rd) = std::fs::read_dir(root) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().into_owned();
            if e.path().is_dir()
                && !n.starts_with('.')
                && !store::RESERVED_HERD_NAMES.contains(&n.as_str())
            {
                tops.push(e.path().join("doks"));
            }
        }
    }
    for t in tops {
        walk_md(&t, &mut out);
    }
    out.sort();
    out.dedup();
    out
}

fn walk_md(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        if e.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let p = e.path();
        if p.is_dir() {
            walk_md(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("md") {
            out.push(p);
        }
    }
}

/// The reference rewrites the plan implies: `(file, new text, edits)`.
fn reference_writes(root: &Path, p: &Plan) -> Vec<(PathBuf, String, usize)> {
    if p.refs.is_empty() {
        return Vec::new();
    }
    referring_files(root)
        .into_iter()
        .filter_map(|path| {
            let text = std::fs::read_to_string(&path).ok()?;
            let (new, n) = rewrite_file(&text, &p.refs);
            (n > 0).then_some((path, new, n))
        })
        .collect()
}

fn report_of(p: &Plan, writes: &[(PathBuf, String, usize)]) -> Report {
    Report {
        sources: p.converts.len(),
        trees: p.converts.iter().filter(|c| c.is_dir).count(),
        duplicates: p
            .converts
            .iter()
            .map(|c| {
                c.remove
                    .iter()
                    .filter(|f| *f != "source.md" && *f != "extracted.md")
                    .count()
            })
            .sum(),
        links: writes.iter().map(|w| w.2).sum(),
        link_files: writes.len(),
    }
}

/// What [`apply`] would do, counted, with nothing written.
pub fn dry_run(root: &Path) -> Result<Report> {
    let p = plan(root)?;
    let writes = reference_writes(root, &p);
    Ok(report_of(&p, &writes))
}

/// Convert every source still in the old layout, and the references to them.
/// The caller holds the farm lock and writes the `schema` marker after.
pub fn apply(root: &Path) -> Result<Report> {
    let p = plan(root)?;
    // Refuse before any write when a target already exists.
    for c in &p.converts {
        if root.join(source_file(&c.dir)).exists() {
            anyhow::bail!(
                "{}: both {} and {}/source.md exist; settle which is the source by hand",
                c.dir,
                source_file(&c.dir),
                c.dir
            );
        }
    }
    let writes = reference_writes(root, &p);
    for (path, text, _) in &writes {
        store::write::atomic_write(path, text)?;
    }
    for c in &p.converts {
        let dir = root.join(&c.dir);
        store::write::atomic_write(&root.join(source_file(&c.dir)), &c.text)?;
        for f in &c.remove {
            let _ = std::fs::remove_file(dir.join(f));
        }
        let files = dir.join("files");
        if c.is_dir && files.is_dir() {
            for e in std::fs::read_dir(&files)?.flatten() {
                let to = dir.join(e.file_name());
                if to.exists() {
                    anyhow::bail!(
                        "{}: cannot move files/{} up a level, {} exists",
                        c.dir,
                        e.file_name().to_string_lossy(),
                        to.display()
                    );
                }
                std::fs::rename(e.path(), &to)?;
            }
            std::fs::remove_dir(&files).with_context(|| format!("removing {}/files", c.dir))?;
        }
        if std::fs::read_dir(&dir).is_ok_and(|mut d| d.next().is_none()) {
            std::fs::remove_dir(&dir)?;
        }
    }
    Ok(report_of(&p, &writes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("yaks-m10-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn put(root: &Path, rel: &str, text: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn read(root: &Path, rel: &str) -> String {
        std::fs::read_to_string(root.join(rel)).unwrap()
    }

    const PAGE: &str = "sources/2026-01-01-aaaaaaaa-page";
    const REPO: &str = "sources/2026-01-02-bbbbbbbb-repo";
    const SHOT: &str = "sources/2026-01-03-cccccccc-shot";

    /// A farm in the old layout: a markdown page (original identical to its
    /// text), a directory source, and an image with no text.
    fn old_farm(tag: &str) -> PathBuf {
        let root = tmp(tag);
        put(
            &root,
            &format!("{PAGE}/source.md"),
            "---\ntype: source\ntitle: A page\nfile: page.md\nmedia_type: text/markdown\nsha256: aa\n---\n\nIts text is in [extracted.md](extracted.md).\n",
        );
        put(&root, &format!("{PAGE}/page.md"), "# Page\n\nText.\n");
        put(&root, &format!("{PAGE}/extracted.md"), "# Page\n\nText.\n");
        put(
            &root,
            &format!("{REPO}/source.md"),
            "---\ntype: source\ntitle: A repo\nfiles:\n  - {file: README.md, size: 3}\n  - {file: docs/a.md, size: 2}\nmedia_type: inode/directory\nsha256: bb\npart_of: sources/2026-01-01-aaaaaaaa-page/source.md\n---\n\nIts text is in [extracted.md](extracted.md).\n\nSkipped 1: `.git/`.\n",
        );
        put(
            &root,
            &format!("{REPO}/files/README.md"),
            "hi, see [a](docs/a.md)\n",
        );
        put(&root, &format!("{REPO}/files/docs/a.md"), "a\n");
        put(
            &root,
            &format!("{REPO}/extracted.md"),
            "## README.md\n\nhi\n\n## docs/a.md\n\na\n",
        );
        put(
            &root,
            &format!("{SHOT}/source.md"),
            "---\ntype: source\ntitle: A shot\nfile: shot.png\nmedia_type: image/png\nsha256: cc\n---\n\nyaks extracts no text from this type; only the original is saved. Convert it and save it again with `--text <file>`.\n\nTaken by hand on the staging site.\n",
        );
        put(&root, &format!("{SHOT}/shot.png"), "PNG");
        put(
            &root,
            "doks/x.md",
            "---\ntitle: X\nsources:\n  - sources/2026-01-01-aaaaaaaa/source.md\n  - sources/2026-01-02-bbbbbbbb-repo/source.md\n---\n\n\
             See [the page](/sources/2026-01-01-aaaaaaaa/extracted.md#page), \
             [a doc](/sources/2026-01-02-bbbbbbbb-repo/files/docs/a.md), \
             [joined](/sources/2026-01-02-bbbbbbbb-repo/extracted.md#docsamd), \
             [the image](/sources/2026-01-03-cccccccc-shot/shot.png) and \
             [pinned](/sources/2026-01-01-aaaaaaaa/source.md@1234567).\n\n\
             `sources/2026-01-01-aaaaaaaa/source.md` in code stays.\n",
        );
        root
    }

    #[test]
    fn sources_become_one_file_each_and_a_tree_moves_up() {
        let root = old_farm("convert");
        let dry = dry_run(&root).unwrap();
        let r = apply(&root).unwrap();
        assert_eq!(dry, r, "a dry run counts what the real run does");
        assert_eq!((r.sources, r.trees, r.duplicates), (3, 1, 1));

        let page = read(&root, &format!("{PAGE}.md"));
        assert!(
            page.contains("title: A page") && !page.contains("file:"),
            "{page}"
        );
        assert!(page.ends_with("# Page\n\nText.\n"), "{page}");
        assert!(!root.join(PAGE).exists(), "nothing was left beside it");

        let repo = read(&root, &format!("{REPO}.md"));
        assert!(
            repo.contains("file_count: 2") && !repo.contains("files:"),
            "{repo}"
        );
        assert!(
            repo.contains("part_of: sources/2026-01-01-aaaaaaaa-page.md"),
            "{repo}"
        );
        assert!(
            repo.contains("[2026-01-02-bbbbbbbb-repo/](2026-01-02-bbbbbbbb-repo/)"),
            "{repo}"
        );
        assert!(
            !repo.contains("Skipped 1") && !repo.contains("extracted.md"),
            "{repo}"
        );
        let d = root.join(REPO);
        assert!(d.join("README.md").is_file() && d.join("docs/a.md").is_file());
        for gone in ["files", "source.md", "extracted.md"] {
            assert!(!d.join(gone).exists(), "{gone}");
        }

        let shot = read(&root, &format!("{SHOT}.md"));
        assert!(shot.contains("file: shot.png") && root.join(SHOT).join("shot.png").is_file());
        assert!(
            shot.contains("Taken by hand on the staging site."),
            "a person's note is kept: {shot}"
        );
        assert_eq!(shot.matches("yaks extracts no text").count(), 1, "{shot}");
    }

    #[test]
    fn references_follow_and_pins_and_code_stay() {
        let root = old_farm("refs");
        apply(&root).unwrap();
        let x = read(&root, "doks/x.md");
        for want in [
            "  - sources/2026-01-01-aaaaaaaa-page.md\n",
            "  - sources/2026-01-02-bbbbbbbb-repo.md\n",
            "(/sources/2026-01-01-aaaaaaaa-page.md#page)",
            "(/sources/2026-01-02-bbbbbbbb-repo/docs/a.md)",
            "(/sources/2026-01-02-bbbbbbbb-repo/docs/a.md)",
            "(/sources/2026-01-03-cccccccc-shot/shot.png)",
            "(/sources/2026-01-01-aaaaaaaa/source.md@1234567)",
            "`sources/2026-01-01-aaaaaaaa/source.md` in code stays",
        ] {
            assert!(x.contains(want), "{want} in:\n{x}");
        }
        assert_eq!(x.matches("bbbbbbbb-repo/docs/a.md)").count(), 2, "{x}");
    }

    #[test]
    fn a_second_run_finds_nothing_to_do() {
        let root = old_farm("again");
        apply(&root).unwrap();
        let r = apply(&root).unwrap();
        assert!(r.is_empty(), "{r:?}");
    }
}
