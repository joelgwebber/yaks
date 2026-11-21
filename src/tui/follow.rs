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
use crate::locate::{Place, Space};

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

/// The row a heading or line anchor lands on in a viewer's rows. A line
/// anchor: the first row on or after that file line (every row records its
/// source line, yaks-15dc; the last row when the line is past them). A
/// heading: the first row whose source covers its line.
pub(crate) fn anchor_row(v: &AttachView, rows: &[detail::DLine], anchor: &Anchor) -> Option<usize> {
    if let Anchor::Lines(a, _) = anchor {
        let lines = v.line_of_rows(rows);
        return lines
            .iter()
            .position(|l| l.is_some_and(|l| l >= *a))
            .or_else(|| lines.iter().rposition(|l| l.is_some()));
    }
    let Anchor::Heading(h) = anchor else {
        return None;
    };
    let body = v.blocks()[0];
    let (_, _, r) = links::sections(body).into_iter().find(|(_, s, _)| s == h)?;
    let line = body[..r.start].lines().count();
    rows.iter().position(|r| {
        r.src
            .is_some_and(|s| s.block == 0 && s.lines.is_some_and(|(a, b)| a <= line && line <= b))
    })
}

impl App {
    /// The farm root, when there is a farm.
    fn farm_root(&self) -> Option<PathBuf> {
        self.farm.as_ref().map(|f| f.root().to_path_buf())
    }

    /// Read farm file `path` (at farm commit `rev`) into a viewer. `Err` is a
    /// one-line message for the status bar.
    pub(crate) fn read_farm_doc(
        &self,
        path: &str,
        rev: Option<&str>,
    ) -> Result<AttachView, String> {
        let root = self
            .farm_root()
            .ok_or_else(|| format!("{path}: no farm to read"))?;
        // `sources/<id>` names a source (the bare `<date>-<sha8>` of a slugged
        // one too): it reads as its file, `sources/<name>.md`.
        let path = &*crate::locate::farm_file_in(&root, path);
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
        let mut v = AttachView::farm_doc(path, file, text, rev.map(String::from));
        if rev.is_none() {
            v.files = self.files_of(path);
        }
        Ok(v)
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
        // A view not drawn yet has no width (0): rows laid out at that width
        // wrap one column per row, so an anchor placed before the first draw
        // (`#heading`, `#L12`) lands thousands of rows down, past the end
        // (yaks-b4d7). Every viewer starts at the pane's width as last drawn.
        if let DocView::Text(v) = &view {
            if v.width.get() == 0 {
                v.width.set(self.doc_width());
            }
        }
        let (from, old) = match self.doc.take() {
            Some(d) => (d.from, Some(d.view)),
            None => (self.focus, None),
        };
        self.doc = Some(DocPane { view, from });
        self.focus = Focus::Detail;
        old
    }

    /// The width a viewer lays out at before it is drawn: the pane's as last
    /// drawn, else the detail pane's, else a full terminal's (an app never
    /// drawn, as in a test that opens a dok straight away).
    pub(crate) fn doc_width(&self) -> u16 {
        [self.pane_width.get(), self.detail_width.get()]
            .into_iter()
            .find(|&w| w > 0)
            .unwrap_or(80)
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

    /// The picture at farm file `path` as it was at commit `rev`, read
    /// through git (an LFS file as its content, not its pointer:
    /// [`links::read_bytes_at`]). `None` when it is not an image, not there at
    /// that commit, too big, or cannot be decoded: the text viewer then says
    /// why.
    fn farm_image_at(&self, path: &str, rev: Option<&str>) -> Option<ImageView> {
        if !crate::store::is_image_name(path) {
            return None;
        }
        let root = self.farm_root()?;
        let path = &*crate::locate::farm_file_in(&root, path);
        let bytes = links::read_bytes_at(&root, path, rev).ok()?;
        if bytes.len() as u64 > super::imageview::IMAGE_MAX_BYTES {
            return None;
        }
        let name = path.rsplit('/').next().unwrap_or(path);
        let mut v = ImageView::from_bytes(path, root.join(path), name, &bytes, &self.picker)?;
        v.rev = rev.map(String::from);
        Some(v)
    }

    // -- one resolver, one opener (yaks-84fe) -------------------------------
    //
    // Every link the pane follows, every document history restores and every
    // row that opens a subject goes link -> [`Place`] ([`App::resolve`], the
    // farm's one resolver, `locate::Space`) -> [`App::open_place`] /
    // [`App::read_place`]. Whether a link names a directory, an attachment,
    // a farm file, a code file or a commit is decided there, once.

    /// What links resolve against here: the farm and the code checkout.
    pub(crate) fn space(&self) -> Option<Space> {
        let root = self.farm_root()?;
        let code = super::source::source_base(&root, self.sheds.cwd.as_deref());
        Some(Space::new(&root, code))
    }

    /// What `link` names, by the farm's resolver. A URL on the code repo's
    /// own host reads as the commit or pinned file it names.
    pub(crate) fn resolve(&self, link: Link) -> Place {
        match (self.space(), link) {
            (Some(s), link) => s.resolve(&s.read_url(link)),
            (None, Link::Url(u)) => Place::Url(u),
            (None, _) => Place::Missing("no farm to resolve the link against".into()),
        }
    }

    /// What pane link `t` names; `None` for one that moves within the pane or
    /// the list instead (a yak, a note, a footnote, an in-page anchor).
    pub(crate) fn place_of(&self, t: &Target) -> Option<Place> {
        let link = match t {
            Target::Url(u) => Link::Url(u.clone()),
            t => t.as_link()?,
        };
        Some(self.resolve(link))
    }

    /// What a link destination names, read as a yak reads it (`<id>/<file>`,
    /// `<id>/` for its attachment directory, `/doks/x.md`, `commit:<sha>`).
    pub(crate) fn place_of_dest(&self, dest: &str) -> Place {
        let link = links::parse(dest, links::From::Yak, |s| self.task(s).is_some());
        self.resolve(link)
    }

    /// Follow pane link `t` to what it names (see [`App::open_place`]).
    pub(crate) fn open_target(&mut self, t: &Target) {
        if let Some(p) = self.place_of(t) {
            self.open_place(&p);
        }
    }

    /// Open what a link destination names (see [`App::place_of_dest`]): a
    /// test's way to follow a link without drawing one.
    #[cfg(test)]
    pub(crate) fn open_dest(&mut self, dest: &str) {
        let p = self.place_of_dest(dest);
        self.open_place(&p);
    }

    /// Open farm file or directory `path` (at farm commit `rev`, at
    /// `anchor`): a dok, a source, any farm path.
    pub(crate) fn open_farm_doc(&mut self, path: &str, rev: Option<&str>, anchor: Option<&Anchor>) {
        let p = self.resolve(Link::Farm {
            path: path.to_string(),
            rev: rev.map(String::from),
            anchor: anchor.cloned(),
        });
        self.open_place(&p);
    }

    /// Show what `place` names, a jump (`o` comes back): a directory in the
    /// directory view, a farm file, attachment, archive member or code file
    /// in the text or image viewer, a commit in the commit view, a yak in the
    /// detail pane. An attachment the pane cannot show, and a URL, go to the
    /// system opener, never an archive ([`App::os_open`]).
    ///
    /// A text view lands on the link's anchor, one rule for every kind of
    /// file (yaks-15dc): a `#heading` or `#L12` puts the cursor on that row;
    /// a range, `#L10-L14`, opens narrowed to those lines (`w` widens).
    pub(crate) fn open_place(&mut self, place: &Place) {
        let anchor = match place {
            Place::FarmFile { anchor, .. }
            | Place::Member { anchor, .. }
            | Place::Attachment { anchor, .. }
            | Place::CodeFile { anchor, .. } => anchor.clone(),
            _ => None,
        };
        match place {
            Place::Yak { id, note: None } => return self.goto_task(id),
            Place::Yak { id, note: Some(n) } => return self.goto_note(id, n),
            Place::FarmDir { rel } => return self.open_dir(rel),
            Place::Commit { sha } => return self.open_commit(sha),
            Place::Url(u) => return self.os_open(u, u),
            Place::Missing(why) => {
                self.notification = Some(why.clone());
                return;
            }
            Place::Attachment { id, name, anchor } => {
                if !self.open_attachment(id, name, anchor.clone()) {
                    return;
                }
            }
            Place::FarmFile { .. } | Place::Member { .. } | Place::CodeFile { .. } => {
                match self.read_place(place) {
                    Ok(view) => self.follow_to_doc(view),
                    Err(e) => {
                        self.notification = Some(e);
                        return;
                    }
                }
            }
        }
        // A code file that could not be read shows why, on its one row.
        if matches!(
            self.attach().map(|v| &v.doc),
            Some(DocKind::Code { error: Some(_), .. })
        ) {
            return;
        }
        self.scroll_to_anchor(anchor.as_ref());
        let label = Space::link(place).map(|l| links::label(&l));
        self.notification = Some(format!("viewing {}", label.unwrap_or_default()));
    }

    /// Attachment `name` of yak `id`, when it is a file of this farm.
    fn attachment_file(&self, id: &str, name: &str) -> Option<crate::farm::FoundAttachment> {
        let root = self.farm_root()?;
        self.task(id)?;
        crate::farm::locate_attachment(&root, id, name)
    }

    /// Open an attachment: in the viewer when it can show it (`true`), else
    /// with the system opener; an archive is never opened (it would unpack
    /// into the farm, yaks-3329) and says how to get its path.
    fn open_attachment(&mut self, id: &str, name: &str, anchor: Option<Anchor>) -> bool {
        let target = format!("{id}/{}", links::percent_encode_path(name));
        let Some(found) = self.attachment_file(id, name) else {
            self.notification = Some(format!("{target}: not in this farm"));
            return false;
        };
        if super::dirview::is_archive_name(&found.name) {
            self.notification = Some(super::dirview::archive_note(&found.name));
            return false;
        }
        let place = Place::Attachment {
            id: id.to_string(),
            name: name.to_string(),
            anchor,
        };
        match self.read_place(&place) {
            Ok(view) => {
                self.follow_to_doc(view);
                true
            }
            Err(_) => {
                self.os_open(&target, &found.path.display().to_string());
                false
            }
        }
    }

    /// The pane's view of `place`, read from disk (what history restores a
    /// document by, and what the openers show). `Err` is a one-line message
    /// for the status bar.
    pub(crate) fn read_place(&self, place: &Place) -> Result<DocView, String> {
        let mut view = self.read_place_whole(place)?;
        // A range link (`#L10-L14`, not a single line) shows only its lines,
        // `w` widens (yaks-15dc): here, so following the link and history
        // restoring it read alike.
        let range = match place {
            Place::FarmFile { anchor, .. }
            | Place::Member { anchor, .. }
            | Place::Attachment { anchor, .. }
            | Place::CodeFile { anchor, .. } => match anchor {
                Some(Anchor::Lines(a, b)) if a != b => Some((*a, *b)),
                _ => None,
            },
            _ => None,
        };
        if let (Some((a, b)), DocView::Text(v)) = (range, &mut view) {
            if !matches!(v.doc, DocKind::Code { error: Some(_), .. }) {
                v.narrow(a, b);
            }
        }
        Ok(view)
    }

    /// [`App::read_place`] before a range narrows it: the whole file.
    fn read_place_whole(&self, place: &Place) -> Result<DocView, String> {
        match place {
            Place::FarmDir { rel } => {
                let root = self.farm_root().ok_or("no farm to read")?;
                super::dirview::DirView::open(&root, rel).map(DocView::Dir)
            }
            Place::FarmFile { rel, rev, .. } => {
                if rev.is_none() {
                    if let Some(v) = self.farm_image(rel) {
                        return Ok(DocView::Image(v));
                    }
                } else if let Some(v) = self.farm_image_at(rel, rev.as_deref()) {
                    return Ok(DocView::Image(v));
                }
                self.read_farm_doc(rel, rev.as_deref()).map(DocView::Text)
            }
            Place::Attachment { id, name, .. } => {
                let target = format!("{id}/{}", links::percent_encode_path(name));
                let found = self
                    .attachment_file(id, name)
                    .ok_or_else(|| format!("{target}: not in this farm"))?;
                if let Some(v) = ImageView::open(&target, &found, &self.picker) {
                    return Ok(DocView::Image(v));
                }
                AttachView::open(&target, &found)
                    .map(DocView::Text)
                    .ok_or_else(|| format!("{target}: cannot be shown here"))
            }
            Place::Member { archive, inner, .. } => self.read_member(archive, inner),
            // The lines the link names are marked in the gutter.
            Place::CodeFile { rel, rev, anchor } => {
                let marked = match anchor {
                    Some(Anchor::Lines(a, b)) => Some((*a, *b)),
                    _ => None,
                };
                self.read_code_view(rel, rev.as_deref(), marked)
                    .map(DocView::Text)
            }
            Place::Commit { sha } => self.read_commit_view(sha).map(DocView::Commit),
            Place::Missing(why) => Err(why.clone()),
            Place::Yak { id, .. } => Err(format!("{id}: a yak, not a document")),
            Place::Url(u) => Err(format!("{u}: a URL")),
        }
    }

    /// File `inner` of the archive at farm path `archive`, in the viewer
    /// that fits it (a picture, else text), read-only and read through
    /// [`crate::entries::read`], so its size is capped as a file's is. The
    /// view's target is the path through the archive, which history keeps.
    fn read_member(&self, archive: &str, inner: &str) -> Result<DocView, String> {
        let root = self.farm_root().ok_or("no farm to read")?;
        let target = format!("{archive}/{inner}");
        let Some((c, top)) = crate::entries::container_at(&root, archive) else {
            return Err(format!("{archive}: not an archive of this farm"));
        };
        if !top.is_empty() {
            return Err(format!("{target}: not a file of an archive"));
        }
        let name = inner.rsplit('/').next().unwrap_or(inner);
        let max = if crate::store::is_image_name(name) {
            super::imageview::IMAGE_MAX_BYTES
        } else {
            VIEW_MAX_BYTES
        };
        let bytes = crate::entries::read(&c, inner, max)
            .map_err(|e| format!("{target}: {e:#}").replace(&format!("{inner}: "), ""))?;
        // Where the farm would have it; never read, so it need not exist.
        let path = root.join(&target);
        if let Some(v) = ImageView::from_bytes(&target, path.clone(), name, &bytes, &self.picker) {
            return Ok(DocView::Image(v));
        }
        let v = AttachView::member(&target, path, name, bytes)
            .ok_or_else(|| format!("{target}: not text or a picture; O then o opens a copy"))?;
        Ok(DocView::Text(v))
    }

    /// The link that names what `view` shows: a subject's one stable name
    /// (yaks-84fe), which history keeps and [`App::read_place`] reads back.
    pub(crate) fn doc_link(&self, view: &DocView) -> Option<Link> {
        let attachment = |target: &str| {
            let target = target.strip_prefix(".yaks/").unwrap_or(target);
            let (id, name) = target.split_once('/')?;
            Some(Link::Attachment {
                id: id.to_string(),
                name: links::percent_decode(name),
                anchor: None,
            })
        };
        let farm = |path: &str, rev: Option<String>| Link::Farm {
            path: path.to_string(),
            rev,
            anchor: None,
        };
        match view {
            // A view narrowed to a range is that range's link (yaks-15dc), so
            // history brings it back narrowed and a star names the range.
            DocView::Text(v) => {
                let mut link = match &v.doc {
                    DocKind::Attachment => attachment(&v.target)?,
                    DocKind::Code { .. } => Link::Code {
                        path: v.code_rel()?.to_string(),
                        rev: v.rev.clone(),
                        anchor: None,
                    },
                    _ => farm(&v.target, v.rev.clone()),
                };
                if let (
                    Some((a, b)),
                    Link::Farm { anchor, .. }
                    | Link::Code { anchor, .. }
                    | Link::Attachment { anchor, .. },
                ) = (v.narrowed(), &mut link)
                {
                    *anchor = Some(Anchor::Lines(a, b));
                }
                Some(link)
            }
            // A farm image's file is its target under the farm root; an
            // attachment's lives under its yak.
            DocView::Image(v) => match self.farm_root() {
                Some(r) if r.join(&v.target) == v.path => Some(farm(&v.target, v.rev.clone())),
                _ => attachment(&v.target),
            },
            DocView::Dir(v) => Some(farm(&format!("{}/", v.rel), None)),
            DocView::Commit(v) => Some(Link::Commit { sha: v.sha.clone() }),
        }
    }

    /// Put the open viewer's cursor on `anchor`'s row (when the anchor is
    /// found; else a notification says so).
    pub(crate) fn scroll_to_anchor(&mut self, anchor: Option<&Anchor>) {
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
    const HUB: &str = "---\ntitle: Hub\n---\n\n# Hub\n\n- See [fts](full-text-search.md).\n- And [indexes](/doks/topics/vector-search.md#indexes).\n- Jump [below](#later).\n- Read [the source](/sources/2026-10-22-1a2b3c4d.md).\n- Lines [two](/doks/howto/release.md#L6-L7).\n- Yak yaks-0001.\n- Log [run](yaks-0001/run.log).\n\n## Later\n\nThe end.\n";

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

    /// `/sources/<id>` names a source: the viewer opens its file,
    /// `sources/<id>.md` (yaks-bede, yaks-e83f), and the history restores it.
    #[test]
    fn a_source_link_opens_its_source_file() {
        let (proj, mut app) = on_hub();
        let sources = app.farm_root().unwrap().join("sources");
        std::fs::create_dir_all(&sources).unwrap();
        std::fs::write(
            sources.join("2026-11-03-81364278.md"),
            "# A source\n\nText.\n",
        )
        .unwrap();
        app.open_farm_doc("sources/2026-11-03-81364278", None, None);
        let v = viewer(&app);
        assert_eq!(v.target, "sources/2026-11-03-81364278.md");
        assert!(
            app.read_place(&app.place_of_dest("/sources/2026-11-03-81364278"))
                .is_ok()
        );
        app.open_farm_doc("sources/nope", None, None);
        assert_eq!(
            app.notification.as_deref(),
            Some("sources/nope.md: not in this farm")
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    /// A source's file carries a slug (`<date>-<sha8>-<slug>.md`, yaks-7729)
    /// and a link names it by the bare `<date>-<sha8>` id: the viewer finds
    /// the one file, and the history re-reads it.
    #[test]
    fn a_bare_source_id_link_opens_the_slugged_file() {
        let (proj, mut app) = on_hub();
        let sources = app.farm_root().unwrap().join("sources");
        std::fs::create_dir_all(&sources).unwrap();
        std::fs::write(
            sources.join("2026-11-03-81364278-a-readable-slug.md"),
            "# A source\n\nText.\n",
        )
        .unwrap();
        app.open_farm_doc("sources/2026-11-03-81364278", None, None);
        assert_eq!(
            viewer(&app).target,
            "sources/2026-11-03-81364278-a-readable-slug.md"
        );
        assert!(
            app.read_place(&app.place_of_dest("/sources/2026-11-03-81364278"))
                .is_ok()
        );
        let _ = std::fs::remove_dir_all(proj);
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
                farm("sources/2026-10-22-1a2b3c4d.md", None),
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
    fn a_source_link_opens_the_source_and_attachments_join_the_history() {
        let (proj, mut app) = on_hub();
        tab_to(&mut app, "sources/2026-10-22-1a2b3c4d.md");
        enter_key(&mut app);
        assert!(matches!(viewer(&app).doc, DocKind::Source));
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

    /// yaks-b4d7: a link followed from a yak opens a viewer that has never
    /// been drawn. Its anchor used to be placed on rows laid out at width 0
    /// (one column each), so it landed far past the end: the last row pinned
    /// at the top and nothing above it on screen. The viewer starts at the
    /// pane's width, so once drawn the cursor is on the heading's row.
    #[test]
    fn a_link_from_a_yak_lands_on_its_heading_in_the_first_frame() {
        let (proj, mut app) = on_yak_with_links();
        tab_key(&mut app);
        enter_key(&mut app);
        assert!(
            viewer(&app).width.get() > 1,
            "the viewer starts at the pane's width"
        );
        let frame = draw(&app, 120, 24);
        // The rows as drawn, at the pane's real width.
        let (rows, _) = app.attachment_rows().unwrap();
        let v = viewer(&app);
        assert_eq!(
            rows.get(v.nav.line).map(|r| r.text.as_str()),
            Some("Indexes"),
            "the cursor is on the heading:\n{frame}"
        );
        assert!(v.nav.scroll as usize <= v.nav.line, "and it is on screen");
        assert!(frame.contains("HNSW"), "{frame}");
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
