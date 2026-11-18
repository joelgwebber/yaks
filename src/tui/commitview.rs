//! The commit view (`DocView::Commit`, yaks-3f90): the pane shows one commit of
//! the code repository, as `git show --stat` would: subject, author and date,
//! the message, then the changed files with their `+added -deleted` counts.
//!
//! - **Reached** by following a commit link: `commit:<sha>`, the code repo's
//!   own host commit URL (read back by [`crate::locate::Space::read_url`]), or
//!   the `commit:` / `source:` hash in a note's header. A jump of the app's
//!   one history ([`super::history`]).
//! - **Keys** are the table's ([`super::keymap`], `Ctx::Commit`): `j`/`k` move,
//!   `Enter` opens the file under the cursor in the code viewer *at this
//!   commit* (the path a pinned `code:path@sha` link takes), or the commit a
//!   `Landed as` row names; `y` copies the hash; `O` takes the commit out
//!   (`g` the host URL, `p` the hash); `Esc` closes. Nothing here writes.
//! - **A squashed worker commit** (the `source:` of a landed note) says
//!   `Landed as <sha>` and links to it.
//! - **A file's diff unfolds inline** (yaks-bdab): `Space` (or `l`/`→`) on a
//!   file row shows its patch under it, hunk headers cyan, `+` green, `-` red;
//!   `h`/`←` folds it back (from a patch line, the cursor goes to its file).
//!   `Enter` on a patch line opens the file at this commit at that line. The
//!   patch is read from git once, when first unfolded; the open folds are part
//!   of the history location, like the directory view's.
//! - **A commit this checkout lacks** says so, with the host link and the
//!   `yaks commits <yak> --fetch` that would bring its branch in, rather than
//!   an error.
//!
//! The data is read once, when the view opens; `o`/`i` read it again.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::PathBuf;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::detail::wrap_ranges;
use super::keymap::Cmd;
use super::render::{disp_width, truncate_disp};
use super::source::source_base;
use super::*;
use crate::gitrepo::CommitInfo;
use crate::links::{self, Link};
use crate::locate::{Place, Space};

/// What the pane can act on: the row's `Enter`.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Item {
    /// A commit to open (`Landed as <sha>`).
    Commit(String),
    /// A file changed by the commit, by repo-relative path.
    File(String),
    /// A line of a file's unfolded patch: the file, and the line in the
    /// commit's version it is at (a deleted line: the next line there).
    Patch(String, u32),
}

/// One line of an unfolded patch.
#[derive(Clone, PartialEq, Debug)]
struct PatchLine {
    text: String,
    style: Style,
    /// The line in the commit's version of the file.
    line: u32,
}

/// A unified diff as display lines: hunk headers, then `+`/`-`/context lines
/// with the new side's line number each lands on.
fn patch_lines(diff: &str) -> Vec<PatchLine> {
    let mut out = Vec::new();
    let mut n: u32 = 1;
    for l in diff.lines() {
        let (style, line) = if let Some(rest) = l.strip_prefix("@@") {
            // `@@ -a,b +c,d @@`: the new side starts at c.
            n = rest
                .split_whitespace()
                .find_map(|w| w.strip_prefix('+'))
                .and_then(|w| w.split(',').next()?.parse().ok())
                .unwrap_or(n);
            (Style::new().fg(Color::Cyan), n)
        } else if l.starts_with('+') {
            n += 1;
            (Style::new().fg(Color::Green), n - 1)
        } else if l.starts_with('-') {
            (Style::new().fg(Color::Red), n)
        } else if l.starts_with('\\') || l.starts_with("Binary files") {
            (dim(), n)
        } else {
            n += 1;
            (Style::new(), n - 1)
        };
        out.push(PatchLine {
            text: l.replace('\t', "    "),
            style,
            line: line.max(1),
        });
    }
    out
}

/// What was read of the commit.
enum State {
    Found(CommitInfo),
    /// Not in this checkout: the reason (`locate::missing_commit`).
    Missing(String),
}

/// A commit open in the pane.
pub(crate) struct CommitView {
    /// The full hash when the checkout has the commit, else as linked.
    pub(crate) sha: String,
    state: State,
    /// The commit a landed note says this (worker) hash was squashed into.
    landed_as: Option<String>,
    /// The host's commit page, or why there is none.
    host: Result<String, String>,
    /// The yak the view was opened from: whose `yaks commits --fetch` is named.
    yak: Option<String>,
    /// The code checkout the commit was read from (patches are read there).
    base: PathBuf,
    /// The files whose patch is unfolded, in the order they were opened.
    open: RefCell<Vec<String>>,
    /// Patches read so far, by path.
    patches: RefCell<HashMap<String, Vec<PatchLine>>>,
    cursor: Cell<usize>,
    /// Whether the cursor has been put on its first stop (needs the width).
    placed: Cell<bool>,
    scroll: Cell<usize>,
    /// Rows at the width last drawn, and that width.
    nrows: Cell<usize>,
    width: Cell<u16>,
}

/// One display row: styled pieces, and what `Enter` does on it.
struct Row {
    spans: Vec<(String, Style)>,
    item: Option<Item>,
}

fn plain(s: impl Into<String>) -> Row {
    Row {
        spans: vec![(s.into(), Style::new())],
        item: None,
    }
}

fn dim() -> Style {
    Style::new().fg(Color::DarkGray)
}

fn short(sha: &str) -> &str {
    &sha[..sha.len().min(7)]
}

impl CommitView {
    pub(crate) fn short(&self) -> &str {
        short(&self.sha)
    }

    /// The row the cursor starts on: the first stop, else the top.
    fn first_stop(rows: &[Row]) -> usize {
        rows.iter().position(|r| r.item.is_some()).unwrap_or(0)
    }

    /// The rows at `width`.
    fn rows(&self, width: usize) -> Vec<Row> {
        let width = width.max(20);
        let mut out = Vec::new();
        let label = |s: &str| (format!("{s:<10}"), dim());
        match &self.state {
            State::Found(c) => {
                out.push(Row {
                    spans: vec![(
                        truncate_disp(&c.subject, width),
                        Style::new().add_modifier(Modifier::BOLD),
                    )],
                    item: None,
                });
                out.push(Row {
                    spans: vec![label("Author"), (c.author.clone(), Style::new())],
                    item: None,
                });
                out.push(Row {
                    spans: vec![label("Date"), (c.date.clone(), Style::new())],
                    item: None,
                });
                out.push(Row {
                    spans: vec![label("Commit"), (c.sha.clone(), Style::new())],
                    item: None,
                });
            }
            State::Missing(why) => {
                out.push(Row {
                    spans: vec![(
                        why.clone(),
                        Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )],
                    item: None,
                });
            }
        }
        if let Some(l) = &self.landed_as {
            out.push(Row {
                spans: vec![
                    label("Landed as"),
                    (
                        short(l).to_string(),
                        Style::new()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    ("  (squashed into the landed commit)".into(), dim()),
                ],
                item: Some(Item::Commit(l.clone())),
            });
        }
        match &self.state {
            State::Found(c) => {
                if !c.body.is_empty() {
                    out.push(plain(""));
                    for line in c.body.lines() {
                        let chars: Vec<char> = line.chars().collect();
                        for (s, e) in wrap_ranges(&chars, width) {
                            out.push(plain(chars[s..e].iter().collect::<String>()));
                        }
                    }
                }
                out.push(plain(""));
                let add: u32 = c.files.iter().filter_map(|f| f.0).sum();
                let del: u32 = c.files.iter().filter_map(|f| f.1).sum();
                let n = c.files.len();
                out.push(Row {
                    spans: vec![(
                        format!(
                            "{n} file{} changed, +{add} -{del}",
                            if n == 1 { "" } else { "s" }
                        ),
                        dim(),
                    )],
                    item: None,
                });
                for (a, d, path) in &c.files {
                    let num = |v: &Option<u32>, sign: char| match v {
                        Some(v) => format!("{sign}{v}"),
                        None => format!("{sign}-"),
                    };
                    let (a, d) = (num(a, '+'), num(d, '-'));
                    let open = self.open.borrow().contains(path);
                    out.push(Row {
                        spans: vec![
                            (
                                if open { "\u{25be}" } else { "\u{25b8}" }.to_string(),
                                dim(),
                            ),
                            (format!("{a:>6} "), Style::new().fg(Color::Green)),
                            (format!("{d:>7}  "), Style::new().fg(Color::Red)),
                            (truncate_disp(path, width.saturating_sub(17)), Style::new()),
                        ],
                        item: Some(Item::File(path.clone())),
                    });
                    if open {
                        for p in self.patch(path) {
                            let chars: Vec<char> = p.text.chars().collect();
                            let indent = "  ";
                            let room = width.saturating_sub(indent.len()).max(1);
                            for (s, e) in wrap_ranges(&chars, room) {
                                out.push(Row {
                                    spans: vec![
                                        (indent.to_string(), Style::new()),
                                        (chars[s..e].iter().collect::<String>(), p.style),
                                    ],
                                    item: Some(Item::Patch(path.clone(), p.line)),
                                });
                            }
                        }
                    }
                }
            }
            State::Missing(_) => {
                out.push(plain(""));
                match &self.host {
                    Ok(url) => out.push(Row {
                        spans: vec![label("Host"), (url.clone(), Style::new())],
                        item: None,
                    }),
                    Err(why) => out.push(Row {
                        spans: vec![label("Host"), (why.clone(), dim())],
                        item: None,
                    }),
                }
                let yak = self.yak.as_deref().unwrap_or("<yak>");
                out.push(Row {
                    spans: vec![
                        label("Fetch"),
                        (format!("yaks commits {yak} --fetch"), Style::new()),
                    ],
                    item: None,
                });
                out.push(plain(""));
                out.push(Row {
                    spans: vec![(
                        "O then g copies the host link; O then p copies the hash".into(),
                        dim(),
                    )],
                    item: None,
                });
            }
        }
        out
    }

    /// `path`'s patch, read from git the first time it is asked for.
    fn patch(&self, path: &str) -> Vec<PatchLine> {
        if let Some(p) = self.patches.borrow().get(path) {
            return p.clone();
        }
        let lines = match crate::gitrepo::commit_file_diff(&self.base, &self.sha, path) {
            Some(d) if !d.trim().is_empty() => patch_lines(&d),
            Some(_) => patch_lines("(no textual change: a mode change or an empty file)"),
            None => patch_lines("(git could not show this file's diff)"),
        };
        self.patches
            .borrow_mut()
            .insert(path.to_string(), lines.clone());
        lines
    }

    /// The file the cursor's row belongs to (a file row or a line of its patch).
    fn cursor_file(&self) -> Option<String> {
        match self.selected()? {
            Item::File(p) | Item::Patch(p, _) => Some(p),
            Item::Commit(_) => None,
        }
    }

    /// Put the cursor on `path`'s file row.
    fn cursor_to_file(&self, path: &str) {
        let rows = self.rows(self.width.get() as usize);
        if let Some(i) = rows
            .iter()
            .position(|r| r.item.as_ref() == Some(&Item::File(path.to_string())))
        {
            self.cursor.set(i);
        }
    }

    /// `Space`: fold or unfold the patch of the cursor's file.
    fn toggle(&self) {
        let Some(path) = self.cursor_file() else {
            return;
        };
        if self.open.borrow().contains(&path) {
            self.fold();
        } else {
            self.unfold();
        }
    }

    /// `l`/`→`: show the cursor's file's patch under it.
    fn unfold(&self) {
        if let Some(Item::File(path)) = self.selected() {
            let mut open = self.open.borrow_mut();
            if !open.contains(&path) {
                open.push(path);
            }
        }
    }

    /// `h`/`←`: fold the cursor's file's patch, the cursor back on the file.
    fn fold(&self) {
        let Some(path) = self.cursor_file() else {
            return;
        };
        self.open.borrow_mut().retain(|p| *p != path);
        self.cursor_to_file(&path);
    }

    /// The unfolded files, for the history.
    pub(crate) fn open_folds(&self) -> Vec<String> {
        self.open.borrow().clone()
    }

    /// Put back the folds a history location recorded.
    pub(crate) fn restore_folds(&self, folds: &[String]) {
        *self.open.borrow_mut() = folds.to_vec();
    }

    fn step(&self, by: i32) {
        let last = self.nrows.get().saturating_sub(1) as i64;
        self.cursor
            .set((self.cursor.get() as i64 + by as i64).clamp(0, last) as usize);
    }

    fn to(&self, end: bool) {
        self.cursor.set(if end {
            self.nrows.get().saturating_sub(1)
        } else {
            0
        });
    }

    /// What `Enter` acts on at the cursor.
    fn selected(&self) -> Option<Item> {
        let rows = self.rows(self.width.get() as usize);
        rows.into_iter().nth(self.cursor.get())?.item
    }

    /// The history's position in this view: the cursor row.
    pub(crate) fn cursor(&self) -> usize {
        self.cursor.get()
    }

    pub(crate) fn restore(&self, cursor: usize) {
        self.cursor.set(cursor);
        self.placed.set(true);
    }

    /// The place `O` takes out: the commit, on any row.
    pub(crate) fn place(&self) -> (Place, String) {
        (
            Place::Commit {
                sha: self.sha.clone(),
            },
            format!("commit:{}", self.short()),
        )
    }
}

/// The pane: a header naming the commit, then the rows from the scroll down,
/// the cursor's on a band.
pub(crate) fn render_commit(app: &App, v: &CommitView, frame: &mut Frame, area: Rect) {
    let [head, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("\u{25c6} commit:{}", v.short()),
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  commit".to_string(), dim()),
        ])),
        head,
    );
    app.hits.borrow_mut().doc = Some(body);
    v.width.set(body.width);
    let rows = v.rows(body.width as usize);
    v.nrows.set(rows.len());
    if !v.placed.get() {
        v.cursor.set(CommitView::first_stop(&rows));
        v.placed.set(true);
    }
    let h = body.height as usize;
    let cursor = v.cursor.get().min(rows.len().saturating_sub(1));
    v.cursor.set(cursor);
    let mut top = v.scroll.get().min(rows.len().saturating_sub(1));
    if cursor < top {
        top = cursor;
    } else if h > 0 && cursor >= top + h {
        top = cursor + 1 - h;
    }
    v.scroll.set(top);
    let lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .skip(top)
        .take(h)
        .map(|(i, r)| {
            let used: usize = r.spans.iter().map(|(s, _)| disp_width(s)).sum();
            let mut spans: Vec<Span> = r
                .spans
                .iter()
                .map(|(s, st)| Span::styled(s.clone(), *st))
                .collect();
            if i == cursor {
                spans.push(Span::raw(
                    " ".repeat((body.width as usize).saturating_sub(used)),
                ));
            }
            let mut line = Line::from(spans);
            if i == cursor && r.item.is_some() {
                line = line.style(Style::new().bg(super::surface::selected()));
            }
            line
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), body);
}

impl App {
    /// The commit view, when the pane shows one.
    pub(crate) fn commit_view(&self) -> Option<&CommitView> {
        match &self.doc {
            Some(DocPane {
                view: DocView::Commit(v),
                ..
            }) => Some(v),
            _ => None,
        }
    }

    /// The landed note that squashed worker commit `sha` (a note's `source:`),
    /// as the commit it landed as. Notes are the truth; this reads them each
    /// time, only for yaks that mention the hash.
    fn landed_as(&self, sha: &str) -> Option<String> {
        let key = short(sha);
        self.all
            .iter()
            .filter(|t| t.body.contains(key))
            .flat_map(|t| crate::store::parse_notes(&t.body))
            .filter(|n| n.kind() == Some("landed"))
            .find_map(|n| {
                let src = n.get("source")?.trim();
                let same = src.len() >= 7 && (src.starts_with(sha) || sha.starts_with(src));
                let to = n.get("commit")?.trim();
                (same && !to.is_empty() && !to.starts_with(sha)).then(|| to.to_string())
            })
    }

    /// Read commit `sha` into a view. `Err` is a one-line message for the
    /// status bar: no farm, or no code checkout to look in. A commit the
    /// checkout lacks is not an error: the view says so.
    pub(crate) fn read_commit_view(&self, sha: &str) -> Result<CommitView, String> {
        let root = self
            .farm
            .as_ref()
            .map(|f| f.root().to_path_buf())
            .ok_or("no farm to resolve the link against")?;
        let base = source_base(&root, self.sheds.cwd.as_deref())
            .ok_or("no code checkout to resolve the link against")?;
        let space = Space::new(&root, Some(base.clone()));
        let state = match crate::gitrepo::commit_info(&base, sha) {
            Some(c) => State::Found(c),
            None => State::Missing(
                crate::locate::missing_commit(&base, sha)
                    .unwrap_or_else(|| format!("commit:{sha}: not in this repository")),
            ),
        };
        let full = match &state {
            State::Found(c) => c.sha.clone(),
            State::Missing(_) => sha.to_string(),
        };
        let host = space
            .web_url(&Place::Commit { sha: full.clone() }, false)
            .map(|w| w.url)
            .map_err(|e| format!("{:#}", e).lines().next().unwrap_or("").to_string());
        Ok(CommitView {
            landed_as: self.landed_as(&full),
            sha: full,
            state,
            host,
            yak: self.selected_id(),
            base,
            open: RefCell::new(Vec::new()),
            patches: RefCell::new(HashMap::new()),
            cursor: Cell::new(0),
            placed: Cell::new(false),
            scroll: Cell::new(0),
            nrows: Cell::new(0),
            width: Cell::new(80),
        })
    }

    /// Show commit `sha` in the pane, a jump (`o` comes back).
    pub(crate) fn open_commit(&mut self, sha: &str) {
        match self.read_commit_view(sha) {
            Ok(v) => {
                self.notification = Some(format!("viewing commit:{}", v.short()));
                self.follow_to_doc(DocView::Commit(v));
            }
            Err(e) => self.notification = Some(e),
        }
    }

    /// Follow a URL: the code repo's own host URLs read back as what they name
    /// (a commit opens the commit view, a file pinned to a commit the code
    /// viewer), everything else goes to the system opener.
    pub(crate) fn follow_url(&mut self, url: &str) {
        let space = self
            .farm
            .as_ref()
            .map(|f| Space::new(f.root(), source_base(f.root(), self.sheds.cwd.as_deref())));
        match space.map(|s| s.read_url(Link::Url(url.to_string()))) {
            Some(Link::Commit { sha }) => self.open_commit(&sha),
            Some(Link::Code { path, rev, anchor }) => {
                let lines = match anchor {
                    Some(links::Anchor::Lines(a, b)) => Some((a, b)),
                    _ => None,
                };
                self.open_code(&path, lines, rev.as_deref());
            }
            _ => self.open_external(url),
        }
    }

    /// Run a commit-view command. The global ones are `App::run_cmd`'s own.
    pub(super) fn run_commit_cmd(&mut self, cmd: Cmd) {
        let page = self.detail_page.saturating_sub(1).max(1) as i32;
        match cmd {
            Cmd::PanePeel | Cmd::PaneClose => return self.close_doc(),
            Cmd::NavBack => return self.history_back(),
            Cmd::NavForward => return self.history_forward(),
            Cmd::PaneEnter => return self.commit_enter(),
            Cmd::RowCopy => return self.commit_copy_hash(),
            _ => {}
        }
        let Some(v) = self.commit_view() else { return };
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

    /// Wheel over the commit: move the cursor `delta` rows.
    pub(crate) fn scroll_commit(&mut self, delta: i32) {
        if let Some(v) = self.commit_view() {
            v.step(delta);
        }
    }

    /// `Enter`: a file opens in the code viewer at this commit (a jump, so `o`
    /// comes back here); `Landed as` opens the landed commit.
    fn commit_enter(&mut self) {
        let Some(v) = self.commit_view() else { return };
        let sha = v.sha.clone();
        match v.selected() {
            Some(Item::File(path)) => self.open_code(&path, None, Some(&sha)),
            Some(Item::Patch(path, line)) => self.open_code(&path, Some((line, line)), Some(&sha)),
            Some(Item::Commit(to)) => self.open_commit(&to),
            None => {}
        }
    }

    /// `y`: copy the full hash.
    fn commit_copy_hash(&mut self) {
        let Some(v) = self.commit_view() else { return };
        let sha = v.sha.clone();
        self.notification = Some(if crate::clipboard::copy_text(&sha) {
            format!("copied {sha}")
        } else {
            "clipboard unavailable".into()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_lines_number_each_line_on_the_new_side() {
        let d = "@@ -3,4 +5,3 @@ fn x()\n ctx\n-gone\n+new\n ctx2\n\\ No newline at end of file\n@@ -20 +30,0 @@\n-tail";
        let got: Vec<(String, u32)> = patch_lines(d)
            .into_iter()
            .map(|p| (p.text, p.line))
            .collect();
        let want: Vec<(&str, u32)> = vec![
            ("@@ -3,4 +5,3 @@ fn x()", 5),
            (" ctx", 5),
            ("-gone", 6),
            ("+new", 6),
            (" ctx2", 7),
            ("\\ No newline at end of file", 8),
            ("@@ -20 +30,0 @@", 30),
            ("-tail", 30),
        ];
        let want: Vec<(String, u32)> = want.into_iter().map(|(t, n)| (t.to_string(), n)).collect();
        assert_eq!(got, want);
    }
}
