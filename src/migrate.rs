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
    /// The schema this fixup brought a farm to. It runs only on a farm marked
    /// below it: on a farm that has had it, it is at best noise, and the
    /// comment-header one is wrong (it would split a note's metadata block
    /// from its header).
    pub schema: u32,
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
        schema: 7,
        unit: "key(s)",
        apply: labels_become_tags,
        front: true,
    },
    Fixup {
        name: "comment-header-blank-line",
        schema: 5,
        unit: "comment(s)",
        apply: comment_header_blank_line,
        front: false,
    },
    Fixup {
        name: "attachment-links",
        schema: 5,
        unit: "link(s)",
        apply: attachment_links,
        front: false,
    },
    Fixup {
        name: "attachment-link-encoding",
        schema: 7,
        unit: "link(s)",
        apply: attachment_link_encoding,
        front: false,
    },
    Fixup {
        name: "wiki-links-become-doks-links",
        schema: 8,
        unit: "link(s)",
        apply: wiki_links_become_doks_links,
        front: false,
    },
];

/// Whether `f` runs on a farm whose marker says `from` (`None`: no usable
/// marker, which is read as the oldest: every fixup runs).
fn fixup_due(f: &Fixup, from: Option<u32>) -> bool {
    from.is_none_or(|v| v < f.schema)
}

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

/// What the schema-9 link step reads about the text it rewrites.
pub struct LinkCx<'a> {
    /// Every yak id of the farm.
    pub ids: &'a HashSet<String>,
    /// Where the text sits, which decides what a relative link means.
    pub from: crate::links::From<'a>,
    /// The yak whose file or attachment this is; `None` for a dok or a session.
    pub own: Option<&'a str>,
    /// `sources::plan_slugs`: old source directory to its slugged name.
    pub sources: &'a [(String, String)],
}

/// How many links of each kind the schema-9 link step rewrote in one text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LinkEdits {
    /// A bare yak id (or its note) to `/<herd>/yaks/<id>.md[#note]`.
    pub yak: usize,
    /// An attachment link to the form that works from where it sits.
    pub attachment: usize,
    /// `/doks/sessions|sources/...` to the real farm path.
    pub namespace: usize,
    /// A source directory to its slugged name.
    pub source: usize,
}

impl LinkEdits {
    pub fn total(&self) -> usize {
        self.yak + self.attachment + self.namespace + self.source
    }
    pub fn add(&mut self, o: LinkEdits) {
        self.yak += o.yak;
        self.attachment += o.attachment;
        self.namespace += o.namespace;
        self.source += o.source;
    }
}

/// `/doks/sessions/...` and `/doks/sources/...` name a path that does not
/// exist: sessions and sources sit beside `doks/`, not inside it (yaks-bede).
/// The farm path they meant, or `None` for any other link.
fn namespace_fix(dest: &str) -> Option<String> {
    let rest = dest.strip_prefix("/doks/")?;
    let first = rest.split(['/', '#', '@']).next()?;
    matches!(first, "sessions" | "sources").then(|| format!("/{rest}"))
}

/// The link `dest` as it should be stored at schema 9, and which kind of edit
/// that was; `None` when it is already right or is not one of ours.
///
/// - a yak (bare id, or `<id>#<note>`) is `/<herd>/yaks/<id>.md`, with a note
///   anchor in its slug form, which is what a markdown host gives the
///   `#### <timestamp>` heading;
/// - an attachment is the bare sibling name inside its own yak's attachments,
///   `<id>/<file>` in a yak of the same herd (the file sits beside the yak's
///   directory, so that already resolves on a host), and the path from the
///   farm root, `/<herd>/yaks/<id>/<file>`, everywhere else;
/// - `/doks/sessions|sources/...` loses its `doks/`, and a source directory
///   gains its slug.
///
/// Code links, URLs, pins and relative document links are left as written.
fn canonical_dest(cx: &LinkCx, dest: &str) -> Option<(String, LinkEdits)> {
    use crate::links::{From as At, Link};
    let mut e = LinkEdits::default();
    let mut cur = dest.to_string();
    if let Some(fixed) = namespace_fix(&cur) {
        cur = fixed;
        e.namespace = 1;
    }
    if let Some(new) = crate::sources::rewrite_ref(&cur, cx.sources) {
        cur = new;
        e.source = 1;
    }
    let link = crate::links::parse(&cur, cx.from, |s| cx.ids.contains(s));
    let frag = match cur.split_once('#') {
        Some((_, f)) => format!("#{f}"),
        None => String::new(),
    };
    let want = match &link {
        Link::Yak { id, note } => {
            let anchor = note
                .as_deref()
                .and_then(crate::links::slug_note_anchor)
                .map(|s| format!("#{s}"))
                .unwrap_or_default();
            e.yak = 1;
            format!("/{}/yaks/{id}.md{anchor}", store::herd_of(id))
        }
        // Already the path from the farm root: it resolves on a host from
        // anywhere, so it is not shortened to a sibling name.
        Link::Attachment { .. } if cur.starts_with('/') => {
            return (cur != dest).then_some((cur, e));
        }
        Link::Attachment { id, name, .. } => {
            let enc = crate::links::percent_encode_path(name);
            let own_attachment = matches!(cx.from, At::Attachment(o) if o == id);
            let beside_yak = matches!(cx.from, At::Yak)
                && cx
                    .own
                    .is_some_and(|o| store::herd_of(o) == store::herd_of(id));
            e.attachment = 1;
            if own_attachment {
                format!("{enc}{frag}")
            } else if beside_yak {
                format!("{id}/{enc}{frag}")
            } else {
                format!("/{}/yaks/{id}/{enc}{frag}", store::herd_of(id))
            }
        }
        _ => return (cur != dest).then_some((cur, e)),
    };
    (want != dest).then_some((want, e))
}

/// Schema 9: rewrite every link of a markdown `text` that [`canonical_dest`]
/// names, code spans and fences untouched ([`crate::links::rewrite`]). The new
/// text and how many links of each kind changed. Idempotent.
pub fn canonical_links(cx: &LinkCx, text: &str) -> (String, LinkEdits) {
    let total = std::cell::Cell::new(LinkEdits::default());
    let (out, _) = crate::links::rewrite(text, |dest| {
        let (new, e) = canonical_dest(cx, dest)?;
        let mut t = total.get();
        t.add(e);
        total.set(t);
        Some(new)
    });
    (out, total.get())
}

/// The farm's `.gitattributes` with its rules moved from the `.yaks/`
/// directory to the branch root, one line at a time: `.yaks/**/*.md
/// merge=yaks` becomes `**/*.md merge=yaks`, and every LFS rule
/// (`.yaks/**/*.[pP][nN][gG] filter=lfs ...`) likewise. A line that does not
/// start with `.yaks/` (another rule, a comment, a blank) is kept as written,
/// and so is the order, so a repository's own rules stay where they are.
/// Returns the new text and the number of lines that moved. Idempotent.
pub fn move_attribute_rules(text: &str) -> (String, usize) {
    let mut moved = 0;
    let mut out: Vec<String> = Vec::new();
    for line in text.split('\n') {
        match line.strip_prefix(".yaks/") {
            Some(rest) => {
                moved += 1;
                out.push(rest.to_string());
            }
            None => out.push(line.to_string()),
        }
    }
    (out.join("\n"), moved)
}

/// The comment above the LFS rules says where they apply: `.yaks/` becomes the
/// farm. Kept apart from [`move_attribute_rules`] so the rule lines stay a pure
/// prefix strip.
pub(crate) fn lfs_marker_text(text: &str) -> String {
    text.replace(
        "# yaks lfs: binary originals under .yaks/ are stored in Git LFS",
        "# yaks lfs: binary originals in the farm are stored in Git LFS",
    )
}

/// Schema 9 (yaks-9f00): a yak's notes in the new shape, a `#### <timestamp>`
/// heading each ([`store::convert_notes`]). A body whose note headers are not
/// timestamps (a hand-written note header with a date and a title) comes back
/// unchanged, still readable in the old shape.
pub fn notes_new_shape(_: &Ctx, body: &str) -> (String, usize) {
    store::convert_notes(body)
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

/// What the schema-9 stage found, or did: the same numbers on a dry run and
/// on the real run, since both come from one plan.
#[derive(Debug, Default)]
pub struct Schema9 {
    /// Links rewritten, by kind, and how many files held one.
    pub links: LinkEdits,
    pub link_files: usize,
    /// Notes put into the new shape, and in how many yaks.
    pub notes: usize,
    pub note_yaks: usize,
    /// Yaks whose note headers are not timestamps, left in the old shape.
    pub old_shape: Vec<String>,
    /// Source directories given a slug: `(old, new)`.
    pub sources: Vec<(String, String)>,
    /// The files the layout stage moved out of `.yaks/`: `(from, to)`, from
    /// the repository top. Empty on a dry run (the count is `files_to_move`).
    pub moved: Vec<(String, String)>,
    /// How many files a dry run says the layout stage would move.
    pub files_to_move: usize,
    /// The farm is a farm branch (it has git to commit to).
    pub on_branch: bool,
    /// The farm sits under `.yaks/` of a farm branch and moves to its root.
    pub move_to_root: bool,
    /// Why the real run refuses; a dry run lists them and goes on. The farm
    /// branch's state (uncommitted changes, behind its upstream, a merge in
    /// progress), which `run` checks before any write.
    pub blocked: Vec<String>,
    /// Why the plan cannot be applied even on a clean farm (two directories
    /// for one source id): also listed by a dry run, found by planning.
    pub plan_blocked: Vec<String>,
    /// The move commit (short hash): every farm file renamed, none changed.
    pub move_commit: Option<String>,
    /// The rewrite commit (short hash): links, notes and the `schema` marker.
    pub commit: Option<String>,
}

impl Schema9 {
    fn is_empty(&self) -> bool {
        self.links.total() == 0 && self.notes == 0 && self.sources.is_empty() && !self.move_to_root
    }
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
    /// The schema-9 stage.
    pub s9: Schema9,
}

impl Report {
    /// Nothing changed and nothing would: the farm is current.
    pub fn is_noop(&self) -> bool {
        self.files == 0
            && self.layout.is_empty()
            && self.doks_roots.is_empty()
            && self.docs.is_empty()
            && self.s9.is_empty()
            && self.from.is_none_or(|v| v == store::SCHEMA)
    }
}

/// The schema-9 content stage, planned and not applied: every file whose
/// text changes (`(path, new text)`), the source renames, and the counts for
/// the report. Reads the farm as it is at `root`, in the layout `run` has
/// already put it in; writes nothing.
///
/// Per file kind: a yak body gets its links and its notes; the attachment
/// documents of a yak get their links (as siblings of that yak); a dok, a
/// schema and a session get their links. A source is a snapshot and is never
/// read. `code:` links and every URL stay as written ([`canonical_dest`]).
fn plan_schema9(root: &Path) -> Result<(Vec<(std::path::PathBuf, String)>, Schema9)> {
    let mut blocked = Vec::new();
    // Two directories carrying one source id is the same source saved twice
    // (two branches): do not guess which to keep.
    let dups = crate::sources::duplicate_ids(root);
    if !dups.is_empty() {
        let list: Vec<String> = dups.iter().map(|d| d.join(" and ")).collect();
        blocked.push(format!(
            "sources saved twice under one id, which the slug rename cannot choose between: {}",
            list.join("; ")
        ));
    }
    let (writes, mut s9) = plan_schema9_with(root, &crate::sources::plan_slugs(root))?;
    s9.plan_blocked = blocked;
    Ok((writes, s9))
}

/// [`plan_schema9`] with the source renames given (`sources`: the plan, taken
/// before the directories were renamed), for the rewrite at the new root:
/// links to a source name its slugged directory, which is by then the real one.
fn plan_schema9_with(
    root: &Path,
    sources: &[(String, String)],
) -> Result<(Vec<(std::path::PathBuf, String)>, Schema9)> {
    let mut s9 = Schema9 {
        sources: sources.to_vec(),
        ..Schema9::default()
    };
    let ids = store::all_ids(root);
    let mut writes: Vec<(std::path::PathBuf, String)> = Vec::new();
    let mut link_files = 0;
    let mut note_yaks = 0;
    let rel = |p: &Path| {
        p.strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy()
            .into_owned()
    };

    // Yaks: links, then notes.
    for path in store::yak_files(root) {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let Some((_, body)) = store::split_frontmatter(&text) else {
            continue;
        };
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let head = &text[..text.len() - body.len()];
        let (body, e) = canonical_links(
            &LinkCx {
                ids: &ids,
                from: crate::links::From::Yak,
                own: Some(&id),
                sources: &s9.sources,
            },
            body,
        );
        let notes_before = store::parse_notes(&body);
        let (body, converted) = store::convert_notes(&body);
        if converted == 0
            && !notes_before.is_empty()
            && store::body_shape(&body) == Some(store::NoteShape::Old)
        {
            s9.old_shape.push(id.clone());
        }
        if e.total() > 0 {
            s9.links.add(e);
            link_files += 1;
        }
        if converted > 0 {
            s9.notes += converted;
            note_yaks += 1;
        }
        let new = format!("{head}{body}");
        if new != text {
            writes.push((path, new));
        }
    }

    // The markdown documents attached to a yak (an evidence write-up): links,
    // as siblings of that yak's attachments.
    for id in &ids {
        for a in crate::farm::list_attachments(root, id) {
            if !crate::links::is_markdown_name(&a.name) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&a.path) else {
                continue;
            };
            let (new, e) = canonical_links(
                &LinkCx {
                    ids: &ids,
                    from: crate::links::From::Attachment(id),
                    own: None,
                    sources: &s9.sources,
                },
                &text,
            );
            if e.total() > 0 {
                s9.links.add(e);
                link_files += 1;
                writes.push((a.path, new));
            }
        }
    }

    // Doks, schemas and sessions.
    for path in document_files(root, true) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel_path = rel(&path);
        let (new, e) = canonical_links(
            &LinkCx {
                ids: &ids,
                from: crate::links::From::Dok(&rel_path),
                own: None,
                sources: &s9.sources,
            },
            &text,
        );
        if e.total() > 0 {
            s9.links.add(e);
            link_files += 1;
            writes.push((path, new));
        }
    }
    s9.link_files = link_files;
    s9.note_yaks = note_yaks;
    writes.sort_by(|a, b| a.0.cmp(&b.0));
    Ok((writes, s9))
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
    // Refuse before the first write: the migration must not be what makes the
    // farm dirty and then trip over it. Only the real run refuses; a dry run
    // reports the same reasons (see `describe_layout_stage`).
    if !dry_run {
        if let Some((_, why)) = layout_refusal(root) {
            anyhow::bail!("{why}");
        }
    }
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
            if !fixup_due(f, from) {
                continue;
            }
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
    let mut report = Report {
        applied: !dry_run,
        from,
        layout,
        fixups,
        doks_roots,
        docs,
        files: writes.len(),
        git_tracked: tracked_by_git(root),
        s9: Schema9::default(),
    };
    if dry_run {
        // Earlier schemas' steps are not applied, so the farm is read as it
        // lies: plan the schema-9 stage only for a farm already at 8, whose
        // layout is the one it assumes.
        if from.is_none_or(|v| v >= 8) {
            let (_, s9) = plan_schema9(root)?;
            report.s9 = s9;
            describe_layout_stage(root, &mut report.s9);
        }
        return Ok(report);
    }
    if report.is_noop() && from.is_none_or(|v| v >= store::SCHEMA) {
        return Ok(report);
    }
    for (path, text) in writes.iter().chain(&doc_writes) {
        store::write::atomic_write(path, text)?;
    }
    report.s9 = apply_schema9(root)?;
    Ok(report)
}

/// The schema-9 stage, applied, on the farm the earlier steps left at schema
/// 8. Two commits on a farm branch, so git can follow every file across it:
///
/// 1. the move: every tracked path renamed with no change to its bytes (a 100%
///    rename, which git pairs by blob id whatever the rename limit), the
///    source directories renamed, `.gitattributes`, the README and the stub;
/// 2. the rewrite: links and notes at the new paths, then the `schema` marker.
///
/// A farm outside git, or one already at the branch root, has no move and
/// gets the rewrite in place. An interrupted run is safe between the two: the
/// farm is at the root with its marker still below 9, so a re-run skips the
/// move and does the rewrite (the stub says 9, so an older yaks says
/// "upgrade" in either state).
fn apply_schema9(root: &Path) -> Result<Schema9> {
    // Plan against the farm as it is, with the source renames the second
    // commit's links point at. Nothing is written until it is known to apply.
    let (_, mut s9) = plan_schema9(root)?;
    describe_layout_stage(root, &mut s9);
    if let Some(why) = s9.plan_blocked.first() {
        anyhow::bail!("{why}");
    }
    // The farm root after the move: the branch top, or where it was.
    let mut now = root.to_path_buf();
    if s9.move_to_root {
        apply_source_renames(root, &s9.sources)?;
        let moved = apply_layout_stage(root)?;
        let top = store::farm_git(root)
            .context("the farm is not on its own farm branch")?
            .top;
        let count = moved.len();
        s9.moved = moved;
        s9.move_commit = Some(commit_all(
            &top,
            &format!(
                "yaks: move the farm to the branch root, schema 9 ({count} file(s), no content change)"
            ),
        )?);
        now = top;
    } else {
        apply_source_renames(root, &s9.sources)?;
    }
    // The rewrite, planned again at the new paths: the plan is pure, and its
    // source renames are now empty (they were applied), so links to a source
    // are rewritten from the plan taken before the move.
    let (writes, again) = plan_schema9_with(&now, &s9.sources)?;
    for (path, text) in &writes {
        store::write::atomic_write(path, text)?;
    }
    s9.links = again.links;
    s9.link_files = again.link_files;
    s9.notes = again.notes;
    s9.note_yaks = again.note_yaks;
    s9.old_shape = again.old_shape;
    store::write::atomic_write(&now.join("schema"), &format!("{}\n", store::SCHEMA))?;
    if s9.move_to_root {
        let n = writes.len() + 1;
        s9.commit = Some(commit_all(
            &now,
            &format!(
                "yaks: migrate the farm to schema {} (links, notes: {n} file(s))",
                store::SCHEMA
            ),
        )?);
    }
    Ok(s9)
}

/// Fill in what the layout stage would do and whether it must refuse: whether
/// the farm is a farm branch, whether it sits under `.yaks/`. The stage itself
/// is [`apply_layout_stage`], run after the content is written.
fn describe_layout_stage(root: &Path, s9: &mut Schema9) {
    let Some(fg) = store::farm_git(root) else {
        return;
    };
    s9.on_branch = true;
    s9.move_to_root = !fg.prefix.is_empty();
    if s9.move_to_root {
        let nested = fg.prefix.trim_end_matches('/').to_string();
        let tracked = crate::branchfarm::git(&fg.top, &["ls-files", "-z", "--", &nested])
            .map(|o| o.split('\0').filter(|p| !p.is_empty()).count())
            .unwrap_or(0);
        let loose = untracked_under(&fg.top, &nested).map_or(0, |u| u.len());
        s9.files_to_move = tracked + loose;
    }
    if let Some((_, why)) = layout_refusal(root) {
        s9.blocked.push(why);
    }
}

/// A farm branch that is not safe to migrate, and why: uncommitted changes
/// (the migration commit would sweep them in, and `--only` could not tell
/// them from its own), a merge in progress, or behind its upstream (what is
/// there would arrive under `.yaks/`). `None` when the farm may go ahead. The
/// branch name comes with it for the message.
fn layout_refusal(root: &Path) -> Option<(String, String)> {
    let fg = store::farm_git(root)?;
    let branch = fg.branch.clone();
    if let Ok(Some((head, what))) = crate::changes::pending_operation(root) {
        return Some((
            branch,
            format!("a {what} is in progress ({head} is set): finish or abort it first"),
        ));
    }
    if let Ok(Some(changes)) = crate::changes::read(root) {
        if !changes.is_empty() {
            return Some((
                branch,
                format!(
                    "the farm has {} uncommitted change(s): `yaks commit` them first, so the \
                     migration is one commit of its own",
                    changes.len()
                ),
            ));
        }
    }
    // Behind (or diverged) is the risk: what the upstream holds that this
    // checkout lacks would arrive under `.yaks/` after the move. Ahead is
    // harmless, the push stays a fast-forward. A farm never published has no
    // upstream and nobody to strand.
    if let Some(s) = crate::status::sync_state(root) {
        if let (Some(up), b) = (&s.upstream, s.behind) {
            if b > 0 {
                return Some((
                    branch,
                    format!(
                        "the farm branch is {b} commit(s) behind {up}: `yaks sync` first, so \
                         work already there is in this checkout before the move"
                    ),
                ));
            }
        }
    }
    None
}

/// What the landing `README.md` of a farm branch says. Static: it is written
/// once by the migration and by `yaks init`, and names no yak, so nothing in it
/// goes stale.
pub const README: &str = "\
# yaks farm

This branch is a [yaks](https://github.com/joelgwebber/yaks) farm: tasks, notes
and working knowledge kept as markdown files, one per thing, so they read on
the web as they do in the terminal.

| Where | What |
|---|---|
| `<herd>/yaks/<id>.md` | a yak (a task), its notes at the end; attachments in `<herd>/yaks/<id>/` |
| `doks/`, `<herd>/doks/` | doks: documents that outlive a yak (designs, research, how-tos) |
| `sessions/` | session notes: how a working session went |
| `sources/` | outside material, saved once and cited |
| `schemas/` | the shape of typed doks, and the known tags |
| `config.yaml`, `schema` | the farm's settings and schema version |

Links between these files are paths from the root of this branch
(`/doks/design/x.md`), which GitHub and most hosts follow.

To work with it, install yaks (`npm i -g @j15r/yaks`) and run `yaks tui` from a
checkout of the repository this branch belongs to.
";

/// What the old-binary stub `.yaks/schema` holds: the new schema number, so a
/// yaks that predates it finds no farm at the old place but, reading the
/// marker, says "newer schema, upgrade" and not "restore .yaks/".
pub const STUB_NOTE: &str = "\
This directory is a stub. The farm moved to the root of this branch at
schema 9; an older yaks reads this file to say it needs upgrading
(`npm i -g @j15r/yaks@latest`). Safe to delete once every machine is upgraded.
";

/// The layout stage, applied: move the farm from `<top>/.yaks/` to `<top>/`,
/// with git, and rewrite the rules that named `.yaks/`. Order matters for an
/// interrupted run: nothing here changes the `schema` marker, which `run`
/// writes after, so a half-moved farm is read as the old layout (its
/// `config.yaml` is still under `.yaks/`) and a re-run finishes the job.
///
/// 1. `git mv` every tracked path, then move anything untracked that sits under
///    `.yaks/` (the lock file is ignored and left behind).
/// 2. `.gitattributes`: the rules lose their `.yaks/`.
/// 3. `README.md`, and the stub `.yaks/schema` holding the new number.
///
/// The caller commits ([`commit_all`]). Returns the paths it moved.
fn apply_layout_stage(root: &Path) -> Result<Vec<(String, String)>> {
    let fg = store::farm_git(root).context("the farm is not on its own farm branch")?;
    let top = fg.top.clone();
    let git = |args: &[&str]| -> Result<String> { crate::branchfarm::git(&top, args) };
    if fg.prefix.is_empty() {
        return Ok(Vec::new());
    }
    let nested = fg.prefix.trim_end_matches('/').to_string(); // ".yaks"
    // The content stage rewrote files and renamed source directories on disk,
    // so the index still lists the old paths. Stage the farm first, so what
    // `git mv` is asked to move is what exists (and a renamed source is a
    // rename git can follow, not a delete and an add).
    git(&["add", "-A", "--", &nested])?;
    let listed = git(&["ls-files", "-z", "--", &nested])?;
    let mut moved: Vec<(String, String)> = Vec::new();
    for rel in listed.split('\0').filter(|p| !p.is_empty()) {
        let Some(to) = rel.strip_prefix(&fg.prefix) else {
            continue;
        };
        if top.join(to).exists() {
            anyhow::bail!(
                "cannot move {rel} to {to}: {to} already exists at the branch root; \
                 nothing was moved"
            );
        }
        moved.push((rel.to_string(), to.to_string()));
    }
    for (from, to) in &moved {
        if let Some(parent) = Path::new(to).parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(top.join(parent))
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        git(&["mv", "--", from, to])?;
    }
    // Untracked files under `.yaks/` (a new yak not yet committed) follow.
    for (from, to) in untracked_under(&top, &nested)? {
        if top.join(&to).exists() {
            anyhow::bail!("cannot move {from} to {to}: {to} already exists at the branch root");
        }
        if let Some(parent) = Path::new(&to)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(top.join(parent))?;
        }
        std::fs::rename(top.join(&from), top.join(&to))
            .with_context(|| format!("moving {from} to {to}"))?;
        moved.push((from, to));
    }
    // The rules that named `.yaks/`.
    let attrs = top.join(".gitattributes");
    if let Ok(text) = std::fs::read_to_string(&attrs) {
        let (text, n) = move_attribute_rules(&text);
        if n > 0 {
            store::write::atomic_write(&attrs, &lfs_marker_text(&text))?;
        }
    }
    // `git mv` leaves the emptied directories behind: remove every directory
    // under `.yaks/` that holds nothing (the ignored lock file is not a file
    // worth keeping a directory for, and goes with it).
    let _ = std::fs::remove_file(top.join(&nested).join(".lock"));
    remove_empty_dirs(&top.join(&nested));
    // The landing page and the stub.
    let readme = top.join("README.md");
    if !readme.exists() {
        store::write::atomic_write(&readme, README)?;
    }
    // The marker moved with everything else, so the farm at the root still
    // reads as the schema it was (the rewrite commit bumps it). The stub the
    // old binaries find already says the new one.
    let stub = top.join(&nested);
    std::fs::create_dir_all(&stub)?;
    store::write::atomic_write(&stub.join("schema"), &format!("{}\n", store::SCHEMA))?;
    store::write::atomic_write(&stub.join("README.md"), STUB_NOTE)?;
    Ok(moved)
}

/// Remove `dir` and every directory under it that holds no file, deepest
/// first. A directory with any file in it is left as it is.
fn remove_empty_dirs(dir: &Path) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        if e.path().is_dir() {
            remove_empty_dirs(&e.path());
        }
    }
    let _ = std::fs::remove_dir(dir);
}

/// Untracked, non-ignored files under `dir` of the repository at `top`, as
/// `(path from top, path with `dir/` removed)`.
fn untracked_under(top: &Path, dir: &str) -> Result<Vec<(String, String)>> {
    let out = crate::branchfarm::git(
        top,
        &[
            "ls-files",
            "-z",
            "--others",
            "--exclude-standard",
            "--",
            dir,
        ],
    )?;
    let prefix = format!("{dir}/");
    Ok(out
        .split('\0')
        .filter(|p| !p.is_empty())
        .filter_map(|p| {
            p.strip_prefix(&prefix)
                .map(|to| (p.to_string(), to.to_string()))
        })
        .collect())
}

/// Commit everything under the farm worktree `top` as one commit, stamped with
/// the actor like every farm commit (`Yaks-Actor` trailer, and the identity
/// fallback for a machine with none). Returns the short hash.
fn commit_all(top: &Path, message: &str) -> Result<String> {
    let git = |args: &[&str]| -> Result<String> { crate::branchfarm::git(top, args) };
    git(&["add", "-A", "--", "."])?;
    let actor = crate::actor::resolve_detail(None)
        .map(|(n, _)| n)
        .unwrap_or_default();
    let ident = crate::autocommit::identity_env(top, &actor);
    let trailer = format!("{}: {actor}", crate::autocommit::ACTOR_TRAILER);
    // `--only` with the whole farm worktree: anything staged elsewhere in the
    // repository stays staged and out of this commit. (The refusal for a dirty
    // farm already makes the tree clean; this is the belt to that brace.)
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(top)
        .args([
            "commit",
            "-q",
            "--only",
            "-m",
            message,
            "-m",
            trailer.as_str(),
            "--",
            ".",
        ])
        .envs(ident.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .output()
        .context("running git commit")?;
    if !out.status.success() {
        anyhow::bail!(
            "git commit failed ({}); the farm is migrated and staged, not committed: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    git(&["rev-parse", "--short", "HEAD"])
}

/// Rename each source directory to its slugged name, in place. The caller has
/// checked [`Schema9::blocked`]; a destination that already exists is an error
/// (nothing is overwritten).
fn apply_source_renames(root: &Path, plan: &[(String, String)]) -> Result<()> {
    for (old, new) in plan {
        let (from, to) = (root.join(old), root.join(new));
        if std::fs::symlink_metadata(&to).is_ok() {
            anyhow::bail!("cannot rename {old} to {new}: {new} already exists");
        }
        std::fs::rename(&from, &to).with_context(|| format!("renaming {old} to {new}"))?;
    }
    Ok(())
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

    fn cx<'a>(
        ids: &'a HashSet<String>,
        from: crate::links::From<'a>,
        own: Option<&'a str>,
        sources: &'a [(String, String)],
    ) -> LinkCx<'a> {
        LinkCx {
            ids,
            from,
            own,
            sources,
        }
    }

    fn links(text: &str, from: crate::links::From, own: Option<&str>) -> (String, LinkEdits) {
        let ids = ctx(&["yaks-1a2b", "yaks-3c4d", "other-0001"]);
        let plan = vec![(
            "sources/2026-11-03-81364278".to_string(),
            "sources/2026-11-03-81364278-a-title".to_string(),
        )];
        canonical_links(&cx(&ids, from, own, &plan), text)
    }

    #[test]
    fn a_bare_yak_id_becomes_the_path_a_host_can_follow() {
        use crate::links::From as At;
        let (out, e) = links(
            "see [a](yaks-1a2b) and [n](yaks-1a2b#2026-11-04T22:13:03Z) and [o](other-0001)",
            At::Yak,
            Some("yaks-3c4d"),
        );
        assert_eq!(
            out,
            "see [a](/yaks/yaks/yaks-1a2b.md) and \
             [n](/yaks/yaks/yaks-1a2b.md#2026-11-04t221303z) and [o](/other/yaks/other-0001.md)"
        );
        assert_eq!((e.yak, e.total()), (3, 3));
        // A `-1` suffix (a second note in one second) keeps its meaning.
        let (out, _) = links("[n](yaks-1a2b#2026-11-04T22:13:03Z-2)", At::Yak, None);
        assert_eq!(out, "[n](/yaks/yaks/yaks-1a2b.md#2026-11-04t221303z-2)");
        // The same from a dok, which sits in another directory.
        let (out, _) = links("[a](yaks-1a2b)", At::Dok("doks/topics/x.md"), None);
        assert_eq!(out, "[a](/yaks/yaks/yaks-1a2b.md)");
    }

    #[test]
    fn only_real_yak_ids_are_touched() {
        use crate::links::From as At;
        for text in [
            "[x](yaks-9999)", // not a yak of this farm
            "[x](https://x.test/yaks-1a2b)",
            "[x](code:src/yaks-1a2b.rs)",
            "[x](#yaks-1a2b)",
            "`[x](yaks-1a2b)` in a code span",
            "```\n[x](yaks-1a2b)\n```",
        ] {
            assert_eq!(
                links(text, At::Yak, None),
                (text.to_string(), LinkEdits::default()),
                "{text}"
            );
        }
    }

    #[test]
    fn an_attachment_link_becomes_the_form_that_works_from_where_it_sits() {
        use crate::links::From as At;
        // In a yak of the same herd the file sits beside its directory: left alone.
        let (out, e) = links("![s](yaks-1a2b/shot.png)", At::Yak, Some("yaks-3c4d"));
        assert_eq!((out.as_str(), e.total()), ("![s](yaks-1a2b/shot.png)", 0));
        // In another herd's yak, in a dok, or in a session: the path from the root.
        let (out, e) = links("![s](yaks-1a2b/shot.png)", At::Yak, Some("other-0001"));
        assert_eq!(out, "![s](/yaks/yaks/yaks-1a2b/shot.png)");
        assert_eq!(e.attachment, 1);
        let (out, _) = links("[d](yaks-1a2b/log.txt)", At::Dok("doks/x.md"), None);
        assert_eq!(out, "[d](/yaks/yaks/yaks-1a2b/log.txt)");
        // Inside the attachment's own yak, a sibling is the bare name, and
        // the `<id>/<file>` form is shortened to it.
        let (out, _) = links(
            "![s](yaks-1a2b/shot.png)",
            At::Attachment("yaks-1a2b"),
            None,
        );
        assert_eq!(out, "![s](shot.png)");
        let (out, e) = links("![s](shot.png)", At::Attachment("yaks-1a2b"), None);
        assert_eq!((out.as_str(), e.total()), ("![s](shot.png)", 0));
        // A path already from the root is left alone, even where a shorter
        // form would do.
        let (out, e) = links(
            "![s](/yaks/yaks/yaks-1a2b/shot.png)",
            At::Yak,
            Some("yaks-3c4d"),
        );
        assert_eq!(
            (out.as_str(), e.total()),
            ("![s](/yaks/yaks/yaks-1a2b/shot.png)", 0)
        );
        // In another yak's attachment, an attachment of a third yak is a path.
        let (out, _) = links("![s](yaks-3c4d/a.png)", At::Attachment("yaks-1a2b"), None);
        assert_eq!(out, "![s](/yaks/yaks/yaks-3c4d/a.png)");
        // An encoded name stays encoded; an anchor is kept.
        let (out, _) = links("[d](yaks-1a2b/my%20doc.md#top)", At::Dok("doks/x.md"), None);
        assert_eq!(out, "[d](/yaks/yaks/yaks-1a2b/my%20doc.md#top)");
    }

    #[test]
    fn sessions_and_sources_links_name_the_real_path_and_the_slugged_directory() {
        use crate::links::From as At;
        let (out, e) = links(
            "[s](/doks/sessions/2026/11/x.md) [src](/doks/sources/2026-11-03-81364278) \
             [ok](/sources/2026-11-03-81364278/source.md#top) [pin](/sources/2026-11-03-81364278/source.md@3f2a9c1)",
            At::Dok("doks/x.md"),
            None,
        );
        assert_eq!(
            out,
            "[s](/sessions/2026/11/x.md) [src](/sources/2026-11-03-81364278-a-title) \
             [ok](/sources/2026-11-03-81364278-a-title/source.md#top) \
             [pin](/sources/2026-11-03-81364278/source.md@3f2a9c1)"
        );
        assert_eq!((e.namespace, e.source), (2, 2));
        // A `/doks/...` path that is not a session or a source is a dok.
        let (out, e) = links("[d](/doks/sessionsx/a.md)", At::Dok("doks/x.md"), None);
        assert_eq!((out.as_str(), e.total()), ("[d](/doks/sessionsx/a.md)", 0));
    }

    #[test]
    fn the_link_step_is_idempotent() {
        use crate::links::From as At;
        let text = "[a](yaks-1a2b) [n](yaks-1a2b#2026-11-04T22:13:03Z) ![s](yaks-3c4d/a.png) \
                    [src](/doks/sources/2026-11-03-81364278)";
        let (once, e) = links(text, At::Dok("doks/x.md"), None);
        // Four links, five edits: the last one needs its `doks/` dropped and
        // its source directory slugged, and each is counted.
        assert_eq!((e.yak, e.attachment, e.namespace, e.source), (2, 1, 1, 1));
        let (twice, e2) = links(&once, At::Dok("doks/x.md"), None);
        assert_eq!((twice, e2.total()), (once, 0));
    }

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
    fn a_schema_7_farm_gets_the_wiki_move_and_goes_on_to_the_current_schema_once() {
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
        // A farm outside git has no branch to move: it stays where it is and
        // gets the marker of the schema this build writes.
        assert!(!r.s9.on_branch && r.s9.commit.is_none());
        assert_eq!(
            std::fs::read_to_string(root.join("schema")).unwrap(),
            format!("{}\n", store::SCHEMA)
        );
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

    /// A schema-8 farm holding one of each thing the schema-9 stage touches:
    /// two yaks (one linking the other by bare id and by note, one with an
    /// evidence document), a dok, a session, a source, and a yak whose note
    /// header is not a timestamp.
    fn schema_8_farm(tag: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("yaks-mig-s9-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let w = |rel: &str, text: &str| {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        };
        let fm = |id: &str, body: &str| {
            format!(
                "---\nid: {id}\ntitle: T\nstatus: hairy\ntype: task\npriority: 3\ncreated: \"2026-01-01T00:00:00Z\"\nupdated: \"2026-01-01T00:00:00Z\"\n---\n\n{body}"
            )
        };
        w("config.yaml", "herd: yak\n");
        w("schema", "8\n");
        w(
            "yak/yaks/yak-0001.md",
            &fm(
                "yak-0001",
                "See [two](yak-0002), [its note](yak-0002#2026-01-02T00:00:00Z) and \
                 [shot](yak-0002/a.png) and [doc](yak-0002/ev.md).\n\n\
                 [code](code:src/a.rs#L1) [url](https://x.test/yak-0002)\n\n---\n\
                 \u{25b8} 2026-01-01T00:00:00Z [a]\n\nhello\n",
            ),
        );
        w(
            "yak/yaks/yak-0002.md",
            &fm(
                "yak-0002",
                "Two.\n\n---\n\u{25b8} 2026-01-02T00:00:00Z [b]\nkind: move\n---\n\nmoved\n",
            ),
        );
        w("yak/yaks/yak-0002/a.png", "png");
        w(
            "yak/yaks/yak-0002/ev.md",
            "Evidence: ![a](a.png) and [back](yak-0001) and [other](yak-0002/a.png).\n",
        );
        w(
            "yak/yaks/yak-0003.md",
            &fm(
                "yak-0003",
                "Three.\n\n---\n\u{25b8} 2026-08-12 E2E VERIFIED (by hand)\n\ndone\n",
            ),
        );
        w(
            "doks/topics/a.md",
            "---\ntype: topic\ntitle: A\n---\n\nSee [y](yak-0001), [s](/doks/sessions/2026/11/s.md) \
             and [src](/doks/sources/2026-01-01-abcd1234) and [att](yak-0002/a.png).\n",
        );
        w(
            "sessions/2026/11/s.md",
            "---\ntype: session-review\n---\n\nRef [y](yak-0002#2026-01-02T00:00:00Z).\n",
        );
        w(
            "sources/2026-01-01-abcd1234/source.md",
            "---\ntype: source\ntitle: A Saved Page\n---\n",
        );
        w(
            "sources/2026-01-01-abcd1234/extracted.md",
            "A page linking [x](yak-0001): a snapshot, never rewritten.\n",
        );
        root
    }

    #[test]
    fn the_schema_9_plan_finds_every_kind_and_writes_nothing() {
        let root = schema_8_farm("plan");
        let before = std::fs::read_to_string(root.join("yak/yaks/yak-0001.md")).unwrap();
        let (writes, s9) = plan_schema9(&root).unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("yak/yaks/yak-0001.md")).unwrap(),
            before,
            "planning writes nothing"
        );
        // yak-0001: 2 yak links, one note anchor slugged, 1 cross-yak attachment
        // from the same herd (left alone) and a document attachment (left alone);
        // code and the URL untouched.
        let text = |p: &str| {
            writes
                .iter()
                .find(|(path, _)| path.ends_with(p))
                .map(|(_, t)| t.clone())
                .unwrap_or_else(|| panic!("{p} not rewritten"))
        };
        let y1 = text("yak-0001.md");
        assert!(y1.contains("[two](/yak/yaks/yak-0002.md)"), "{y1}");
        assert!(
            y1.contains("[its note](/yak/yaks/yak-0002.md#2026-01-02t000000z)"),
            "{y1}"
        );
        assert!(y1.contains("[shot](yak-0002/a.png)"), "same herd: kept");
        assert!(y1.contains("[code](code:src/a.rs#L1) [url](https://x.test/yak-0002)"));
        assert!(
            y1.contains("\n#### 2026-01-01T00:00:00Z\n\n*a*\n\nhello"),
            "notes converted: {y1}"
        );
        // yak-0002's note carries meta.
        assert!(
            text("yak-0002.md").contains("#### 2026-01-02T00:00:00Z\n\n*b \u{b7} kind: move*\n")
        );
        // The evidence document: a sibling of yak-0002's attachments.
        let ev = text("yak-0002/ev.md");
        assert!(ev.contains("[back](/yak/yaks/yak-0001.md)"), "{ev}");
        assert!(ev.contains("[other](a.png)"), "{ev}");
        // A dok, a session.
        let dok = text("doks/topics/a.md");
        assert!(dok.contains("[y](/yak/yaks/yak-0001.md)"), "{dok}");
        assert!(dok.contains("[s](/sessions/2026/11/s.md)"), "{dok}");
        assert!(
            dok.contains("[src](/sources/2026-01-01-abcd1234-a-saved-page)"),
            "{dok}"
        );
        assert!(dok.contains("[att](/yak/yaks/yak-0002/a.png)"), "{dok}");
        assert!(
            text("sessions/2026/11/s.md").contains("(/yak/yaks/yak-0002.md#2026-01-02t000000z)")
        );
        // The hand-headed yak is left alone, and reported.
        assert!(!writes.iter().any(|(p, _)| p.ends_with("yak-0003.md")));
        assert_eq!(s9.old_shape, ["yak-0003".to_string()]);
        // A source is a snapshot: not in the plan, but renamed.
        assert!(
            !writes
                .iter()
                .any(|(p, _)| p.to_string_lossy().contains("sources/"))
        );
        assert_eq!(
            s9.sources,
            [(
                "sources/2026-01-01-abcd1234".to_string(),
                "sources/2026-01-01-abcd1234-a-saved-page".to_string()
            )]
        );
        assert_eq!((s9.notes, s9.note_yaks), (2, 2));
        assert!(s9.blocked.is_empty() && s9.plan_blocked.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// A code repository with a farm branch under `.yaks/` (the schema-8
    /// layout), as the farm worktree would be. Returns `(code, farm worktree)`.
    pub(super) fn farm_branch_at_8_pub(tag: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        farm_branch_at_8(tag)
    }

    fn farm_branch_at_8(tag: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let base = std::env::temp_dir().join(format!("yaks-mig-git-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let code = base.join("code");
        std::fs::create_dir_all(&code).unwrap();
        git(&code, &["init", "-q", "-b", "main"]);
        std::fs::write(code.join("a.txt"), "a\n").unwrap();
        git(&code, &["add", "."]);
        git(&code, &["commit", "-q", "-m", "code"]);
        let wt = base.join("farm");
        git(
            &code,
            &[
                "worktree",
                "add",
                "-q",
                "--orphan",
                "-b",
                "yaks",
                &wt.display().to_string(),
            ],
        );
        // The same farm `schema_8_farm` makes, under `.yaks/`.
        let scratch = schema_8_farm(&format!("git-{tag}"));
        copy_dir(&scratch, &wt.join(".yaks"));
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::write(
            wt.join(".gitattributes"),
            ".yaks/**/*.md merge=yaks\n\n# yaks lfs: binary originals under .yaks/ are stored in Git LFS\n\
             .yaks/**/*.[pP][nN][gG] filter=lfs diff=lfs merge=lfs -text\n",
        )
        .unwrap();
        std::fs::write(wt.join(".yaks/.gitignore"), ".lock\n").unwrap();
        git(&wt, &["add", "-A"]);
        git(&wt, &["commit", "-q", "-m", "seed"]);
        (code, wt)
    }

    fn copy_dir(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for e in std::fs::read_dir(from).unwrap().flatten() {
            let (p, q) = (e.path(), to.join(e.file_name()));
            if p.is_dir() {
                copy_dir(&p, &q);
            } else {
                std::fs::copy(&p, &q).unwrap();
            }
        }
    }

    #[test]
    fn a_farm_branch_moves_to_the_branch_root_in_two_commits_a_move_and_a_rewrite() {
        let (code, wt) = farm_branch_at_8("move");
        let root = wt.join(".yaks");
        let commits_before = git(&wt, &["rev-list", "--count", "HEAD"])
            .trim()
            .to_string();
        let tracked_before: Vec<String> = git(&wt, &["ls-files"])
            .lines()
            .filter(|l| l.starts_with(".yaks/"))
            .map(String::from)
            .collect();

        // A dry run says what it would do and writes nothing.
        let dry = run(&root, true).unwrap();
        assert!(dry.s9.on_branch && dry.s9.move_to_root, "{dry:?}");
        assert_eq!(dry.s9.files_to_move, tracked_before.len());
        assert!(dry.s9.blocked.is_empty(), "{:?}", dry.s9.blocked);
        assert!(root.join("yak/yaks/yak-0001.md").exists());
        assert_eq!(std::fs::read_to_string(root.join("schema")).unwrap(), "8\n");

        let done = run(&root, false).unwrap();
        assert!(done.applied);
        let (mv, rewrite) = (
            done.s9.move_commit.clone().expect("the move commit"),
            done.s9.commit.clone().expect("the rewrite commit"),
        );
        assert_eq!(
            git(&wt, &["rev-list", "--count", "HEAD"])
                .trim()
                .parse::<usize>()
                .unwrap(),
            commits_before.parse::<usize>().unwrap() + 2,
            "exactly two commits"
        );
        assert_eq!(git(&wt, &["rev-parse", "--short", "HEAD"]).trim(), rewrite);
        assert_eq!(git(&wt, &["rev-parse", "--short", "HEAD~1"]).trim(), mv);
        for sha in [&mv, &rewrite] {
            assert!(git(&wt, &["log", "-1", "--format=%B", sha]).contains("Yaks-Actor:"));
        }
        // The move changes no bytes: every farm file in it is a 100% rename
        // (the new files, the README and the stub, are additions).
        let status = git(&wt, &["show", "-M", "--name-status", "--format=", &mv]);
        for l in status.lines() {
            let kind = l.split('\t').next().unwrap();
            assert!(
                kind == "R100" || kind == "A" || kind == "M" || kind == "D",
                "{l}"
            );
        }
        let r100 = status.lines().filter(|l| l.starts_with("R100")).count();
        assert!(r100 >= tracked_before.len() - 3, "{status}");
        assert!(
            !status
                .lines()
                .any(|l| l.starts_with('R') && !l.starts_with("R100")),
            "a rename git is unsure of would break --follow: {status}"
        );
        // The rewrite is modifications only, at the new paths.
        let rw = git(&wt, &["show", "-M", "--name-status", "--format=", &rewrite]);
        assert!(rw.lines().all(|l| l.starts_with('M')), "{rw}");
        assert!(
            rw.contains("yak/yaks/yak-0001.md") && rw.contains("\tschema"),
            "{rw}"
        );
        assert!(
            git(&wt, &["status", "--porcelain"]).trim().is_empty(),
            "clean after"
        );

        // The farm is at the root; `.yaks/` holds only the stub.
        for f in [
            "config.yaml",
            "yak/yaks/yak-0001.md",
            "doks/topics/a.md",
            "README.md",
        ] {
            assert!(wt.join(f).is_file(), "{f}");
        }
        assert!(!wt.join(".yaks/yak").exists() && !wt.join(".yaks/doks").exists());
        let stub: Vec<String> = git(&wt, &["ls-files", ".yaks"])
            .lines()
            .map(String::from)
            .collect();
        assert_eq!(stub, [".yaks/README.md", ".yaks/schema"]);
        assert_eq!(
            std::fs::read_to_string(wt.join(".yaks/schema"))
                .unwrap()
                .trim(),
            store::SCHEMA.to_string()
        );
        assert_eq!(
            std::fs::read_to_string(wt.join("schema")).unwrap().trim(),
            store::SCHEMA.to_string()
        );
        assert_eq!(
            std::fs::read_to_string(wt.join(".gitattributes")).unwrap(),
            "**/*.md merge=yaks\n\n# yaks lfs: binary originals in the farm are stored in Git LFS\n\
             **/*.[pP][nN][gG] filter=lfs diff=lfs merge=lfs -text\n"
        );
        assert_eq!(
            std::fs::read_to_string(wt.join(".gitignore")).unwrap(),
            ".lock\n"
        );
        // History follows a yak across both commits.
        assert!(
            git(
                &wt,
                &[
                    "log",
                    "--follow",
                    "--format=%h",
                    "--",
                    "yak/yaks/yak-0001.md"
                ]
            )
            .lines()
            .count()
                >= 3,
            "--follow reaches back past the move"
        );
        // The sources were renamed to their slugs; the page is a snapshot.
        assert!(
            wt.join("sources/2026-01-01-abcd1234-a-saved-page/source.md")
                .is_file()
        );
        assert!(!wt.join("sources/2026-01-01-abcd1234").exists());

        // The farm is found and read from the new root, and a second run is a no-op.
        assert_eq!(
            store::discover_with(&code, None)
                .unwrap()
                .root
                .canonicalize()
                .unwrap(),
            wt.canonicalize().unwrap()
        );
        assert!(run(&wt, false).unwrap().is_noop());
        let _ = std::fs::remove_dir_all(wt.parent().unwrap());
    }

    #[test]
    fn a_dirty_or_behind_farm_branch_refuses_before_writing_anything() {
        let (_code, wt) = farm_branch_at_8("refuse");
        let root = wt.join(".yaks");
        std::fs::write(
            root.join("yak/yaks/yak-0001.md"),
            "---\nid: yak-0001\n---\nedited\n",
        )
        .unwrap();
        let dry = run(&root, true).unwrap();
        assert!(
            dry.s9.blocked.iter().any(|b| b.contains("uncommitted")),
            "{:?}",
            dry.s9.blocked
        );
        let before = std::fs::read(root.join("yak/yaks/yak-0001.md")).unwrap();
        let err = run(&root, false).unwrap_err().to_string();
        assert!(err.contains("uncommitted"), "{err}");
        assert_eq!(
            std::fs::read(root.join("yak/yaks/yak-0001.md")).unwrap(),
            before
        );
        assert_eq!(std::fs::read_to_string(root.join("schema")).unwrap(), "8\n");
        let _ = std::fs::remove_dir_all(wt.parent().unwrap());
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
