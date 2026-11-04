//! Schema migrations: the named, per-file fixups `yaks migrate` applies to a
//! farm that is behind [`crate::store::SCHEMA`].
//!
//! Each [`Fixup`] is a pure function from one yak's body to its fixed body
//! (plus how many edits it made). They are idempotent and independent, so a
//! later schema's fixup is one more entry in [`FIXUPS`]; `yaks migrate` runs
//! the whole list over every yak file (schema 4 -> 5), and a re-run on a
//! current farm changes nothing. Readers accept the shapes every fixup
//! rewrites *from*, so an un-migrated checkout or an old commit still parses.
//!
//! Schema 8 (the rename of wiki pages to doks) adds a second LAYOUT step,
//! [`plan_doks`], and one fixup over yak bodies: `wiki/` and `<herd>/wiki/` move
//! to `doks/` and `<herd>/doks/`, `/wiki/...` links become `/doks/...` (in yaks,
//! doks, schemas and sessions; never in a source, which is a snapshot), and a
//! schema's `type: Page Type` becomes `Dok Type`.
//!
//! Schema 6 adds one LAYOUT step that runs first ([`plan_layout`]): every yak
//! file and attachment directory at the farm root moves into its herd's
//! directory, `<herd>/yaks/` ([`store::task_path`]). Bodies are not touched by
//! it (attachment links are relative to the yak file, which moves with them).

use crate::store;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::Path;

/// What a fixup may look at besides the body: the farm's yak ids.
pub struct Ctx<'a> {
    pub ids: &'a HashSet<String>,
}

/// One named rewrite of a yak's body. `apply` returns the new body and the
/// number of edits made (0 = body returned unchanged).
pub struct Fixup {
    pub name: &'static str,
    /// What one counted edit is, for the report ("comment headers", ...).
    pub unit: &'static str,
    pub apply: fn(&Ctx, &str) -> (String, usize),
    /// Whether `apply` rewrites the frontmatter text (between the `---`
    /// lines) instead of the body.
    pub front: bool,
}

/// The fixups of schemas 5, 7 and 8, in the order they run.
pub const FIXUPS: &[Fixup] = &[
    Fixup {
        name: "labels-become-tags",
        unit: "key(s)",
        apply: labels_become_tags,
        front: true,
    },
    Fixup {
        name: "comment-header-blank-line",
        unit: "comment(s)",
        apply: comment_header_blank_line,
        front: false,
    },
    Fixup {
        name: "attachment-links",
        unit: "link(s)",
        apply: attachment_links,
        front: false,
    },
    Fixup {
        name: "attachment-link-encoding",
        unit: "link(s)",
        apply: attachment_link_encoding,
        front: false,
    },
    Fixup {
        name: "wiki-links-become-doks-links",
        unit: "link(s)",
        apply: wiki_links_become_doks_links,
        front: false,
    },
];

/// The farm-root link `dest` with its `wiki` directory renamed: `/wiki/x.md`
/// becomes `/doks/x.md` and `/<herd>/wiki/x.md` becomes `/<herd>/doks/x.md`,
/// its `@rev` and `#anchor` as written. `None` for any other link (`code:`
/// links, URLs, relative links, `/sources/wiki/...`).
pub fn wiki_link_to_doks(dest: &str) -> Option<String> {
    let rest = dest.strip_prefix('/')?;
    let (first, tail) = rest.split_once('/')?;
    if first == "wiki" {
        return Some(format!("/doks/{tail}"));
    }
    let (second, tail) = tail.split_once('/')?;
    (second == "wiki" && !store::RESERVED_HERD_NAMES.contains(&first))
        .then(|| format!("/{first}/doks/{tail}"))
}

/// Schema 8: every `/wiki/...` link of a markdown body becomes `/doks/...`
/// ([`wiki_link_to_doks`]). A pinned link keeps its `@rev`: the reader falls
/// back to `wiki/` at a rev from before the rename ([`crate::links::read_at`]).
pub fn wiki_links_become_doks_links(_: &Ctx, body: &str) -> (String, usize) {
    crate::links::rewrite(body, wiki_link_to_doks)
}

/// Schema 7 (yaks-fd7a): `](<id>/my shot.png)`, an attachment link whose file
/// name holds a space (which CommonMark does not read as a link), becomes
/// `](<id>/my%20shot.png)`, for any `<id>` that is a yak of this farm.
/// Fenced code blocks are skipped.
pub fn attachment_link_encoding(ctx: &Ctx, body: &str) -> (String, usize) {
    let mut edits = 0;
    let mut in_fence = false;
    let mut out: Vec<String> = Vec::new();
    for line in body.split('\n') {
        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            in_fence = !in_fence;
            out.push(line.to_string());
            continue;
        }
        if in_fence || !line.contains("](") {
            out.push(line.to_string());
            continue;
        }
        let mut fixed = String::with_capacity(line.len());
        let mut rest = line;
        while let Some(i) = rest.find("](") {
            fixed.push_str(&rest[..i + 2]);
            rest = &rest[i + 2..];
            let Some(close) = rest.find(')') else {
                continue;
            };
            let target = &rest[..close];
            let fix = target.split_once('/').filter(|(id, file)| {
                ctx.ids.contains(*id) && file.contains(' ') && !file.contains('/')
            });
            if let Some((id, file)) = fix {
                fixed.push_str(&format!("{id}/{}", crate::links::percent_encode_path(file)));
                rest = &rest[close..];
                edits += 1;
            }
        }
        fixed.push_str(rest);
        out.push(fixed);
    }
    (out.join("\n"), edits)
}

/// Schema 7 (yaks-37da): the top-level `labels:` key becomes `tags:`, its
/// value and every other line untouched. A file with both keeps both (doctor
/// has nothing to say about it; the reader prefers whichever comes last).
pub fn labels_become_tags(_: &Ctx, front: &str) -> (String, usize) {
    let has_tags = front
        .lines()
        .any(|l| l == "tags:" || l.starts_with("tags: "));
    if has_tags {
        return (front.to_string(), 0);
    }
    let mut edits = 0;
    let out: Vec<String> = front
        .split('\n')
        .map(|l| match l.strip_prefix("labels:") {
            Some(rest) if rest.is_empty() || rest.starts_with([' ', '\t']) => {
                edits += 1;
                format!("tags:{rest}")
            }
            _ => l.to_string(),
        })
        .collect();
    (out.join("\n"), edits)
}

/// `---` / `▸ <ts> [actor]` / text  ->  `---` / `▸ <ts> [actor]` / blank / text
/// (yaks-2362): the header becomes a paragraph of its own in CommonMark. A
/// comment with no text, or already followed by a blank line, is left alone.
pub fn comment_header_blank_line(_: &Ctx, body: &str) -> (String, usize) {
    let lines: Vec<&str> = body.split('\n').collect();
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut edits = 0;
    for i in 0..lines.len() {
        out.push(lines[i]);
        let header = i > 0 && store::is_note_delim(&lines, i - 1);
        if header
            && lines
                .get(i + 1)
                .is_some_and(|n| !n.trim().is_empty() && !store::is_note_delim(&lines, i + 1))
        {
            out.push("");
            edits += 1;
        }
    }
    (out.join("\n"), edits)
}

/// `![stem](<id>/<file>)` where `<file>` is not an image -> `[stem](<id>/<file>)`
/// (yaks-3b8d), for any `<id>` that is a yak of this farm. Fenced code blocks
/// are skipped. Images keep the `!`.
pub fn attachment_links(ctx: &Ctx, body: &str) -> (String, usize) {
    let mut edits = 0;
    let mut in_fence = false;
    let mut out: Vec<String> = Vec::new();
    for line in body.split('\n') {
        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            in_fence = !in_fence;
            out.push(line.to_string());
        } else if in_fence {
            out.push(line.to_string());
        } else {
            let (l, n) = fix_line(ctx, line);
            edits += n;
            out.push(l);
        }
    }
    (out.join("\n"), edits)
}

/// Drop the `!` of every `![stem](<id>/<file>)` in `line` whose file is not an
/// image.
fn fix_line(ctx: &Ctx, line: &str) -> (String, usize) {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    let mut edits = 0;
    while let Some(i) = rest.find("![") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 2..];
        // `stem](target)` with no nested `]` or `[` in the stem.
        let link = after.split_once("](").and_then(|(stem, tail)| {
            let (target, _) = tail.split_once(')')?;
            (!stem.contains(['[', ']'])).then_some(target)
        });
        let drop_bang = link.is_some_and(|target| {
            target.split_once('/').is_some_and(|(id, file)| {
                ctx.ids.contains(id)
                    && !file.is_empty()
                    && !file.contains(['/', ' '])
                    && !store::is_image_name(file)
            })
        });
        if drop_bang {
            edits += 1;
            out.push('[');
        } else {
            out.push_str("![");
        }
        rest = after;
    }
    out.push_str(rest);
    (out, edits)
}

/// What one fixup changed (or would change): per yak, the edit count.
#[derive(Debug)]
pub struct FixupReport {
    pub name: &'static str,
    pub unit: &'static str,
    /// `(yak id, edits)` for each yak the fixup touches, sorted by id.
    pub yaks: Vec<(String, usize)>,
}

/// One file or directory the layout step moves, farm-relative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    pub from: std::path::PathBuf,
    pub to: std::path::PathBuf,
}

/// The schema-6 layout step: every yak file at the farm root (`<id>.md` with
/// frontmatter naming that `id`) and its attachment directory (`<id>/`) moves
/// to `<herd>/yaks/` ([`store::task_path`], [`store::artifact_dir`]). A root
/// `.md` with no frontmatter, or whose `id:` is not its file name, is not a
/// yak file and stays. Refuses (before any move) when a destination already
/// exists, or when a yak's herd is a reserved name (`docs`). Returns the moves
/// in a stable order: yak files first, then attachment dirs.
pub fn plan_layout(root: &Path) -> Result<Vec<Move>> {
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return Ok(Vec::new());
    };
    let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    let mut ids = HashSet::new();
    for p in &entries {
        if !p.is_file() || p.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let text =
            std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
        let Some(task) = store::parse_task(&text, crate::model::Status::Hairy) else {
            continue;
        };
        if task.id != stem {
            continue;
        }
        ids.insert(stem.to_string());
        files.push(Move {
            from: p.clone(),
            to: store::task_path(root, stem),
        });
    }
    for p in &entries {
        let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if p.is_dir() && ids.contains(name) {
            dirs.push(Move {
                from: p.clone(),
                to: store::artifact_dir(root, name),
            });
        }
    }
    for id in &ids {
        let herd = store::herd_of(id);
        if store::RESERVED_HERD_NAMES.contains(&herd) {
            anyhow::bail!(
                "{id}'s herd `{herd}` is a reserved name (it would collide with the farm's {herd} \
                 directory); rename the herd first (`yaks rename-herd {herd} <new>`) with yaks 0.0.31"
            );
        }
    }
    // Schema 7 reserves more names: a herd directory already using one
    // would become invisible, so refuse before touching anything.
    for name in store::RESERVED_HERD_NAMES {
        if root.join(name).join(store::YAKS_SUBDIR).is_dir() {
            anyhow::bail!(
                "herd `{name}` is now a reserved name (the farm's {name} directory); rename the herd \
                 first (`yaks rename-herd {name} <new>`) with yaks 0.0.31"
            );
        }
    }
    let moves: Vec<Move> = files.into_iter().chain(dirs).collect();
    for m in &moves {
        if std::fs::symlink_metadata(&m.to).is_ok() {
            anyhow::bail!(
                "cannot move {} into its herd directory: {} already exists",
                m.from.display(),
                m.to.display()
            );
        }
    }
    Ok(moves)
}

/// The schema-8 layout step: `wiki/` becomes `doks/`, and so does each herd's
/// `<herd>/wiki/`. Refuses (before any move) when a destination already
/// exists, or when a herd is literally named `doks`. Farm-relative
/// `(from, to)` pairs, farm-wide first, then by herd.
pub fn plan_doks(root: &Path) -> Result<Vec<(String, String)>> {
    let Ok(rd) = std::fs::read_dir(root) else {
        return Ok(Vec::new());
    };
    let mut herds: Vec<String> = rd
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| {
            !n.starts_with('.') && n != "wiki" && !store::RESERVED_HERD_NAMES.contains(&n.as_str())
        })
        .collect();
    herds.sort();
    let mut moves = Vec::new();
    let mut candidates = vec![String::new()];
    candidates.extend(herds);
    for h in candidates {
        let (from, to) = if h.is_empty() {
            ("wiki".to_string(), "doks".to_string())
        } else {
            (format!("{h}/wiki"), format!("{h}/doks"))
        };
        if root.join(&from).is_dir() {
            if root.join(&to).exists() {
                anyhow::bail!("cannot move {from} to {to}: {to} already exists");
            }
            moves.push((from, to));
        }
    }
    Ok(moves)
}

/// Every markdown file of the non-yak documents the schema-8 link rewrite
/// reads: each doks root (at `wiki/` when `moved` is false), `schemas/` and
/// `sessions/`. Sources are not here: a source is a snapshot.
fn document_files(root: &Path, moved: bool) -> Vec<std::path::PathBuf> {
    let dir = if moved { "doks" } else { "wiki" };
    let mut tops: Vec<std::path::PathBuf> =
        vec![root.join(dir), root.join("schemas"), root.join("sessions")];
    if let Ok(rd) = std::fs::read_dir(root) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().into_owned();
            if e.path().is_dir()
                && !n.starts_with('.')
                && n != "wiki"
                && !store::RESERVED_HERD_NAMES.contains(&n.as_str())
            {
                tops.push(e.path().join(dir));
            }
        }
    }
    let mut out = Vec::new();
    let mut stack = tops;
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if e.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|x| x.to_str()) == Some("md") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// A document's schema-8 text and how many edits it took: its body's
/// `/wiki/` links, and a schema's `type: Page Type` in its frontmatter.
fn dok_text(text: &str) -> (String, usize) {
    let Some((_, body)) = store::split_frontmatter(text) else {
        return (text.to_string(), 0);
    };
    // `body` is the tail of `text`; everything before it is the frontmatter.
    let head = &text[..text.len() - body.len()];
    let (body, links) = wiki_links_become_doks_links(
        &Ctx {
            ids: &HashSet::new(),
        },
        body,
    );
    let mut types = 0;
    let head = head
        .split('\n')
        .map(|l| match l.trim_end() {
            "type: Page Type" | "type: \"Page Type\"" | "type: 'Page Type'" => {
                types += 1;
                l.replacen("Page Type", "Dok Type", 1)
            }
            _ => l.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n");
    (format!("{head}{body}"), links + types)
}

fn apply_layout(moves: &[Move]) -> Result<()> {
    for m in moves {
        if let Some(parent) = m.to.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::rename(&m.from, &m.to)
            .with_context(|| format!("moving {} to {}", m.from.display(), m.to.display()))?;
    }
    Ok(())
}

/// Result of a [`run`].
#[derive(Debug)]
pub struct Report {
    /// False for a dry run: nothing was written.
    pub applied: bool,
    /// The farm's schema marker before the run (`None`: no usable marker).
    pub from: Option<u32>,
    /// The layout step's moves (schema 6): per herd, `(herd, yak files,
    /// attachment dirs)`, sorted by herd.
    pub layout: Vec<(String, usize, usize)>,
    pub fixups: Vec<FixupReport>,
    /// The schema-8 layout step: `(from, to)` for each `wiki` directory moved.
    pub doks_roots: Vec<(String, String)>,
    /// Non-yak documents (doks, schemas, sessions) the schema-8 step rewrites
    /// (or would): `(farm-relative path, edits)`.
    pub docs: Vec<(String, usize)>,
    /// Yak files rewritten (or that would be): the union over the fixups.
    pub files: usize,
    /// Whether anything under the farm is tracked by git (so the result
    /// should be committed with `yaks commit`); false for a farm outside git.
    pub git_tracked: bool,
}

impl Report {
    /// Nothing changed and nothing would: the farm is current.
    pub fn is_noop(&self) -> bool {
        self.files == 0
            && self.layout.is_empty()
            && self.doks_roots.is_empty()
            && self.docs.is_empty()
            && self.from.is_none_or(|v| v == store::SCHEMA)
    }
}

/// Apply every fixup to every yak file under `root` (rewriting each file
/// once, frontmatter byte-for-byte as it was, `updated:` untouched, so a
/// migration commit is only the shape change), then write the `schema` marker
/// LAST: an interrupted run leaves the old marker and is simply re-run.
/// `dry_run` writes nothing. The caller holds the farm lock.
pub fn run(root: &Path, dry_run: bool) -> Result<Report> {
    let from = std::fs::read_to_string(root.join("schema"))
        .ok()
        .and_then(|raw| raw.trim().parse::<u32>().ok());
    // The layout step first, so the fixups below read every yak at its
    // schema-6 path. A dry run plans it and reads the files where they are.
    let moves = plan_layout(root)?;
    let mut layout: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
    for m in &moves {
        let name = m
            .from
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let id = name.strip_suffix(".md").unwrap_or(name);
        let e = layout.entry(store::herd_of(id).to_string()).or_default();
        if m.from.is_dir() {
            e.1 += 1;
        } else {
            e.0 += 1;
        }
    }
    let layout: Vec<(String, usize, usize)> =
        layout.into_iter().map(|(h, (f, d))| (h, f, d)).collect();
    let doks_roots = plan_doks(root)?;
    if !dry_run {
        apply_layout(&moves)?;
        for (from, to) in &doks_roots {
            std::fs::rename(root.join(from), root.join(to))
                .with_context(|| format!("moving {from} to {to}"))?;
        }
    }
    let mut paths: Vec<std::path::PathBuf> = if dry_run {
        let moved: HashSet<_> = moves.iter().map(|m| m.to.clone()).collect();
        store::yak_files(root)
            .into_iter()
            .filter(|p| !moved.contains(p))
            .chain(
                moves
                    .iter()
                    .filter(|m| m.from.is_file())
                    .map(|m| m.from.clone()),
            )
            .collect()
    } else {
        store::yak_files(root)
    };
    paths.sort();
    let ids: HashSet<String> = paths
        .iter()
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    let ctx = Ctx { ids: &ids };
    let mut fixups: Vec<FixupReport> = FIXUPS
        .iter()
        .map(|f| FixupReport {
            name: f.name,
            unit: f.unit,
            yaks: Vec::new(),
        })
        .collect();
    let mut writes: Vec<(std::path::PathBuf, String)> = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let Some((_, body)) = store::split_frontmatter(&text) else {
            continue; // not a yak file (no frontmatter): leave it alone
        };
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        // `head` is everything before the body: `---`, the frontmatter lines,
        // and the closing `---` with any blank lines after it.
        let mut head = text[..text.len() - body.len()].to_string();
        let mut body = body.to_string();
        for (f, report) in FIXUPS.iter().zip(&mut fixups) {
            let target = if f.front { &mut head } else { &mut body };
            let (next, edits) = (f.apply)(&ctx, target);
            if edits > 0 {
                report.yaks.push((id.to_string(), edits));
                *target = next;
            }
        }
        let new = format!("{head}{body}");
        if new != text {
            writes.push((path, new));
        }
    }
    // The non-yak documents: at `wiki/` on a dry run, at `doks/` once moved.
    let mut docs: Vec<(String, usize)> = Vec::new();
    let mut doc_writes: Vec<(std::path::PathBuf, String)> = Vec::new();
    for path in document_files(root, !dry_run) {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let (new, edits) = dok_text(&text);
        if edits > 0 {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            docs.push((rel, edits));
            doc_writes.push((path, new));
        }
    }
    let report = Report {
        applied: !dry_run,
        from,
        layout,
        fixups,
        doks_roots,
        docs,
        files: writes.len(),
        git_tracked: tracked_by_git(root),
    };
    if dry_run || report.is_noop() {
        return Ok(report);
    }
    for (path, text) in writes.iter().chain(&doc_writes) {
        store::write::atomic_write(path, text)?;
    }
    let schema = root.join("schema");
    store::write::atomic_write(&schema, &format!("{}\n", store::SCHEMA))?;
    Ok(report)
}

/// Whether git tracks anything under the farm at `root` (a farm branch).
/// False for a farm outside git.
fn tracked_by_git(root: &Path) -> bool {
    std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--", "."])
        .output()
        .is_ok_and(|o| o.status.success() && !o.stdout.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    const OLD: &str = "desc\n\n---\n\u{25b8} 2026-01-01T00:00:00Z [a]\nhello\nworld\n\n---\n\u{25b8} 2026-01-02T00:00:00Z\nsecond";
    const NEW: &str = "desc\n\n---\n\u{25b8} 2026-01-01T00:00:00Z [a]\n\nhello\nworld\n\n---\n\u{25b8} 2026-01-02T00:00:00Z\n\nsecond";

    #[test]
    fn labels_become_tags_renames_only_the_top_level_key() {
        let c = ctx(&[]);
        let front = "---\nid: t-1\nlabels:\n- ui\nnotes:\n  labels: keep\n---\n\n";
        let (out, n) = labels_become_tags(&Ctx { ids: &c }, front);
        assert_eq!(n, 1);
        assert_eq!(
            out,
            "---\nid: t-1\ntags:\n- ui\nnotes:\n  labels: keep\n---\n\n"
        );
        let (again, n) = labels_become_tags(&Ctx { ids: &c }, &out);
        assert_eq!((again.as_str(), n), (out.as_str(), 0), "idempotent");
        let inline = "---\nlabels: [a, b]\n---\n";
        assert_eq!(
            labels_become_tags(&Ctx { ids: &c }, inline).0,
            "---\ntags: [a, b]\n---\n"
        );
        // A file that already has tags keeps both keys untouched.
        let both = "---\ntags: [a]\nlabels: [b]\n---\n";
        assert_eq!(
            labels_become_tags(&Ctx { ids: &c }, both),
            (both.to_string(), 0)
        );
    }

    #[test]
    fn a_herd_named_like_a_new_reserved_dir_refuses_the_migration() {
        let root = std::env::temp_dir().join(format!("yaks-mig-reserved-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("doks/yaks")).unwrap();
        std::fs::write(root.join("schema"), "6\n").unwrap();
        let err = run(&root, true).unwrap_err().to_string();
        assert!(err.contains("herd `doks` is now a reserved name"), "{err}");
        assert_eq!(std::fs::read_to_string(root.join("schema")).unwrap(), "6\n");
        let _ = std::fs::remove_dir_all(&root);
    }

    // -- schema 8: wiki -> doks --------------------------------------------

    #[test]
    fn wiki_links_become_doks_links_only_where_they_are_farm_links() {
        let c = Ctx { ids: &ctx(&[]) };
        let body = "[a](/wiki/topics/a.md#x) [b](/yaks/wiki/b.md@3f2a9c1) [c](/sources/wiki/c.md) \
                    [d](code:wiki/d.md) [e](https://x.org/wiki/e) [f](wiki/f.md) [g](/doks/g.md)\n\n\
                    `[h](/wiki/h.md)`\n";
        let (out, n) = wiki_links_become_doks_links(&c, body);
        assert_eq!(n, 2, "{out}");
        assert!(out.contains("[a](/doks/topics/a.md#x)"), "{out}");
        assert!(out.contains("[b](/yaks/doks/b.md@3f2a9c1)"), "{out}");
        // Everything else is as written, code spans included.
        for keep in [
            "[c](/sources/wiki/c.md)",
            "[d](code:wiki/d.md)",
            "[e](https://x.org/wiki/e)",
            "[f](wiki/f.md)",
            "`[h](/wiki/h.md)`",
        ] {
            assert!(out.contains(keep), "{keep} in {out}");
        }
        assert_eq!(wiki_links_become_doks_links(&c, &out).1, 0, "idempotent");
    }

    /// A schema-7 farm with a farm-wide wiki, a herd wiki, links in a yak, a
    /// dok, a schema and a session, and a source that mentions `/wiki/`.
    fn schema_7_farm(tag: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("yaks-mig-doks-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let w = |rel: &str, text: &str| {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        };
        w("schema", "7\n");
        w(
            "yak/yaks/yak-0001.md",
            "---\nid: yak-0001\ntitle: T\nstatus: hairy\ntype: task\npriority: 3\ncreated: \"2026-01-01T00:00:00Z\"\nupdated: \"2026-01-01T00:00:00Z\"\n---\n\nSee [a](/wiki/topics/a.md) and [b](/yak/wiki/b.md#h).\n",
        );
        w(
            "wiki/topics/a.md",
            "---\ntype: topic\ntitle: A\n---\n\nSee [b](/yak/wiki/b.md) and [local](b.md).\n",
        );
        w(
            "yak/wiki/b.md",
            "---\ntype: note\ntitle: B\n---\n\nBack to [a](/wiki/topics/a.md@3f2a9c1).\n",
        );
        w(
            "schemas/thing.md",
            "---\ntype: Page Type\ntitle: Thing\n---\n",
        );
        w(
            "sessions/s.md",
            "---\ntype: session-review\n---\n\n[a](/wiki/topics/a.md)\n",
        );
        w(
            "sources/2026-01-01-abcd1234/extracted.md",
            "A saved page that links [x](/wiki/x.md): a snapshot, never rewritten.\n",
        );
        root
    }

    #[test]
    fn schema_8_moves_the_wikis_and_rewrites_links_once() {
        let root = schema_7_farm("run");
        let src_before =
            std::fs::read(root.join("sources/2026-01-01-abcd1234/extracted.md")).unwrap();

        // A dry run says what it would do and writes nothing.
        let dry = run(&root, true).unwrap();
        assert!(!dry.applied);
        assert_eq!(
            dry.doks_roots,
            vec![
                ("wiki".to_string(), "doks".to_string()),
                ("yak/wiki".to_string(), "yak/doks".to_string())
            ]
        );
        assert!(root.join("wiki").is_dir() && !root.join("doks").exists());
        assert_eq!(std::fs::read_to_string(root.join("schema")).unwrap(), "7\n");

        let r = run(&root, false).unwrap();
        assert!(r.applied && !r.is_noop());
        assert_eq!(std::fs::read_to_string(root.join("schema")).unwrap(), "8\n");
        assert!(!root.join("wiki").exists() && !root.join("yak/wiki").exists());
        let read = |rel: &str| std::fs::read_to_string(root.join(rel)).unwrap();
        let yak = read("yak/yaks/yak-0001.md");
        assert!(
            yak.contains("[a](/doks/topics/a.md) and [b](/yak/doks/b.md#h)"),
            "{yak}"
        );
        assert!(read("doks/topics/a.md").contains("[b](/yak/doks/b.md) and [local](b.md)"));
        assert!(
            read("yak/doks/b.md").contains("[a](/doks/topics/a.md@3f2a9c1)"),
            "pins keep their rev"
        );
        assert!(read("schemas/thing.md").contains("type: Dok Type\n"));
        assert!(read("sessions/s.md").contains("(/doks/topics/a.md)"));
        // A source is a snapshot: byte for byte as it was.
        assert_eq!(
            std::fs::read(root.join("sources/2026-01-01-abcd1234/extracted.md")).unwrap(),
            src_before
        );

        // Idempotent: a second run changes nothing.
        let again = run(&root, false).unwrap();
        assert!(again.is_noop(), "{again:?}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn schema_8_refuses_before_moving_when_a_destination_exists() {
        let root = schema_7_farm("clash");
        std::fs::create_dir_all(root.join("doks")).unwrap();
        let err = run(&root, false).unwrap_err().to_string();
        assert!(err.contains("doks already exists"), "{err}");
        assert!(root.join("wiki/topics/a.md").is_file(), "nothing moved");
        assert_eq!(std::fs::read_to_string(root.join("schema")).unwrap(), "7\n");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn header_fixup_inserts_the_blank_line_once() {
        let ids = ctx(&[]);
        let c = Ctx { ids: &ids };
        assert_eq!(comment_header_blank_line(&c, OLD), (NEW.to_string(), 2));
        // Idempotent, and the new shape is untouched.
        assert_eq!(comment_header_blank_line(&c, NEW), (NEW.to_string(), 0));
    }

    #[test]
    fn header_fixup_leaves_empty_comments_and_prose_rules_alone() {
        let ids = ctx(&[]);
        let c = Ctx { ids: &ids };
        let body = "a\n\n---\n\nb\n\n---\n\u{25b8} 2026-01-01T00:00:00Z\n---\n\u{25b8} 2026-01-02T00:00:00Z";
        assert_eq!(comment_header_blank_line(&c, body), (body.to_string(), 0));
        // A trailing newline survives.
        let (out, n) = comment_header_blank_line(&c, "---\n\u{25b8} 2026-01-01T00:00:00Z\nx\n");
        assert_eq!(
            (out.as_str(), n),
            ("---\n\u{25b8} 2026-01-01T00:00:00Z\n\nx\n", 1)
        );
    }

    #[test]
    fn link_fixup_unbangs_non_images_only() {
        let ids = ctx(&["yaks-a1b2", "yaks-c3d4"]);
        let c = Ctx { ids: &ids };
        let old = "![log](yaks-a1b2/log.txt) and ![shot](yaks-a1b2/shot.PNG)\n![n](yaks-c3d4/notes.md) ![v](yaks-a1b2/f.svg)";
        let new = "[log](yaks-a1b2/log.txt) and ![shot](yaks-a1b2/shot.PNG)\n[n](yaks-c3d4/notes.md) ![v](yaks-a1b2/f.svg)";
        assert_eq!(attachment_links(&c, old), (new.to_string(), 2));
        assert_eq!(attachment_links(&c, new), (new.to_string(), 0));
    }

    #[test]
    fn link_fixup_ignores_foreign_links_and_code_fences() {
        let ids = ctx(&["yaks-a1b2"]);
        let c = Ctx { ids: &ids };
        let body = "![x](other/log.txt) ![y](https://e.com/a.txt) ![z](yaks-a1b2/a/b.txt)\n```\n![c](yaks-a1b2/log.txt)\n```";
        assert_eq!(attachment_links(&c, body), (body.to_string(), 0));
    }

    #[test]
    fn attachment_link_names_with_spaces_are_encoded() {
        let ids: HashSet<String> = ["t-1".to_string()].into_iter().collect();
        let c = Ctx { ids: &ids };
        let old = "See ![my shot](t-1/my shot.png) and [a b](t-1/a b.txt).\n```\n[x](t-1/a b.txt)\n```\n[o](other/a b.txt)";
        let new = "See ![my shot](t-1/my%20shot.png) and [a b](t-1/a%20b.txt).\n```\n[x](t-1/a b.txt)\n```\n[o](other/a b.txt)";
        assert_eq!(attachment_link_encoding(&c, old), (new.to_string(), 2));
        assert_eq!(attachment_link_encoding(&c, new), (new.to_string(), 0));
    }
}
