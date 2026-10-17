//! Structured, navigable detail-pane model. Ported from Python `yaktui.detail`
//! + `yaklib.links`. `build` turns a task into display lines where the parent,
//! dependencies, children, source, and any task-id references in the body carry
//! link targets; `jumplist` flattens those targets in reading order for the
//! Tab/Enter jump navigation. No regex (hand-rolled id/URL scan).

use std::collections::{HashMap, HashSet};

use super::markdown::MdSpan;
use super::when::Clock;
use super::{content, mdrender, palette};
use crate::model::{Status, Task};
use crate::sheds::{Shed, YakChange};
use ratatui::style::{Color, Modifier, Style};

/// Where a link points.
#[derive(Clone, PartialEq, Debug)]
pub enum Target {
    Task(String),
    Url(String),
    /// A markdown image link `![alt](path)` — an attached artifact, opened
    /// externally (path is relative to the farm's `.yaks/` root).
    Artifact(String),
}

/// Styling hint for a detail line.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Field,
    Section,
    /// A field carrying a warning accent (e.g. the `needs` blocker), styled
    /// distinctly from a plain `Field` in `render_dline`.
    Warn,
    /// One line of the Sheds section (what another shed changed about the
    /// yak); its colours ride in `md`, like a body line's, but it is not body
    /// text (no content block, no link scan).
    Shed,
    Body,
    Empty,
}

/// One display line plus any link spans it carries (col + len in chars).
pub struct DLine {
    pub text: String,
    pub kind: Kind,
    pub links: Vec<(usize, usize, Target)>,
    /// Markdown highlight spans (char col + len) for body lines. Rendered
    /// beneath the link/find layers so a link inside emphasis still wins.
    pub md: Vec<MdSpan>,
    /// True when this line is a wrapped continuation of the previous one (set by
    /// [`wrap`]). Lets yank rejoin a soft-wrapped line back into one logical line.
    pub cont: bool,
    /// For a body row: where it came from, so yank can copy source and `E`
    /// can open the right block at the right line.
    pub src: Option<Src>,
}

/// A body row's origin: content block (0 = description, 1.. = comments) and
/// the inclusive 0-based line range within that block's text. A comment's
/// header row has `lines: None`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Src {
    pub block: usize,
    pub lines: Option<(usize, usize)>,
}

/// A single navigable target, located at a line + starting column.
pub struct Jump {
    pub line: usize,
    pub col: usize,
    pub target: Target,
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn field(label: &str, value: &str) -> DLine {
    DLine {
        text: format!("{label:<13}{value}"),
        kind: Kind::Field,
        links: vec![],
        md: vec![],
        cont: false,
        src: None,
    }
}

/// Like [`field`] but flagged [`Kind::Warn`] so the whole line reads as a
/// blocker accent (the `needs` block).
fn warn_field(label: &str, value: &str) -> DLine {
    DLine {
        text: format!("{label:<13}{value}"),
        kind: Kind::Warn,
        links: vec![],
        md: vec![],
        cont: false,
        src: None,
    }
}

/// A blank line (for callers outside this module).
pub fn blank() -> DLine {
    empty()
}

/// The banner on top of a read-only detail (`only in shed <name>`, yaks-9631):
/// a warning-accent line.
pub fn banner(text: &str) -> DLine {
    DLine {
        text: text.into(),
        kind: Kind::Warn,
        links: vec![],
        md: vec![],
        cont: false,
        src: None,
    }
}

fn section(text: &str) -> DLine {
    DLine {
        text: text.into(),
        kind: Kind::Section,
        links: vec![],
        md: vec![],
        cont: false,
        src: None,
    }
}

fn empty() -> DLine {
    DLine {
        text: String::new(),
        kind: Kind::Empty,
        links: vec![],
        md: vec![],
        cont: false,
        src: None,
    }
}

fn status_word(s: Status) -> &'static str {
    match s {
        Status::Hairy => "hairy",
        Status::Shaving => "shaving",
        Status::Shorn => "shorn",
        Status::Dead => "dead",
    }
}

fn opt_date(o: &Option<String>, clock: &Clock) -> String {
    match o {
        Some(s) => clock.format(s),
        None => "-".to_string(),
    }
}

/// A "glyph id  title" reference line whose id is a link target. `glyph` is a
/// status emoji (display width 2 but a single `char`); link offsets below are
/// char indices, which `render_dline` styles as relative span flow, so the
/// wide emoji doesn't skew the id highlight.
fn ref_line(prefix: String, glyph: &str, id: &str, title: &str, exists: bool) -> DLine {
    let head = format!("{prefix}{glyph} ");
    let col = head.chars().count();
    let text = format!("{head}{id}  {title}");
    let links = if exists {
        vec![(col, id.chars().count(), Target::Task(id.to_string()))]
    } else {
        vec![]
    };
    DLine {
        text,
        kind: Kind::Field,
        links,
        md: vec![],
        cont: false,
        src: None,
    }
}

/// The Sheds section: one line per shed that changed the yak, the TUI form of
/// `yaks show <id> --sheds`. The shed name, then the bracket words of `yaks
/// changes` (`hairy -> shaving`, `new, hairy`, `edited`), `needs: <v>` and
/// `+N notes`, then (dim) the newest note's actor and first line. Nothing at
/// all (no header) when no shed changed the yak.
fn sheds_section(sheds: &[(&Shed, &YakChange)]) -> Vec<DLine> {
    if sheds.is_empty() {
        return Vec::new();
    }
    let mut out = vec![empty(), section("Sheds:")];
    // Two or more sheds changed the same yak: a conflict to settle before
    // they land (yaks-9631).
    if sheds.len() >= 2 {
        out.push(DLine {
            text: format!("  \u{26a0} changed in {} sheds", sheds.len()),
            kind: Kind::Warn,
            links: vec![],
            md: vec![],
            cont: false,
            src: None,
        });
    }
    for (shed, y) in sheds {
        let mut what = match (y.added, y.moved) {
            (true, _) => format!("new, {}", y.status),
            (false, Some((from, to))) => format!("{from} -> {to}"),
            (false, None) => y.status.to_string(),
        };
        if y.edited {
            what.push_str(", edited");
        }
        let mut text = format!("  {}  ", shed.name());
        let mut md = Vec::new();
        // Append `piece` to the line, styled when `style` is given.
        let mut push = |text: &mut String, piece: String, style: Option<Style>| {
            if let Some(style) = style {
                md.push(MdSpan {
                    start: text.chars().count(),
                    len: piece.chars().count(),
                    style,
                });
            }
            text.push_str(&piece);
        };
        push(
            &mut text,
            format!("[{what}]"),
            Some(Style::new().fg(Color::Yellow)),
        );
        if let Some(v) = &y.needs {
            let warn = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
            push(&mut text, "  ".into(), None);
            push(&mut text, format!("needs: {v}"), Some(warn));
        }
        if !y.notes.is_empty() {
            push(&mut text, format!("  +{} notes", y.notes.len()), None);
        }
        if let Some(n) = y.notes.last() {
            let first = n
                .text
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .unwrap_or("");
            let dim = Style::new().add_modifier(Modifier::DIM);
            let actor = n.actor.as_deref().unwrap_or("-");
            push(&mut text, format!("  {actor}: {first}"), Some(dim));
        }
        out.push(DLine {
            text,
            kind: Kind::Shed,
            links: vec![],
            md,
            cont: false,
            src: None,
        });
    }
    out
}

/// [`build_at`] against the pinned test clock (see [`Clock::app_default`]).
#[cfg(test)]
pub fn build(task: &Task, all: &[Task], sheds: &[(&Shed, &YakChange)]) -> Vec<DLine> {
    build_at(task, all, sheds, &Clock::app_default(), 0)
}

/// `sheds` is what the other sheds changed about this yak (`Snapshot::copies`);
/// when non-empty it adds a Sheds section between the fields and the body.
/// `clock` renders the Created/Updated timestamps (relative to its "now", in
/// its timezone).
pub fn build_at(
    task: &Task,
    all: &[Task],
    sheds: &[(&Shed, &YakChange)],
    clock: &Clock,
    width: usize,
) -> Vec<DLine> {
    let by_id: HashMap<&str, &Task> = all.iter().map(|t| (t.id.as_str(), t)).collect();
    let known: HashSet<&str> = by_id.keys().copied().collect();

    let labels = if task.labels.is_empty() {
        "-".to_string()
    } else {
        task.labels.join(", ")
    };
    // Whether the farm spans more than one herd (id prefix). Mirrors the
    // App-level `is_multi_herd` present-prefix rule (a herd is a dashed id
    // prefix), so the Herd field appears exactly when the id colour / create
    // picker do — and stays absent (no noise) on a single-herd farm.
    let multi_herd = {
        let mut prefixes: HashSet<&str> = HashSet::new();
        for t in all {
            if let Some((p, _)) = t.id.split_once('-') {
                prefixes.insert(p);
            }
        }
        prefixes.len() > 1
    };
    let mut out = vec![
        section(&format!("Task: {}", task.id)),
        empty(),
        field("Title:", &task.title),
        field(
            "Status:",
            &format!("{} {}", task.status.emoji(), status_word(task.status)),
        ),
    ];
    // A `needs` block is state worth seeing next to Status (mirrors CLI `show`).
    // An open question carries a warning accent (Kind::Warn) so it reads as a
    // blocker; an answered one (`needs: agent`) is plain: it blocks nothing.
    if task.awaiting_agent() {
        out.push(field("Needs:", "agent (answered, awaiting pickup)"));
    } else if let Some(n) = &task.needs {
        out.push(warn_field("Needs:", n));
    }
    out.push(field("Type:", &task.kind));
    // On a multi-herd farm, surface the yak's herd (its id prefix) as a field
    // between Type and Priority, so it's visible and the `H` key's effect has
    // somewhere to land (yaks-71d1). A herd is `prefix-tail`; an id without a
    // `-` (test fixtures) names no herd.
    if multi_herd {
        if let Some((prefix, _)) = task.id.split_once('-') {
            out.push(field("Herd:", prefix));
        }
    }
    out.push(field("Priority:", &format!("p{}", task.priority)));
    out.push(field("Created:", &opt_date(&task.created, clock)));
    out.push(field("Updated:", &opt_date(&task.updated, clock)));
    out.push(field("Labels:", &labels));

    if !task.depends_on.is_empty() {
        out.push(empty());
        out.push(section("Depends on:"));
        for d in &task.depends_on {
            let (glyph, title, exists) = match by_id.get(d.as_str()) {
                Some(t) => (t.status.emoji(), t.title.clone(), true),
                None => (" ", "(missing)".into(), false),
            };
            out.push(ref_line("  ".into(), glyph, d, &title, exists));
        }
    }

    // Blocks: reverse dependencies -- tasks that depend on this one.
    let mut blocks: Vec<&Task> = all
        .iter()
        .filter(|t| t.depends_on.iter().any(|d| d.as_str() == task.id.as_str()))
        .collect();
    blocks.sort_by(|a, b| a.id.cmp(&b.id));
    if !blocks.is_empty() {
        out.push(empty());
        out.push(section("Blocks:"));
        for b in blocks {
            out.push(ref_line(
                "  ".into(),
                b.status.emoji(),
                &b.id,
                &b.title,
                true,
            ));
        }
    }

    if let Some(p) = &task.parent {
        let (glyph, title, exists) = match by_id.get(p.as_str()) {
            Some(t) => (t.status.emoji(), t.title.as_str(), true),
            None => (" ", "(missing)", false),
        };
        out.push(empty());
        out.push(section("Parent:"));
        out.push(ref_line("  ".into(), glyph, p, title, exists));
    }

    let mut kids: Vec<&Task> = all
        .iter()
        .filter(|c| c.parent.as_deref() == Some(task.id.as_str()))
        .collect();
    kids.sort_by(|a, b| a.id.cmp(&b.id));
    if !kids.is_empty() {
        out.push(empty());
        out.push(section("Children:"));
        for c in kids {
            out.push(ref_line(
                "  ".into(),
                c.status.emoji(),
                &c.id,
                &c.title,
                true,
            ));
        }
    }

    if let Some(s) = &task.source {
        let head = format!("{:<13}", "Source:");
        let col = head.chars().count();
        let links = if is_url(s) {
            vec![(col, s.chars().count(), Target::Url(s.clone()))]
        } else {
            vec![]
        };
        out.push(empty());
        out.push(DLine {
            text: format!("{head}{s}"),
            kind: Kind::Field,
            links,
            md: vec![],
            cont: false,
            src: None,
        });
    }

    // Unmodeled frontmatter (`task.extra`) kept verbatim on parse. Surfaced
    // read-only so hand-added/newer keys are visible instead of silently
    // invisible; the raw lines are echoed as-is (they are not label/value
    // pairs we own the formatting of).
    if !task.extra.is_empty() {
        out.push(empty());
        out.push(section("Other fields:"));
        for line in &task.extra {
            out.push(DLine {
                text: format!("  {line}"),
                kind: Kind::Field,
                links: vec![],
                md: vec![],
                cont: false,
                src: None,
            });
        }
    }

    out.extend(sheds_section(sheds));

    out.extend(body_rows(task, &known, clock, width));
    out
}

/// Width the body is laid out at before the first render has measured the
/// pane (`width == 0`).
const FALLBACK_WIDTH: usize = 80;

/// The body, rendered: the description and each comment parsed as its own
/// markdown document (`content::parse` first, so nothing leaks between them),
/// each comment under a header row. Rows are already wrapped to `width` and
/// carry their [`Src`], so [`wrap`] leaves them alone.
fn body_rows(task: &Task, known: &HashSet<&str>, clock: &Clock, width: usize) -> Vec<DLine> {
    let width = if width == 0 { FALLBACK_WIDTH } else { width };
    let mut out = Vec::new();
    let blocks = content::parse(&task.body);
    for (bi, block) in blocks.iter().enumerate() {
        let is_comment = matches!(block.kind, content::BlockKind::Comment { .. });
        if block.text.trim().is_empty() && !is_comment {
            continue;
        }
        out.push(empty());
        if is_comment {
            out.push(comment_header(&block.kind, clock, width, bi));
        }
        for row in mdrender::render(&block.text, width, known) {
            out.push(DLine {
                text: row.text,
                kind: Kind::Body,
                links: row.links,
                md: row.spans,
                cont: false,
                src: Some(Src {
                    block: bi,
                    lines: Some(row.lines),
                }),
            });
        }
    }
    out
}

/// The label of a comment block: `actor · when` (or just `when`). One design
/// for the detail pane's comment headers and the edit form's separators.
pub fn comment_label(kind: &content::BlockKind, clock: &Clock) -> String {
    match kind {
        content::BlockKind::Description => "description".into(),
        content::BlockKind::Comment { timestamp, actor } => {
            let when = clock.format(timestamp);
            match actor {
                Some(a) => format!("{a} · {when}"),
                None => when,
            }
        }
    }
}

/// A comment's header row in the detail pane: `▸ actor · when ────`.
fn comment_header(kind: &content::BlockKind, clock: &Clock, width: usize, block: usize) -> DLine {
    let head = format!("\u{25b8} {} ", comment_label(kind, clock));
    let head_len = head.chars().count();
    let dashes = width.saturating_sub(head.chars().map(char_disp).sum());
    let text = format!("{head}{}", palette::RULE_CHAR.to_string().repeat(dashes));
    DLine {
        md: vec![
            MdSpan {
                start: 0,
                len: head_len,
                style: palette::comment_header(true),
            },
            MdSpan {
                start: head_len,
                len: dashes,
                style: palette::rule(),
            },
        ],
        text,
        kind: Kind::Body,
        links: vec![],
        cont: false,
        src: Some(Src { block, lines: None }),
    }
}

/// For each display line, the content-block index it belongs to (0 =
/// description, 1.. = comments), or `None` for non-body lines, from the rows'
/// [`Src`]. Lets the detail pane map its line cursor to a content block for
/// block navigation and context-sensitive editing.
pub fn block_index_per_line(lines: &[DLine]) -> Vec<Option<usize>> {
    lines.iter().map(|l| l.src.map(|s| s.block)).collect()
}

/// Display width of a char (wide glyphs like status emoji count as 2).
fn char_disp(c: char) -> usize {
    use unicode_width::UnicodeWidthChar;
    c.width().unwrap_or(0)
}

/// Greedy word-wrap the char slice to `width` display columns, returning the
/// `[start, end)` char ranges of each visual row. Breaks at the last space that
/// fits (consuming it); hard-breaks a single word longer than `width`. Leading
/// whitespace of the first row is preserved.
fn wrap_ranges(chars: &[char], width: usize) -> Vec<(usize, usize)> {
    let n = chars.len();
    let mut rows = Vec::new();
    let mut start = 0;
    while start < n {
        let mut w = 0;
        let mut end = start;
        while end < n {
            let cw = char_disp(chars[end]);
            if w + cw > width && end > start {
                break;
            }
            w += cw;
            end += 1;
        }
        if end >= n {
            rows.push((start, n));
            break;
        }
        // `end` is the first char that didn't fit. Prefer a word break: the
        // rightmost space in (start, end].
        let mut brk = None;
        let mut k = end;
        while k > start {
            if chars[k] == ' ' {
                brk = Some(k);
                break;
            }
            k -= 1;
        }
        match brk {
            Some(sp) => {
                rows.push((start, sp)); // row excludes the break space
                start = sp + 1; // consume it
            }
            None => {
                rows.push((start, end)); // hard break mid-word
                start = end;
            }
        }
    }
    if rows.is_empty() {
        rows.push((0, 0));
    }
    rows
}

/// Soft-wrap detail lines to `width` display columns so that one `DLine` still
/// equals one screen row (the whole line-cursor / jumplist / find / scroll model
/// is row-indexed). Link `(col, len)` spans are remapped to their new per-row
/// columns; continuation rows are marked `cont` so yank can rejoin them.
/// `width == 0` (not yet rendered) is a no-op.
pub fn wrap(lines: Vec<DLine>, width: usize) -> Vec<DLine> {
    if width == 0 {
        return lines;
    }
    let mut out = Vec::new();
    for dl in lines {
        let chars: Vec<char> = dl.text.chars().collect();
        let total: usize = chars.iter().map(|&c| char_disp(c)).sum();
        // Rendered body rows are already laid out (or deliberately unwrapped:
        // code, tables).
        if total <= width || dl.src.is_some() {
            out.push(dl);
            continue;
        }
        let ranges = wrap_ranges(&chars, width);
        for (ri, (s, e)) in ranges.iter().enumerate() {
            let (s, e) = (*s, *e);
            let text: String = chars[s..e].iter().collect();
            // Keep the portion of each link that falls within this row.
            let links = dl
                .links
                .iter()
                .filter_map(|(c, l, t)| {
                    let ls = (*c).max(s);
                    let le = (*c + *l).min(e);
                    (ls < le).then(|| (ls - s, le - ls, t.clone()))
                })
                .collect();
            // Remap markdown spans onto this row the same way.
            let md = dl
                .md
                .iter()
                .filter_map(|sp| {
                    let ls = sp.start.max(s);
                    let le = (sp.start + sp.len).min(e);
                    (ls < le).then(|| MdSpan {
                        start: ls - s,
                        len: le - ls,
                        style: sp.style,
                    })
                })
                .collect();
            out.push(DLine {
                text,
                kind: dl.kind,
                links,
                md,
                cont: ri > 0,
                src: dl.src,
            });
        }
    }
    out
}

#[cfg(test)]
mod detail_tests {
    use super::*;

    fn t(id: &str, parent: Option<&str>, deps: &[&str]) -> Task {
        Task {
            id: id.into(),
            title: format!("title {id}"),
            kind: "task".into(),
            priority: 3,
            status: Status::Hairy,
            created: Some("2025-12-31T19:00:00Z".into()),
            updated: None,
            parent: parent.map(String::from),
            labels: vec![],
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            source: None,
            needs: None,
            verify: None,
            extra: Vec::new(),
            body: String::new(),
        }
    }

    #[test]
    fn header_fields_and_reverse_deps() {
        let all = vec![
            t("yak-0001", None, &[]),
            t("yak-0002", None, &["yak-0001"]), // 0002 depends on 0001
        ];
        let lines = build(&all[0], &all, &[]);
        let text: Vec<&str> = lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(text[0], "Task: yak-0001");
        assert!(text.iter().any(|l| l.starts_with("Status:")));
        assert!(
            text.iter()
                .any(|l| l.starts_with("Created:") && l.contains("Dec 31, 2025 19:00"))
        );
        assert!(text.iter().any(|l| *l == "Blocks:"));
        let jumps = jumplist(&lines);
        assert!(
            jumps
                .iter()
                .any(|j| j.target == Target::Task("yak-0002".into()))
        );
    }

    #[test]
    fn needs_block_renders_after_status_when_set() {
        let mut task = t("yak-0001", None, &[]);
        // Unset: no Needs line.
        let plain: Vec<String> = build(&task, &[task.clone()], &[])
            .iter()
            .map(|l| l.text.clone())
            .collect();
        assert!(!plain.iter().any(|l| l.starts_with("Needs:")));
        // Set: a Needs line appears, immediately after Status.
        task.needs = Some("human".into());
        let lines: Vec<String> = build(&task, &[task.clone()], &[])
            .iter()
            .map(|l| l.text.clone())
            .collect();
        let status_i = lines.iter().position(|l| l.starts_with("Status:")).unwrap();
        let needs_i = lines.iter().position(|l| l.starts_with("Needs:")).unwrap();
        assert_eq!(needs_i, status_i + 1);
        assert!(lines[needs_i].contains("human"));
        // The Needs line carries the warning accent, not a plain field.
        let dlines = build(&task, &[task.clone()], &[]);
        assert_eq!(dlines[needs_i].kind, Kind::Warn);
    }

    #[test]
    fn answered_needs_is_a_plain_field_not_a_warning() {
        let mut task = t("yak-0001", None, &[]);
        task.needs = Some("agent".into());
        let dlines = build(&task, &[task.clone()], &[]);
        let needs = dlines
            .iter()
            .find(|l| l.text.starts_with("Needs:"))
            .unwrap();
        assert!(needs.text.contains("answered"), "{}", needs.text);
        assert_eq!(needs.kind, Kind::Field);
    }

    #[test]
    fn herd_line_shows_only_on_a_multi_herd_farm() {
        // Single-herd farm (one dashed prefix): no Herd line — it would just
        // echo the id prefix and add noise (yaks-71d1).
        let solo = vec![t("web-0001", None, &[]), t("web-0002", None, &[])];
        let single: Vec<String> = build(&solo[0], &solo, &[])
            .iter()
            .map(|l| l.text.clone())
            .collect();
        assert!(!single.iter().any(|l| l.starts_with("Herd:")));
        // Multi-herd farm (two prefixes): a Herd line appears, between Type and
        // Priority, naming this yak's prefix.
        let mixed = vec![t("web-0001", None, &[]), t("api-0001", None, &[])];
        let lines: Vec<String> = build(&mixed[0], &mixed, &[])
            .iter()
            .map(|l| l.text.clone())
            .collect();
        let herd_i = lines.iter().position(|l| l.starts_with("Herd:")).unwrap();
        let type_i = lines.iter().position(|l| l.starts_with("Type:")).unwrap();
        let pri_i = lines
            .iter()
            .position(|l| l.starts_with("Priority:"))
            .unwrap();
        assert_eq!(herd_i, type_i + 1);
        assert_eq!(pri_i, herd_i + 1);
        assert!(lines[herd_i].contains("web"));
    }

    #[test]
    fn extra_renders_read_only_other_fields_section() {
        let mut task = t("yak-0001", None, &[]);
        // Empty: no section.
        let plain: Vec<String> = build(&task, &[task.clone()], &[])
            .iter()
            .map(|l| l.text.clone())
            .collect();
        assert!(!plain.iter().any(|l| l == "Other fields:"));
        // Non-empty: verbatim raw frontmatter lines under a section header.
        task.extra = vec![
            "assignee: alice".into(),
            "reviewers:".into(),
            "- bob".into(),
        ];
        let lines = build(&task, &[task.clone()], &[]);
        let text: Vec<String> = lines.iter().map(|l| l.text.clone()).collect();
        let sec = text
            .iter()
            .position(|l| l == "Other fields:")
            .expect("Other fields section present");
        assert_eq!(text[sec + 1], "  assignee: alice");
        assert_eq!(text[sec + 2], "  reviewers:");
        assert_eq!(text[sec + 3], "  - bob");
        // The section carries no link targets (read-only).
        assert!(lines[sec + 1].links.is_empty());
    }

    #[test]
    fn created_and_updated_use_the_clock() {
        let mut task = t("a", None, &[]);
        task.updated = Some("2026-09-10T11:00:00Z".into());
        task.created = Some("bogus".into());
        let lines: Vec<String> = build(&task, &[task.clone()], &[])
            .iter()
            .map(|l| l.text.clone())
            .collect();
        assert!(
            lines
                .iter()
                .any(|l| l.ends_with("1h ago") && l.contains("Updated:"))
        );
        assert!(
            lines
                .iter()
                .any(|l| l.contains("Created:") && l.ends_with("bogus"))
        );
    }
}

/// Flatten every link across the lines, in reading order.
pub fn jumplist(lines: &[DLine]) -> Vec<Jump> {
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let mut links: Vec<&(usize, usize, Target)> = l.links.iter().collect();
        links.sort_by_key(|(c, _, _)| *c);
        for (col, _len, t) in links {
            out.push(Jump {
                line: i,
                col: *col,
                target: t.clone(),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;

    fn task(id: &str, parent: Option<&str>, deps: &[&str], body: &str) -> Task {
        Task {
            id: id.into(),
            title: format!("title {id}"),
            kind: "task".into(),
            priority: 3,
            status: Status::Hairy,
            created: None,
            updated: None,
            parent: parent.map(String::from),
            labels: vec![],
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            source: None,
            needs: None,
            verify: None,
            extra: Vec::new(),
            body: body.into(),
        }
    }

    #[test]
    fn links_parent_deps_children_and_body_refs() {
        let all = vec![
            task(
                "yak-0001",
                None,
                &["yak-0002"],
                "see yak-0003 and [[yak-0002]] here",
            ),
            task("yak-0002", None, &[], ""),
            task("yak-0003", Some("yak-0001"), &[], ""), // child of 0001
        ];
        let lines = build(&all[0], &all, &[]);
        let jumps = jumplist(&lines);
        let targets: Vec<Target> = jumps.iter().map(|j| j.target.clone()).collect();
        // parent: none; dep yak-0002; child yak-0003; body refs yak-0003, yak-0002.
        assert_eq!(
            targets,
            vec![
                Target::Task("yak-0002".into()), // depends on
                Target::Task("yak-0003".into()), // children
                Target::Task("yak-0003".into()), // body ref
                Target::Task("yak-0002".into()), // body ref (brackets stripped)
            ]
        );
    }

    #[test]
    fn build_attaches_markdown_spans_to_body_lines() {
        let all = vec![task("yak-0001", None, &[], "# Heading\n\nplain line")];
        let lines = build(&all[0], &all, &[]);
        let body: Vec<&DLine> = lines.iter().filter(|l| l.kind == Kind::Body).collect();
        // The rendered heading hides its `# ` marker and carries one heading
        // span; the plain line carries none. Each row knows its source line.
        let heading = body.iter().find(|l| l.text == "Heading").unwrap();
        assert_eq!(heading.md.len(), 1);
        assert_eq!((heading.md[0].start, heading.md[0].len), (0, 7));
        assert_eq!(heading.src.unwrap().lines, Some((0, 0)));
        let plain = body.iter().find(|l| l.text == "plain line").unwrap();
        assert!(plain.md.is_empty());
        assert_eq!(plain.src.unwrap().lines, Some((2, 2)));
    }

    #[test]
    fn comments_render_under_a_header_with_their_block_index() {
        let body = crate::store::append_note("Desc.", "2026-09-10T11:00:00Z", Some("joel"), "a\nb");
        let all = vec![task("yak-0001", None, &[], &body)];
        let lines = build(&all[0], &all, &[]);
        let body: Vec<&DLine> = lines.iter().filter(|l| l.kind == Kind::Body).collect();
        let texts: Vec<&str> = body.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts[0], "Desc.");
        assert!(texts[1].starts_with("\u{25b8} joel · 1h ago "), "{texts:?}");
        // CommonMark: the comment's soft break joins its two lines.
        assert_eq!(texts[2], "a b");
        assert_eq!(
            body.iter()
                .map(|l| l.src.unwrap().block)
                .collect::<Vec<_>>(),
            vec![0, 1, 1]
        );
        // No raw `---` / `▸ <timestamp>` lines leak through.
        assert!(
            !texts
                .iter()
                .any(|t| *t == "---" || t.contains("2026-09-10T"))
        );
    }

    #[test]
    fn wrap_remaps_markdown_spans_onto_rows() {
        let dl = DLine {
            text: "aaaa `code here` bbbb".into(),
            kind: Kind::Body,
            links: vec![],
            md: vec![MdSpan {
                start: 5,
                len: 11,
                style: ratatui::style::Style::new(),
            }],
            cont: false,
            src: None,
        };
        // Width 10 splits the line; the code span (cols 5..16) must be clipped to
        // whatever falls on each row and re-based to that row's columns.
        let rows = wrap(vec![dl], 10);
        let covered: usize = rows.iter().flat_map(|r| &r.md).map(|s| s.len).sum();
        assert!(covered > 0 && covered <= 11);
        for r in &rows {
            for s in &r.md {
                assert!(s.start + s.len <= r.text.chars().count());
            }
        }
    }

    #[test]
    fn body_link_span_points_at_the_id() {
        let all = vec![task("yak-0001", None, &[], "x yak-0001 y")];
        // self-id still scans in body (resolution is the caller's concern here).
        let lines = build(&all[0], &all, &[]);
        let body = lines.last().unwrap();
        assert_eq!(body.text, "x yak-0001 y");
        assert_eq!(body.links.len(), 1);
        let (col, len, _) = &body.links[0];
        let slice: String = body.text.chars().skip(*col).take(*len).collect();
        assert_eq!(slice, "yak-0001");
    }

    #[test]
    fn wrap_splits_on_words_and_marks_continuations() {
        let dl = DLine {
            text: "aaaa bbbb cccc dddd".into(),
            kind: Kind::Body,
            links: vec![],
            md: vec![],
            cont: false,
            src: None,
        };
        let rows = wrap(vec![dl], 10);
        let texts: Vec<&str> = rows.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(texts, vec!["aaaa bbbb", "cccc dddd"]);
        assert!(!rows[0].cont);
        assert!(rows[1].cont);
    }

    #[test]
    fn wrap_remaps_link_columns_onto_the_row() {
        let dl = DLine {
            text: "see yak-0001 now".into(),
            kind: Kind::Body,
            links: vec![(4, 8, Target::Task("yak-0001".into()))],
            md: vec![],
            cont: false,
            src: None,
        };
        let rows = wrap(vec![dl], 8);
        let link_row = rows.iter().find(|r| !r.links.is_empty()).unwrap();
        assert_eq!(link_row.text, "yak-0001");
        let (col, len, _) = &link_row.links[0];
        assert_eq!((*col, *len), (0, 8));
    }

    #[test]
    fn wrap_preserves_indent_and_hard_breaks_long_words() {
        let dl = DLine {
            text: "  alpha beta gamma".into(),
            kind: Kind::Body,
            links: vec![],
            md: vec![],
            cont: false,
            src: None,
        };
        assert_eq!(wrap(vec![dl], 10)[0].text, "  alpha");

        let dl2 = DLine {
            text: "abcdefghij".into(),
            kind: Kind::Body,
            links: vec![],
            md: vec![],
            cont: false,
            src: None,
        };
        let texts: Vec<String> = wrap(vec![dl2], 4).iter().map(|r| r.text.clone()).collect();
        assert_eq!(texts, vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn wrap_zero_width_is_noop() {
        let dl = DLine {
            text: "long line here".into(),
            kind: Kind::Body,
            links: vec![],
            md: vec![],
            cont: false,
            src: None,
        };
        assert_eq!(wrap(vec![dl], 0).len(), 1);
    }

    #[test]
    fn url_in_body_is_a_link() {
        let all = vec![task(
            "yak-0001",
            None,
            &[],
            "docs at https://example.com/x ok",
        )];
        let lines = build(&all[0], &all, &[]);
        let jumps = jumplist(&lines);
        assert_eq!(
            jumps.iter().map(|j| j.target.clone()).collect::<Vec<_>>(),
            vec![Target::Url("https://example.com/x".into())]
        );
    }
}
