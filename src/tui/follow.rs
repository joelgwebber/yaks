//! Following links to farm documents, and the peek preview (yaks-7395).
//!
//! Every markdown link is classified by the farm's link grammar
//! ([`crate::links::parse`], through `mdrender::link_target`), so the TUI and
//! `yaks peek`/`doctor` agree on what a link means. This module is the
//! TUI's opening side for the targets that grammar adds:
//!
//! - **A farm document** ([`Target::Farm`], `/doks/x.md#setup`, a dok-relative
//!   `notes.md`, `/sources/<id>/source.md`, a pinned `...@<sha>`): opens in
//!   the text viewer ([`App::open_farm_doc`]) at its `#heading` or `#L12`
//!   row. A pinned one shows the file as it was at that farm commit.
//! - **An in-document anchor** ([`Target::Local`], `#section`): moves the
//!   viewer's cursor to that heading (`o` returns).
//!
//! **History.** Following a link, to a document or within one, is a jump of
//! the app's one history ([`super::history`]): `o` goes back, re-reading a
//! document from disk ([`App::read_doc`]); `i` goes forward.
//!
//! **Peek.** While the cursor of the viewer or of a yak's detail pane is on a
//! link it can read, a pane at the bottom shows `links::peek` of it (the
//! section, the line range, or the document's head), cut to fit; `p` hides and
//! shows it, in both, by the one flag `App::peek_shown`. Peeks are cached by
//! link and farm generation ([`PeekCache`]), so a frame reads each target
//! once.

use super::detail::{self, Target};
use super::handlers::{AttachView, DocKind};
use super::imageview::ImageView;
use super::linenav::Viewport;
use super::*;
use crate::links::{self, Anchor, Link};

/// Largest farm file the viewer reads (as for attachments).
const VIEW_MAX_BYTES: u64 = 1 << 20;

/// Rows the peek pane takes, in the viewer and in the detail pane: a rule and
/// the peek's text.
pub(crate) const PEEK_ROWS: u16 = 7;

/// Peeks already read, by link label, for the farm generation they were read
/// at (`App::data_gen`): a reload drops them.
#[derive(Default)]
pub(crate) struct PeekCache {
    at_gen: u64,
    map: HashMap<String, Result<String, String>>,
}

/// The row a heading or line anchor lands on in a viewer's rows: the first
/// row whose source covers the heading's line (a heading in block `block`).
fn anchor_row(v: &AttachView, rows: &[detail::DLine], anchor: &Anchor) -> Option<usize> {
    // Within a dok or source, block 0 is the body below the frontmatter;
    // `#L12` counts lines of the whole file, so shift past the frontmatter.
    let body = v.blocks()[0];
    let front_lines = v.text[..v.text.len() - body.len()].lines().count();
    let line = match anchor {
        Anchor::Lines(a, _) => (*a as usize).saturating_sub(1).checked_sub(front_lines)?,
        Anchor::Heading(h) => {
            let (_, _, r) = links::sections(body).into_iter().find(|(_, s, _)| s == h)?;
            body[..r.start].lines().count()
        }
    };
    rows.iter()
        .position(|r| {
            r.src.is_some_and(|s| {
                s.block == 0 && s.lines.is_some_and(|(a, b)| a <= line && line <= b)
            })
        })
        .or_else(|| {
            // A line past every rendered row (trailing blanks): the last row of
            // the block.
            rows.iter().rposition(|r| {
                r.src
                    .is_some_and(|s| s.block == 0 && s.lines.is_some_and(|(_, b)| b < line))
            })
        })
}

impl App {
    /// The farm root, when there is a farm.
    fn farm_root(&self) -> Option<PathBuf> {
        self.farm.as_ref().map(|f| f.root().to_path_buf())
    }

    /// Read farm file `path` (at farm commit `rev`) into a viewer. A
    /// `source.md` brings its `extracted.md` along. `Err` is a one-line
    /// message for the status bar.
    pub(crate) fn read_farm_doc(
        &self,
        path: &str,
        rev: Option<&str>,
    ) -> Result<AttachView, String> {
        let root = self
            .farm_root()
            .ok_or_else(|| format!("{path}: no farm to read"))?;
        let file = root.join(path);
        if rev.is_none() {
            if !file.is_file() {
                return Err(format!("{path}: not in this farm"));
            }
            if std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0) > VIEW_MAX_BYTES {
                return Err(format!("{path}: too big to view here (O opens it)"));
            }
        }
        let text = links::read_at(&root, path, rev).map_err(|e| format!("{e:#}"))?;
        if text.contains('\0') || text.starts_with("(binary file") {
            return Err(format!("{path}: not a text file"));
        }
        let extracted = path
            .strip_suffix("/source.md")
            .filter(|d| d.starts_with("sources/"))
            .and_then(|d| links::read_at(&root, &format!("{d}/extracted.md"), rev).ok());
        Ok(AttachView::farm_doc(
            path,
            file,
            text,
            rev.map(String::from),
            extracted,
        ))
    }

    /// The text viewer's document, when the pane shows one.
    pub(crate) fn attach(&self) -> Option<&AttachView> {
        match &self.doc {
            Some(DocPane {
                view: DocView::Text(v),
                ..
            }) => Some(v),
            _ => None,
        }
    }

    pub(crate) fn attach_mut(&mut self) -> Option<&mut AttachView> {
        match &mut self.doc {
            Some(DocPane {
                view: DocView::Text(v),
                ..
            }) => Some(v),
            _ => None,
        }
    }

    /// The image viewer's picture, when the pane shows one.
    pub(crate) fn image(&self) -> Option<&ImageView> {
        match &self.doc {
            Some(DocPane {
                view: DocView::Image(v),
                ..
            }) => Some(v),
            _ => None,
        }
    }

    /// Show `view` in the pane and focus it. The pane closes back to where
    /// the first document was opened from (the list, or a yak's detail); a
    /// document replacing another keeps that. Returns the view it replaced.
    pub(crate) fn show_doc(&mut self, view: DocView) -> Option<DocView> {
        let (from, old) = match self.doc.take() {
            Some(d) => (d.from, Some(d.view)),
            None => (self.focus, None),
        };
        self.doc = Some(DocPane { view, from });
        self.focus = Focus::Detail;
        old
    }

    /// Show a document followed from a link: a jump, so `o` comes back to
    /// where the link was (the dok open now, or the yak or list row under
    /// it).
    pub(crate) fn follow_to_doc(&mut self, view: DocView) {
        self.jump(|app| {
            app.show_doc(view);
        });
    }

    /// Close the pane's document: focus goes back to where it was opened
    /// from. The history keeps the doks it passed through: `o` still goes
    /// back to them.
    pub(crate) fn close_doc(&mut self) {
        if let Some(d) = self.doc.take() {
            self.focus = d.from;
        }
    }

    /// The picture at farm file `path`, in the image viewer's form (`None`
    /// when it is not an image, not there, or cannot be decoded).
    fn farm_image(&self, path: &str) -> Option<ImageView> {
        let found = crate::farm::FoundAttachment {
            name: path.rsplit('/').next().unwrap_or(path).to_string(),
            path: self.farm_root()?.join(path),
        };
        if !crate::store::is_image_name(path) || !found.path.is_file() {
            return None;
        }
        ImageView::open(path, &found, &self.picker)
    }

    /// A document as the pane showed it, read again from disk: `path` is its
    /// name as the viewer gives it (`<id>/<file>` for an attachment, else a
    /// farm path) and `rev` the farm commit of a pinned one. What
    /// [`super::history`] restores a dok by.
    pub(crate) fn read_doc(&self, path: &str, rev: Option<&str>) -> Result<DocView, String> {
        // A farm file is under the farm root; an attachment is not (it lives
        // under its yak), so the path tells them apart.
        if rev.is_some() || self.farm_root().is_some_and(|r| r.join(path).is_file()) {
            if rev.is_none() {
                if let Some(v) = self.farm_image(path) {
                    return Ok(DocView::Image(v));
                }
            }
            return self.read_farm_doc(path, rev).map(DocView::Text);
        }
        let found = self
            .resolve_attachment(path)
            .ok_or_else(|| format!("{path}: not in this farm"))?;
        if let Some(v) = ImageView::open(path, &found, &self.picker) {
            return Ok(DocView::Image(v));
        }
        AttachView::open(path, &found)
            .map(DocView::Text)
            .ok_or_else(|| format!("{path}: cannot be shown here"))
    }

    /// Open farm file `path` in the viewer at `anchor`, a jump. An image
    /// opens in the image viewer.
    pub(crate) fn open_farm_doc(&mut self, path: &str, rev: Option<&str>, anchor: Option<&Anchor>) {
        if rev.is_none() {
            if let Some(v) = self.farm_image(path) {
                self.follow_to_doc(DocView::Image(v));
                self.notification = Some(format!("viewing {path}"));
                return;
            }
        }
        let v = match self.read_farm_doc(path, rev) {
            Ok(v) => v,
            Err(e) => {
                self.notification = Some(e);
                return;
            }
        };
        self.follow_to_doc(DocView::Text(v));
        self.scroll_to_anchor(anchor);
        let label = links::label(&Link::Farm {
            path: path.to_string(),
            rev: rev.map(String::from),
            anchor: anchor.cloned(),
        });
        self.notification = Some(format!("viewing {label}"));
    }

    /// Put the open viewer's cursor on `anchor`'s row (when the anchor is
    /// found; else a notification says so).
    fn scroll_to_anchor(&mut self, anchor: Option<&Anchor>) {
        let Some(anchor) = anchor else { return };
        let Some((rows, vp)) = self.attachment_rows() else {
            return;
        };
        let Some(v) = self.attach_mut() else {
            return;
        };
        match anchor_row(v, &rows, anchor) {
            Some(to) => {
                // The anchor's row at the top of the dok, as a browser does.
                v.nav.line = to;
                v.nav.scroll = to.min(vp.max_scroll(rows.len())) as u16;
            }
            None => {
                let what = links::label(&Link::Local(anchor.clone()));
                let note = format!("{}: no {what}", v.target);
                self.notification = Some(note);
            }
        }
    }

    /// Enter on an in-document anchor in the viewer: move to its heading, a
    /// jump (`o` returns).
    pub(crate) fn follow_local_anchor(
        &mut self,
        rows: &[detail::DLine],
        vp: Viewport,
        anchor: &Anchor,
    ) {
        let Some(v) = self.attach() else {
            return;
        };
        let what = links::label(&Link::Local(anchor.clone()));
        match anchor_row(v, rows, anchor) {
            Some(to) => {
                self.jump_in_dok(rows, vp, to);
                self.notification = Some(format!("→ {what} (o to go back)"));
            }
            None => self.notification = Some(format!("no {what} in {}", v.target)),
        }
    }

    /// Move the open dok's cursor to row `to`: a jump (a footnote, a
    /// `#section`), so `o` returns to where it was.
    pub(crate) fn jump_in_dok(&mut self, rows: &[detail::DLine], vp: Viewport, to: usize) {
        self.jump(|app| {
            if let Some(v) = app.attach_mut() {
                v.nav.goto(rows, vp, to);
            }
        });
    }

    // -- peek ---------------------------------------------------------------

    /// `links::peek` of `target`, from the cache when this farm generation
    /// already read it. `None` for a target peek can't read (a yak id, a
    /// URL, a footnote); an in-document anchor is peeked in `doc`, the
    /// viewer's own farm file.
    pub(crate) fn peek_target(
        &self,
        target: &Target,
        doc: Option<&AttachView>,
    ) -> Option<(String, Result<String, String>)> {
        let link = match (target, doc) {
            (Target::Local(a), Some(v)) if v.doc != DocKind::Attachment => Link::Farm {
                path: v.target.clone(),
                rev: v.rev.clone(),
                anchor: Some(a.clone()),
            },
            _ => target.as_link()?,
        };
        let label = links::label(&link);
        let root = self.farm_root()?;
        {
            let cache = self.peeks.borrow();
            if cache.at_gen == self.data_gen {
                if let Some(r) = cache.map.get(&label) {
                    return Some((label, r.clone()));
                }
            }
        }
        let code = links::code_base(&root, self.sheds.cwd.as_deref());
        let bases = links::Bases {
            farm: &root,
            code: code.as_deref(),
        };
        let r = links::peek(&link, &bases, 0)
            .map(|p| p.text)
            .map_err(|e| format!("{e:#}"));
        let mut cache = self.peeks.borrow_mut();
        if cache.at_gen != self.data_gen {
            cache.at_gen = self.data_gen;
            cache.map.clear();
        }
        cache.map.insert(label.clone(), r.clone());
        Some((label, r))
    }

    /// Whether the open viewer shows a peek pane: it is not hidden (`p`),
    /// there is a farm to read from, and the document has a link peek can
    /// read. Reserved whenever such a link exists, so the dok geometry does
    /// not jump as the cursor moves.
    pub(crate) fn viewer_has_peek(&self, rows: &[detail::DLine]) -> bool {
        let Some(v) = self.attach() else {
            return false;
        };
        self.peek_shown
            && self.farm.is_some()
            && rows.iter().any(|r| {
                r.links.iter().any(|(_, _, t)| match t {
                    Target::Local(_) => v.doc != DocKind::Attachment,
                    t => t.as_link().is_some(),
                })
            })
    }

    /// Whether the yak detail pane, focused, shows a peek pane: the same
    /// rule as [`App::viewer_has_peek`] (shown, a farm, a link peek can read
    /// among the yak's rows), and the pane tall enough to spare
    /// [`PEEK_ROWS`]. An unfocused pane has no cursor to peek from, and the
    /// in-place editor wants the rows. The pane's page
    /// ([`App::detail_viewport`]) is what is left. The cheap tests come
    /// first: the editor's own slot asks for the viewport while the rows are
    /// being built.
    pub(crate) fn detail_has_peek(&self) -> bool {
        self.peek_shown
            && self.focus == Focus::Detail
            && self.doc.is_none()
            && self.farm.is_some()
            && self.detail_page > PEEK_ROWS + 2
            && self.block_edit().is_none()
            && self
                .detail_dlines()
                .iter()
                .any(|r| r.links.iter().any(|(_, _, t)| t.as_link().is_some()))
    }

    /// `p`: show or hide the peek pane, in the viewer and the yak detail
    /// pane alike, then keep the cursor in view of the page the pane leaves.
    pub(crate) fn toggle_peek(&mut self) {
        self.peek_shown = !self.peek_shown;
        self.notification = Some(
            if self.peek_shown {
                "peek shown"
            } else {
                "peek hidden"
            }
            .into(),
        );
        if let Some((rows, vp)) = self.attachment_rows() {
            if let Some(v) = self.attach_mut() {
                // `goto` forgets the targeted link; the cursor hasn't moved.
                let link = v.nav.link;
                let line = v.nav.line;
                v.nav.goto(&rows, vp, line);
                v.nav.link = link;
            }
        } else if self.focus == Focus::Detail {
            self.scroll_line_into_view(self.detail.line as u16);
        }
    }
}

/// A peek's text as display lines: the frontmatter fence and blank runs
/// dropped, so a dok head shows its fields and first paragraph.
pub(crate) fn peek_lines(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for l in text.lines() {
        let l = l.trim_end();
        if l == "---" || (l.is_empty() && out.last().is_none_or(|p| p.is_empty())) {
            continue;
        }
        out.push(l.replace('\t', "    "));
    }
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_support::*;

    /// A dok linking every way the grammar allows: dok-relative, from the
    /// farm root at a heading, an in-page anchor, a source, a line range, a
    /// yak, and an attachment.
    const HUB: &str = "---\ntitle: Hub\n---\n\n# Hub\n\n- See [fts](full-text-search.md).\n- And [indexes](/doks/topics/vector-search.md#indexes).\n- Jump [below](#later).\n- Read [the source](/sources/2026-10-22-1a2b3c4d/source.md).\n- Lines [two](/doks/howto/release.md#L6-L7).\n- Yak yaks-0001.\n- Log [run](yaks-0001/run.log).\n\n## Later\n\nThe end.\n";

    /// A live app on the fixture doks plus `doks/topics/hub.md`, with the hub
    /// open in the viewer and its pane measured.
    fn on_hub() -> (PathBuf, App) {
        let (proj, farm) = temp_farm(&[task(
            "yaks-0001",
            "Search",
            crate::model::Status::Hairy,
            2,
            None,
        )]);
        crate::tui::doks_tab::fixture::write(farm.root());
        std::fs::write(farm.root().join("doks/topics/hub.md"), HUB).unwrap();
        farm.attach("yaks-0001", "run.log", b"step one\nstep two\n")
            .unwrap();
        let mut app = App::with_farm(farm).unwrap();
        app.detail_page = 30;
        app.open_farm_doc("doks/topics/hub.md", None, None);
        draw(&app, 150, 34);
        (proj, app)
    }

    fn viewer(app: &App) -> &AttachView {
        app.attach().unwrap_or_else(|| {
            panic!(
                "the viewer should be open, overlay={}",
                super::super::render::overlay_name(&app.overlay)
            )
        })
    }

    /// Tab until the cursor's link is `want` (by its status-bar label).
    fn tab_to(app: &mut App, want: &str) {
        for _ in 0..12 {
            tab_key(app);
            let (rows, _) = app.attachment_rows().unwrap();
            let j = viewer(app).nav.link_at(&rows).unwrap();
            let label = match &j.target {
                Target::Task(id) => id.clone(),
                t => super::super::linenav::link_dest(t.clone()).unwrap_or_default(),
            };
            if label == want {
                return;
            }
        }
        panic!("no link {want}");
    }

    #[test]
    fn links_in_a_dok_resolve_by_the_link_grammar() {
        let (proj, app) = on_hub();
        let (rows, _) = app.attachment_rows().unwrap();
        let targets: Vec<Target> = detail::jumplist(&rows)
            .into_iter()
            .map(|j| j.target)
            .collect();
        let farm = |p: &str, a: Option<Anchor>| Target::Farm {
            path: p.into(),
            rev: None,
            anchor: a,
        };
        assert_eq!(
            targets,
            vec![
                farm("doks/topics/full-text-search.md", None),
                farm(
                    "doks/topics/vector-search.md",
                    Some(Anchor::Heading("indexes".into()))
                ),
                Target::Local(Anchor::Heading("later".into())),
                farm("sources/2026-10-22-1a2b3c4d/source.md", None),
                farm("doks/howto/release.md", Some(Anchor::Lines(6, 7))),
                Target::Task("yaks-0001".into()),
                Target::Artifact("yaks-0001/run.log".into()),
            ]
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn enter_opens_a_dok_at_its_heading_and_o_comes_back() {
        let (proj, mut app) = on_hub();
        tab_to(&mut app, "doks/topics/vector-search.md#indexes");
        let hub_line = viewer(&app).nav.line;
        enter_key(&mut app);
        let v = viewer(&app);
        assert_eq!(v.target, "doks/topics/vector-search.md");
        let (rows, _) = app.attachment_rows().unwrap();
        assert_eq!(
            rows[viewer(&app).nav.line].text,
            "Indexes",
            "on the heading"
        );
        // o goes back to the hub, cursor where it was; i forward again.
        press(&mut app, "o");
        assert_eq!(viewer(&app).target, "doks/topics/hub.md");
        assert_eq!(viewer(&app).nav.line, hub_line);
        press(&mut app, "i");
        assert_eq!(viewer(&app).target, "doks/topics/vector-search.md");
        // A dok-relative link from there, then two hops back.
        tab_to(&mut app, "doks/topics/full-text-search.md");
        enter_key(&mut app);
        assert_eq!(viewer(&app).target, "doks/topics/full-text-search.md");
        press(&mut app, "oo");
        assert_eq!(viewer(&app).target, "doks/topics/hub.md");
        // One step further is where the hub was opened from: the list.
        press(&mut app, "o");
        assert!(app.doc.is_none(), "back out of the first dok, to the list");
        assert_eq!(app.focus, Focus::List);
        press(&mut app, "o");
        assert_eq!(app.notification.as_deref(), Some("no earlier location"));
        // Closing a dok does not forget the history: `i` reopens it.
        press(&mut app, "i");
        assert_eq!(viewer(&app).target, "doks/topics/hub.md");
        esc_key(&mut app);
        assert!(app.doc.is_none());
        assert!(!app.history.fwd.is_empty() || !app.history.back.is_empty());
        press(&mut app, "i");
        assert_eq!(viewer(&app).target, "doks/topics/vector-search.md");
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_followed_dok_closes_back_to_where_the_first_was_opened_from() {
        let (proj, mut app) = on_hub();
        // `on_hub` opened the hub from the list: that is where it closes to.
        assert_eq!(app.doc.as_ref().map(|d| d.from), Some(Focus::List));
        tab_to(&mut app, "doks/topics/vector-search.md#indexes");
        enter_key(&mut app);
        assert_eq!(viewer(&app).target, "doks/topics/vector-search.md");
        assert_eq!(app.doc.as_ref().map(|d| d.from), Some(Focus::List));
        esc_key(&mut app);
        assert!(app.doc.is_none());
        assert_eq!(app.focus, Focus::List);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn an_in_dok_anchor_and_a_line_range_land_on_their_rows() {
        let (proj, mut app) = on_hub();
        tab_to(&mut app, "#later");
        let from = viewer(&app).nav.line;
        enter_key(&mut app);
        let (rows, _) = app.attachment_rows().unwrap();
        assert_eq!(rows[viewer(&app).nav.line].text, "Later");
        press(&mut app, "o");
        assert_eq!(
            viewer(&app).nav.line,
            from,
            "o undoes the in-page jump first"
        );
        assert_eq!(viewer(&app).target, "doks/topics/hub.md");
        // `#L6-L7` counts file lines, frontmatter included: release.md's
        // line 6 is `1. Tag ...`.
        tab_to(&mut app, "doks/howto/release.md:L6-L7");
        enter_key(&mut app);
        let (rows, _) = app.attachment_rows().unwrap();
        assert!(
            rows[viewer(&app).nav.line].text.contains("Tag"),
            "{:?}",
            rows[viewer(&app).nav.line].text
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_source_link_opens_with_extracted_text_and_attachments_join_the_history() {
        let (proj, mut app) = on_hub();
        tab_to(&mut app, "sources/2026-10-22-1a2b3c4d/source.md");
        enter_key(&mut app);
        assert!(matches!(
            viewer(&app).doc,
            DocKind::Source { extracted: Some(_) }
        ));
        press(&mut app, "o");
        tab_to(&mut app, "yaks-0001/run.log");
        enter_key(&mut app);
        assert_eq!(viewer(&app).target, "yaks-0001/run.log");
        press(&mut app, "o");
        assert_eq!(viewer(&app).target, "doks/topics/hub.md");
        // A yak closes the viewer and opens the yak.
        tab_to(&mut app, "yaks-0001");
        enter_key(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(app.selected_id().as_deref(), Some("yaks-0001"));
        let _ = std::fs::remove_dir_all(proj);
    }

    /// yaks-0e15 / yaks-d0b0: a markdown link whose destination is a yak id,
    /// in a dok under `doks/topics/`, opens the yak; it is not read as the
    /// dok-relative file `doks/topics/yaks-0001`.
    #[test]
    fn a_markdown_link_to_a_yak_from_a_dok_opens_the_yak() {
        let (proj, mut app) = on_hub();
        let root = app.farm.as_ref().unwrap().root().to_path_buf();
        std::fs::write(
            root.join("doks/topics/hub.md"),
            "---\ntitle: Hub\n---\n\n# Hub\n\nBuilt under [the search yak](yaks-0001).\n",
        )
        .unwrap();
        app.close_doc();
        app.open_farm_doc("doks/topics/hub.md", None, None);
        draw(&app, 150, 34);
        tab_to(&mut app, "yaks-0001");
        enter_key(&mut app);
        assert_eq!(
            app.selected_id().as_deref(),
            Some("yaks-0001"),
            "{:?}",
            app.notification
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_missing_dok_says_so_and_stays_put() {
        let (proj, mut app) = on_hub();
        std::fs::remove_file(
            app.farm
                .as_ref()
                .unwrap()
                .root()
                .join("doks/topics/full-text-search.md"),
        )
        .unwrap();
        tab_to(&mut app, "doks/topics/full-text-search.md");
        enter_key(&mut app);
        assert_eq!(viewer(&app).target, "doks/topics/hub.md");
        assert_eq!(
            app.notification.as_deref(),
            Some("doks/topics/full-text-search.md: not in this farm")
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn the_peek_pane_shows_the_section_under_the_cursor() {
        let (proj, mut app) = on_hub();
        tab_to(&mut app, "doks/topics/vector-search.md#indexes");
        let frame = draw(&app, 150, 34);
        assert!(
            frame.contains("── peek: doks/topics/vector-search.md#indexes"),
            "{frame}"
        );
        assert!(
            frame.contains("- HNSW: fast queries, slow builds."),
            "{frame}"
        );
        // A dok head: its fields and first paragraph.
        tab_to(&mut app, "doks/topics/full-text-search.md");
        let frame = draw(&app, 150, 34);
        assert!(frame.contains("title: Full-text search"), "{frame}");
        // An in-page anchor peeks this dok's section.
        tab_to(&mut app, "#later");
        let frame = draw(&app, 150, 34);
        assert!(frame.contains("peek: doks/topics/hub.md#later"), "{frame}");
        assert!(frame.contains("The end."), "{frame}");
        // `p` hides it, and the dok gets the rows back.
        press(&mut app, "p");
        let frame = draw(&app, 150, 34);
        assert!(!frame.contains("── peek"), "{frame}");
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_pinned_link_opens_the_dok_as_it_was() {
        let (proj, mut app) = on_hub();
        let root = app.farm.as_ref().unwrap().root().to_path_buf();
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(&proj)
                .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
                .args(args)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        git(&["init", "-q"]);
        git(&["add", "-A"]);
        git(&["commit", "-qm", "doks"]);
        let sha = git(&["rev-parse", "HEAD"]);
        let dok = root.join("doks/topics/full-text-search.md");
        std::fs::write(&dok, "---\ntitle: Full-text search\n---\n\nRewritten.\n").unwrap();
        app.open_farm_doc("doks/topics/full-text-search.md", Some(&sha), None);
        let frame = draw(&app, 150, 34);
        assert!(
            frame.contains(&format!("full-text-search.md@{}", &sha[..7])),
            "{frame}"
        );
        assert!(frame.contains("Use tsvector and a GIN index."), "{frame}");
        assert!(!frame.contains("Rewritten."), "{frame}");
        press(&mut app, "E");
        assert!(app.attach().is_some(), "a past version is read-only");
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .ends_with("a past version is read-only")
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    /// A live app on a yak whose body links a dok section, a source, a code
    /// range and a yak, with the detail pane open and measured.
    fn on_yak_with_links() -> (PathBuf, App) {
        let mut t = task("yaks-0001", "Search", crate::model::Status::Hairy, 2, None);
        t.body = "Background: [vector search](/doks/topics/vector-search.md#indexes).\n\nPlain line.\n\nSee yaks-0002.".into();
        let (proj, farm) = temp_farm(&[t]);
        crate::tui::doks_tab::fixture::write(farm.root());
        let mut app = App::with_farm(farm).unwrap();
        app.detail_page = 21;
        enter_key(&mut app);
        draw(&app, 120, 24);
        (proj, app)
    }

    #[test]
    fn the_detail_pane_follows_a_farm_link_and_peeks_it_in_a_pane() {
        let (proj, mut app) = on_yak_with_links();
        tab_key(&mut app);
        let frame = draw(&app, 120, 24);
        assert!(
            frame.contains("── peek: doks/topics/vector-search.md#indexes"),
            "{frame}"
        );
        assert!(
            frame.contains("- HNSW: fast queries, slow builds."),
            "{frame}"
        );
        // The status bar names the destination; the peek is in the pane.
        let status = frame.lines().last().unwrap();
        assert!(
            status.starts_with("→ doks/topics/vector-search.md#indexes"),
            "{status}"
        );
        assert!(!status.contains("·"), "{status}");
        enter_key(&mut app);
        assert_eq!(viewer(&app).target, "doks/topics/vector-search.md");
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn p_hides_and_shows_the_peek_in_the_yak_pane_and_the_flag_is_the_viewers() {
        let (proj, mut app) = on_yak_with_links();
        tab_key(&mut app);
        assert!(draw(&app, 120, 24).contains("── peek: "));
        press(&mut app, "p");
        let frame = draw(&app, 120, 24);
        assert!(!frame.contains("── peek"), "{frame}");
        assert!(!app.peek_shown);
        // One flag: hidden here, hidden in a dok opened from the pane.
        enter_key(&mut app);
        tab_key(&mut app);
        assert!(!draw(&app, 120, 24).contains("── peek"));
        press(&mut app, "p");
        assert!(app.peek_shown);
        assert!(draw(&app, 120, 24).contains("── peek"));
        // And shown again when back in the yak.
        esc_key(&mut app);
        assert!(app.attach().is_none());
        tab_key(&mut app);
        assert!(draw(&app, 120, 24).contains("── peek: "));
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn the_yak_pane_peek_is_only_for_a_focused_pane_and_gives_the_page_back() {
        let (proj, mut app) = on_yak_with_links();
        // On the list there is no cursor to peek from.
        esc_key(&mut app);
        assert!(!draw(&app, 120, 24).contains("── peek"));
        enter_key(&mut app);
        let page = app.detail_viewport().page;
        assert!(draw(&app, 120, 24).contains("── peek"));
        // Hiding gives the rows back to the page, showing takes them again.
        press(&mut app, "p");
        assert_eq!(app.detail_viewport().page, page + PEEK_ROWS as usize);
        press(&mut app, "p");
        assert_eq!(app.detail_viewport().page, page);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn showing_the_peek_scrolls_a_cursor_it_would_cover_back_into_view() {
        let mut t = task("yaks-0001", "Long", crate::model::Status::Hairy, 2, None);
        t.body = format!(
            "{}[vector search](/doks/topics/vector-search.md#indexes)\n",
            "Line.\n\n".repeat(12)
        );
        let (proj, farm) = temp_farm(&[t]);
        crate::tui::doks_tab::fixture::write(farm.root());
        let mut app = App::with_farm(farm).unwrap();
        app.detail_page = 21;
        enter_key(&mut app);
        draw(&app, 120, 24);
        press(&mut app, "p"); // hidden: the page is the whole pane
        press(&mut app, "G");
        draw(&app, 120, 24);
        press(&mut app, "p"); // shown: 7 rows fewer
        let (line, scroll) = (app.detail.line, app.detail.scroll as usize);
        let shown = app.detail_rows_shown(app.detail.scroll);
        assert!(
            (scroll..scroll + shown).contains(&line),
            "cursor {line} outside {scroll}+{shown}"
        );
        let frame = draw(&app, 120, 24);
        assert!(frame.contains("── peek: "), "{frame}");
        assert!(frame.contains("[vector search]") || frame.contains("vector search"));
        let _ = std::fs::remove_dir_all(proj);
    }

    /// The viewer reached from the Doks tab (the hub's row, then Enter).
    fn hub_from_the_doks_tab() -> (PathBuf, App) {
        let (proj, farm) = temp_farm(&[task(
            "yaks-0001",
            "Search",
            crate::model::Status::Hairy,
            2,
            None,
        )]);
        crate::tui::doks_tab::fixture::write(farm.root());
        std::fs::write(farm.root().join("doks/topics/hub.md"), HUB).unwrap();
        let mut app = App::with_farm(farm).unwrap();
        let i = app
            .views
            .iter()
            .position(|v| v.key == view::DOKS_KEY)
            .unwrap();
        app.set_view(i);
        draw(&app, 150, 34);
        while app.selected_doc().map(|d| d.path.as_str()) != Some("doks/topics/hub.md") {
            press(&mut app, "j");
        }
        enter_key(&mut app);
        draw(&app, 150, 34);
        (proj, app)
    }

    #[test]
    fn p_hides_and_shows_the_peek_in_a_dok_opened_from_the_doks_tab() {
        let (proj, mut app) = hub_from_the_doks_tab();
        tab_to(&mut app, "doks/topics/vector-search.md#indexes");
        let frame = draw(&app, 150, 34);
        assert!(frame.contains("── peek: "), "{frame}");
        press(&mut app, "p");
        let frame = draw(&app, 150, 34);
        assert!(!frame.contains("── peek"), "{frame}");
        press(&mut app, "p");
        let frame = draw(&app, 150, 34);
        assert!(frame.contains("── peek: "), "{frame}");
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn p_hides_and_shows_the_peek_in_a_dok_opened_from_a_yak_link() {
        let mut t = task("yaks-0001", "Search", crate::model::Status::Hairy, 2, None);
        t.body = "See [the hub](/doks/topics/hub.md).".into();
        let (proj, farm) = temp_farm(&[t]);
        crate::tui::doks_tab::fixture::write(farm.root());
        std::fs::write(farm.root().join("doks/topics/hub.md"), HUB).unwrap();
        let mut app = App::with_farm(farm).unwrap();
        enter_key(&mut app);
        draw(&app, 150, 34);
        tab_key(&mut app);
        enter_key(&mut app);
        draw(&app, 150, 34);
        tab_to(&mut app, "doks/topics/vector-search.md#indexes");
        assert!(draw(&app, 150, 34).contains("── peek: "));
        press(&mut app, "p");
        assert!(!draw(&app, 150, 34).contains("── peek"));
        press(&mut app, "p");
        assert!(draw(&app, 150, 34).contains("── peek: "));
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn the_viewer_following_a_dok_link_with_a_peek() {
        let (proj, mut app) = on_hub();
        tab_to(&mut app, "doks/topics/vector-search.md#indexes");
        insta::assert_snapshot!(draw(&app, 120, 28));
        let _ = std::fs::remove_dir_all(proj);
    }
}
