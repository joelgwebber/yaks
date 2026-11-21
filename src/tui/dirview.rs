//! The directory view (`DocView::Dir`, yaks-3329): the pane lists the files of
//! a directory of the farm as a tree, so a *set* of files can be looked
//! through: a yak's attachment directory `<herd>/yaks/<id>/`, a dok's sibling
//! directory (`doks/x.md` + `doks/x/`), a directory source's `files/`.
//!
//! - **Reached** by following a link that names a directory (`<id>/proj/`,
//!   `/sources/<id>/files/`), by the `Attachments:` row of a yak's detail pane
//!   and of a dok in the viewer, and by the `Files:` row of a source's view.
//!   Each is a jump of the app's one history ([`super::history`]).
//! - **Keys** are the table's ([`super::keymap`], `Ctx::Dir`): `j`/`k` move,
//!   `Enter` opens a file in the text or image viewer (the path a followed
//!   link takes, so `o`/`i` work) and folds or unfolds a directory, `l`/`h`
//!   unfold/fold, `y` copies the row's path, `O` takes the file out, `Esc`
//!   closes. Nothing here writes to the farm.
//! - **Archives are never system-opened** from the TUI: `Enter` on one, and the
//!   take-out menu's `o`, say how to get its path instead
//!   ([`archive_note`], enforced by `App::os_open`).
//!
//! Only the folds that are open are read from disk. Symlinks are not listed (one
//! could point out of the farm), nor is a `.git` directory, and the tree lists
//! no more than [`MAX_ROWS`] rows. Other dot files are listed: directory
//! sources and attached directories keep them.

use std::cell::Cell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::detail::{self, Target};
use super::handlers::DocKind;
use super::keymap::Cmd;
use super::render::{disp_width, truncate_disp};
use super::*;

/// Most rows the tree lists in all; past it the listing stops.
const MAX_ROWS: usize = 2000;

/// Most files [`count_files`] visits.
const COUNT_CAP: usize = 999;

/// File extensions (lowercase, without the dot) of archives: the OS opener
/// would unpack these into the farm.
const ARCHIVE_EXTS: &[&str] = &["tgz", "tar", "gz", "zip", "7z", "bz2", "xz"];

/// Whether `name` is an archive by its extension (`.tgz .tar .tar.gz .zip .7z
/// .gz .bz2 .xz`, any case).
pub(crate) fn is_archive_name(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| ARCHIVE_EXTS.iter().any(|a| a.eq_ignore_ascii_case(e)))
}

/// The notification an archive gets in place of being opened.
pub(crate) fn archive_note(name: &str) -> String {
    let name = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name);
    format!("{name}: an archive; O \u{2192} p copies its path")
}

/// Whether a directory entry is left out of the view: `.git`.
fn hidden(name: &str) -> bool {
    name == ".git"
}

/// Files under `dir`, counted to [`COUNT_CAP`] (so a huge tree costs a
/// bounded walk): symlinks and `.git` are not counted.
pub(crate) fn count_files(dir: &Path) -> usize {
    fn walk(dir: &Path, n: &mut usize) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            if *n >= COUNT_CAP {
                return;
            }
            if hidden(&e.file_name().to_string_lossy()) {
                continue;
            }
            match e.file_type() {
                Ok(t) if t.is_dir() => walk(&e.path(), n),
                Ok(t) if t.is_file() => *n += 1,
                _ => {}
            }
        }
    }
    let mut n = 0;
    walk(dir, &mut n);
    n
}

/// `3 files`, `1 file`, `999+ files`.
pub(crate) fn files_phrase(n: usize) -> String {
    match n {
        1 => "1 file".into(),
        n if n >= COUNT_CAP => format!("{COUNT_CAP}+ files"),
        n => format!("{n} files"),
    }
}

/// The row that leads into a directory from a yak's detail pane or a dok's
/// viewer: `Attachments:  3 files`, the count a link to the directory.
pub(crate) fn files_row(label: &str, target: Target, n: usize) -> detail::DLine {
    let head = format!("{label:<13}");
    let value = files_phrase(n);
    detail::DLine {
        links: vec![(head.chars().count(), value.chars().count(), target)],
        text: format!("{head}{value}"),
        kind: detail::Kind::Field,
        md: vec![],
        cont: false,
        src: None,
        nowrap: None,
        image: None,
    }
}

/// One listed entry of a directory.
struct Entry {
    name: String,
    is_dir: bool,
    size: u64,
}

/// `dir`'s entries, directories first then files, each sorted by name
/// (case-insensitive). Symlinks and `.git` are left out.
fn list(dir: &Path) -> Vec<Entry> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<Entry> = rd
        .flatten()
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
            Some(Entry {
                name,
                is_dir: t.is_dir(),
                size,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

/// One visible row of the tree.
pub(crate) struct DirRow {
    /// Path below the directory (`src/deep/x.rs`).
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) depth: usize,
    pub(crate) is_dir: bool,
    pub(crate) expanded: bool,
    pub(crate) size: u64,
}

/// A directory open in the pane: the visible rows, the folds that are open,
/// the cursor. Only open folds are read from disk.
pub(crate) struct DirView {
    /// The directory's farm-relative path, no trailing `/`: the pane's header
    /// and what the history re-reads it by.
    pub(crate) rel: String,
    dir: PathBuf,
    open: BTreeSet<String>,
    rows: Vec<DirRow>,
    pub(crate) cursor: usize,
    /// The first visible row, kept in step with the cursor by the render.
    scroll: Cell<usize>,
    /// Files in all, counted when it was opened.
    total: usize,
}

impl DirView {
    /// The view of farm directory `rel` under `root`.
    pub(crate) fn open(root: &Path, rel: &str) -> Result<DirView, String> {
        let rel = rel.trim_matches('/');
        let dir = root.join(rel);
        if rel.is_empty() || !dir.is_dir() {
            return Err(format!("{rel}: not a directory of this farm"));
        }
        let mut v = DirView {
            rel: rel.to_string(),
            total: count_files(&dir),
            dir,
            open: BTreeSet::new(),
            rows: Vec::new(),
            cursor: 0,
            scroll: Cell::new(0),
        };
        v.rebuild();
        Ok(v)
    }

    fn rebuild(&mut self) {
        fn walk(
            dir: &Path,
            prefix: &str,
            depth: usize,
            open: &BTreeSet<String>,
            out: &mut Vec<DirRow>,
        ) {
            for e in list(dir) {
                if out.len() >= MAX_ROWS {
                    return;
                }
                let path = if prefix.is_empty() {
                    e.name.clone()
                } else {
                    format!("{prefix}/{}", e.name)
                };
                let expanded = e.is_dir && open.contains(&path);
                out.push(DirRow {
                    path: path.clone(),
                    name: e.name.clone(),
                    depth,
                    is_dir: e.is_dir,
                    expanded,
                    size: e.size,
                });
                if expanded {
                    walk(&dir.join(&e.name), &path, depth + 1, open, out);
                }
            }
        }
        let mut rows = Vec::new();
        walk(&self.dir, "", 0, &self.open, &mut rows);
        self.rows = rows;
        self.cursor = self.cursor.min(self.rows.len().saturating_sub(1));
    }

    pub(crate) fn selected(&self) -> Option<&DirRow> {
        self.rows.get(self.cursor)
    }

    /// The farm-relative path of the selected row.
    pub(crate) fn selected_rel(&self) -> Option<String> {
        self.selected().map(|r| format!("{}/{}", self.rel, r.path))
    }

    /// The folds that are open, for the history.
    pub(crate) fn open_folds(&self) -> Vec<String> {
        self.open.iter().cloned().collect()
    }

    /// Put back the folds and cursor a history location recorded (clamped to
    /// what is there now).
    pub(crate) fn restore(&mut self, open: &[String], cursor: usize) {
        self.open = open.iter().cloned().collect();
        self.cursor = cursor;
        self.rebuild();
    }

    /// Move the cursor `by` rows, clamped.
    pub(crate) fn step(&mut self, by: i32) {
        let last = self.rows.len().saturating_sub(1) as i64;
        self.cursor = (self.cursor as i64 + by as i64).clamp(0, last) as usize;
    }

    pub(crate) fn to(&mut self, end: bool) {
        self.cursor = if end {
            self.rows.len().saturating_sub(1)
        } else {
            0
        };
    }

    /// Fold or unfold the directory under the cursor (a file: nothing).
    pub(crate) fn toggle(&mut self) {
        let Some(r) = self.selected().filter(|r| r.is_dir) else {
            return;
        };
        let path = r.path.clone();
        if !self.open.remove(&path) {
            self.open.insert(path);
        }
        self.rebuild();
    }

    /// `l`: unfold the directory, else (already open) go to its first entry.
    pub(crate) fn unfold(&mut self) {
        let Some(r) = self.selected().filter(|r| r.is_dir) else {
            return;
        };
        if r.expanded {
            self.step(1);
        } else {
            self.open.insert(r.path.clone());
            self.rebuild();
        }
    }

    /// `h`: fold the directory, else go to the directory that holds the row.
    pub(crate) fn fold(&mut self) {
        let Some(r) = self.selected() else { return };
        if r.is_dir && r.expanded {
            let path = r.path.clone();
            self.open.remove(&path);
            self.rebuild();
        } else if r.depth > 0 {
            let depth = r.depth;
            if let Some(up) = self.rows[..self.cursor]
                .iter()
                .rposition(|p| p.is_dir && p.depth + 1 == depth)
            {
                self.cursor = up;
            }
        }
    }
}

/// A listing of the directory's top level for the peek pane: directories
/// first (`name/`), then files, one per line.
pub(crate) fn listing(dir: &Path) -> String {
    let entries = list(dir);
    if entries.is_empty() {
        return "(empty directory)\n".into();
    }
    let mut s = String::new();
    for e in entries {
        s.push_str(&e.name);
        if e.is_dir {
            s.push('/');
        }
        s.push('\n');
    }
    s
}

/// `1234` -> `1.2 KiB`, as `yaks weight` writes sizes.
fn size_label(bytes: u64) -> String {
    crate::weight::human(bytes)
}

/// The pane: a header naming the directory, then the rows from the scroll
/// down, the cursor's on a band.
pub(crate) fn render_dir(app: &App, v: &DirView, frame: &mut Frame, area: Rect) {
    let [head, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("\u{1f4c1} {}/", v.rel),
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", files_phrase(v.total)),
                Style::new().fg(Color::DarkGray),
            ),
        ])),
        head,
    );
    app.hits.borrow_mut().doc = Some(body);
    let h = body.height as usize;
    let w = body.width as usize;
    if v.rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "(empty directory)",
                Style::new().fg(Color::DarkGray),
            )),
            body,
        );
        return;
    }
    // Scroll only as far as it takes to keep the cursor on screen.
    let mut top = v.scroll.get().min(v.rows.len().saturating_sub(1));
    if v.cursor < top {
        top = v.cursor;
    } else if h > 0 && v.cursor >= top + h {
        top = v.cursor + 1 - h;
    }
    v.scroll.set(top);
    let lines: Vec<Line> = v
        .rows
        .iter()
        .enumerate()
        .skip(top)
        .take(h)
        .map(|(i, r)| {
            let marker = match (r.is_dir, r.expanded) {
                (true, true) => "\u{25be}",
                (true, false) => "\u{25b8}",
                (false, _) => " ",
            };
            let name = if r.is_dir {
                format!("{}/", r.name)
            } else {
                r.name.clone()
            };
            let left = format!("{}{marker} {name}", "  ".repeat(r.depth));
            let right = if r.is_dir {
                String::new()
            } else if is_archive_name(&r.name) {
                format!("archive \u{b7} {}", size_label(r.size))
            } else {
                size_label(r.size)
            };
            // The size column is right-aligned; the name gives way to it.
            let room = w.saturating_sub(right.chars().count() + 2);
            let left = truncate_disp(&left, room.max(1));
            let gap = w.saturating_sub(disp_width(&left) + right.chars().count());
            let base = if r.is_dir {
                Style::new().fg(Color::Cyan)
            } else {
                Style::new()
            };
            let mut line = Line::from(vec![
                Span::styled(left, base),
                Span::raw(" ".repeat(gap)),
                Span::styled(right, Style::new().fg(Color::DarkGray)),
            ]);
            if i == v.cursor {
                line = line.style(Style::new().bg(toque::theme::selected()));
            }
            line
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), body);
}

impl App {
    /// The directory view, when the pane shows one.
    pub(crate) fn dir(&self) -> Option<&DirView> {
        match &self.doc {
            Some(DocPane {
                view: DocView::Dir(v),
                ..
            }) => Some(v),
            _ => None,
        }
    }

    pub(crate) fn dir_mut(&mut self) -> Option<&mut DirView> {
        match &mut self.doc {
            Some(DocPane {
                view: DocView::Dir(v),
                ..
            }) => Some(v),
            _ => None,
        }
    }

    /// The farm directory a link target names, as a farm-relative path with no
    /// trailing `/`, when there is one. `target` is an attachment link
    /// (`<id>/proj/`, `.yaks/<id>/proj`, `<id>/` for the yak's whole
    /// attachment directory) or a farm path (`/sources/<id>/files/`; a
    /// `sources/<id>` without the slug is found like a file's is). Never a
    /// path with a `.` or `..` component, so a link cannot climb out of the
    /// farm, nor with a `.git` one.
    pub(crate) fn farm_dir_of(&self, target: &str) -> Option<String> {
        let root = self.farm.as_ref()?.root();
        let decoded = crate::links::percent_decode(target.strip_prefix(".yaks/").unwrap_or(target));
        let rel = if let Some(abs) = decoded.strip_prefix('/') {
            abs.trim_matches('/').to_string()
        } else {
            let t = decoded.trim_matches('/');
            let (id, name) = t.split_once('/').unwrap_or((t, ""));
            if self.task(id).is_some() {
                let dir = crate::store::artifact_dir(root, id);
                let dir = if name.is_empty() { dir } else { dir.join(name) };
                dir.strip_prefix(root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/")
            } else {
                t.to_string()
            }
        };
        if rel.is_empty()
            || rel
                .split('/')
                .any(|c| matches!(c, "" | "." | ".." | ".git") || c.contains('\\'))
        {
            return None;
        }
        if root.join(&rel).is_dir() {
            return Some(rel);
        }
        // A source named by its bare `<date>-<sha8>` id.
        let rest = rel.strip_prefix("sources/")?;
        let (id, tail) = rest.split_once('/').unwrap_or((rest, ""));
        let dir = crate::sources::dir_for(root, id)?;
        let found = if tail.is_empty() {
            dir
        } else {
            format!("{dir}/{tail}")
        };
        root.join(&found).is_dir().then_some(found)
    }

    /// The directory a pane link points at (what Enter browses), when it
    /// points at one. `doc` is the viewer's document when the link is in one:
    /// a relative link (`sketches/`) is written from its folder, as the
    /// grammar reads a dok's links.
    pub(crate) fn target_dir(
        &self,
        t: &Target,
        doc: Option<&super::handlers::AttachView>,
    ) -> Option<String> {
        match t {
            Target::Farm {
                path, rev: None, ..
            } if self.farm_path_is_dir(path) => self.farm_dir_of(&format!("/{path}")),
            Target::Artifact(p) => {
                let from_doc = doc
                    .filter(|v| !matches!(v.doc, DocKind::Attachment | DocKind::Code { .. }))
                    .filter(|_| !p.starts_with('/') && !p.contains("://"))
                    .and_then(|v| {
                        let base = v.target.rsplit_once('/').map_or("", |(d, _)| d);
                        self.farm_dir_of(&format!("/{base}/{p}"))
                    });
                from_doc.or_else(|| self.farm_dir_of(p))
            }
            _ => None,
        }
    }

    /// The directory of files that goes with farm document `path`, as the
    /// viewer's `Attachments:` / `Files:` row, with its file count: the
    /// document's sibling directory (its path without `.md`), for a dok, a
    /// session note or a source alike. `None` when there is none or it is empty.
    ///
    /// Until the one-file source layout lands (yaks-e83f), a source's files sit
    /// in `<dir>/files/` beside its `source.md`: that is the only
    /// layout-specific line here, and goes with that layout.
    pub(crate) fn files_of(&self, path: &str) -> Option<(&'static str, String, usize)> {
        let root = self.farm.as_ref()?.root();
        let is_source = path.starts_with("sources/");
        let label = if is_source { "Files:" } else { "Attachments:" };
        let sibling = || path.strip_suffix(".md").map(String::from);
        // Every document's files are its sibling directory: a dok's, a
        // session note's, a source's (yaks-e83f).
        let rel = match () {
            _ if crate::sources::is_source_path(path)
                || crate::doks::is_dok_path(path)
                || path.starts_with(&format!("{}/", crate::sessions::DIR)) =>
            {
                sibling()?
            }
            _ => return None,
        };
        let n = count_files(&root.join(&rel));
        (n > 0).then_some((label, rel, n))
    }

    /// Whether farm path `path` as written names a directory rather than a
    /// file: it ends in `/`, or there is no file by that name (`sources/<id>`
    /// is its file, `sources/<id>.md`, not the directory).
    pub(crate) fn farm_path_is_dir(&self, path: &str) -> bool {
        let Some(root) = self.farm.as_ref().map(|f| f.root()) else {
            return false;
        };
        path.ends_with('/')
            || !root
                .join(&*crate::locate::farm_file_in(root, path))
                .is_file()
    }

    /// Show farm directory `rel` in the pane, a jump (`o` comes back).
    pub(crate) fn open_dir(&mut self, rel: &str) {
        let Some(root) = self.farm.as_ref().map(|f| f.root().to_path_buf()) else {
            return;
        };
        match DirView::open(&root, rel) {
            Ok(v) => {
                self.notification = Some(format!("browsing {}/", v.rel));
                self.follow_to_doc(DocView::Dir(v));
            }
            Err(e) => self.notification = Some(e),
        }
    }

    /// Run a directory-view command. The global ones (`q`, `?`, `O`, `#`, `W`,
    /// `U`, `{`/`}`) are `App::run_cmd`'s own.
    pub(super) fn run_dir_cmd(&mut self, cmd: Cmd) {
        let page = self.detail_page.saturating_sub(1).max(1) as i32;
        match cmd {
            Cmd::PanePeel | Cmd::PaneClose => return self.close_doc(),
            Cmd::NavBack => return self.history_back(),
            Cmd::NavForward => return self.history_forward(),
            Cmd::PaneEnter => return self.dir_enter(),
            Cmd::RowCopy => return self.dir_copy_path(),
            _ => {}
        }
        let Some(v) = self.dir_mut() else { return };
        match cmd {
            Cmd::MoveDown => v.step(1),
            Cmd::MoveUp => v.step(-1),
            Cmd::MoveHalfDown => v.step((page / 2).max(1)),
            Cmd::MoveHalfUp => v.step(-(page / 2).max(1)),
            Cmd::MovePageDown => v.step(page),
            Cmd::MovePageUp => v.step(-page),
            Cmd::MoveTop => v.to(false),
            Cmd::MoveBottom => v.to(true),
            Cmd::TreeToggle => v.toggle(),
            Cmd::TreeRight => v.unfold(),
            Cmd::TreeLeft => v.fold(),
            _ => {}
        }
    }

    /// Wheel over the directory: move the cursor `delta` rows.
    pub(crate) fn scroll_dir(&mut self, delta: i32) {
        if let Some(v) = self.dir_mut() {
            v.step(delta);
        }
    }

    /// `Enter`: a directory folds or unfolds; a file opens in the text or
    /// image viewer, as a followed link does (so `o` comes back to this tree);
    /// an archive says how to get its path and is never handed to the system.
    fn dir_enter(&mut self) {
        let Some(v) = self.dir() else { return };
        let Some(row) = v.selected() else { return };
        if row.is_dir {
            if let Some(v) = self.dir_mut() {
                v.toggle();
            }
            return;
        }
        if is_archive_name(&row.name) {
            self.notification = Some(archive_note(&row.name));
            return;
        }
        if let Some(rel) = v.selected_rel() {
            self.open_farm_doc(&rel, None, None);
        }
    }

    /// `y`: copy the selected row's farm-relative path (a directory's too, which
    /// the take-out menu cannot give).
    fn dir_copy_path(&mut self) {
        let Some(v) = self.dir() else { return };
        let Some(path) = v.selected_rel().or_else(|| Some(v.rel.clone())) else {
            return;
        };
        self.notification = Some(if crate::clipboard::copy_text(&path) {
            format!("copied {path}")
        } else {
            "clipboard unavailable".into()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;
    use crate::tui::handlers::OS_OPENED;
    use crate::tui::test_support::*;

    fn put(root: &Path, rel: &str, data: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, data).unwrap();
    }

    fn opened() -> Vec<String> {
        OS_OPENED.with(|o| o.borrow().clone())
    }

    /// A farm whose yak `t0` links its attachment directory and archive, with
    /// a nested tree, a dot file, and a `.tgz`.
    fn attached() -> (PathBuf, App) {
        let mut t0 = task("t0", "solo", Status::Hairy, 3, None);
        t0.body = "See [proj/](t0/proj/) and [pack](t0/pack.tgz).".into();
        let (dir, farm) = temp_farm(&[t0]);
        let a = crate::store::artifact_dir(farm.root(), "t0");
        for (rel, data) in [
            ("proj/src/deep/notes.md", "# deep\n"),
            ("proj/src/main.rs", "fn main() {}\n"),
            ("proj/Cargo.toml", "x\n"),
            ("proj/.hidden", "h\n"),
            ("pack.tgz", "not really a tarball"),
            ("run.log", "log\n"),
        ] {
            put(&a, rel, data);
        }
        let mut app = App::with_farm(farm).unwrap();
        app.detail_page = 20;
        (dir, app)
    }

    /// The visible rows, indented by depth.
    fn names(app: &App) -> Vec<String> {
        let v = app.dir().expect("the directory view is open");
        v.rows
            .iter()
            .map(|r| format!("{}{}", "  ".repeat(r.depth), r.name))
            .collect()
    }

    #[test]
    fn archives_are_recognised_by_extension() {
        for n in [
            "a.tgz", "a.tar", "a.tar.gz", "a.zip", "a.7z", "a.gz", "a.bz2", "a.xz", "A.ZIP",
        ] {
            assert!(is_archive_name(n), "{n}");
        }
        for n in ["a.md", "tgz", "a.tgz.txt", "gz/x", "a.png"] {
            assert!(!is_archive_name(n), "{n}");
        }
    }

    #[test]
    fn a_link_to_a_directory_opens_the_tree_dirs_first_and_o_comes_back() {
        let (dir, mut app) = attached();
        enter_key(&mut app);
        draw(&app, 100, 24);
        app.open_external("t0/");
        assert_eq!(app.surface_ctx(), keymap::Ctx::Dir);
        // Directories first, then files by name; the dot file inside is kept.
        assert_eq!(names(&app), ["proj", "pack.tgz", "run.log"]);
        press(&mut app, "l"); // unfold proj
        assert_eq!(
            names(&app),
            [
                "proj",
                "  src",
                "  .hidden",
                "  Cargo.toml",
                "pack.tgz",
                "run.log"
            ]
        );
        // `l` on an open directory goes in; `h` on its entry goes back out.
        press(&mut app, "l");
        assert_eq!(app.dir().unwrap().selected().unwrap().name, "src");
        press(&mut app, "l");
        press(&mut app, "l");
        assert_eq!(app.dir().unwrap().selected().unwrap().name, "deep");
        press(&mut app, "l");
        press(&mut app, "l");
        assert_eq!(app.dir().unwrap().selected().unwrap().name, "notes.md");
        // Enter on a file is the followed-link path: the text viewer.
        enter_key(&mut app);
        let v = app.attach().expect("the file opens in the viewer");
        assert!(v.target.ends_with("proj/src/deep/notes.md"), "{}", v.target);
        // `o` returns to the same tree: folds and cursor.
        press(&mut app, "o");
        let d = app.dir().expect("back to the directory view");
        assert_eq!(d.selected().unwrap().name, "notes.md");
        assert_eq!(d.open_folds(), ["proj", "proj/src", "proj/src/deep"]);
        press(&mut app, "i");
        assert!(app.attach().is_some(), "i goes forward to the file again");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn enter_folds_a_directory_and_esc_closes_the_view() {
        let (dir, mut app) = attached();
        enter_key(&mut app);
        draw(&app, 100, 24);
        app.open_external("t0/proj/");
        assert_eq!(names(&app), ["src", ".hidden", "Cargo.toml"]);
        enter_key(&mut app);
        assert_eq!(
            names(&app),
            ["src", "  deep", "  main.rs", ".hidden", "Cargo.toml"]
        );
        enter_key(&mut app);
        assert_eq!(names(&app), ["src", ".hidden", "Cargo.toml"]);
        esc_key(&mut app);
        assert!(app.doc.is_none());
        assert_eq!(
            app.focus,
            Focus::Detail,
            "back to the yak it was opened from"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    /// yaks-3329: an archive is never handed to the OS opener (it would unpack
    /// into the farm): not from `Enter` in the tree, not from a followed
    /// attachment link, not from the take-out menu's `o`.
    #[test]
    fn enter_on_an_archive_never_calls_the_os_opener() {
        let (dir, mut app) = attached();
        OS_OPENED.with(|o| o.borrow_mut().clear());
        enter_key(&mut app);
        draw(&app, 100, 24);
        // In the tree.
        app.open_external("t0/");
        press(&mut app, "j"); // pack.tgz
        assert_eq!(app.dir().unwrap().selected().unwrap().name, "pack.tgz");
        enter_key(&mut app);
        assert_eq!(
            app.notification.as_deref(),
            Some("pack.tgz: an archive; O \u{2192} p copies its path")
        );
        assert!(app.dir().is_some(), "the tree stays");
        // From a link.
        app.close_doc();
        app.notification = None;
        app.open_external("t0/pack.tgz");
        assert_eq!(
            app.notification.as_deref(),
            Some("pack.tgz: an archive; O \u{2192} p copies its path")
        );
        // From the take-out menu: `o` on it, and `p` still copies the path.
        app.open_external("t0/");
        press(&mut app, "j");
        press(&mut app, "O");
        let Overlay::TakeOut(t) = std::mem::replace(&mut app.overlay, Overlay::None) else {
            panic!("the take-out menu should be up");
        };
        app.take_out_with(None, &t, 'o');
        assert!(
            app.notification.as_deref().unwrap().contains("an archive"),
            "{:?}",
            app.notification
        );
        assert_eq!(
            opened(),
            Vec::<String>::new(),
            "the opener was never called"
        );
        app.take_out_with(None, &t, 'p');
        assert!(
            crate::tui::takeout::copied()
                .unwrap()
                .ends_with("t0/pack.tgz"),
            "p still copies the path"
        );
        // And the stub does see a file that is not an archive.
        app.os_open("run.log", "/tmp/run.log");
        assert_eq!(opened(), ["/tmp/run.log"]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_detail_pane_has_an_attachments_row_only_when_there_are_files() {
        let (dir, mut app) = attached();
        enter_key(&mut app);
        let rows = app.detail_dlines();
        let row = rows
            .iter()
            .find(|r| r.text.starts_with("Attachments:"))
            .unwrap();
        assert_eq!(row.text, "Attachments: 6 files");
        drop(rows);
        // Tab to it and Enter: the directory view.
        tab_key(&mut app);
        enter_key(&mut app);
        assert_eq!(app.dir().unwrap().rel, "t0/yaks/t0");
        let _ = std::fs::remove_dir_all(dir);

        let (dir, farm) = temp_farm(&[task("t1", "bare", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        enter_key(&mut app);
        assert!(
            !app.detail_dlines()
                .iter()
                .any(|r| r.text.contains("Attachments"))
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_dok_and_a_source_have_a_files_row() {
        let (dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let root = farm.root().to_path_buf();
        put(
            &root,
            "doks/topics/notes.md",
            "---\ntitle: N\n---\n\nSee [s](notes/).\n",
        );
        put(&root, "doks/topics/notes/a.png.txt", "a");
        put(&root, "doks/topics/bare.md", "---\ntitle: B\n---\n\nbare\n");
        // A directory source: its tree is directly in the sibling directory.
        let src = "sources/2026-11-17-abcd1234-snap";
        put(
            &root,
            &format!("{src}.md"),
            "---\ntitle: S\nfile_count: 2\n---\n\nsnap\n",
        );
        put(&root, &format!("{src}/lib/x.c"), "x");
        put(&root, &format!("{src}/.gitignore"), "x");
        let mut app = App::with_farm(farm).unwrap();
        assert_eq!(
            app.files_of("doks/topics/notes.md"),
            Some(("Attachments:", "doks/topics/notes".to_string(), 1))
        );
        assert_eq!(app.files_of("doks/topics/bare.md"), None);
        // A source that keeps an original beside its text is a dok in this
        // respect.
        put(
            &root,
            "sources/2026-11-18-ffff0000-one.md",
            "---\ntitle: O\n---\n\none\n",
        );
        put(&root, "sources/2026-11-18-ffff0000-one/a.c", "x");
        assert_eq!(
            app.files_of("sources/2026-11-18-ffff0000-one.md"),
            Some(("Files:", "sources/2026-11-18-ffff0000-one".to_string(), 1))
        );
        assert_eq!(
            app.files_of(&format!("{src}.md")),
            Some(("Files:", src.to_string(), 2)),
            "dot files count"
        );
        // A source with nothing beside its text has no row.
        put(
            &root,
            "sources/2026-11-19-00000000-bare.md",
            "---\ntitle: B\n---\n\nbare\n",
        );
        assert_eq!(app.files_of("sources/2026-11-19-00000000-bare.md"), None);
        // The row is in the viewer, and a relative link from the dok opens
        // the directory (peek and Enter alike).
        app.detail_page = 20;
        app.open_farm_doc("doks/topics/notes.md", None, None);
        draw(&app, 100, 24);
        let (rows, _) = app.attachment_rows().unwrap();
        assert!(
            rows.iter().any(|r| r.text == "Attachments: 1 file"),
            "{:?}",
            rows.iter().map(|r| &r.text).collect::<Vec<_>>()
        );
        tab_key(&mut app);
        tab_key(&mut app);
        enter_key(&mut app);
        assert_eq!(app.dir().unwrap().rel, "doks/topics/notes");
        // `/sources/<bare id>/` finds the slugged directory.
        app.open_farm_doc("sources/2026-11-17-abcd1234/", None, None);
        assert_eq!(app.dir().unwrap().rel, src);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_link_cannot_climb_out_of_the_farm_or_into_git() {
        let (dir, app) = attached();
        for bad in [
            "/../",
            "/doks/../..",
            "/.git/",
            "t0/../../x/",
            "/sources/x/./",
            "/",
        ] {
            assert_eq!(app.farm_dir_of(bad), None, "{bad}");
        }
        assert_eq!(
            app.farm_dir_of("t0/proj/").as_deref(),
            Some("t0/yaks/t0/proj")
        );
        assert_eq!(
            app.farm_dir_of("/t0/yaks/t0/proj").as_deref(),
            Some("t0/yaks/t0/proj")
        );
        assert_eq!(app.farm_dir_of("t0/nope/"), None);
        assert_eq!(
            app.farm_dir_of("t0/run.log"),
            None,
            "a file is not a directory"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_missing_directory_says_so() {
        let (dir, mut app) = attached();
        enter_key(&mut app);
        app.open_farm_doc("doks/nothing/", None, None);
        assert!(app.dir().is_none());
        assert!(app.attach().is_none());
        let _ = std::fs::remove_dir_all(dir);
    }
}
