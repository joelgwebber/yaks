//! The directory view (`DocView::Dir`, yaks-3329): the pane lists the files of
//! a directory of the farm as a tree, so a *set* of files can be looked
//! through: a yak's attachment directory `<herd>/yaks/<id>/`, a dok's sibling
//! directory (`doks/x.md` + `doks/x/`), a directory source's `files/`.
//!
//! - **Reached** by following a link that names a directory (`<id>/proj/`,
//!   `/doks/x/sketches/`; the resolver decides, `locate::Place::FarmDir`),
//!   by the `Attachments:` row of a yak's detail pane
//!   and of a dok in the viewer, and by the `Files:` row of a source's view.
//!   Each is a jump of the app's one history ([`super::history`]).
//! - **Keys** are the table's ([`super::keymap`], `Ctx::Dir`): `j`/`k` move,
//!   `Enter` opens a file in the text or image viewer (the path a followed
//!   link takes, so `o`/`i` work) and folds or unfolds a directory, `l`/`h`
//!   unfold/fold, `y` copies the row's path, `O` takes the file out, `Esc`
//!   closes. Nothing here writes to the farm.
//! - **Archives** (`.zip .tar .tar.gz .tgz`, yaks-93ee) read as read-only
//!   directories: the row has a fold marker and unfolds like one, its members
//!   open in the viewer, nothing is ever extracted into the farm. Every other
//!   archive kind (`.7z .bz2 .xz`, a bare `.gz`, or a file that is not
//!   really one) is a plain row. **No archive is ever system-opened** from the
//!   TUI: `Enter` on an unsupported one, and the take-out menu's `o`, say how
//!   to get its path instead ([`archive_note`], enforced by `App::os_open`).
//!
//! Only the folds that are open are read from disk, through the farm's one
//! lister ([`crate::entries`], so `yaks peek` lists a directory the same
//! way): no symlinks, no `.git`, every other dot file. The tree lists no more
//! than [`MAX_ROWS`] rows.

use std::cell::Cell;
use std::collections::BTreeSet;
use std::path::Path;

use crate::entries::Container;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::detail::{self, Target};
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

/// Files in farm directory `dir` (absolute), counted to [`COUNT_CAP`] by
/// the farm's listing rules ([`crate::entries`]): what the `Attachments:` /
/// `Files:` rows and the view's header show.
pub(crate) fn count_files(dir: &Path) -> usize {
    crate::entries::count_files(
        &crate::entries::Container::Dir(dir.to_path_buf()),
        "",
        COUNT_CAP,
    )
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

/// One visible row of the tree.
pub(crate) struct DirRow {
    /// Path below the directory (`src/deep/x.rs`).
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) depth: usize,
    /// A directory, or an archive that unfolds like one.
    pub(crate) is_dir: bool,
    /// An archive file ([`crate::entries::Entry::archive`]): `is_dir`, with a
    /// size and no trailing `/`.
    pub(crate) archive: bool,
    pub(crate) expanded: bool,
    pub(crate) size: u64,
}

/// A directory open in the pane: the visible rows, the folds that are open,
/// the cursor. Only open folds are read from disk.
pub(crate) struct DirView {
    /// The directory's farm-relative path, no trailing `/`: the pane's header
    /// and what the history re-reads it by.
    pub(crate) rel: String,
    /// What it lists, read through [`crate::entries`], and where in it the
    /// view starts (`""` unless `rel` is a directory inside an archive).
    container: Container,
    base: String,
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
        let Some((container, base)) = crate::entries::dir_at(root, rel) else {
            return Err(format!("{rel}: not a directory of this farm"));
        };
        let mut v = DirView {
            rel: rel.to_string(),
            total: crate::entries::count_files(&container, &base, COUNT_CAP),
            container,
            base,
            open: BTreeSet::new(),
            rows: Vec::new(),
            cursor: 0,
            scroll: Cell::new(0),
        };
        v.rebuild();
        Ok(v)
    }

    fn rebuild(&mut self) {
        /// `prefix` is the path below the view's directory; the container
        /// is asked for `base/prefix`.
        fn walk(
            c: &Container,
            base: &str,
            prefix: &str,
            depth: usize,
            open: &BTreeSet<String>,
            out: &mut Vec<DirRow>,
        ) {
            let join = |a: &str, b: &str| {
                if a.is_empty() {
                    b.to_string()
                } else {
                    format!("{a}/{b}")
                }
            };
            for e in crate::entries::list(c, &join(base, prefix)) {
                if out.len() >= MAX_ROWS {
                    return;
                }
                let path = join(prefix, &e.name);
                let expanded = e.is_dir && open.contains(&path);
                out.push(DirRow {
                    path: path.clone(),
                    name: e.name,
                    depth,
                    is_dir: e.is_dir,
                    archive: e.archive,
                    expanded,
                    size: e.size,
                });
                if expanded {
                    walk(c, base, &path, depth + 1, open, out);
                }
            }
        }
        let mut rows = Vec::new();
        walk(&self.container, &self.base, "", 0, &self.open, &mut rows);
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
            let name = if r.is_dir && !r.archive {
                format!("{}/", r.name)
            } else {
                r.name.clone()
            };
            let left = format!("{}{marker} {name}", "  ".repeat(r.depth));
            let right = if r.is_dir && !r.archive {
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
            let base = if r.is_dir && !r.archive {
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
                line = line.style(Style::new().bg(super::surface::selected()));
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
    use crate::locate::Place;
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
        app.open_dest("t0/");
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
        app.open_dest("t0/proj/");
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
        app.open_dest("t0/");
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
        app.open_dest("t0/pack.tgz");
        assert_eq!(
            app.notification.as_deref(),
            Some("pack.tgz: an archive; O \u{2192} p copies its path")
        );
        // From the take-out menu: `o` on it, and `p` still copies the path.
        app.open_dest("t0/");
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

    /// The one resolver decides what a link names: a directory link never
    /// climbs out of the farm or into git, and a file is not a directory.
    #[test]
    fn a_link_cannot_climb_out_of_the_farm_or_into_git() {
        let (dir, app) = attached();
        let dir_of = |dest: &str| match app.place_of_dest(dest) {
            Place::FarmDir { rel } => Some(rel),
            _ => None,
        };
        for bad in [
            "/../",
            "/doks/../..",
            "/.git/",
            "t0/../../x/",
            "/sources/x/./",
            "/",
        ] {
            assert_eq!(dir_of(bad), None, "{bad}");
        }
        assert_eq!(dir_of("t0/proj/").as_deref(), Some("t0/yaks/t0/proj"));
        assert_eq!(
            dir_of("/t0/yaks/t0/proj/").as_deref(),
            Some("t0/yaks/t0/proj")
        );
        assert_eq!(dir_of("t0/").as_deref(), Some("t0/yaks/t0"));
        assert_eq!(dir_of("t0/nope/"), None);
        assert_eq!(dir_of("t0/run.log"), None, "a file is not a directory");
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

    /// A farm whose yak `t0` has a real `code.zip` and `pack.tar.gz`
    /// attached (with a hostile member each) and a `bad.7z`.
    fn archived() -> (PathBuf, App) {
        use crate::entries::fixtures;
        let t0 = task("t0", "solo", Status::Hairy, 3, None);
        let (dir, farm) = temp_farm(&[t0]);
        let a = crate::store::artifact_dir(farm.root(), "t0");
        std::fs::create_dir_all(&a).unwrap();
        let members: &[(&str, &[u8])] = &[
            ("src/main.rs", b"fn main() {}\nlet two = 2;\n"),
            ("src/deep/notes.md", b"# deep\n"),
            ("README.md", b"# top\n"),
            ("../evil.txt", b"evil"),
        ];
        fixtures::zip(&a.join("code.zip"), members);
        fixtures::tgz(&a.join("pack.tar.gz"), members, &[]);
        std::fs::write(a.join("bad.7z"), "7z").unwrap();
        let mut app = App::with_farm(farm).unwrap();
        app.detail_page = 20;
        (dir, app)
    }

    #[test]
    fn the_tree_unfolds_into_an_archive_and_a_member_opens_read_only() {
        let (dir, mut app) = archived();
        enter_key(&mut app);
        app.open_dest("t0/");
        // An archive row is a fold row (no trailing `/`), its size column
        // keeps `archive · <size>`; an unsupported kind is a plain file.
        // An archive is a directory to the tree, so it sorts with them.
        assert_eq!(names(&app), ["code.zip", "pack.tar.gz", "bad.7z"]);
        let shown = draw(&app, 100, 24);
        assert!(shown.contains("\u{25b8} code.zip "), "{shown}");
        assert!(shown.contains("archive \u{b7} "), "{shown}");
        assert!(!shown.contains("code.zip/"), "{shown}");
        press(&mut app, "l"); // unfold code.zip
        assert_eq!(
            names(&app),
            ["code.zip", "  src", "  README.md", "pack.tar.gz", "bad.7z"]
        );
        press(&mut app, "l"); // into it
        press(&mut app, " "); // `src`: unfold with Space
        assert_eq!(app.dir().unwrap().selected().unwrap().name, "src");
        let n = names(&app);
        assert!(n.contains(&"    main.rs".to_string()), "{n:?}");
        assert!(!n.iter().any(|r| r.contains("evil")), "{n:?}");
        press(&mut app, "j"); // deep
        press(&mut app, "j"); // main.rs
        assert_eq!(app.dir().unwrap().selected().unwrap().name, "main.rs");
        enter_key(&mut app);
        let v = app.attach().expect("the member opens in the viewer");
        assert!(
            v.target.ends_with("t0/code.zip/src/main.rs"),
            "{}",
            v.target
        );
        assert!(v.text.contains("two"));
        // Read-only, like a pinned dok.
        press(&mut app, "E");
        assert!(
            app.notification.as_deref().unwrap().contains("read-only"),
            "{:?}",
            app.notification
        );
        assert!(app.attach().is_some(), "still the viewer");
        // `o` comes back to the same tree, folds and cursor.
        press(&mut app, "o");
        let d = app.dir().expect("back in the tree");
        assert_eq!(d.selected().unwrap().name, "main.rs");
        assert_eq!(d.open_folds(), ["code.zip", "code.zip/src"]);
        assert!(opened().is_empty(), "nothing went to the OS opener");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_link_into_an_archive_opens_the_viewer_or_the_tree() {
        let (dir, mut app) = archived();
        enter_key(&mut app);
        draw(&app, 100, 24);
        app.open_dest("t0/pack.tar.gz/src/main.rs#L2");
        let v = app.attach().expect("the member opens");
        assert!(v.target.ends_with("t0/pack.tar.gz/src/main.rs"));
        // `#L2` lands on the file's second line.
        let (rows, _) = app.attachment_rows().unwrap();
        assert_eq!(rows[app.attach().unwrap().nav.line].text, "let two = 2;");
        app.close_doc();
        app.open_dest("t0/pack.tar.gz/src/");
        assert_eq!(
            app.dir().unwrap().rel,
            "t0/yaks/t0/pack.tar.gz/src".replace("t0/yaks/t0", "t0/yaks/t0")
        );
        assert_eq!(names(&app), ["deep", "main.rs"]);
        app.close_doc();
        app.open_dest("t0/code.zip/");
        assert_eq!(names(&app), ["src", "README.md"]);
        // A missing or hostile member says so and opens nothing.
        app.close_doc();
        app.open_dest("t0/code.zip/../evil.txt");
        assert!(app.dir().is_none() && app.attach().is_none());
        // An unsupported archive keeps the note and is never opened.
        app.open_dest("t0/bad.7z");
        assert_eq!(
            app.notification.as_deref(),
            Some("bad.7z: an archive; O \u{2192} p copies its path")
        );
        assert!(opened().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn take_out_on_a_member_gives_the_path_through_the_archive_and_a_read_only_copy() {
        let (dir, mut app) = archived();
        enter_key(&mut app);
        app.open_dest("t0/code.zip/src/main.rs");
        press(&mut app, "O");
        let Overlay::TakeOut(t) = std::mem::replace(&mut app.overlay, Overlay::None) else {
            panic!("the take-out menu should be up");
        };
        assert!(matches!(t.place, Place::Member { .. }));
        let root = app.farm.as_ref().unwrap().root().to_path_buf();
        let through = "t0/yaks/t0/code.zip/src/main.rs";
        app.take_out_with(None, &t, 'r');
        assert_eq!(crate::tui::takeout::copied().as_deref(), Some(through));
        app.take_out_with(None, &t, 'p');
        assert_eq!(
            crate::tui::takeout::copied(),
            Some(root.join(through).display().to_string())
        );
        // `e` queues the editor on a read-only extracted copy.
        app.take_out_with(Some("ed"), &t, 'e');
        let note = app.notification.clone().unwrap();
        assert!(note.contains("read-only copy"), "{note}");
        assert!(app.pending_editor.is_some());
        let (copy, snap) = crate::locate::Space::new(&root, None)
            .local_path(&t.place)
            .unwrap();
        assert!(snap);
        assert!(std::fs::metadata(&copy).unwrap().permissions().readonly());
        assert!(std::fs::read_to_string(&copy).unwrap().contains("two"));
        // `o` opens that copy with the system.
        app.take_out_with(None, &t, 'o');
        assert_eq!(opened(), [copy.display().to_string()]);
        // `g`: a host cannot link into an archive.
        app.take_out_with(None, &t, 'g');
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .contains("host cannot link into an archive"),
            "{:?}",
            app.notification
        );
        // From the tree, an archive row is the archive file (never system-opened).
        app.open_dest("t0/");
        press(&mut app, "O"); // on code.zip
        let Overlay::TakeOut(t) = std::mem::replace(&mut app.overlay, Overlay::None) else {
            panic!("the take-out menu should be up");
        };
        assert!(matches!(t.place, Place::FarmFile { .. }));
        let _ = std::fs::remove_dir_all(dir);
    }
}
