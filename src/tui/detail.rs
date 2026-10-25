//! Structured, navigable detail-pane model. Ported from Python `yaktui.detail`
//! + `yaklib.links`. `build` turns a task into display lines where the parent,
//! dependencies, children, source, and any task-id references in the body carry
//! link targets; `jumplist` flattens those targets in reading order for the
//! Tab/Enter jump navigation. No regex (hand-rolled id/URL scan).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::markdown::MdSpan;
use super::when::Clock;
use super::{content, mdrender, palette};
use crate::model::{Status, Task};
use ratatui::style::Modifier;

/// Where a link points.
#[derive(Clone, PartialEq, Debug)]
pub enum Target {
    Task(String),
    Url(String),
    /// A markdown image link `![alt](path)` — an attached artifact, opened
    /// externally (path is relative to the farm's `.yaks/` root).
    Artifact(String),
    /// A link to a repo source file (`../src/foo.rs#L12-L40`, optionally
    /// pinned `../blob/<sha>/…`), see [`super::source`]. `path` is relative
    /// to the repo root; `lines` is 1-based inclusive.
    Source {
        path: String,
        lines: Option<(u32, u32)>,
        rev: Option<String>,
    },
    /// A footnote reference `[n]` (the number it renders as): following it
    /// moves the line cursor to the definition at the end of the same block.
    Footnote(String),
}

/// Styling hint for a detail line.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Field,
    Section,
    /// A field carrying a warning accent (e.g. the `needs` blocker), styled
    /// distinctly from a plain `Field` in `render_dline`.
    Warn,
    Body,
    Empty,
    /// One row of the in-place block editor's region (yaks-597d): the row
    /// model reserves the rows, `render_detail` draws the editor over them.
    Editor,
}

/// One display line plus any link spans it carries (col + len in chars).
#[derive(PartialEq, Debug)]
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
    /// For a row of a no-wrap block (code, table): which block, and the gutter
    /// that stays put when the block scrolls sideways (yaks-c733). Such a row
    /// may be wider than the pane; `render_dline` clips it.
    pub nowrap: Option<mdrender::NoWrap>,
    /// For one of the rows an inline image reserves (yaks-0bf0): which image
    /// and which of its rows this is. The row's text is blank; `render_detail`
    /// draws the image over the whole run of rows.
    pub image: Option<ImageSlot>,
}

/// One row of the block an inline image reserves in the detail pane.
#[derive(Clone, PartialEq, Debug)]
pub struct ImageSlot {
    /// The image file on disk.
    pub path: PathBuf,
    /// The image's size in cells.
    pub cols: u16,
    pub rows: u16,
    /// This row's index within the image's `rows` (0 = its first row).
    pub row: u16,
}

/// What the layout needs to reserve rows for inline images: where attachments
/// live, the terminal's cell size in pixels, and the tallest an image without
/// an explicit size may be. `None` (no farm, or `inline_images: false`) keeps
/// every image a `🖼 alt` link row.
#[derive(Clone, Copy)]
pub struct ImageEnv<'a> {
    pub root: &'a Path,
    pub cell_px: (u16, u16),
    pub max_rows: u16,
}

impl DLine {
    /// Identity of the no-wrap block this row belongs to: (content block,
    /// block id within it). Rows sharing a key scroll together.
    pub fn nowrap_key(&self) -> Option<(usize, usize)> {
        Some((self.src?.block, self.nowrap?.id))
    }
}

/// A body row's origin: content block (0 = description, 1.. = comments) and
/// the inclusive 0-based line range within that block's text. A comment's
/// header row has `lines: None`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Src {
    pub block: usize,
    pub lines: Option<(usize, usize)>,
}

/// The widest row (display columns, prefix included) of each no-wrap block,
/// keyed by [`DLine::nowrap_key`]. A block scrolls sideways at most this much
/// minus the pane width.
pub fn nowrap_extents(lines: &[DLine]) -> std::collections::HashMap<(usize, usize), usize> {
    let mut out = std::collections::HashMap::new();
    for l in lines {
        if let Some(key) = l.nowrap_key() {
            let w = l.text.chars().map(char_disp).sum();
            let e: &mut usize = out.entry(key).or_default();
            *e = (*e).max(w);
        }
    }
    out
}

/// A content block being edited in place: its rows in the detail model are
/// replaced by a header and `rows` editor rows (`Kind::Editor`), so scroll,
/// cursor and hit arithmetic see the editor as N ordinary rows.
#[derive(Clone, PartialEq, Debug)]
pub struct EditSlot {
    /// The content block being edited; `None` = a new comment after the last.
    pub block: Option<usize>,
    /// Rows the editor occupies (below the header).
    pub rows: usize,
    /// Header label of a new comment (`you · now`); an existing block keeps
    /// its own.
    pub label: String,
    /// Candidate hints for the header, longest first; the first that fits the
    /// pane is drawn.
    pub hints: Vec<String>,
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
        nowrap: None,
        image: None,
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
        nowrap: None,
        image: None,
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
        nowrap: None,
        image: None,
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
        nowrap: None,
        image: None,
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
        nowrap: None,
        image: None,
    }
}

/// [`build_at`] against the pinned test clock (see [`Clock::app_default`]).
#[cfg(test)]
pub fn build(task: &Task, all: &[Task]) -> Vec<DLine> {
    build_at(task, all, &Clock::app_default(), 0, None, None)
}

/// `clock` renders the Created/Updated timestamps (relative to its "now", in
/// its timezone).
pub fn build_at(
    task: &Task,
    all: &[Task],
    clock: &Clock,
    width: usize,
    edit: Option<&EditSlot>,
    images: Option<&ImageEnv>,
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
            nowrap: None,
            image: None,
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
                nowrap: None,
                image: None,
            });
        }
    }

    out.extend(body_rows(task, &known, clock, width, edit, images));
    out
}

/// Width the body is laid out at before the first render has measured the
/// pane (`width == 0`).
const FALLBACK_WIDTH: usize = 80;

/// The body, rendered: the description and each comment parsed as its own
/// markdown document (`content::parse` first, so nothing leaks between them),
/// each comment under a header row. Rows are already wrapped to `width` and
/// carry their [`Src`], so [`wrap`] leaves them alone.
fn body_rows(
    task: &Task,
    known: &HashSet<&str>,
    clock: &Clock,
    width: usize,
    edit: Option<&EditSlot>,
    images: Option<&ImageEnv>,
) -> Vec<DLine> {
    let width = if width == 0 { FALLBACK_WIDTH } else { width };
    let mut out = Vec::new();
    let blocks = content::parse(&task.body);
    for (bi, block) in blocks.iter().enumerate() {
        let is_comment = matches!(block.kind, content::BlockKind::Comment { .. });
        let slot = edit.filter(|e| e.block == Some(bi));
        if block.text.trim().is_empty() && !is_comment && slot.is_none() {
            continue;
        }
        out.push(empty());
        if let Some(slot) = slot {
            // The block is being edited: its rendered rows give way to the
            // editor's, under a header (the description gets one too, to
            // carry the hint).
            out.push(header_row(
                &comment_label(&block.kind, clock),
                width,
                bi,
                Some(slot),
            ));
            out.extend(editor_rows(slot.rows));
            continue;
        }
        if is_comment {
            out.push(comment_header(&block.kind, clock, width, bi));
        }
        // An image that is a whole paragraph reserves rows after the
        // paragraph's last row; `pending` holds it until that row has gone out.
        let mut pending: Option<(Src, ImageSlot)> = None;
        for row in mdrender::render(&block.text, width, known) {
            // The blank row after the paragraph (a lone image has no container,
            // so its rows are never blank) ends it: the image goes here.
            if row.text.trim().is_empty() {
                if let Some(p) = pending.take() {
                    out.extend(image_rows(p));
                }
            }
            let src = Src {
                block: bi,
                lines: Some(row.lines),
            };
            if pending.is_none() {
                pending = images
                    .and_then(|env| {
                        row.images
                            .iter()
                            .find_map(|img| image_slot(env, img, width))
                    })
                    .map(|slot| (src, slot));
            }
            out.push(DLine {
                text: row.text,
                kind: Kind::Body,
                links: row.links,
                md: row.spans,
                cont: false,
                src: Some(src),
                nowrap: row.nowrap,
                image: None,
            });
        }
        if let Some(p) = pending.take() {
            out.extend(image_rows(p));
        }
    }
    // A new comment: a provisional header and the editor after the last block.
    if let Some(slot) = edit.filter(|e| e.block.is_none()) {
        out.push(empty());
        out.push(header_row(&slot.label, width, blocks.len(), Some(slot)));
        out.extend(editor_rows(slot.rows));
    }
    out
}

/// The image of `img` if it is drawn in place: it is a whole top-level
/// paragraph, a local attachment (`<id>/<file>`) whose header an image decoder
/// can read, sized by [`imagesize::cells`] from that header alone (no decode,
/// so laying out a yak never waits on its pictures). The slot is for row 0.
fn image_slot(env: &ImageEnv, img: &mdrender::MdImage, pane_cols: usize) -> Option<ImageSlot> {
    if !img.alone || img.target.contains("://") {
        return None;
    }
    let rel = img.target.strip_prefix(".yaks/").unwrap_or(&img.target);
    let (id, name) = rel.split_once('/')?;
    if !crate::store::is_image_name(name) {
        return None;
    }
    let found = crate::farm::locate_attachment(env.root, id, name)?;
    let (w, h) = image::ImageReader::open(&found.path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()?;
    let pane = pane_cols.min(u16::MAX as usize) as u16;
    let (cols, rows) = super::imagesize::cells(img.size, (w, h), env.cell_px, pane, env.max_rows);
    Some(ImageSlot {
        path: found.path,
        cols,
        rows,
        row: 0,
    })
}

/// The rows the line cursor can rest on: every row except the blank rows an
/// inline image reserves after its first one. The image is one stop, on the
/// row above its picture (its `🖼 alt` link row), so crossing it takes one
/// step, however tall it is.
pub fn cursor_stops(lines: &[DLine]) -> Vec<usize> {
    (0..lines.len())
        .filter(|&i| lines[i].image.is_none())
        .collect()
}

/// The blank rows an inline image reserves, each tagged with its slot. They
/// are body rows of the image's paragraph (same [`Src`]), so `E`, yank and
/// block navigation treat them as the paragraph they belong to.
fn image_rows((src, slot): (Src, ImageSlot)) -> impl Iterator<Item = DLine> {
    (0..slot.rows).map(move |row| DLine {
        text: String::new(),
        kind: Kind::Body,
        links: vec![],
        md: vec![],
        cont: false,
        src: Some(src),
        nowrap: None,
        image: Some(ImageSlot {
            row,
            ..slot.clone()
        }),
    })
}

/// The rows an in-place editor reserves.
fn editor_rows(n: usize) -> impl Iterator<Item = DLine> {
    (0..n).map(|_| DLine {
        text: String::new(),
        kind: Kind::Editor,
        links: vec![],
        md: vec![],
        cont: false,
        src: None,
        nowrap: None,
        image: None,
    })
}

/// The label of a comment block: `actor · when` (or just `when`). One design
/// for the detail pane's comment headers.
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
    header_row(&comment_label(kind, clock), width, block, None)
}

/// A block header row: `▸ label ────` to the pane width. For the block under
/// the in-place editor (`editing`) the label is bold and the longest hint that
/// fits sits at the right end of the rule: `── editing · Ctrl-S save ──`.
fn header_row(label: &str, width: usize, block: usize, editing: Option<&EditSlot>) -> DLine {
    let head = format!("\u{25b8} {label} ");
    let head_len = head.chars().count();
    let head_w: usize = head.chars().map(char_disp).sum();
    let rule = palette::RULE_CHAR.to_string();
    // Room after the head for `── <hint> ─`: at least two dashes before the
    // hint and one after.
    let hint = editing.and_then(|e| {
        e.hints
            .iter()
            .find(|h| head_w + 2 + 1 + h.chars().map(char_disp).sum::<usize>() + 1 + 1 <= width)
    });
    let mut label_style = palette::comment_header(true);
    if editing.is_some() {
        label_style = label_style.add_modifier(Modifier::BOLD);
    }
    let mut text = head;
    let mut md = vec![MdSpan {
        start: 0,
        len: head_len,
        style: label_style,
    }];
    let push_rule = |text: &mut String, md: &mut Vec<MdSpan>, n: usize| {
        if n > 0 {
            md.push(MdSpan {
                start: text.chars().count(),
                len: n,
                style: palette::rule(),
            });
            text.push_str(&rule.repeat(n));
        }
    };
    match hint {
        Some(h) => {
            let hint_w: usize = h.chars().map(char_disp).sum();
            let dashes = width - head_w - hint_w - 2 - 1;
            push_rule(&mut text, &mut md, dashes);
            text.push(' ');
            md.push(MdSpan {
                start: text.chars().count(),
                len: h.chars().count(),
                style: palette::comment_header(true),
            });
            text.push_str(h);
            text.push(' ');
            push_rule(&mut text, &mut md, 1);
        }
        None => push_rule(&mut text, &mut md, width.saturating_sub(head_w)),
    }
    DLine {
        md,
        text,
        kind: Kind::Body,
        links: vec![],
        cont: false,
        src: Some(Src { block, lines: None }),
        nowrap: None,
        image: None,
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
                nowrap: dl.nowrap,
                image: None,
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
        let lines = build(&all[0], &all);
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
        let plain: Vec<String> = build(&task, &[task.clone()])
            .iter()
            .map(|l| l.text.clone())
            .collect();
        assert!(!plain.iter().any(|l| l.starts_with("Needs:")));
        // Set: a Needs line appears, immediately after Status.
        task.needs = Some("human".into());
        let lines: Vec<String> = build(&task, &[task.clone()])
            .iter()
            .map(|l| l.text.clone())
            .collect();
        let status_i = lines.iter().position(|l| l.starts_with("Status:")).unwrap();
        let needs_i = lines.iter().position(|l| l.starts_with("Needs:")).unwrap();
        assert_eq!(needs_i, status_i + 1);
        assert!(lines[needs_i].contains("human"));
        // The Needs line carries the warning accent, not a plain field.
        let dlines = build(&task, &[task.clone()]);
        assert_eq!(dlines[needs_i].kind, Kind::Warn);
    }

    #[test]
    fn answered_needs_is_a_plain_field_not_a_warning() {
        let mut task = t("yak-0001", None, &[]);
        task.needs = Some("agent".into());
        let dlines = build(&task, &[task.clone()]);
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
        let single: Vec<String> = build(&solo[0], &solo)
            .iter()
            .map(|l| l.text.clone())
            .collect();
        assert!(!single.iter().any(|l| l.starts_with("Herd:")));
        // Multi-herd farm (two prefixes): a Herd line appears, between Type and
        // Priority, naming this yak's prefix.
        let mixed = vec![t("web-0001", None, &[]), t("api-0001", None, &[])];
        let lines: Vec<String> = build(&mixed[0], &mixed)
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
        let plain: Vec<String> = build(&task, &[task.clone()])
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
        let lines = build(&task, &[task.clone()]);
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
        let lines: Vec<String> = build(&task, &[task.clone()])
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

/// The row of footnote `n`'s definition for a reference on row `from`: the
/// last row of the same content block that starts `[n] ` (definitions sit at
/// the end of their block, so this cannot be mistaken for body text).
pub fn footnote_line(lines: &[DLine], from: usize, n: &str) -> Option<usize> {
    let block = lines.get(from)?.src?.block;
    let head = format!("[{n}] ");
    lines
        .iter()
        .rposition(|l| l.src.is_some_and(|s| s.block == block) && l.text.starts_with(&head))
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
        let lines = build(&all[0], &all);
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
        let lines = build(&all[0], &all);
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
        let lines = build(&all[0], &all);
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
            nowrap: None,
            image: None,
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
        let lines = build(&all[0], &all);
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
            nowrap: None,
            image: None,
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
            nowrap: None,
            image: None,
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
            nowrap: None,
            image: None,
        };
        assert_eq!(wrap(vec![dl], 10)[0].text, "  alpha");

        let dl2 = DLine {
            text: "abcdefghij".into(),
            kind: Kind::Body,
            links: vec![],
            md: vec![],
            cont: false,
            src: None,
            nowrap: None,
            image: None,
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
            nowrap: None,
            image: None,
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
        let lines = build(&all[0], &all);
        let jumps = jumplist(&lines);
        assert_eq!(
            jumps.iter().map(|j| j.target.clone()).collect::<Vec<_>>(),
            vec![Target::Url("https://example.com/x".into())]
        );
    }
}
