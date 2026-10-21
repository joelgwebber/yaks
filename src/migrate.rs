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
}

/// The fixups of schema 5, in the order they run.
pub const FIXUPS: &[Fixup] = &[
    Fixup {
        name: "comment-header-blank-line",
        unit: "comment(s)",
        apply: comment_header_blank_line,
    },
    Fixup {
        name: "attachment-links",
        unit: "link(s)",
        apply: attachment_links,
    },
];

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
                "{id}'s herd `{herd}` is a reserved name (it would collide with the farm's docs \
                 directory); rename the herd first (`yaks rename-herd {herd} <new>`) with yaks 0.0.21"
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
    /// Yak files rewritten (or that would be): the union over the fixups.
    pub files: usize,
    /// Whether anything under the farm is tracked by git (so the result
    /// should be committed); false for a private or pointer farm.
    pub git_tracked: bool,
}

impl Report {
    /// Nothing changed and nothing would: the farm is current.
    pub fn is_noop(&self) -> bool {
        self.files == 0 && self.layout.is_empty() && self.from.is_none_or(|v| v == store::SCHEMA)
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
    if !dry_run {
        apply_layout(&moves)?;
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
        let front = &text[..text.len() - body.len()];
        let mut body = body.to_string();
        for (f, report) in FIXUPS.iter().zip(&mut fixups) {
            let (next, edits) = (f.apply)(&ctx, &body);
            if edits > 0 {
                report.yaks.push((id.to_string(), edits));
                body = next;
            }
        }
        let new = format!("{front}{body}");
        if new != text {
            writes.push((path, new));
        }
    }
    let report = Report {
        applied: !dry_run,
        from,
        layout,
        fixups,
        files: writes.len(),
        git_tracked: tracked_by_git(root),
    };
    if dry_run || report.is_noop() {
        return Ok(report);
    }
    for (path, text) in &writes {
        store::write::atomic_write(path, text)?;
    }
    let schema = root.join("schema");
    store::write::atomic_write(&schema, &format!("{}\n", store::SCHEMA))?;
    Ok(report)
}

/// Whether git tracks anything under the farm at `root` (a team farm). False
/// for a private or pointer farm, or one outside a repository.
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
}
