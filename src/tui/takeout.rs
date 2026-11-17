//! "Take it out" (yaks-c3d0): `O` opens a small modal menu for the thing under
//! the cursor, whatever it is: the file in the viewer, the link on a yak
//! pane's line, a Doks row, a yak row. It never changes a yak.
//!
//! - `e` the editor (`$VISUAL`/`$EDITOR`) at the line, with the TUI suspended;
//! - `o` the system opener;
//! - `p` copy the absolute path;
//! - `r` copy the farm- or repo-relative path;
//! - `g` copy the host link (GitHub, GitLab, Gitea/Forgejo).
//!
//! What it acts on is a [`Place`] ([`crate::locate`]): the same resolver
//! `yaks peek` and `yaks url` use, so a pinned `code:path@sha` file gives the
//! working-tree path when it is unchanged since the pin and a read-only temp
//! copy when not, and `g` gives the host's permalink. Nothing local is needed
//! for `g`; a commit this checkout has not got makes `p`/`e`/`o` say so in a
//! one-line notification ([`crate::locate::missing_commit`]), never an error
//! dialog and never a git call that waits.
//!
//! The menu is a modal [`Overlay::TakeOut`], so its keys are rows of
//! `keymap::EXTRAS`, not of the table; the key that opens it is the table's
//! `take.out`.

use super::detail::Target;
use super::handlers::DocKind;
use super::keymap::Ctx;
use super::source::{editor_command, editor_from_env, source_base};
use super::*;
use crate::links::{self, Anchor, Link};
use crate::locate::{Place, Space};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};

/// The menu's subject: what it takes out, and where the editor lands.
pub(crate) struct TakeOut {
    pub(crate) place: Place,
    /// The 1-based line `e` opens the editor at.
    pub(crate) line: Option<u32>,
    /// What the menu's title calls it.
    pub(crate) label: String,
}

/// The keys of the menu, in the order it lists them.
pub(crate) const KEYS: &str = "eopgr";

/// The menu's rows: key, then what it does.
pub(crate) const ROWS: [(char, &str); 5] = [
    ('e', "editor at the line ($VISUAL / $EDITOR)"),
    ('o', "system open"),
    ('p', "copy the absolute path"),
    ('r', "copy the farm- or repo-relative path"),
    ('g', "copy the host link (GitHub, GitLab, Gitea)"),
];

#[cfg(test)]
thread_local! {
    static COPIED: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Put `text` on the system clipboard. Under `cargo test` it is remembered
/// for [`copied`] instead, so a test never touches the developer's clipboard.
fn copy(text: &str) -> bool {
    #[cfg(test)]
    {
        COPIED.with(|c| *c.borrow_mut() = Some(text.to_string()));
        true
    }
    #[cfg(not(test))]
    crate::clipboard::copy_text(text)
}

/// What the last `p`, `r` or `g` copied on this thread (tests only).
#[cfg(test)]
pub(crate) fn copied() -> Option<String> {
    COPIED.with(|c| c.borrow().clone())
}

/// A pane link as the resolver reads it. `None` for what names nothing to
/// take out (a footnote, an in-page anchor).
fn target_link(t: &Target) -> Option<Link> {
    match t {
        Target::Task(id) => Some(Link::Yak {
            id: id.clone(),
            note: None,
        }),
        Target::Note { id, anchor } => Some(Link::Yak {
            id: id.clone(),
            note: Some(anchor.clone()),
        }),
        Target::Url(u) => Some(Link::Url(u.clone())),
        other => other.as_link(),
    }
}

/// The first line a place's anchor names.
fn anchor_line(place: &Place) -> Option<u32> {
    match place {
        Place::CodeFile {
            anchor: Some(Anchor::Lines(a, _)),
            ..
        }
        | Place::FarmFile {
            anchor: Some(Anchor::Lines(a, _)),
            ..
        } => Some(*a),
        _ => None,
    }
}

impl App {
    /// What links resolve against: the farm and the code checkout.
    fn take_space(&self) -> Option<Space> {
        let root = self.farm.as_ref()?.root().to_path_buf();
        let code = source_base(&root, self.sheds.cwd.as_deref());
        Some(Space::new(&root, code))
    }

    /// `O`: open the menu for what is under the cursor on surface `ctx`, or
    /// say there is nothing to take out.
    pub(crate) fn open_take_out(&mut self, ctx: Ctx) {
        let Some(space) = self.take_space() else {
            self.notification = Some("no farm to take anything out of".into());
            return;
        };
        let subject = match ctx {
            Ctx::Viewer => self.viewer_subject(&space),
            Ctx::Image => self.image_subject(&space),
            Ctx::Dir => self.dir_subject(),
            Ctx::Detail => self.pane_subject(&space),
            Ctx::List => self.row_subject(),
        };
        match subject {
            Some(Ok(t)) => self.overlay = Overlay::TakeOut(t),
            Some(Err(why)) => self.notification = Some(why),
            None => self.notification = Some("nothing to take out here".into()),
        }
    }

    /// The farm-relative path of `file`, a path under the farm root.
    fn farm_relative(&self, file: &Path) -> Option<String> {
        let root = self.farm.as_ref()?.root();
        file.strip_prefix(root)
            .ok()
            .map(|r| r.to_string_lossy().replace('\\', "/"))
    }

    /// The open text viewer's file; in a code file, the cursor's line (or
    /// the selected lines) for `e` and `g`.
    fn viewer_subject(&self, space: &Space) -> Option<Result<TakeOut, String>> {
        let v = self.attach()?;
        let (rows, _) = self.attachment_rows()?;
        let line = v.source_line(&rows);
        let (place, label) = match v.code_rel() {
            Some(rel) => {
                let anchor = match v.nav.selection() {
                    Some((lo, hi)) => Some(Anchor::Lines(lo as u32 + 1, hi as u32 + 1)),
                    None => line.map(|n| Anchor::Lines(n, n)),
                };
                let place = Place::CodeFile {
                    rel: rel.to_string(),
                    rev: v.rev.clone(),
                    anchor,
                };
                let label = Space::link(&place).map(|l| links::label(&l));
                (place, label.unwrap_or_else(|| v.target.clone()))
            }
            None => {
                let rel = match v.doc {
                    // An attachment's target is `<id>/<file>`; its file is
                    // somewhere under the farm.
                    DocKind::Attachment => match self.farm_relative(&v.path) {
                        Some(r) => r,
                        None => return Some(Err(format!("{}: not a file of the farm", v.target))),
                    },
                    _ => v.target.clone(),
                };
                let pin = v
                    .rev
                    .as_ref()
                    .map(|r| format!("@{}", &r[..r.len().min(7)]))
                    .unwrap_or_default();
                let label = format!("{}{pin}", v.target);
                (
                    Place::FarmFile {
                        rel,
                        rev: v.rev.clone(),
                        anchor: None,
                    },
                    label,
                )
            }
        };
        let _ = space;
        Some(Ok(TakeOut { place, line, label }))
    }

    /// The directory view's selected file. A directory has no file to take
    /// out: `y` copies its path.
    fn dir_subject(&self) -> Option<Result<TakeOut, String>> {
        let v = self.dir()?;
        let row = v.selected()?;
        if row.is_dir {
            return Some(Err(format!(
                "{}/: a directory; y copies its path",
                row.name
            )));
        }
        let rel = v.selected_rel()?;
        Some(Ok(TakeOut {
            place: Place::FarmFile {
                rel: rel.clone(),
                rev: None,
                anchor: None,
            },
            line: None,
            label: rel,
        }))
    }

    /// The open image's file.
    fn image_subject(&self, _space: &Space) -> Option<Result<TakeOut, String>> {
        let v = self.image()?;
        let Some(rel) = self.farm_relative(&v.path) else {
            return Some(Err(format!("{}: not a file of the farm", v.target)));
        };
        Some(Ok(TakeOut {
            place: Place::FarmFile {
                rel,
                rev: None,
                anchor: None,
            },
            line: None,
            label: v.target.clone(),
        }))
    }

    /// A yak pane: the link on the cursor's line, else the yak itself.
    fn pane_subject(&self, space: &Space) -> Option<Result<TakeOut, String>> {
        if let Some(link) = self.detail_link().and_then(|j| target_link(&j.target)) {
            let place = space.resolve(&link);
            return Some(Ok(TakeOut {
                line: anchor_line(&place),
                label: links::label(&link),
                place,
            }));
        }
        let id = self.selected_id()?;
        Some(Ok(TakeOut {
            place: Place::Yak {
                id: id.clone(),
                note: None,
            },
            line: None,
            label: id,
        }))
    }

    /// The list's row: a dok or source, or a yak.
    fn row_subject(&self) -> Option<Result<TakeOut, String>> {
        if let Some(e) = self.selected_doc() {
            return Some(Ok(TakeOut {
                place: Place::FarmFile {
                    rel: e.path.clone(),
                    rev: None,
                    anchor: None,
                },
                line: None,
                label: e.shown_path().to_string(),
            }));
        }
        let id = self.selected_id()?;
        Some(Ok(TakeOut {
            place: Place::Yak {
                id: id.clone(),
                note: None,
            },
            line: None,
            label: id,
        }))
    }

    /// One key in the open menu: a menu key acts and closes it, `Esc` or `q`
    /// closes it, any other key leaves it up.
    pub(crate) fn take_out_key(&mut self, t: TakeOut, k: KeyEvent) {
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => self.notification = Some("cancelled".into()),
            KeyCode::Char(c) if KEYS.contains(c) => {
                self.take_out_with(editor_from_env().as_deref(), &t, c)
            }
            _ => self.overlay = Overlay::TakeOut(t),
        }
    }

    /// Run menu key `c` on `t`, with `editor` the `$VISUAL`/`$EDITOR` text.
    /// Every outcome is a notification; the editor is queued for the event
    /// loop ([`App::pending_editor`]).
    pub(crate) fn take_out_with(&mut self, editor: Option<&str>, t: &TakeOut, c: char) {
        let Some(space) = self.take_space() else {
            return;
        };
        let place = &t.place;
        self.notification = Some(match c {
            'e' => {
                let Some(editor) = editor else {
                    self.notification =
                        Some("no $VISUAL or $EDITOR is set (o opens it with the system)".into());
                    return;
                };
                match space.local_path(place) {
                    Err(why) => why,
                    Ok((file, snap)) => match editor_command(editor, &file, t.line) {
                        Some(cmd) => {
                            self.pending_editor = Some(cmd);
                            format!("editing {}{}", t.label, copy_note(snap))
                        }
                        None => "no $VISUAL or $EDITOR is set (o opens it with the system)".into(),
                    },
                }
            }
            'o' => {
                if let Place::Url(u) = place {
                    self.os_open(u, u);
                    return;
                }
                match space.local_path(place) {
                    Err(why) => why,
                    Ok((file, snap)) => {
                        self.os_open(&t.label, &file.display().to_string());
                        if let Some(n) = self.notification.as_mut() {
                            n.push_str(copy_note(snap));
                        }
                        return;
                    }
                }
            }
            'p' => match space.local_path(place) {
                Err(why) => why,
                Ok((file, snap)) => {
                    let path = file.display().to_string();
                    copied_note(copy(&path), &path, copy_note(snap))
                }
            },
            'r' => match space.rel_path(place) {
                None => "this link names no file".into(),
                Some(rel) => copied_note(copy(&rel), &rel, ""),
            },
            'g' => match space.web_url(place, false) {
                Err(e) => host_problem(&format!("{e:#}")),
                Ok(w) => {
                    let note = w.note.map(|n| format!(" ({n})")).unwrap_or_default();
                    copied_note(copy(&w.url), &w.url, &note)
                }
            },
            _ => return,
        });
    }
}

/// What a read-only copy adds to a message.
fn copy_note(snapshot: bool) -> &'static str {
    if snapshot {
        " (read-only copy at the pinned commit)"
    } else {
        ""
    }
}

fn copied_note(ok: bool, what: &str, then: &str) -> String {
    if ok {
        format!("copied {what}{then}")
    } else {
        format!("clipboard unavailable: {what}{then}")
    }
}

/// `g`'s refusal as one line that names `git config yaks.host`: the
/// resolver's message for an unknown domain already does; a missing remote
/// gets the pointer added.
fn host_problem(why: &str) -> String {
    let why = why.lines().next().unwrap_or(why).trim_end_matches('.');
    if why.contains("yaks.host") || !why.contains("remote") {
        why.to_string()
    } else {
        format!("{why} (a custom domain: `git config yaks.host github|gitlab|gitea`)")
    }
}

/// The menu, over the foot of the main area like the `?` popup.
pub(crate) fn render(t: &TakeOut, frame: &mut Frame, mid: Rect) {
    let height = (ROWS.len() as u16 + 2).min(mid.height);
    let area = Rect {
        y: mid.y + mid.height - height,
        height,
        ..mid
    };
    let title = format!(
        " Take out · {} ",
        super::render::truncate_disp(&t.label, 60)
    );
    let block = Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(Color::Cyan))
        .padding(Padding::horizontal(1))
        .title(Span::styled(
            title,
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    let lines: Vec<Line> = ROWS
        .iter()
        .map(|(key, what)| {
            Line::from(vec![
                Span::styled(format!("{key}  "), Style::new().fg(Color::Yellow)),
                Span::raw(*what),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;
    use crate::tui::test_support::*;
    use std::process::Command;

    fn git_ok(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {out:?}");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    const RUST: &str = "// a comment\nfn main() {\n    let n = 42;\n    println!(\"hi\");\n}\n";

    /// A code checkout (the farm's parent) with `src/a.rs` committed, and a
    /// yak `t0` whose body links `links`. Returns the checkout, the pinned
    /// commit and the app with the yak's pane open and measured.
    fn on_links(links: &str) -> (PathBuf, String, App) {
        let mut t = task("t0", "solo", Status::Hairy, 3, None);
        t.body = links.to_string();
        let (proj, farm) = temp_farm(&[t]);
        std::fs::create_dir_all(proj.join("src")).unwrap();
        std::fs::write(proj.join("src/a.rs"), RUST).unwrap();
        std::fs::write(proj.join("notes.txt"), "plain\ntext\n").unwrap();
        git_ok(&proj, &["init", "-q", "-b", "main"]);
        git_ok(&proj, &["add", "src", "notes.txt"]);
        git_ok(&proj, &["commit", "-q", "-m", "first"]);
        let rev = git_ok(&proj, &["rev-parse", "--short=10", "HEAD"]);
        let mut app = App::with_farm(farm).unwrap();
        enter_key(&mut app);
        draw(&app, 100, 24);
        (proj, rev, app)
    }

    /// Tab onto the pane's first link and press `keys`.
    fn tab_and(app: &mut App, keys: &str) {
        tab_key(app);
        draw(app, 100, 24);
        press(app, keys);
    }

    fn rows_of(app: &App) -> std::rc::Rc<Vec<crate::tui::detail::DLine>> {
        app.attachment_rows().expect("the viewer is open").0
    }

    #[test]
    fn enter_on_a_code_link_opens_the_viewer_at_its_lines_coloured() {
        let (proj, _, mut app) = on_links("See [x](code:src/a.rs#L3-L4).\n");
        tab_key(&mut app);
        enter_key(&mut app);
        let v = app.attach().expect("a viewer, not an editor");
        assert_eq!(v.target, "code:src/a.rs");
        assert!(app.pending_editor.is_none());
        let frame = draw(&app, 100, 24);
        assert!(frame.contains("{ } code:src/a.rs  code"), "{frame}");
        assert!(frame.contains("3 │     let n = 42;"), "{frame}");
        // Landed on line 3 (row 2), at the top of the page.
        assert_eq!(app.attach().unwrap().nav.line, 2);
        // Coloured by extension: a keyword and a number carry the palette's
        // styles; the gutter of the named lines is the accent.
        let rows = rows_of(&app);
        let styles = |row: usize| -> Vec<ratatui::style::Style> {
            rows[row].md.iter().map(|s| s.style).collect()
        };
        assert!(styles(2).contains(&crate::tui::palette::syntax_keyword()));
        assert!(styles(2).contains(&crate::tui::palette::syntax_number()));
        assert_eq!(styles(2)[0], crate::tui::palette::line_mark());
        assert_eq!(styles(0)[0], crate::tui::palette::line_number());
        assert!(styles(0).contains(&crate::tui::palette::syntax_comment()));
        // Esc comes back to the yak.
        esc_key(&mut app);
        assert!(app.attach().is_none());
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn plain_text_has_no_colour_but_its_gutter() {
        let (proj, _, mut app) = on_links("See [x](code:notes.txt).\n");
        tab_key(&mut app);
        enter_key(&mut app);
        let rows = rows_of(&app);
        assert_eq!(rows[0].text, "1 │ plain");
        assert_eq!(rows[0].md.len(), 1, "only the gutter: {:?}", rows[0].md);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_pinned_code_link_shows_the_file_as_it_was_then() {
        let (proj, rev, mut app) = on_links("See [x](code:src/a.rs).\n");
        std::fs::write(proj.join("src/a.rs"), "changed\n").unwrap();
        app.open_code("src/a.rs", Some((2, 2)), Some(&rev));
        let v = app.attach().unwrap();
        assert_eq!(v.rev.as_deref(), Some(rev.as_str()));
        let frame = draw(&app, 100, 24);
        assert!(frame.contains("code:src/a.rs@"), "{frame}");
        assert!(
            frame.contains("fn main()"),
            "git show, not the work file: {frame}"
        );
        assert!(!frame.contains("changed"), "{frame}");
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_commit_this_checkout_lacks_is_one_line_never_a_blank_pane_or_panic() {
        let (proj, _, mut app) = on_links("See [x](code:src/a.rs@deadbeefdeadbeef#L2).\n");
        tab_key(&mut app);
        enter_key(&mut app);
        let frame = draw(&app, 100, 24);
        assert!(
            frame.contains("commit deadbee is not in this checkout; fetch it or open on the host"),
            "{frame}"
        );
        // Every key still works on it, and a file the commit lacks says so.
        press(&mut app, "jkGg");
        app.open_code("nope.rs", None, None);
        assert!(draw(&app, 100, 24).contains("nope.rs: no such file"));
        let rev = git_ok(&proj, &["rev-parse", "HEAD"]);
        app.open_code("src/gone.rs", None, Some(&rev));
        assert!(draw(&app, 100, 24).contains("src/gone.rs is not in"));
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn o_opens_the_menu_from_the_pane_the_viewer_a_doks_row_and_a_yak_row() {
        let (proj, _, mut app) = on_links("See [x](code:src/a.rs#L3).\n");
        // The yak row, from the list.
        esc_key(&mut app);
        press(&mut app, "O");
        assert!(matches!(&app.overlay, Overlay::TakeOut(t) if t.label == "t0"));
        let frame = draw(&app, 100, 24);
        assert!(frame.contains("Take out · t0"), "{frame}");
        assert!(frame.contains("p  copy the absolute path"), "{frame}");
        assert!(frame.contains("e editor · o system open"), "{frame}");
        press(&mut app, "x");
        assert!(matches!(app.overlay, Overlay::TakeOut(_)), "x keeps it up");
        esc_key(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        // The link on the pane's line.
        enter_key(&mut app);
        draw(&app, 100, 24);
        tab_and(&mut app, "O");
        let Overlay::TakeOut(t) = &app.overlay else {
            panic!("no menu");
        };
        assert_eq!(t.label, "code:src/a.rs:L3");
        assert_eq!(t.line, Some(3));
        esc_key(&mut app);
        // Enter on the same link still follows it, into the viewer.
        enter_key(&mut app);
        assert!(app.attach().is_some());
        // The viewer: the cursor's line.
        press(&mut app, "j");
        press(&mut app, "O");
        let Overlay::TakeOut(t) = &app.overlay else {
            panic!("no menu");
        };
        assert_eq!((t.line, t.label.as_str()), (Some(4), "code:src/a.rs:L4"));
        esc_key(&mut app);
        // A Doks row.
        esc_key(&mut app);
        crate::tui::doks_tab::fixture::write(app.farm.as_ref().unwrap().root());
        app.reload_doks();
        let i = app
            .views
            .iter()
            .position(|v| v.key == "doks")
            .expect("a Doks view");
        app.set_view(i);
        press(&mut app, "O");
        assert!(
            matches!(&app.overlay, Overlay::TakeOut(t) if matches!(t.place, Place::FarmFile { .. })),
            "a Doks row opens it"
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn the_menu_keys_copy_paths_and_open() {
        let (proj, rev, mut app) = on_links("See [x](code:src/a.rs#L3).\n");
        let abs = crate::gitrepo::canonical(&proj).join("src/a.rs");
        let mut run = |app: &mut App, c: char| -> String {
            tab_key(app);
            press(app, "O");
            press(app, &c.to_string());
            assert!(matches!(app.overlay, Overlay::None), "{c} closes the menu");
            app.notification.clone().unwrap()
        };
        let n = run(&mut app, 'r');
        assert_eq!(n, "copied src/a.rs");
        assert_eq!(copied().as_deref(), Some("src/a.rs"));
        let n = run(&mut app, 'p');
        assert!(n.starts_with("copied ") && n.ends_with("src/a.rs"), "{n}");
        let got = copied().unwrap();
        assert_eq!(
            crate::gitrepo::canonical(Path::new(&got)),
            abs,
            "the working file, not a temp copy"
        );
        let n = run(&mut app, 'o');
        assert_eq!(n, "opened code:src/a.rs:L3");
        // `e` needs an editor: with none set it says so.
        let t = TakeOut {
            place: Place::CodeFile {
                rel: "src/a.rs".into(),
                rev: None,
                anchor: None,
            },
            line: Some(3),
            label: "code:src/a.rs:L3".into(),
        };
        app.take_out_with(None, &t, 'e');
        assert!(app.pending_editor.is_none());
        assert!(
            app.notification.as_deref().unwrap().contains("$EDITOR"),
            "{:?}",
            app.notification
        );
        app.take_out_with(Some("vim"), &t, 'e');
        let cmd = app.pending_editor.take().unwrap();
        assert_eq!(cmd.args[0], "+3");
        // Pinned and unchanged: the working file; changed: a read-only copy.
        let pinned = TakeOut {
            place: Place::CodeFile {
                rel: "src/a.rs".into(),
                rev: Some(rev),
                anchor: None,
            },
            line: None,
            label: "pin".into(),
        };
        app.take_out_with(None, &pinned, 'p');
        assert_eq!(
            crate::gitrepo::canonical(Path::new(&copied().unwrap())),
            abs
        );
        std::fs::write(proj.join("src/a.rs"), "edited\n").unwrap();
        app.take_out_with(None, &pinned, 'p');
        let n = app.notification.clone().unwrap();
        assert!(
            n.contains("yaks-src") && n.contains("read-only copy"),
            "{n}"
        );
        assert_eq!(
            std::fs::read_to_string(copied().unwrap()).unwrap(),
            RUST,
            "the copy is the pinned content"
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn g_is_the_host_link_or_one_line_naming_yaks_host() {
        let (proj, rev, mut app) = on_links("See [x](code:src/a.rs#L3).\n");
        let t = |rev: Option<String>| TakeOut {
            place: Place::CodeFile {
                rel: "src/a.rs".into(),
                rev,
                anchor: Some(Anchor::Lines(3, 4)),
            },
            line: None,
            label: "x".into(),
        };
        // No remote: why, and the setting; a notification, no overlay.
        app.take_out_with(None, &t(None), 'g');
        let n = app.notification.clone().unwrap();
        assert!(n.contains("no remote") && n.contains("yaks.host"), "{n}");
        assert!(!n.contains('\n') && matches!(app.overlay, Overlay::None));
        // An unknown domain.
        git_ok(
            &proj,
            &["remote", "add", "origin", "https://git.example.com/o/r.git"],
        );
        git_ok(&proj, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
        app.take_out_with(None, &t(None), 'g');
        let n = app.notification.clone().unwrap();
        assert!(
            n.contains("not a recognised host") && n.contains("yaks.host"),
            "{n}"
        );
        // GitHub.
        git_ok(
            &proj,
            &["remote", "set-url", "origin", "https://github.com/o/r.git"],
        );
        app.take_out_with(None, &t(None), 'g');
        assert_eq!(
            copied().as_deref(),
            Some("https://github.com/o/r/blob/main/src/a.rs#L3-L4")
        );
        // A pin: the permalink, with the commit as written.
        app.take_out_with(None, &t(Some(rev.clone())), 'g');
        assert!(
            copied()
                .unwrap()
                .starts_with("https://github.com/o/r/blob/"),
            "{:?}",
            copied()
        );
        // A commit this checkout lacks: `g` still gives the URL, and says
        // the remote has not got it as far as this checkout knows.
        let ghost = "deadbeefdeadbeef".to_string();
        app.take_out_with(None, &t(Some(ghost.clone())), 'g');
        assert_eq!(
            copied().unwrap(),
            format!("https://github.com/o/r/blob/{ghost}/src/a.rs#L3-L4")
        );
        let n = app.notification.clone().unwrap();
        assert!(n.contains("deadbee") && n.contains("not on origin"), "{n}");
        // `p`, `e` and `o` on it say the commit is missing.
        for c in ['p', 'e', 'o'] {
            app.take_out_with(Some("vim"), &t(Some(ghost.clone())), c);
            assert_eq!(
                app.notification.as_deref(),
                Some("commit deadbee is not in this checkout; fetch it or open on the host"),
                "{c}"
            );
        }
        assert!(app.pending_editor.is_none());
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn the_code_viewer_is_read_only() {
        let (proj, _, mut app) = on_links("See [x](code:src/a.rs).\n");
        tab_key(&mut app);
        enter_key(&mut app);
        press(&mut app, "E");
        assert!(app.attach().is_some(), "no editor opened");
        assert!(matches!(app.overlay, Overlay::None));
        assert!(
            app.notification.as_deref().unwrap().contains("read-only"),
            "{:?}",
            app.notification
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn o_back_from_a_code_file_returns_to_the_yak() {
        let (proj, _, mut app) = on_links("See [x](code:src/a.rs#L3).\n");
        tab_key(&mut app);
        enter_key(&mut app);
        assert!(app.attach().is_some());
        press(&mut app, "o");
        assert!(app.attach().is_none());
        press(&mut app, "i");
        assert_eq!(
            app.attach().map(|v| v.target.as_str()),
            Some("code:src/a.rs")
        );
        let _ = std::fs::remove_dir_all(proj);
    }
}
