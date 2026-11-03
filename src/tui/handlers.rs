//! `App`'s key handling, form commits, and overlay dispatch: `handle_create_key`,
//! `handle_drawer_key`, `handle_help_key`, `handle_cmdline_key`, the big
//! `handle_overlay_key` router, the `:`-command runners, `field_cancel`/
//! double-Esc, and the commit/resolve paths (`commit_*`, `apply_edit`,
//! `resolve_*`) — plus the top-level free `handle_key` dispatcher. Split out of
//! `tui.rs` (yaks-b1cc / b1cc/7). `use super::*` inherits `tui.rs`'s scope.

use super::imageview::ImageView;
use super::linenav::{LineNav, Viewport};
use super::*;

/// Largest attachment the in-TUI viewer shows; a bigger file opens externally.
const VIEW_MAX_BYTES: u64 = 1 << 20;

/// A text attachment open in the viewer (`DocView::Text`, yaks-699b): read once on open, a `.md`/`.markdown` file rendered by the
/// detail pane's markdown renderer and anything else shown verbatim, wrapped
/// to the pane. It navigates like the detail pane, by the same code
/// ([`LineNav`], yaks-84e0): line cursor, visual selection and yank, find,
/// Tab among links and Enter to follow one. `E` edits it in place (yaks-56c1:
/// [`App::open_attachment_edit`], saved by [`App::commit_attachment_edit`]);
/// `O` hands it to the OS opener.
///
/// The Wiki tab (yaks-ebb2) opens farm pages and sources in the same viewer
/// ([`DocKind`]): a page's frontmatter shows as field rows above its rendered
/// body, and a source shows `source.md` then its `extracted.md`.
pub(crate) struct AttachView {
    /// The link target as written (`<id>/<file>`), or a page's farm-relative
    /// path, shown as the header.
    pub(crate) target: String,
    pub(crate) path: PathBuf,
    /// The file's whole text as read (a page's frontmatter included).
    pub(crate) text: String,
    pub(crate) markdown: bool,
    /// What the file is: an attachment, a wiki page or a saved source.
    pub(crate) doc: DocKind,
    /// A farm file shown as it was at this farm commit (a pinned link,
    /// yaks-7395): read-only, so `E` refuses.
    pub(crate) rev: Option<String>,
    /// The line cursor, scroll, selection and find state.
    pub(crate) nav: LineNav,
    /// Content width at the last render, so key handling clamps the scroll
    /// to the rows actually on screen.
    pub(crate) width: Cell<u16>,
}

impl AttachView {
    /// The viewer for `found`, or `None` when it should open externally: an
    /// image, a file over [`VIEW_MAX_BYTES`], or one that is not UTF-8 text
    /// (invalid UTF-8, or a NUL byte).
    pub(crate) fn open(target: &str, found: &crate::farm::FoundAttachment) -> Option<Self> {
        if crate::store::is_image_name(&found.name) {
            return None;
        }
        if std::fs::metadata(&found.path).ok()?.len() > VIEW_MAX_BYTES {
            return None;
        }
        let text = String::from_utf8(std::fs::read(&found.path).ok()?).ok()?;
        if text.contains('\0') {
            return None;
        }
        let markdown = Path::new(&found.name)
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"));
        Some(AttachView {
            target: target.to_string(),
            path: found.path.clone(),
            text,
            markdown,
            doc: DocKind::Attachment,
            rev: None,
            nav: LineNav::default(),
            width: Cell::new(0),
        })
    }

    /// The viewer for farm file `rel` holding `text` (read from `path`, or
    /// from farm commit `rev`): a page (`wiki/...md`), a source's
    /// `source.md` (with its `extracted.md`, when given), or any other farm
    /// file, rendered when it is markdown.
    pub(crate) fn farm_doc(
        rel: &str,
        path: PathBuf,
        text: String,
        rev: Option<String>,
        extracted: Option<String>,
    ) -> Self {
        let is_source = rel.starts_with("sources/") && rel.ends_with("/source.md");
        let doc = if crate::pages::is_page_path(rel) {
            DocKind::Page
        } else if is_source {
            DocKind::Source { extracted }
        } else {
            DocKind::File
        };
        let markdown = Path::new(rel)
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"));
        AttachView {
            target: rel.to_string(),
            path,
            text,
            markdown,
            doc,
            rev,
            nav: LineNav::default(),
            width: Cell::new(0),
        }
    }

    /// The document links in it are written from: a yak (an attachment), or
    /// the farm file at its path, so a relative link means what
    /// [`crate::links::parse`] says it means there.
    pub(crate) fn from(&self) -> crate::links::From<'_> {
        match self.doc {
            DocKind::Attachment => crate::links::From::Yak,
            _ => crate::links::From::Page(&self.target),
        }
    }

    /// The header's kind label: `markdown`/`text` for an attachment, the
    /// page's kind (`topic`, ...), or `source`.
    pub(crate) fn kind_label(&self) -> String {
        match &self.doc {
            DocKind::Attachment | DocKind::File if self.markdown => "markdown".into(),
            DocKind::Attachment | DocKind::File => "text".into(),
            DocKind::Page => crate::pages::Page::parse(&self.target, &self.text).kind(),
            DocKind::Source { .. } => "source".into(),
        }
    }

    /// The markdown each rendered block comes from, indexed like
    /// [`detail::Src::block`]: the file (an attachment), the body below the
    /// frontmatter (a page, a source's note), then a source's extracted text.
    pub(crate) fn blocks(&self) -> Vec<&str> {
        match &self.doc {
            DocKind::Attachment | DocKind::File => vec![self.text.as_str()],
            DocKind::Page => vec![self.body()],
            DocKind::Source { extracted } => {
                let mut v = vec![self.body()];
                if let Some(x) = extracted {
                    v.push(x.as_str());
                }
                v
            }
        }
    }

    /// The text below the frontmatter (all of it when there is none).
    pub(crate) fn body(&self) -> &str {
        crate::store::split_frontmatter(&self.text).map_or(self.text.as_str(), |(_, b)| b)
    }

    /// The content laid out to `width` as detail rows: markdown through
    /// [`mdrender::render`] (so it looks as it would in a yak's body), plain
    /// text one row per line, soft-wrapped by [`detail::wrap`]. A markdown
    /// row records its source lines (as block 0), so yank copies the source,
    /// a footnote reference finds its definition, and a code block or table
    /// scrolls sideways, as in the detail pane.
    ///
    /// A page or source starts with its frontmatter as `key  value` field
    /// rows and a blank row; a source's extracted text follows its note
    /// under an `extracted.md` section row.
    pub(crate) fn rows(&self, width: u16, known: &HashSet<&str>) -> Vec<detail::DLine> {
        let width = width.max(1) as usize;
        let row = |text: String, md, links, nowrap, src| detail::DLine {
            text,
            kind: detail::Kind::Body,
            links,
            md,
            cont: false,
            src,
            nowrap,
            image: None,
        };
        let md_rows = |text: &str, block: usize| -> Vec<detail::DLine> {
            mdrender::render_in(text, width, known, self.from())
                .into_iter()
                .map(|r| {
                    let src = detail::Src {
                        block,
                        lines: Some(r.lines),
                    };
                    row(r.text, r.spans, r.links, r.nowrap, Some(src))
                })
                .collect()
        };
        if matches!(self.doc, DocKind::Page | DocKind::Source { .. }) {
            let blocks = self.blocks();
            let mut out = frontmatter_rows(&self.text);
            if !out.is_empty() {
                out.push(plain_row(String::new(), detail::Kind::Empty));
            }
            out = detail::wrap(out, width);
            out.extend(md_rows(blocks[0], 0));
            if let Some(x) = blocks.get(1) {
                out.push(plain_row(String::new(), detail::Kind::Empty));
                out.push(plain_row(
                    "\u{2500}\u{2500} extracted.md \u{2500}\u{2500}".into(),
                    detail::Kind::Section,
                ));
                out.extend(md_rows(x, 1));
            }
            out
        } else if self.markdown {
            md_rows(&self.text, 0)
        } else {
            let lines = self
                .text
                .lines()
                .map(|l| row(l.replace('\t', "    "), vec![], vec![], None, None))
                .collect();
            detail::wrap(lines, width)
        }
    }
}

/// What a viewer shows: a yak attachment, a wiki page, a saved source with
/// its extracted text (`None` when it has none), or another farm file (a
/// link to `/sources/<id>/extracted.md`, an image's sibling note, ...).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DocKind {
    Attachment,
    Page,
    Source { extracted: Option<String> },
    File,
}

fn plain_row(text: String, kind: detail::Kind) -> detail::DLine {
    detail::DLine {
        text,
        kind,
        links: vec![],
        md: vec![],
        cont: false,
        src: None,
        nowrap: None,
        image: None,
    }
}

/// A page's frontmatter as detail-style field rows: `key:` in the dim label
/// column, then a scalar, or a list of scalars joined with `, `. A value the
/// row can't flatten (a list of maps, a block) shows its YAML lines as
/// written, indented under the key.
fn frontmatter_rows(text: &str) -> Vec<detail::DLine> {
    let Some((front, _)) = crate::store::split_frontmatter(text) else {
        return Vec::new();
    };
    let fields = crate::fields::Fields::parse(front);
    let mut out = Vec::new();
    for (key, value) in fields.iter() {
        let label = format!("{key}:");
        let flat = value.as_ref().and_then(|v| match v {
            crate::fields::Value::Null => Some(String::new()),
            crate::fields::Value::List(items) => items
                .iter()
                .map(|i| i.as_str().map(String::from))
                .collect::<Option<Vec<_>>>()
                .map(|v| v.join(", ")),
            crate::fields::Value::Map(_) => None,
            other => other.as_str().map(String::from),
        });
        match flat {
            Some(v) => out.push(plain_row(format!("{label:<13}{v}"), detail::Kind::Field)),
            None => {
                for l in fields.raw(key).unwrap_or(&[]) {
                    out.push(plain_row(l.trim_end().to_string(), detail::Kind::Field));
                }
            }
        }
    }
    out
}

/// Whether cancelling editor `ed` would lose work: an attachment edit when its
/// text differs from the file as opened; otherwise a non-empty multi-line
/// editor (single-line fields never ask).
fn edit_is_dirty(ed: &Editor) -> bool {
    match &ed.action {
        EditAction::EditAttachment(v, _) => ed.text() != v.text,
        _ => !ed.single_line && !ed.text().trim().is_empty(),
    }
}

/// What one key did to an embedded multi-line or single-line editor overlay
/// (`Overlay::Edit`, `Overlay::BlockEdit`): the shared key routing, decided in
/// one place so the two editors cannot drift.
pub(crate) enum EditKey {
    Commit,
    Cancel,
    /// Open the ref completer: `(chars to replace, seed)`.
    Complete(usize, String),
    /// Vim `:` in Normal mode.
    Cmdline,
    /// Handed to edtui (or nothing to do).
    Handled,
}

/// Route key `k` for editor `ed`: Ctrl-S (or Enter in a single-line field)
/// commits, Ctrl-C / a double-Esc (or Esc in a single-line emacs field)
/// cancels, Tab in a `<prefix>-…` context completes, `:` in vim Normal opens
/// the command line, and everything else flows to edtui.
pub(crate) fn editor_key(
    ed: &mut Editor,
    k: KeyEvent,
    double_esc: bool,
    ref_prefix: &str,
) -> EditKey {
    let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
    let commit =
        (ctrl && k.code == KeyCode::Char('s')) || (ed.single_line && k.code == KeyCode::Enter);
    // `:` in a multiline editor's Normal mode opens the command line.
    let open_cmd = !ed.single_line
        && ed.vim
        && k.code == KeyCode::Char(':')
        && ed.state.borrow().mode == EditorMode::Normal;
    // A single-line field is fully modal in vim: a lone Esc only drops to
    // Normal (handed to edtui) and never cancels, so the whole normal-mode
    // keymap (b/w/0/$/dd/x/yy/p/...) is reachable. Cancel is Ctrl-C or a rapid
    // double-Esc; emacs keeps plain single-Esc-cancels. In a multiline editor
    // a lone Esc always just drops to Normal, so double-Esc is the only Esc-way
    // out.
    let esc_cancels = ed.single_line && k.code == KeyCode::Esc && !ed.vim;
    let cancel = (ctrl && k.code == KeyCode::Char('c')) || esc_cancels || double_esc;
    // Tab opens the ref completer when the cursor is in a `<prefix>-…` context;
    // otherwise it falls through as a normal editor key.
    let complete = if k.code == KeyCode::Tab {
        completion_context(&ed.state, ref_prefix)
    } else {
        None
    };
    if commit {
        EditKey::Commit
    } else if let Some((len, seed)) = complete {
        EditKey::Complete(len, seed)
    } else if open_cmd {
        EditKey::Cmdline
    } else if cancel {
        EditKey::Cancel
    } else {
        ed.handler.on_key_event(k, &mut ed.state.borrow_mut());
        EditKey::Handled
    }
}

impl App {
    pub(crate) fn handle_create_key(&mut self, k: KeyEvent, double_esc: bool) {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let vim = self.editor_vim;
        let (is_content, is_line_text) = match &self.overlay {
            Overlay::Create(f) => (f.is_content_row(), f.is_line_text_row()),
            _ => return,
        };
        // Commit (Ctrl-S) / cancel (Ctrl-C, or Esc outside a content block —
        // inside one Esc belongs to the editor, e.g. vim normal mode).
        if ctrl && k.code == KeyCode::Char('s') {
            let has_title =
                matches!(&self.overlay, Overlay::Create(f) if !f.title_text().is_empty());
            if has_title {
                self.commit_form();
            }
            return;
        }
        // In vim every field is modal: a lone Esc drops the focused field to
        // Normal (or is a no-op on chip rows) and never cancels the form — use
        // Ctrl-C or a rapid double-Esc. Emacs keeps single-Esc-cancels on the
        // non-content rows.
        let esc_cancels = k.code == KeyCode::Esc && !vim && !is_content;
        if (ctrl && k.code == KeyCode::Char('c')) || double_esc || esc_cancels {
            let (editing, dirty) = match &self.overlay {
                Overlay::Create(f) => (f.is_editing(), self.form_is_dirty(f)),
                _ => (false, false),
            };
            let msg = if editing {
                "edit cancelled"
            } else {
                "create cancelled"
            };
            self.request_cancel(dirty, msg);
            return;
        }
        // Row navigation: Tab / Shift-Tab / Ctrl-N / Ctrl-P always move rows.
        // On chip rows j/k also navigate; on single-line text rows Up/Down do;
        // content blocks keep Up/Down for their own cursor.
        let is_chip = !is_content && !is_line_text;
        // In Normal mode a single-line field's j/k are free (no vertical motion
        // within one line), so they move between rows like Tab does on chips.
        let line_normal = is_line_text
            && matches!(&self.overlay, Overlay::Create(f)
                if f.line_editor().is_some_and(|e| e.borrow().mode == EditorMode::Normal));
        // Tab/BackTab move fields on chip + single-line rows, but inside a
        // content block they're normal editor keys (Ctrl-N/P still navigate).
        let nav_down = (matches!(k.code, KeyCode::Tab) && !is_content)
            || (ctrl && k.code == KeyCode::Char('n'))
            || (is_line_text && k.code == KeyCode::Down)
            || (line_normal && k.code == KeyCode::Char('j'))
            || (is_chip && matches!(k.code, KeyCode::Down | KeyCode::Char('j')));
        // BackTab (shift-tab) focuses the previous field on *every* row —
        // including content blocks, where forwarding it to edtui would panic
        // (`KeyCode::from` is `unimplemented!` for BackTab). It mirrors Tab's
        // focus-next, so it's field navigation even inside a content block.
        let nav_up = matches!(k.code, KeyCode::BackTab)
            || (ctrl && k.code == KeyCode::Char('p'))
            || (is_line_text && k.code == KeyCode::Up)
            || (line_normal && k.code == KeyCode::Char('k'))
            || (is_chip && matches!(k.code, KeyCode::Up | KeyCode::Char('k')));
        if nav_down || nav_up {
            // Carry Normal across a j/k field-nav so it keeps moving instead of
            // dropping back to Insert (typing) on the next single-line row.
            let carry = line_normal && matches!(k.code, KeyCode::Char('j') | KeyCode::Char('k'));
            if let Overlay::Create(f) = &mut self.overlay {
                let n = f.row_count();
                f.row = if nav_down {
                    (f.row + 1) % n
                } else {
                    (f.row + n - 1) % n
                };
                if carry {
                    if let Some(e) = f.line_editor() {
                        e.borrow_mut().mode = EditorMode::Normal;
                    }
                }
            }
            return;
        }
        // Enter on a single-line/chip row advances to the next row; in a content
        // block it inserts a newline (handled by the editor below).
        if k.code == KeyCode::Enter && !is_content {
            if let Overlay::Create(f) = &mut self.overlay {
                let n = f.row_count();
                f.row = (f.row + 1) % n;
            }
            return;
        }
        if is_content {
            // `:` in a content block's Normal mode opens the command line.
            let open_cmd = vim
                && k.code == KeyCode::Char(':')
                && matches!(&self.overlay, Overlay::Create(f)
                    if f.description_editor()
                        .is_some_and(|e| e.borrow().mode == EditorMode::Normal));
            if open_cmd {
                self.cmdline = Some(String::new());
                return;
            }
            // Tab opens the ref completer when the cursor is in a `<prefix>-…`
            // context; otherwise Tab is a normal editor key here.
            let complete = if k.code == KeyCode::Tab {
                match &self.overlay {
                    Overlay::Create(f) => f
                        .description_editor()
                        .and_then(|e| completion_context(e, &self.ref_prefix)),
                    _ => None,
                }
            } else {
                None
            };
            if let Some((replace_len, seed)) = complete {
                self.open_ref_picker(replace_len, &seed);
                return;
            }
            // Never hand edtui a key it can't convert (it would panic).
            if edtui_can_handle(k.code) {
                if let Overlay::Create(f) = &mut self.overlay {
                    if let Some(e) = &f.description {
                        f.handler.on_key_event(k, &mut e.borrow_mut());
                    }
                }
            }
            return;
        }
        if is_line_text {
            // Same guard: drop keys edtui can't handle rather than panic.
            if edtui_can_handle(k.code) {
                if let Overlay::Create(f) = &mut self.overlay {
                    match f.row {
                        0 => f.handler.on_key_event(k, &mut f.title.borrow_mut()),
                        3 => f.handler.on_key_event(k, &mut f.tags.borrow_mut()),
                        4 => f.handler.on_key_event(k, &mut f.source.borrow_mut()),
                        _ => {}
                    }
                }
            }
            return;
        }
        // Chip rows: left/right (h/l) move the single-select cursor = value.
        if let Overlay::Create(f) = &mut self.overlay {
            match k.code {
                KeyCode::Left | KeyCode::Char('h') => f.move_chip(-1),
                KeyCode::Right | KeyCode::Char('l') => f.move_chip(1),
                _ => {}
            }
        }
    }

    pub(crate) fn commit_form(&mut self) {
        let editing = matches!(&self.overlay, Overlay::Create(f) if f.is_editing());
        if editing {
            self.commit_edit_form();
        } else {
            self.commit_create();
        }
    }

    pub(crate) fn commit_create(&mut self) {
        let new = match std::mem::replace(&mut self.overlay, Overlay::None) {
            Overlay::Create(f) => NewTask {
                title: f.title_text(),
                prefix: f.target_herd(),
                kind: Some(TYPE_CHOICES[f.kind_idx].to_string()),
                priority: Some(PRI_CHOICES[f.pri_idx]),
                parent: f.parent.clone(),
                tags: f.tags_vec(),
                depends_on: vec![],
                source: Some(f.source_text()).filter(|s| !s.is_empty()),
                prs: vec![],
                description: f.body_opt(),
                verify: None,
                fields: vec![],
            },
            other => {
                self.overlay = other;
                return;
            }
        };
        let Some(h) = &self.farm else { return };
        match h.create(new) {
            Ok(CreateOutcome::Created(t)) => {
                let id = t.id.clone();
                self.reload();
                self.select_id(&id);
                self.notification = Some(format!("created {id}"));
            }
            Ok(CreateOutcome::ParentNotFound(p)) => {
                self.notification = Some(format!("parent {p} not found"))
            }
            Ok(CreateOutcome::ParentFinished(msg)) => {
                self.notification = Some(format!("error: {msg}"))
            }
            // The create form has no prefix field yet, so this is unreachable in
            // practice; surface it as a notification rather than panicking.
            Ok(CreateOutcome::InvalidPrefix(p)) => {
                self.notification = Some(format!("invalid prefix {p:?}"))
            }
            Err(e) => self.notification = Some(format!("error: {e}")),
        }
    }

    /// Commit an edit: diff the form against the current task so unchanged
    /// fields don't rewrite the file or bump `updated`.
    pub(crate) fn commit_edit_form(&mut self) {
        let Overlay::Create(f) = std::mem::replace(&mut self.overlay, Overlay::None) else {
            return;
        };
        let Some(id) = f.edit_id.clone() else { return };
        let Some(cur) = self.task(&id).cloned() else {
            self.notification = Some(format!("{id} not found"));
            return;
        };
        let title = f.title_text();
        let kind = TYPE_CHOICES[f.kind_idx].to_string();
        let priority = PRI_CHOICES[f.pri_idx];
        let new_tags = f.tags_vec();
        let mut edit = TaskEdit::default();
        if title != cur.title {
            edit.title = Some(title);
        }
        if kind != cur.kind {
            edit.kind = Some(kind);
        }
        if priority != cur.priority {
            edit.priority = Some(priority);
        }
        // Source: empty clears it (farm `update` treats "" as clear).
        let source = f.source_text();
        if source != cur.source.clone().unwrap_or_default() {
            edit.source = Some(source);
        }
        if let Some((add, remove)) = crate::model::tag_edit(&cur.tags, &new_tags) {
            edit.add_tags = add;
            edit.remove_tags = remove;
        }
        // A herd change from the edit form's chip row is an id-prefix rename
        // (tail preserved), folded into this same commit (yaks-71d1). Detect it
        // against the yak's current prefix.
        let new_id = f.target_herd().and_then(|nh| {
            id.split_once('-')
                .and_then(|(cur, tail)| (cur != nh).then(|| format!("{nh}-{tail}")))
        });
        let Some(new_id) = new_id else {
            self.apply_edit(&id, edit, format!("{id} updated"));
            return;
        };
        // Field edits land on the current id first, then the prefix rename moves
        // the (now-updated) file. Two farm calls, each in its own borrow scope.
        match &self.farm {
            Some(h) => {
                if let Err(e) = h.update(&id, edit) {
                    self.notification = Some(format!("error: {e}"));
                    return;
                }
            }
            None => return,
        }
        let outcome = match &self.farm {
            Some(h) => h.rename(&id, &new_id, false),
            None => return,
        };
        match outcome {
            Ok(RenameOutcome::Done(_)) => {
                self.reload();
                self.select_id(&new_id);
                let herd = new_id.split_once('-').map(|(p, _)| p).unwrap_or(&new_id);
                self.notification = Some(format!("{id} updated, moved to {herd}"));
            }
            Ok(RenameOutcome::Collision(t)) => {
                self.reload_preserving_selection();
                self.notification = Some(format!("{t} already exists"));
            }
            Ok(RenameOutcome::NotFound(t)) => {
                self.reload_preserving_selection();
                self.notification = Some(format!("{t} not found"));
            }
            Ok(RenameOutcome::Invalid(t)) => {
                self.reload_preserving_selection();
                self.notification = Some(format!("invalid id {t}"));
            }
            Ok(RenameOutcome::NothingToRename) => {
                self.reload_preserving_selection();
                self.notification = Some(format!("{id} updated"));
            }
            Err(e) => self.notification = Some(format!("error: {e}")),
        }
    }

    pub(crate) fn open_dep_picker(&mut self) {
        let Some(id) = self.selected_id() else { return };
        let mut exclude: HashSet<String> = HashSet::new();
        exclude.insert(id.clone());
        if let Some(t) = self.task(&id) {
            for d in &t.depends_on {
                exclude.insert(d.clone());
            }
        }
        // Exclude any task that already reaches `id`, which would form a cycle.
        for t in &self.all {
            if t.id != id && filter::depends_on_transitively(&self.all, &t.id, &id) {
                exclude.insert(t.id.clone());
            }
        }
        self.overlay = Overlay::Fuzzy(FuzzyPick::new(
            self.editor_vim,
            format!("Add dependency to {id}"),
            exclude,
            false,
            FuzzyAction::AddDep(id),
        ));
    }

    /// `#` — go to any yak by id or title: the fuzzy picker over every yak;
    /// Enter jumps to its detail with o/i history (yaks-63f3).
    pub(crate) fn open_goto_picker(&mut self) {
        self.overlay = Overlay::Fuzzy(FuzzyPick::new(
            self.editor_vim,
            "Go to yak".into(),
            HashSet::new(),
            false,
            FuzzyAction::Goto,
        ));
    }

    /// `B` — find references: the fuzzy picker over the yaks that point AT the
    /// selected one (children, dependents, id mentions in a title, description
    /// or note; `farm::backrefs_in`), each row tagged with how it refers;
    /// Enter goes to it like `#` does.
    pub(crate) fn open_backrefs_picker(&mut self) {
        let Some(id) = self.selected_id() else { return };
        let mut only: std::collections::BTreeMap<String, String> = Default::default();
        for r in crate::farm::backrefs_in(&self.all, &id) {
            let tag = only.entry(r.id).or_default();
            if !tag.is_empty() {
                tag.push_str(", ");
            }
            tag.push_str(&r.via.label());
        }
        if only.is_empty() {
            self.notification = Some(format!("no yak refers to {id}"));
            return;
        }
        let mut fp = FuzzyPick::new(
            self.editor_vim,
            format!("References to {id}"),
            HashSet::new(),
            false,
            FuzzyAction::Goto,
        );
        fp.only = Some(only);
        self.overlay = Overlay::Fuzzy(fp);
    }

    pub(crate) fn open_reparent_picker(&mut self) {
        let Some(id) = self.selected_id() else { return };
        let mut exclude = filter::descendant_ids(&self.all, &id, true);
        exclude.insert(id.clone());
        let has_parent = self.task(&id).map(|t| t.parent.is_some()).unwrap_or(false);
        if let Some(cur) = self.task(&id).and_then(|t| t.parent.clone()) {
            exclude.insert(cur);
        }
        self.overlay = Overlay::Fuzzy(FuzzyPick::new(
            self.editor_vim,
            format!("Reparent {id}"),
            exclude,
            has_parent,
            FuzzyAction::Reparent(id),
        ));
    }

    /// Open the yak-ref picker over the active editor so the chosen id is
    /// inserted at the editor's cursor (ref autocomplete, yaks-5656). The editor
    /// is stashed in the picker's `return_to`, so both commit and cancel come
    /// back to it with its content intact.
    /// Open the yak-ref picker over whatever editor overlay is active (the
    /// comment `Overlay::Edit` or the create/edit form `Overlay::Create`),
    /// stashing it so commit/cancel return to it. `replace_len` is the typed
    /// `<prefix>-` to drop before inserting the chosen id.
    pub(crate) fn open_ref_picker(&mut self, replace_len: usize, seed: &str) {
        let back = std::mem::replace(&mut self.overlay, Overlay::None);
        let mut fp = FuzzyPick::new(
            self.editor_vim,
            "Insert yak reference".into(),
            HashSet::new(),
            false,
            FuzzyAction::InsertRef,
        );
        fp.return_to = Some(Box::new(back));
        fp.replace_len = replace_len;
        // Pre-filter by whatever tail was already typed after `<prefix>-`.
        if !seed.is_empty() {
            insert_into(&mut fp.handler, &fp.query, seed);
        }
        self.overlay = Overlay::Fuzzy(fp);
    }

    /// n / N on a list: with no query, the search register becomes the query;
    /// with one, nothing (the list has no matches to step through).
    pub(crate) fn list_find(&mut self) {
        if self.query.is_some() {
            return;
        }
        match self.register.clone() {
            Some(q) => {
                self.query = Some(q);
                self.clamp_cursor();
            }
            None => self.notification = Some("nothing to find: / sets the search".into()),
        }
    }

    pub(crate) fn open_search(&mut self) {
        let cur = self.query.clone();
        self.overlay = Overlay::Search(SearchBox::new(self.editor_vim, cur));
    }

    pub(crate) fn open_drawer(&mut self) {
        let choices = self.herd_choices();
        self.overlay = Overlay::Drawer(Drawer::from_filter(self.editor_vim, &self.filter, choices));
    }

    pub(crate) fn open_help(&mut self) {
        self.overlay = Overlay::Help(0);
    }

    /// `l` / `→` / Enter on the list: show the selected yak (or shed) in the
    /// detail pane, from its top.
    pub(crate) fn open_detail_pane(&mut self) {
        if matches!(
            self.selected_row(),
            Some(tree::Item::Yak(_) | tree::Item::Shed(_))
        ) {
            self.focus = Focus::Detail;
            self.pane_yak = None;
            self.detail.to_top();
        }
    }

    /// Esc on the list: clear the query first, then drop the edits to the
    /// active view's facets.
    pub(crate) fn revert_list_view(&mut self) {
        if self.query.take().is_some() {
            self.clamp_cursor();
            self.notification = Some("search cleared".into());
        } else if self.is_view_modified() {
            self.revert_filter_to_view();
            self.notification = Some("reverted to view".into());
        }
    }

    /// A — attach an artifact (a file path, or the clipboard PNG when blank).
    pub(crate) fn open_attach(&mut self) {
        if let Some(id) = self.selected_id() {
            self.overlay = Overlay::Edit(Editor::new(
                self.editor_vim,
                true,
                "Attach path (empty = clipboard PNG): ".into(),
                "",
                EditAction::Attach(id),
            ));
        }
    }

    pub(crate) fn commit_attach(&mut self, id: String, path_input: String) {
        let path_input = path_input.trim();
        if path_input.is_empty() {
            match crate::clipboard::read_png() {
                Some(bytes) => self.open_paste_name(id, bytes),
                None => self.notification = Some("no PNG image on clipboard".into()),
            }
            return;
        }
        let p = std::path::Path::new(path_input);
        let Ok(bytes) = std::fs::read(p) else {
            self.notification = Some(format!("not a file: {path_input}"));
            return;
        };
        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("attachment")
            .to_string();
        self.attach_bytes(&id, &name, &bytes);
    }

    /// Second step of a clipboard-PNG attach: prompt for a filename (empty
    /// keeps the timestamped `paste-….png` default) so the pasted image doesn't
    /// land with an ugly name (yaks-fe19).
    pub(crate) fn open_paste_name(&mut self, id: String, data: Vec<u8>) {
        let default = format!("paste-{}.png", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
        self.overlay = Overlay::Edit(Editor::new(
            self.editor_vim,
            true,
            format!("Name pasted PNG (empty = {default}): "),
            "",
            EditAction::AttachPaste { id, default, data },
        ));
    }

    fn attach_bytes(&mut self, id: &str, name: &str, data: &[u8]) {
        let Some(h) = &self.farm else { return };
        match h.attach(id, name, data) {
            Ok(AttachOutcome::Attached(n)) => {
                self.reload();
                self.notification = Some(format!("attached {n}"));
            }
            Ok(AttachOutcome::NotFound) => self.notification = Some(format!("{id} not found")),
            Err(e) => self.notification = Some(format!("attach failed: {e}")),
        }
    }

    /// r (detail pane) — rename the attachment whose image link the detail
    /// cursor sits on (Tab/[ ] cycle to one). Opens a one-line prompt; the
    /// rename moves the file and rewrites its links farm-wide (yaks-a09b).
    pub(crate) fn open_rename_attachment(&mut self) {
        let target = self
            .detail_jumps()
            .into_iter()
            .find_map(|j| match j.target {
                detail::Target::Artifact(p) if j.line == self.detail.line => Some(p),
                _ => None,
            });
        let parsed = target.as_deref().and_then(|p| {
            // `<id>/<file>`, relative to the farm root (the yak's own directory).
            let rest = p.strip_prefix(".yaks/").unwrap_or(p);
            let (id, name) = rest.split_once('/')?;
            let name = crate::links::percent_decode(name);
            (!name.contains('/')).then(|| (id.to_string(), name))
        });
        let Some((id, old)) = parsed else {
            self.notification = Some("no attachment on this line (Tab to an image link)".into());
            return;
        };
        self.overlay = Overlay::Edit(Editor::new(
            self.editor_vim,
            true,
            format!("Rename {old} to (ext kept if omitted): "),
            "",
            EditAction::RenameAttachment { id, old },
        ));
    }

    pub(crate) fn commit_rename_attachment(&mut self, id: String, old: String, new: String) {
        use crate::farm::RenameAttachmentOutcome as R;
        if new.trim().is_empty() {
            self.notification = Some("rename cancelled".into());
            return;
        }
        let Some(h) = &self.farm else { return };
        self.notification = Some(match h.rename_attachment(&id, &old, &new) {
            Ok(R::Renamed { name, .. }) => {
                self.reload();
                format!("renamed {old} → {name}")
            }
            Ok(R::Unchanged) => format!("{old}: name unchanged"),
            Ok(R::Collision(n)) => format!("{id} already has {n}"),
            Ok(R::Invalid(n)) => format!("invalid name: {n}"),
            Ok(R::NoSuchAttachment(n)) => format!("no attachment {n} on {id}"),
            Ok(R::TaskNotFound) => format!("{id} not found"),
            Err(e) => format!("rename failed: {e}"),
        });
    }

    /// Open a URL or attachment (`Enter` on its link, or `O`). An attachment
    /// (`<id>/<file>`) is resolved by [`App::resolve_attachment`]. A text
    /// attachment opens in the in-TUI viewer ([`AttachView`], yaks-699b) and a
    /// decodable image in the image viewer ([`ImageView`], yaks-4305); a URL or
    /// any other file goes to the OS default application (best-effort).
    pub(crate) fn open_external(&mut self, target: &str) {
        let arg = if target.contains("://") || target.starts_with("mailto:") {
            target.to_string()
        } else {
            match self.resolve_attachment(target) {
                Some(found) => {
                    if let Some(v) = ImageView::open(target, &found, &self.picker) {
                        self.notification = Some(format!("viewing {target}"));
                        self.follow_to_doc(DocView::Image(v));
                        return;
                    }
                    if let Some(v) = AttachView::open(target, &found) {
                        self.notification = Some(format!("viewing {target}"));
                        self.follow_to_doc(DocView::Text(v));
                        return;
                    }
                    found.path.display().to_string()
                }
                None => {
                    self.notification = Some(format!("{target}: not in this farm"));
                    return;
                }
            }
        };
        self.os_open(target, &arg);
    }

    /// After cancelling editor `ov`: an attachment edit puts its viewer back
    /// in the pane (the file unchanged); anything else just closes.
    fn drop_editor(&mut self, ov: Overlay) {
        self.overlay = Overlay::None;
        if let Overlay::Edit(Editor {
            action: EditAction::EditAttachment(v, from),
            ..
        }) = ov
        {
            self.doc = Some(DocPane {
                view: DocView::Text(v),
                from,
            });
        }
    }

    /// Hand `arg` (a URL or file path) to the OS opener, reporting it as
    /// `target`. Under `cargo test` nothing is launched (a test must never
    /// pop up an image viewer); the notification is the same.
    pub(super) fn os_open(&mut self, target: &str, arg: &str) {
        #[cfg(not(test))]
        let opened = super::source::os_open(arg).map(drop);
        #[cfg(test)]
        let opened: std::io::Result<()> = {
            let _ = arg;
            Ok(())
        };
        self.notification = Some(match opened {
            Ok(()) => format!("opened {target}"),
            Err(e) => format!("open failed: {e}"),
        });
    }

    /// The open viewer's rows at the width last rendered, and its viewport
    /// (the pane less the header row).
    pub(crate) fn attachment_rows(&self) -> Option<(Vec<detail::DLine>, Viewport)> {
        let v = self.attach()?;
        let known: HashSet<&str> = self.all.iter().map(|t| t.id.as_str()).collect();
        let rows = v.rows(v.width.get(), &known);
        // The header row, and the peek pane when it is shown (yaks-7395).
        let peek = if self.viewer_has_peek(&rows) {
            super::follow::PEEK_ROWS
        } else {
            0
        };
        let vp = Viewport {
            page: self.detail_page.saturating_sub(1 + peek).max(1) as usize,
            pinned: 0,
        };
        Some((rows, vp))
    }

    /// Keys in the pane's find box, which edits the search register for the
    /// subject the pane shows: the open page, else the yak's detail, by the
    /// same [`LineNav`] (yaks-84e0). Other keys in the viewer are the
    /// [`keymap::Ctx::Viewer`] commands, dispatched like the list's.
    fn handle_find_key(&mut self, k: KeyEvent, double_esc: bool) {
        let cancel = self.field_cancel(k, double_esc);
        let Overlay::Find(mut sb) = std::mem::replace(&mut self.overlay, Overlay::None) else {
            return;
        };
        let mut register = self.register.take();
        let closed = match self.attachment_rows() {
            Some((lines, _)) => match self.attach_mut() {
                Some(v) => v.nav.find_key(&mut sb, k, cancel, &lines, &mut register),
                None => true,
            },
            None => {
                let lines = self.detail_dlines();
                self.detail
                    .find_key(&mut sb, k, cancel, &lines, &mut register)
            }
        };
        self.register = register;
        if !closed {
            self.overlay = Overlay::Find(sb);
        }
    }

    /// Run a viewer command: `j`/`k`/`d`/`u`/`g`/`G` move the line cursor,
    /// `Tab`/`[`/`]` cycle links, `Enter` follows one (or yanks a selection),
    /// `v`/Shift-↑↓ select, `y` yanks, `/` `n` `N` find, `o`/`i` go back /
    /// forward in the history, `h`/`l` scroll a code block or table
    /// sideways. `E` edits the file, `O` hands it to the OS opener; `Esc`
    /// peels back selection, then find, then closes, as do `q` and `h`/`←`
    /// (off a scrolled block).
    pub(super) fn run_viewer_cmd(&mut self, cmd: Cmd) {
        let Some((lines, vp)) = self.attachment_rows() else {
            return;
        };
        // The search register is the app's, not the page's: these commands
        // read or write it, so they run before the page's state is borrowed.
        match cmd {
            Cmd::FindOpen => return self.open_find(),
            Cmd::FindNext | Cmd::FindPrev => {
                let delta = if cmd == Cmd::FindNext { 1 } else { -1 };
                let register = self.register.clone();
                let found = self
                    .attach_mut()
                    .is_some_and(|v| v.nav.find_step(&lines, register.as_deref(), delta, true));
                if !found {
                    self.notification = Some(match register {
                        Some(q) => format!("no match for \"{q}\""),
                        None => "nothing to find: / sets the search".into(),
                    });
                }
                return;
            }
            // Esc peels a selection first, then the register, then closes.
            Cmd::PanePeel
                if self.register.is_some()
                    && self.attach().is_some_and(|v| v.nav.anchor.is_none()) =>
            {
                self.register = None;
                if let Some(v) = self.attach_mut() {
                    v.nav.matched = None;
                }
                return;
            }
            _ => {}
        }
        let Some(v) = self.attach_mut() else {
            return;
        };
        let width = v.width.get() as usize;
        let nav = &mut v.nav;
        let half = (vp.page / 2).max(1) as i32;
        match cmd {
            Cmd::SelectExtendDown | Cmd::SelectExtendUp => {
                nav.extend_selection(
                    &lines,
                    vp,
                    if cmd == Cmd::SelectExtendDown { 1 } else { -1 },
                );
                return;
            }
            Cmd::PanePeel => {
                if nav.anchor.is_some() {
                    nav.anchor = None;
                } else {
                    self.close_doc();
                }
            }
            Cmd::PaneClose => self.close_doc(),
            Cmd::PaneBack => {
                if !nav.scroll_block(&lines, width, -1) {
                    self.close_doc();
                }
            }
            Cmd::PeekToggle => {
                self.peek_shown = !self.peek_shown;
                self.notification = Some(
                    if self.peek_shown {
                        "peek shown"
                    } else {
                        "peek hidden"
                    }
                    .into(),
                );
            }
            Cmd::PaneScrollRight => {
                nav.scroll_block(&lines, width, 1);
            }
            Cmd::MoveDown => nav.step(&lines, vp, 1),
            Cmd::MoveUp => nav.step(&lines, vp, -1),
            Cmd::MoveHalfDown => nav.step(&lines, vp, half),
            Cmd::MoveHalfUp => nav.step(&lines, vp, -half),
            Cmd::MoveTop => nav.to_end(&lines, vp, false),
            Cmd::MoveBottom => nav.to_end(&lines, vp, true),
            Cmd::LinkNext => nav.jump_link(&lines, vp, 1),
            Cmd::LinkPrev => nav.jump_link(&lines, vp, -1),
            Cmd::SelectVisual => nav.toggle_visual(),
            Cmd::NavBack => return self.history_back(),
            Cmd::NavForward => return self.history_forward(),
            Cmd::SelectYank => self.yank_attachment(&lines),
            Cmd::PaneEnter => {
                if nav.anchor.is_some() {
                    self.yank_attachment(&lines);
                } else {
                    self.follow_attachment_link(&lines, vp);
                }
            }
            Cmd::DocEdit => self.open_attachment_edit(),
            Cmd::FileOpenExternal => {
                let (target, arg) = (v.target.clone(), v.path.display().to_string());
                self.close_doc();
                self.os_open(&target, &arg);
            }
            _ => {}
        }
        // A no-wrap block's sideways scroll lives only while the cursor is on it.
        if let Some(v) = self.attach_mut() {
            v.nav.drop_stale_hscroll(&lines);
        }
    }

    /// y / Enter with a selection (viewer) — copy the selected rows (source
    /// markdown for a `.md` file) and clear the selection.
    fn yank_attachment(&mut self, lines: &[detail::DLine]) {
        let Some(v) = self.attach_mut() else {
            return;
        };
        let (lo, hi) = v.nav.selection().unwrap_or((v.nav.line, v.nav.line));
        v.nav.anchor = None;
        let Some(text) = super::linenav::yank_text(lines, lo, hi, &v.blocks()) else {
            return;
        };
        let n = hi.min(lines.len().saturating_sub(1)) + 1 - lo;
        self.notification = Some(if crate::clipboard::copy_text(&text) {
            format!("copied {n} line(s)")
        } else {
            "clipboard unavailable".into()
        });
    }

    /// Enter (viewer) — follow the link on the cursor's row with the detail
    /// pane's rules: a footnote or `#heading` moves within the file (`o`
    /// returns); a yak replaces the viewer in the detail pane; a
    /// farm document opens in the viewer at its anchor (yaks-7395, `o` comes
    /// back); an attachment, URL or source file opens as from the detail pane.
    fn follow_attachment_link(&mut self, lines: &[detail::DLine], vp: Viewport) {
        let Some(v) = self.attach() else {
            return;
        };
        let Some(j) = v.nav.link_at(lines) else {
            return;
        };
        match j.target {
            detail::Target::Footnote(n) => match detail::footnote_line(lines, v.nav.line, &n) {
                Some(to) => {
                    self.jump_in_page(lines, vp, to);
                    self.notification = Some(format!("→ [{n}] (o to go back)"));
                }
                None => self.notification = Some(format!("no definition for [{n}]")),
            },
            // Closes the viewer (the yak replaces it) and opens the yak.
            detail::Target::Task(id) => self.goto_task(&id),
            detail::Target::Url(u) | detail::Target::Artifact(u) => self.open_external(&u),
            detail::Target::Source { path, lines, rev } => {
                self.open_source(&path, lines, rev.as_deref())
            }
            detail::Target::Farm { path, rev, anchor } => {
                self.open_farm_doc(&path, rev.as_deref(), anchor.as_ref())
            }
            detail::Target::Local(a) => self.follow_local_anchor(lines, vp, &a),
        }
    }

    /// E (attachment viewer) — edit the viewed text file in the multi-line
    /// editor (yaks-56c1).
    pub(crate) fn open_attachment_edit(&mut self) {
        let Some(v) = self.attach() else {
            return;
        };
        if let Some(rev) = &v.rev {
            self.notification = Some(format!(
                "{}@{}: a past version is read-only",
                v.target,
                &rev[..rev.len().min(7)]
            ));
            return;
        }
        // A source is a snapshot, written once (yaks-1532).
        if matches!(v.doc, DocKind::Source { .. }) {
            self.notification = Some(format!(
                "{}: a source is a snapshot, written once; cite it from a page",
                v.target
            ));
            return;
        }
        let Some(DocPane {
            view: DocView::Text(v),
            from,
        }) = self.doc.take()
        else {
            unreachable!()
        };
        let label = format!("Edit {} (Ctrl-S save · Ctrl-C cancel)", v.target);
        let text = v.text.clone();
        self.overlay = Overlay::Edit(Editor::new(
            self.editor_vim,
            false,
            label,
            &text,
            EditAction::EditAttachment(v, from),
        ));
    }

    /// Save an attachment or page edit: write `text` to the viewed file under
    /// the farm lock (so a farm-branch farm commits it as one change,
    /// yaks-a838), then reopen the viewer on it (markdown re-rendered). A page
    /// (yaks-1532) is parsed and written by `pages::save`, frontmatter and
    /// body, and must keep its `---` frontmatter block. If the file changed on
    /// disk since the edit began, nothing is written and the editor stays open
    /// with the typed text; a second Ctrl-S overwrites.
    pub(crate) fn commit_attachment_edit(&mut self, mut v: AttachView, from: Focus, text: String) {
        let back = |v| {
            Some(DocPane {
                view: DocView::Text(v),
                from,
            })
        };
        if text == v.text {
            self.notification = Some(format!("{}: unchanged", v.target));
            self.doc = back(v);
            return;
        }
        let Some(root) = self.farm.as_ref().map(|h| h.root().to_path_buf()) else {
            return;
        };
        let page = v.doc == DocKind::Page;
        if page && crate::store::split_frontmatter(&text).is_none() {
            self.notification = Some(format!(
                "{}: not saved: a page starts with a --- frontmatter block closed by ---",
                v.target
            ));
            self.reopen_attachment_edit(v, from, &text);
            return;
        }
        let saved = (|| -> anyhow::Result<Result<String, String>> {
            let _lock = crate::store::lock(&root)?;
            let on_disk = std::fs::read_to_string(&v.path)?;
            if on_disk != v.text {
                return Ok(Err(on_disk));
            }
            if page {
                crate::pages::save(&root, &crate::pages::Page::parse(&v.target, &text))?;
            } else {
                crate::store::write::atomic_write(&v.path, &text)?;
            }
            // What is on disk now (a page is written in its normal form).
            Ok(Ok(std::fs::read_to_string(&v.path)?))
        })();
        match saved {
            Ok(Ok(written)) => {
                self.notification = Some(format!("saved {}", v.target));
                v.text = written;
                self.doc = back(v);
                if page {
                    // Its title, kind or tags may have changed (the Wiki tab).
                    self.reload_wiki();
                    self.refresh_farm_sync();
                }
                // The file may have shrunk: keep the cursor on it.
                if let Some((lines, vp)) = self.attachment_rows() {
                    if let Some(v) = self.attach_mut() {
                        v.nav.clamp(lines.len(), vp);
                    }
                }
            }
            Ok(Err(on_disk)) => {
                self.notification = Some(format!(
                    "{} changed on disk: not saved (Ctrl-S again overwrites)",
                    v.target
                ));
                v.text = on_disk;
                self.reopen_attachment_edit(v, from, &text);
            }
            Err(e) => {
                self.notification = Some(format!("save failed: {e}"));
                self.reopen_attachment_edit(v, from, &text);
            }
        }
    }

    /// Put the attachment editor back with the typed `text` after a save that
    /// did not write, so no edit is lost.
    fn reopen_attachment_edit(&mut self, v: AttachView, from: Focus, text: &str) {
        let label = format!("Edit {} (Ctrl-S save · Ctrl-C cancel)", v.target);
        self.overlay = Overlay::Edit(Editor::new(
            self.editor_vim,
            false,
            label,
            text,
            EditAction::EditAttachment(v, from),
        ));
    }

    /// Where attachment link `target` (`<id>/<file>`, farm-relative) is on
    /// disk in this farm: the same rule as `yaks attachment`.
    pub(crate) fn resolve_attachment(&self, target: &str) -> Option<crate::farm::FoundAttachment> {
        let rel = target.strip_prefix(".yaks/").unwrap_or(target);
        let (id, name) = rel.split_once('/')?;
        crate::farm::locate_attachment(
            self.farm.as_ref()?.root(),
            id,
            &crate::links::percent_decode(name),
        )
    }

    /// O — open the artifact/URL/source link on the current detail line (a
    /// text attachment in the viewer, anything else externally).
    pub(crate) fn open_current_external(&mut self) {
        let jumps = self.detail_jumps();
        match jumps.into_iter().find(|j| j.line == self.detail.line) {
            Some(j) => match j.target {
                detail::Target::Artifact(p) => self.open_external(&p),
                detail::Target::Url(u) => self.open_external(&u),
                detail::Target::Source { path, lines, rev } => {
                    self.open_source(&path, lines, rev.as_deref())
                }
                detail::Target::Farm { path, rev, anchor } => {
                    self.open_farm_doc(&path, rev.as_deref(), anchor.as_ref())
                }
                detail::Target::Task(_)
                | detail::Target::Footnote(_)
                | detail::Target::Local(_) => {
                    self.notification = Some("not an artifact/link".into())
                }
            },
            None => self.notification = Some("no link on this line".into()),
        }
    }

    /// M — append a timestamped comment/note to the selected task (multi-line).
    pub(crate) fn open_comment(&mut self) {
        self.open_block_edit(None);
    }

    /// `a`, the human's hand-off key: answer the selected yak if it is awaiting
    /// a human, otherwise request work on it from an agent. Either way it ends
    /// `needs: agent`. One key, two verbs — the editor's header states which.
    /// Both open the multi-line comment editor (`M`'s), whose save goes
    /// through `needs_edit`. Asking is `Q` ([`Self::open_ask`]).
    pub(crate) fn open_answer_or_request(&mut self) {
        let blocked = self.selected().is_some_and(|t| t.awaiting_human());
        self.open_needs_edit(if blocked {
            NeedsVerb::Answer
        } else {
            NeedsVerb::Request
        });
    }

    /// `Q` — ask: Ctrl-S sets `needs: human`, recording the typed question (on
    /// a `needs: agent` yak that flips it back to a human).
    pub(crate) fn open_ask(&mut self) {
        self.open_needs_edit(NeedsVerb::Ask);
    }

    /// Ask, answer or request via the farm, appending the typed text as an
    /// attributed note when non-empty. Backs the `Ask`/`Answer`/`Request` edit
    /// actions.
    pub(crate) fn needs_edit(&mut self, id: &str, verb: NeedsVerb, note: Option<&str>) {
        let actor = crate::actor::resolve(None);
        let Some(h) = &self.farm else { return };
        let res = match verb {
            NeedsVerb::Answer => h
                .answer(id, actor.as_deref(), note, false)
                .map(|c| c.is_some()),
            NeedsVerb::Ask => h
                .ask(id, "human", actor.as_deref(), note)
                .map(|c| c.is_some()),
            NeedsVerb::Request => match h.request(id, actor.as_deref(), note) {
                Ok(crate::farm::Request::NotFound) => Ok(false),
                Ok(crate::farm::Request::AwaitingHuman(_)) => {
                    self.notification =
                        Some(format!("{id} is awaiting a human: answer it instead (a)"));
                    return;
                }
                Ok(crate::farm::Request::Requested(_)) => Ok(true),
                Err(e) => Err(e),
            },
        };
        match res {
            Ok(true) => {
                self.reload();
                self.notification = Some(match verb {
                    NeedsVerb::Answer => format!("answered {id}: needs agent (awaiting pickup)"),
                    NeedsVerb::Request => format!("requested {id}: needs agent (awaiting pickup)"),
                    NeedsVerb::Ask => {
                        format!("asked {id}: needs human (out of next until answered)")
                    }
                });
            }
            Ok(false) => self.notification = Some(format!("{id} not found")),
            Err(e) => self.notification = Some(format!("error: {e}")),
        }
    }

    pub(crate) fn handle_help_key(&mut self, k: KeyEvent) {
        // Any of Esc/q/? dismisses the reference.
        if matches!(
            k.code,
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')
        ) {
            self.overlay = Overlay::None;
            return;
        }
        let vh = self.detail_page.max(1);
        let max_scroll = (help_content().len() as u16).saturating_sub(vh);
        let half = (vh / 2).max(1);
        if let Overlay::Help(scroll) = &mut self.overlay {
            match k.code {
                KeyCode::Char('j') | KeyCode::Down => *scroll = (*scroll + 1).min(max_scroll),
                KeyCode::Char('k') | KeyCode::Up => *scroll = scroll.saturating_sub(1),
                KeyCode::Char('d') | KeyCode::PageDown => {
                    *scroll = (*scroll + half).min(max_scroll)
                }
                KeyCode::Char('u') | KeyCode::PageUp => *scroll = scroll.saturating_sub(half),
                KeyCode::Char('g') => *scroll = 0,
                KeyCode::Char('G') => *scroll = max_scroll,
                _ => {}
            }
        }
    }

    pub(crate) fn open_save_view(&mut self) {
        self.overlay = Overlay::Edit(Editor::new(
            self.editor_vim,
            true,
            "Save view as: ".into(),
            "",
            EditAction::SaveView,
        ));
    }

    pub(crate) fn drawer_live_preview(&mut self) {
        let spec = match &self.overlay {
            Overlay::Drawer(d) => Some(d.build_spec()),
            _ => None,
        };
        if let Some(s) = spec {
            self.filter = s;
            self.clamp_cursor();
        }
    }

    pub(crate) fn close_drawer(&mut self, commit: bool) {
        if let Overlay::Drawer(d) = std::mem::replace(&mut self.overlay, Overlay::None) {
            self.filter = if commit { d.build_spec() } else { d.saved };
            self.clamp_cursor();
            self.notification = Some(if commit {
                "filter applied".into()
            } else {
                "filter unchanged".into()
            });
        }
    }

    pub(crate) fn handle_drawer_key(&mut self, k: KeyEvent, double_esc: bool) {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let (row, is_text) = match &self.overlay {
            Overlay::Drawer(d) => (d.row, d.is_text_row()),
            _ => return,
        };
        // Global apply / cancel / clear. Esc closes the drawer, except on a text
        // row in vim, where a lone Esc drops the field to Normal (motions); a
        // Ctrl-C or a rapid double-Esc always closes.
        let close = (ctrl && k.code == KeyCode::Char('c'))
            || double_esc
            || (k.code == KeyCode::Esc && !(self.editor_vim && is_text));
        if k.code == KeyCode::Enter {
            return self.close_drawer(true);
        }
        if close {
            return self.close_drawer(false);
        }
        if k.code == KeyCode::Char('C') {
            if let Overlay::Drawer(d) = &mut self.overlay {
                d.clear();
            }
            return self.drawer_live_preview();
        }
        // Row navigation. On text rows only Tab/arrows/Ctrl move rows (so j/k
        // remain typeable); on chip rows j/k also navigate.
        // On a text row in Normal mode, j/k are free, so they move rows too.
        let text_normal = is_text
            && matches!(&self.overlay, Overlay::Drawer(d)
                if d.text_editor().is_some_and(|e| e.borrow().mode == EditorMode::Normal));
        let nav_down = matches!(k.code, KeyCode::Down | KeyCode::Tab)
            || (ctrl && k.code == KeyCode::Char('n'))
            || (!is_text && k.code == KeyCode::Char('j'))
            || (text_normal && k.code == KeyCode::Char('j'));
        let nav_up = matches!(k.code, KeyCode::Up | KeyCode::BackTab)
            || (ctrl && k.code == KeyCode::Char('p'))
            || (!is_text && k.code == KeyCode::Char('k'))
            || (text_normal && k.code == KeyCode::Char('k'));
        if nav_down || nav_up {
            let carry = text_normal && matches!(k.code, KeyCode::Char('j') | KeyCode::Char('k'));
            if let Overlay::Drawer(d) = &mut self.overlay {
                let n = d.row_count();
                d.row = if nav_down {
                    (d.row + 1) % n
                } else {
                    (d.row + n - 1) % n
                };
                d.chip_idx = 0;
                if carry {
                    if let Some(e) = d.text_editor() {
                        e.borrow_mut().mode = EditorMode::Normal;
                    }
                }
            }
            return;
        }
        if is_text {
            if let Overlay::Drawer(d) = &mut self.overlay {
                match row {
                    3 => d.handler.on_key_event(k, &mut d.tags.borrow_mut()),
                    4 => d.handler.on_key_event(k, &mut d.search.borrow_mut()),
                    5 => d.handler.on_key_event(k, &mut d.parent.borrow_mut()),
                    _ => {}
                }
            }
            return self.drawer_live_preview();
        }
        // Chip rows: left/right move the cursor, Space toggles.
        let mut changed = false;
        if let Overlay::Drawer(d) = &mut self.overlay {
            let n = d.chip_count();
            match k.code {
                KeyCode::Left | KeyCode::Char('h') if n > 0 => {
                    d.chip_idx = (d.chip_idx + n - 1) % n
                }
                KeyCode::Right | KeyCode::Char('l') if n > 0 => d.chip_idx = (d.chip_idx + 1) % n,
                KeyCode::Char(' ') => {
                    d.toggle_chip();
                    changed = true;
                }
                _ => {}
            }
        }
        if changed {
            self.drawer_live_preview();
        }
    }

    /// Record an `Esc` in an editor overlay and report whether it's a *rapid*
    /// second `Esc` (within [`DOUBLE_ESC_MS`] of the previous one) — the
    /// Ctrl-C-equivalent cancel gesture. A lone `Esc` keeps its usual meaning
    /// (leave Insert for Normal); only the fast second press cancels.
    pub(crate) fn register_double_esc(&mut self) -> bool {
        let now = std::time::Instant::now();
        let double = self.last_esc.is_some_and(|t| {
            now.duration_since(t) <= std::time::Duration::from_millis(DOUBLE_ESC_MS)
        });
        // Clear after a double so a third quick Esc doesn't also fire.
        self.last_esc = if double { None } else { Some(now) };
        double
    }

    /// Cancel the current edit overlay. When `dirty`, stash it behind a "discard
    /// changes?" confirmation (restored if declined) instead of dropping the work;
    /// otherwise cancel immediately with `msg`.
    pub(crate) fn request_cancel(&mut self, dirty: bool, msg: &str) {
        if dirty {
            let stashed = std::mem::replace(&mut self.overlay, Overlay::None);
            self.dirty_cancel = Some(stashed);
            self.overlay = Overlay::Confirm {
                prompt: "Discard changes? (y/N): ".into(),
                action: ConfirmAction::DiscardEdit,
            };
        } else {
            let ov = std::mem::replace(&mut self.overlay, Overlay::None);
            self.drop_editor(ov);
            self.notification = Some(msg.to_string());
        }
    }

    /// Whether the create/edit form differs from its starting point (the seeded
    /// task when editing, or the empty defaults when creating).
    pub(crate) fn form_is_dirty(&self, f: &CreateForm) -> bool {
        match &f.edit_id {
            Some(id) => match self.task(id) {
                Some(cur) => {
                    let mut a = f.tags_vec();
                    let mut b = crate::model::normalize_tags(&cur.tags);
                    a.sort();
                    b.sort();
                    f.title_text() != cur.title
                        || TYPE_CHOICES[f.kind_idx] != cur.kind
                        || PRI_CHOICES[f.pri_idx] != cur.priority
                        || a != b
                }
                None => true,
            },
            None => {
                !f.title_text().is_empty()
                    || !f.description_text().trim().is_empty()
                    || !f.tags_vec().is_empty()
                    || f.kind_idx != 0
                    || f.pri_idx != pri_index(3)
            }
        }
    }

    /// Edit the active `:` command line. Enter runs it, Esc dismisses it,
    /// Backspace deletes, and printable chars are appended.
    pub(crate) fn handle_cmdline_key(&mut self, k: KeyEvent) {
        match k.code {
            KeyCode::Esc => self.cmdline = None,
            KeyCode::Enter => {
                let cmd = self.cmdline.take().unwrap_or_default();
                self.run_command(&cmd);
            }
            KeyCode::Backspace => {
                if let Some(s) = &mut self.cmdline {
                    s.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Some(s) = &mut self.cmdline {
                    s.push(c);
                }
            }
            _ => {}
        }
    }

    /// Run a `:` command against the active editor overlay. The verb set is
    /// deliberately small (`w`/`q`/`wq`/`x`/`q!`); the match's catch-all is the
    /// extension point for any future `:` commands.
    pub(crate) fn run_command(&mut self, cmd: &str) {
        match cmd.trim() {
            "" => {}
            "w" | "wq" | "x" => self.cmd_write(),
            "q" => self.cmd_quit(false),
            "q!" => self.cmd_quit(true),
            other => self.notification = Some(format!("unknown command: :{other}")),
        }
    }

    /// `:w` / `:wq` / `:x` — commit the active overlay (a modal editor saves and
    /// closes, so all three behave alike).
    pub(crate) fn cmd_write(&mut self) {
        match &self.overlay {
            Overlay::Create(f) => {
                if f.title_text().is_empty() {
                    self.notification = Some("need a title".into());
                } else {
                    self.commit_form();
                }
            }
            Overlay::Edit(_) => {
                if let Overlay::Edit(ed) = std::mem::replace(&mut self.overlay, Overlay::None) {
                    self.commit_edit(ed);
                }
            }
            Overlay::BlockEdit(_) => {
                if let Overlay::BlockEdit(be) = std::mem::replace(&mut self.overlay, Overlay::None)
                {
                    self.commit_edit(be.editor);
                }
            }
            _ => {}
        }
    }

    /// `:q` — cancel the active overlay (respecting the dirty-discard confirm);
    /// `:q!` force-cancels, discarding unsaved work outright.
    pub(crate) fn cmd_quit(&mut self, force: bool) {
        if force {
            let ov = std::mem::replace(&mut self.overlay, Overlay::None);
            self.drop_editor(ov);
            self.notification = Some("cancelled".into());
            return;
        }
        let (dirty, msg) = match &self.overlay {
            Overlay::Create(f) => (
                self.form_is_dirty(f),
                if f.is_editing() {
                    "edit cancelled"
                } else {
                    "create cancelled"
                },
            ),
            Overlay::Edit(ed) => (edit_is_dirty(ed), "cancelled"),
            Overlay::BlockEdit(be) => (be.is_dirty(), "cancelled"),
            _ => (false, "cancelled"),
        };
        self.request_cancel(dirty, msg);
    }

    /// Whether a key should trigger a single-line field overlay's cancel/close
    /// path. In vim a lone Esc instead drops the field to Normal (so the
    /// normal-mode motions are reachable) and only Ctrl-C or a rapid double-Esc
    /// cancels; emacs keeps the familiar single-Esc-cancels.
    pub(crate) fn field_cancel(&self, k: KeyEvent, double_esc: bool) -> bool {
        (k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c'))
            || double_esc
            || (k.code == KeyCode::Esc && !self.editor_vim)
    }

    pub(crate) fn handle_overlay_key(&mut self, k: KeyEvent) {
        // An active `:` command line intercepts everything until it resolves.
        if self.cmdline.is_some() {
            self.handle_cmdline_key(k);
            return;
        }
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        // A rapid double-Esc acts as Ctrl-C (cancel) in editor overlays, so Esc
        // can otherwise mean "leave Insert for Normal" without trapping the user.
        let double_esc = k.code == KeyCode::Esc && self.register_double_esc();
        if matches!(self.overlay, Overlay::Drawer(_)) {
            self.handle_drawer_key(k, double_esc);
            return;
        }
        if matches!(self.overlay, Overlay::Create(_)) {
            self.handle_create_key(k, double_esc);
            return;
        }
        if matches!(self.overlay, Overlay::Help(_)) {
            self.handle_help_key(k);
            return;
        }
        if matches!(self.overlay, Overlay::ViewPicker(_)) {
            self.handle_view_picker_key(k);
            return;
        }
        // The fuzzy picker: nav/commit/cancel are intercepted; everything else
        // edits the query (and resets the selection to the top match).
        if matches!(self.overlay, Overlay::Fuzzy(_)) {
            // In Normal mode j/k move the result selection (the query's own j/k
            // have no single-line meaning).
            let q_normal = matches!(&self.overlay, Overlay::Fuzzy(fp)
                if fp.query.borrow().mode == EditorMode::Normal);
            let up = k.code == KeyCode::Up
                || (ctrl && k.code == KeyCode::Char('p'))
                || (q_normal && k.code == KeyCode::Char('k'));
            let down = matches!(k.code, KeyCode::Down | KeyCode::Tab)
                || (ctrl && k.code == KeyCode::Char('n'))
                || (q_normal && k.code == KeyCode::Char('j'));
            // Once the query is in Normal mode (after the first Esc dropped it
            // there), a single further Esc closes the picker — no rapid
            // double-Esc needed. Ctrl-C still cancels from anywhere.
            let esc_from_normal = q_normal && k.code == KeyCode::Esc;
            if self.field_cancel(k, double_esc) || esc_from_normal {
                // A picker opened over an editor (ref autocomplete) returns there;
                // a top-level picker just closes.
                let back = if let Overlay::Fuzzy(fp) = &mut self.overlay {
                    fp.return_to.take()
                } else {
                    None
                };
                match back {
                    Some(b) => self.overlay = *b,
                    None => {
                        self.overlay = Overlay::None;
                        self.notification = Some("cancelled".into());
                    }
                }
            } else if k.code == KeyCode::Enter {
                if let Overlay::Fuzzy(fp) = std::mem::replace(&mut self.overlay, Overlay::None) {
                    self.commit_fuzzy(fp);
                }
            } else if up {
                if let Overlay::Fuzzy(fp) = &mut self.overlay {
                    fp.sel = fp.sel.saturating_sub(1);
                }
            } else if down {
                let total = match &self.overlay {
                    Overlay::Fuzzy(fp) => fuzzy_total(&self.all, fp),
                    _ => 0,
                };
                if let Overlay::Fuzzy(fp) = &mut self.overlay {
                    if total > 0 {
                        fp.sel = (fp.sel + 1).min(total - 1);
                    }
                }
            } else if let Overlay::Fuzzy(fp) = &mut self.overlay {
                fp.handler.on_key_event(k, &mut fp.query.borrow_mut());
                fp.sel = 0;
            }
            return;
        }
        // Inline search: every keystroke edits the global query for instant
        // preview; Enter keeps it, Esc restores the pre-search query.
        if matches!(self.overlay, Overlay::Search(_)) {
            if self.field_cancel(k, double_esc) {
                let saved = match &self.overlay {
                    Overlay::Search(sb) => sb.saved.clone(),
                    _ => None,
                };
                self.query = saved;
                self.overlay = Overlay::None;
                self.clamp_cursor();
                self.notification = Some("search cleared".into());
            } else if k.code == KeyCode::Enter {
                let q = match &self.overlay {
                    Overlay::Search(sb) => sb.query_text(),
                    _ => String::new(),
                };
                self.overlay = Overlay::None;
                // A kept query is already visible as its chip; only the
                // empty commit (which clears it) says anything.
                if q.is_empty() {
                    self.notification = Some("search cleared".into());
                }
            } else {
                if let Overlay::Search(sb) = &mut self.overlay {
                    sb.handler.on_key_event(k, &mut sb.query.borrow_mut());
                }
                let q = match &self.overlay {
                    Overlay::Search(sb) => sb.query_text(),
                    _ => String::new(),
                };
                self.query = if q.is_empty() { None } else { Some(q) };
                self.clamp_cursor();
            }
            return;
        }
        // The pane's find edits the search register: live-highlight matches;
        // Enter keeps, Esc restores (yaks-84e0, yaks-c215).
        if matches!(self.overlay, Overlay::Find(_)) {
            self.handle_find_key(k, double_esc);
            return;
        }
        // Editors are handled in place (most keys flow to edtui); only the
        // Pick/Confirm variants move their action out on resolution.
        if matches!(self.overlay, Overlay::Edit(_) | Overlay::BlockEdit(_)) {
            let ref_prefix = self.ref_prefix.clone();
            let (key, dirty) = match &mut self.overlay {
                Overlay::Edit(ed) => {
                    let key = editor_key(ed, k, double_esc, &ref_prefix);
                    (key, edit_is_dirty(ed))
                }
                Overlay::BlockEdit(be) => {
                    let key = editor_key(&mut be.editor, k, double_esc, &ref_prefix);
                    (key, be.is_dirty())
                }
                _ => unreachable!(),
            };
            match key {
                EditKey::Commit => match std::mem::replace(&mut self.overlay, Overlay::None) {
                    Overlay::Edit(ed) => self.commit_edit(ed),
                    Overlay::BlockEdit(be) => self.commit_edit(be.editor),
                    _ => {}
                },
                EditKey::Complete(replace_len, seed) => self.open_ref_picker(replace_len, &seed),
                EditKey::Cmdline => self.cmdline = Some(String::new()),
                EditKey::Cancel => self.request_cancel(dirty, "cancelled"),
                EditKey::Handled => {}
            }
            // The editor may have grown or shrunk: keep all of it on screen.
            self.show_block_editor();
            return;
        }
        match std::mem::replace(&mut self.overlay, Overlay::None) {
            Overlay::None
            | Overlay::Edit(_)
            | Overlay::BlockEdit(_)
            | Overlay::Fuzzy(_)
            | Overlay::Search(_)
            | Overlay::Drawer(_)
            | Overlay::Create(_)
            | Overlay::Find(_)
            | Overlay::ViewPicker(_)
            | Overlay::Help(_) => {}
            Overlay::Pick {
                prompt,
                keys,
                action,
            } => match k.code {
                KeyCode::Esc => self.notification = Some("cancelled".into()),
                KeyCode::Char(c) if keys.contains(c) => self.resolve_pick(action, c),
                _ => {
                    self.overlay = Overlay::Pick {
                        prompt,
                        keys,
                        action,
                    }
                }
            },
            Overlay::Confirm { prompt, action } => match k.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => self.resolve_confirm(action),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Enter => {
                    // Declining a discard-confirm returns to the stashed editor.
                    if matches!(action, ConfirmAction::DiscardEdit) {
                        if let Some(ov) = self.dirty_cancel.take() {
                            self.overlay = ov;
                        }
                    } else {
                        self.notification = Some("cancelled".into());
                    }
                }
                _ => self.overlay = Overlay::Confirm { prompt, action },
            },
        }
    }

    pub(crate) fn commit_edit(&mut self, ed: Editor) {
        let text = ed.text();
        let cursor_row = ed.state.borrow().cursor.row;
        match ed.action {
            EditAction::Labels(id) => {
                let new = crate::model::normalize_tags([&text]);
                let cur = self.task(&id).map(|t| t.tags.clone()).unwrap_or_default();
                let Some((add, remove)) = crate::model::tag_edit(&cur, &new) else {
                    self.notification = Some(format!("{id} tags unchanged"));
                    return;
                };
                let shown = if new.is_empty() {
                    "(none)".to_string()
                } else {
                    new.join(", ")
                };
                self.apply_edit(
                    &id,
                    TaskEdit {
                        add_tags: add,
                        remove_tags: remove,
                        ..Default::default()
                    },
                    format!("{id} tags: {shown}"),
                );
            }
            EditAction::Comment(id) => self.commit_comment(&id, text),
            EditAction::Block { id, block } => self.commit_block(&id, block, text, cursor_row),
            EditAction::Attach(id) => self.commit_attach(id, text),
            EditAction::AttachPaste { id, default, data } => {
                let name = if text.trim().is_empty() {
                    Some(default)
                } else {
                    crate::farm::attachment_name(&text, "paste.png")
                };
                let taken = |n: &str| {
                    self.farm
                        .as_ref()
                        .is_some_and(|h| crate::store::artifact_dir(h.root(), &id).join(n).exists())
                };
                match name {
                    Some(n) if taken(&n) => {
                        self.notification = Some(format!("{id} already has {n}"))
                    }
                    Some(n) => self.attach_bytes(&id, &n, &data),
                    None => self.notification = Some(format!("invalid name: {}", text.trim())),
                }
            }
            EditAction::RenameAttachment { id, old } => {
                self.commit_rename_attachment(id, old, text)
            }
            EditAction::EditAttachment(v, from) => self.commit_attachment_edit(v, from, text),
            EditAction::NewPage => self.commit_new_page(&text),
            EditAction::Ask(id) => self.commit_needs(&id, NeedsVerb::Ask, &text),
            EditAction::Answer(id) => self.commit_needs(&id, NeedsVerb::Answer, &text),
            EditAction::Request(id) => self.commit_needs(&id, NeedsVerb::Request, &text),
            EditAction::Shear(target) => self.commit_shear(target, &text),
            EditAction::SaveView => self.save_current_view(text),
            EditAction::RenameView { index } => {
                let name = text.trim().to_string();
                if !name.is_empty() && index < self.views.len() {
                    self.views[index].name = name;
                    self.persist_views();
                }
                // Return to the picker on the renamed row.
                self.overlay = Overlay::ViewPicker(index.min(self.views.len().saturating_sub(1)));
            }
        }
    }

    /// Save the ask, answer or request editor: `needs_edit` with the trimmed
    /// text as the note (none when empty), then, as after `M`, the line cursor
    /// goes to the new note.
    fn commit_needs(&mut self, id: &str, verb: NeedsVerb, text: &str) {
        let note = text.trim();
        self.needs_edit(id, verb, (!note.is_empty()).then_some(note));
        self.focus_new_comment();
    }

    /// Append `text` to `id` as a new comment (an empty one is a no-op).
    pub(crate) fn commit_comment(&mut self, id: &str, text: String) {
        if text.trim().is_empty() {
            self.notification = Some("comment cancelled".into());
            return;
        }
        // Attribute TUI comments like CLI ones: $YAKS_ACTOR, else git
        // user (no --as in the TUI). Resolved lazily here so the
        // read-only/snapshot constructor stays pure (no git subprocess).
        self.apply_edit(
            id,
            TaskEdit {
                note: Some(text),
                actor: crate::actor::resolve(None),
                ..Default::default()
            },
            format!("comment added to {id}"),
        );
        self.focus_new_comment();
    }

    pub(crate) fn commit_fuzzy(&mut self, mut fp: FuzzyPick) {
        let cands = fuzzy_candidates(&self.all, &fp);
        // Resolve the selection to a target: `None` = clear-parent row,
        // `Some(id)` = a task, or bail if the selection points at nothing.
        let target: Option<Option<String>> = if fp.allow_none {
            if fp.sel == 0 {
                Some(None)
            } else {
                cands.get(fp.sel - 1).map(|t| Some(t.id.clone()))
            }
        } else {
            cands.get(fp.sel).map(|t| Some(t.id.clone()))
        };
        // Ref autocomplete: insert the picked id (if any) and always return to
        // the stashed editor — even on an empty pick, so the editor is never lost.
        if let FuzzyAction::InsertRef = fp.action {
            let picked = target.flatten();
            if let Some(back) = fp.return_to.take() {
                let mut ov = *back;
                if let Some(id) = &picked {
                    // Replace the typed `<prefix>-` with the chosen id, in
                    // whichever editor the picker was opened over.
                    match &mut ov {
                        Overlay::Edit(ed) => {
                            delete_into(&mut ed.handler, &ed.state, fp.replace_len);
                            insert_into(&mut ed.handler, &ed.state, id);
                        }
                        Overlay::BlockEdit(be) => {
                            let ed = &mut be.editor;
                            delete_into(&mut ed.handler, &ed.state, fp.replace_len);
                            insert_into(&mut ed.handler, &ed.state, id);
                        }
                        Overlay::Create(f) => {
                            if let Some(e) = &f.description {
                                delete_into(&mut f.handler, e, fp.replace_len);
                                insert_into(&mut f.handler, e, id);
                            }
                        }
                        _ => {}
                    }
                }
                self.overlay = ov;
            }
            return;
        }
        let Some(target) = target else {
            self.notification = Some("nothing selected".into());
            return;
        };
        match fp.action {
            FuzzyAction::AddDep(id) => {
                let Some(dep) = target else { return };
                let Some(h) = &self.farm else { return };
                match h.dep_add(&id, &dep) {
                    Ok(DepOutcome::Added) => {
                        self.reload();
                        self.notification = Some(format!("{id} depends on {dep}"));
                    }
                    Ok(DepOutcome::AlreadyDep) => {
                        self.notification = Some(format!("{id} already depends on {dep}"))
                    }
                    Ok(_) => self.notification = Some("dependency not added".into()),
                    Err(e) => self.notification = Some(format!("error: {e}")),
                }
            }
            FuzzyAction::Reparent(id) => {
                let Some(h) = &self.farm else { return };
                match h.reparent(&id, target.clone()) {
                    Ok(Reparent::Done { new_parent }) => {
                        self.reload();
                        self.notification = Some(match new_parent {
                            Some(p) => format!("{id} reparented under {p}"),
                            None => format!("{id} moved to top level"),
                        });
                    }
                    Ok(Reparent::Error(m)) => self.notification = Some(m),
                    Err(e) => self.notification = Some(format!("error: {e}")),
                }
            }
            FuzzyAction::Goto => {
                if let Some(id) = target {
                    self.goto_task(&id);
                }
            }
            FuzzyAction::InsertRef => unreachable!("InsertRef is handled before this match"),
        }
    }

    pub(crate) fn resolve_pick(&mut self, action: PickAction, c: char) {
        match action {
            PickAction::BulkState(ids) => {
                let dest = match c {
                    'h' => Status::Hairy,
                    's' => Status::Shaving,
                    'n' => Status::Shorn,
                    'x' => Status::Dead,
                    _ => return,
                };
                if dest == Status::Shorn {
                    self.open_shear_comment(ShearTarget::Bulk(ids));
                } else {
                    self.bulk_transition(&ids, dest, None);
                }
            }
            PickAction::State(id) => {
                let dest = match c {
                    'h' => Status::Hairy,
                    's' => Status::Shaving,
                    'n' => Status::Shorn,
                    'x' => Status::Dead,
                    _ => return,
                };
                if self.task(&id).map(|t| t.status) == Some(dest) {
                    self.notification = Some(format!("{id} already {}", status_word(dest)));
                    return;
                }
                if dest == Status::Shorn {
                    self.open_shear_comment(ShearTarget::One(id));
                } else {
                    self.transition_one(&id, dest, None);
                }
            }
            PickAction::Priority(id) => {
                let Some(p) = c.to_digit(10).map(|d| d as u8) else {
                    return;
                };
                if self.task(&id).map(|t| t.priority) == Some(p) {
                    self.notification = Some(format!("{id} already p{p}"));
                    return;
                }
                self.apply_edit(
                    &id,
                    TaskEdit {
                        priority: Some(p),
                        ..Default::default()
                    },
                    format!("{id} → p{p}"),
                );
            }
            PickAction::Type(id) => {
                let kind = match c {
                    't' => "task",
                    'b' => "bug",
                    'f' => "feature",
                    'i' => "idea",
                    _ => return,
                };
                if self.task(&id).map(|t| t.kind.as_str()) == Some(kind) {
                    self.notification = Some(format!("{id} already {kind}"));
                    return;
                }
                self.apply_edit(
                    &id,
                    TaskEdit {
                        kind: Some(kind.to_string()),
                        ..Default::default()
                    },
                    format!("{id} → {kind}"),
                );
            }
            PickAction::Herd(id, herds) => {
                // The digit key is 1-based into the candidate herds.
                let Some(idx) = c.to_digit(10).map(|d| d as usize) else {
                    return;
                };
                let Some(dest) = idx.checked_sub(1).and_then(|i| herds.get(i)) else {
                    return;
                };
                // Moving herd = renaming the id's prefix, keeping the tail.
                let Some((_, tail)) = id.split_once('-') else {
                    self.notification = Some(format!("{id} has no herd"));
                    return;
                };
                let new_id = format!("{dest}-{tail}");
                let Some(h) = &self.farm else { return };
                match h.rename(&id, &new_id, false) {
                    Ok(RenameOutcome::Done(_)) => {
                        self.reload();
                        self.select_id(&new_id);
                        self.notification = Some(format!("moved to {dest}"));
                    }
                    Ok(RenameOutcome::Collision(t)) => {
                        self.notification = Some(format!("{t} already exists in {dest}"));
                    }
                    Ok(RenameOutcome::NotFound(t)) => {
                        self.notification = Some(format!("{t} not found"));
                    }
                    Ok(RenameOutcome::Invalid(t)) => {
                        self.notification = Some(format!("invalid id {t}"));
                    }
                    Ok(RenameOutcome::NothingToRename) => {
                        self.notification = Some(format!("{id} unchanged"));
                    }
                    Err(e) => self.notification = Some(format!("error: {e}")),
                }
            }
        }
    }

    /// Shearing from a state picker first asks for an optional comment, so a
    /// TUI shear can carry the evidence note `doctor --strict` looks for
    /// (yaks-27c6). Enter on an empty line shears with no note; cancel
    /// (Ctrl-C, or Esc in emacs) leaves the yak(s) as they were.
    pub(crate) fn open_shear_comment(&mut self, target: ShearTarget) {
        let what = match &target {
            ShearTarget::One(id) => id.clone(),
            ShearTarget::Bulk(ids) => format!("{} marked", ids.len()),
        };
        self.overlay = Overlay::Edit(Editor::new(
            self.editor_vim,
            true,
            format!("Shear {what} — optional comment (Enter shears · Ctrl-C cancel): "),
            "",
            EditAction::Shear(target),
        ));
    }

    /// Commit a shear-comment prompt: the non-empty note goes on each yak
    /// before its move, then the yak(s) move to shorn.
    pub(crate) fn commit_shear(&mut self, target: ShearTarget, text: &str) {
        let note = text.trim();
        let note = (!note.is_empty()).then_some(note);
        match target {
            ShearTarget::One(id) => self.transition_one(&id, Status::Shorn, note),
            ShearTarget::Bulk(ids) => self.bulk_transition(&ids, Status::Shorn, note),
        }
    }

    /// Append `note` (attributed like other TUI notes) to `id` ahead of a
    /// transition, so the note is in place when the yak lands in `dest`.
    fn note_before_move(&self, id: &str, note: Option<&str>, actor: Option<&str>) -> Result<()> {
        let (Some(h), Some(note)) = (&self.farm, note) else {
            return Ok(());
        };
        h.update(
            id,
            TaskEdit {
                note: Some(note.to_string()),
                actor: actor.map(str::to_string),
                ..Default::default()
            },
        )
        .map(|_| ())
    }

    /// Move one yak to `dest`, first recording `note` on it when given.
    pub(crate) fn transition_one(&mut self, id: &str, dest: Status, note: Option<&str>) {
        let actor = crate::actor::resolve(None);
        // A refused move must not leave its note behind (yaks-35a4).
        if let Some(Err(e)) = self.farm.as_ref().map(|h| h.check_transition(id, dest)) {
            self.notification = Some(format!("error: {e}"));
            return;
        }
        if let Err(e) = self.note_before_move(id, note, actor.as_deref()) {
            self.notification = Some(format!("error: {e}"));
            return;
        }
        let Some(h) = &self.farm else { return };
        match h.transition_lifting(id, dest, actor.as_deref()) {
            Ok((MoveOutcome::Moved, lifted)) => {
                self.reload();
                let mut msg = format!("{id} → {}", status_word(dest));
                if !lifted.is_empty() {
                    // Shaving took hairy ancestors along (yaks-cfa6).
                    msg.push_str(&format!(" (with {})", lifted.join(", ")));
                }
                self.notification = Some(msg);
            }
            Ok((MoveOutcome::AlreadyThere, _)) => {
                self.notification = Some(format!("{id} already {}", status_word(dest)))
            }
            Ok((MoveOutcome::NotFound, _)) => self.notification = Some(format!("{id} not found")),
            Err(e) => self.notification = Some(format!("error: {e}")),
        }
    }

    /// Move every marked id to `dest` (recording `note` on each first when
    /// given), then clear the marks and report the count.
    pub(crate) fn bulk_transition(&mut self, ids: &[String], dest: Status, note: Option<&str>) {
        // The family invariant (yaks-9b41) is the farm's to enforce: a
        // finishing move refuses a yak with open children, an opening one a yak
        // under a finished parent. Walk the marks in the farm's batch order, so
        // a marked family moves whole, and report what was refused as skipped
        // rather than failed (the single slaughter path, open_slaughter_confirm,
        // offers to take the family instead; yaks-5c51).
        let Some(h) = &self.farm else { return };
        let ordered = h.batch_order(ids, dest).unwrap_or_else(|_| ids.to_vec());
        let (mut skipped_kids, mut skipped_parent) = (0usize, 0usize);
        let actor = crate::actor::resolve(None);
        let (mut moved, mut failed) = (0usize, 0usize);
        for id in &ordered {
            // Never note a yak that is already there: the move would be a no-op.
            if self.task(id).map(|t| t.status) == Some(dest) {
                continue;
            }
            let Some(h) = &self.farm else { return };
            // Check first, so a refused move leaves no note behind.
            if let Err(e) = h.check_transition(id, dest) {
                if e.downcast_ref::<crate::farm::UnfinishedChildren>()
                    .is_some()
                {
                    skipped_kids += 1;
                } else if e.downcast_ref::<crate::farm::FinishedParent>().is_some() {
                    skipped_parent += 1;
                } else {
                    failed += 1;
                }
                continue;
            }
            if self.note_before_move(id, note, actor.as_deref()).is_err() {
                failed += 1;
                continue;
            }
            let Some(h) = &self.farm else { return };
            match h.transition(id, dest, actor.as_deref()) {
                Ok(MoveOutcome::Moved) => moved += 1,
                Ok(_) => {}
                Err(_) => failed += 1,
            }
        }
        let total = ids.len();
        self.selected.clear();
        self.reload();
        let mut msg = if failed > 0 {
            format!("{moved}/{total} → {} ({failed} failed)", status_word(dest))
        } else {
            format!("{moved}/{total} → {}", status_word(dest))
        };
        if skipped_kids > 0 {
            msg.push_str(&format!(" · {skipped_kids} skipped: unfinished children"));
        }
        if skipped_parent > 0 {
            msg.push_str(&format!(" · {skipped_parent} skipped: parent is finished"));
        }
        self.notification = Some(msg);
    }

    /// Apply `edit` to `id` and report the result as a notification; returns
    /// the outcome (`None` without a farm or on an error) so a caller can act
    /// on a [`UpdateOutcome::Conflict`].
    pub(crate) fn apply_edit(
        &mut self,
        id: &str,
        edit: TaskEdit,
        ok_msg: String,
    ) -> Option<UpdateOutcome> {
        let h = self.farm.as_ref()?;
        let outcome = h.update(id, edit);
        match outcome {
            Ok(UpdateOutcome::Updated) => {
                // Follow the edited yak to its new sorted slot. The cursor is
                // index-based, so an edit that changes the sort key (e.g.
                // priority) would otherwise leave it pointing at whatever yak
                // now sits in that slot (yaks-f207). Capture the id first so we
                // can tell whether the edit filtered it out of view.
                let edited = self.selected_id();
                self.reload_preserving_selection();
                // If the edit pushed the yak out of the current filtered/sorted
                // list entirely, close the detail pane -- what you were looking
                // at is no longer here (human-confirmed drop-out behavior).
                // (A yak the pane shows from outside the list was never in it.)
                if self.focus == Focus::Detail
                    && self.pane_yak_id().is_none()
                    && edited.is_some_and(|id| {
                        !self
                            .rows()
                            .iter()
                            .any(|r| r.yak().is_some_and(|t| t.id == id))
                    })
                {
                    self.back_to_list();
                }
                self.notification = Some(ok_msg);
            }
            Ok(UpdateOutcome::NoChanges) => self.notification = Some(format!("{id} unchanged")),
            Ok(UpdateOutcome::NotFound) => self.notification = Some(format!("{id} not found")),
            // Nothing was written; show what is on disk now (the caller keeps
            // the user's text, yaks-9f5c).
            Ok(UpdateOutcome::Conflict) => {
                self.reload_preserving_selection();
                self.notification = Some(format!("{id} changed on disk; not saved"));
            }
            Err(ref e) => self.notification = Some(format!("error: {e}")),
        }
        outcome.ok()
    }

    pub(crate) fn resolve_confirm(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::DiscardEdit => {
                // Drop the stashed editor (an attachment edit returns to its viewer).
                if let Some(ov) = self.dirty_cancel.take() {
                    self.drop_editor(ov);
                }
                self.notification = Some("changes discarded".into());
            }
            ConfirmAction::Slaughter(id) => {
                let Some(h) = &self.farm else { return };
                let actor = crate::actor::resolve(None);
                match h.transition(&id, Status::Dead, actor.as_deref()) {
                    Ok(MoveOutcome::Moved) => {
                        self.reload();
                        self.notification = Some(format!("slaughtered {id}"));
                    }
                    Ok(_) => self.notification = Some(format!("{id} not slaughtered")),
                    Err(e) => self.notification = Some(format!("error: {e}")),
                }
            }
            ConfirmAction::SlaughterFamily(id) => {
                let Some(h) = &self.farm else { return };
                let actor = crate::actor::resolve(None);
                match h.slaughter(&id, true, actor.as_deref()) {
                    Ok(crate::farm::SlaughterOutcome::Slaughtered(moved)) => {
                        self.reload();
                        let n = moved.len();
                        self.notification = Some(format!("slaughtered {id} + family ({n} yaks)"));
                    }
                    Ok(_) => self.notification = Some(format!("{id} not slaughtered")),
                    Err(e) => self.notification = Some(format!("error: {e}")),
                }
            }
        }
    }
}

/// Route one terminal event: key presses to [`handle_key`], a bracketed paste
/// to [`handle_paste`], mouse to `handle_mouse`. Everything else (resize, focus, releases) is a no-op
/// here — the next frame picks up a resize on its own.
pub(crate) fn handle_event(app: &mut App, ev: Event) {
    match ev {
        Event::Key(k) if k.kind == KeyEventKind::Press => handle_key(app, k),
        Event::Paste(text) => handle_paste(app, &text),
        Event::Mouse(m) => app.handle_mouse(m),
        _ => {}
    }
}

/// Handle `first`, then keep draining already-queued events (`next` returns
/// `None` when the queue is empty) until the queue empties, the app quits, or
/// `budget` elapses — so a burst (a terminal that sends a paste as a keystroke
/// stream) costs one redraw, not one per char (yaks-f2aa), while a flood still
/// can't starve the screen. Returns how many events were handled.
pub(crate) fn pump_events(
    app: &mut App,
    first: Event,
    budget: Duration,
    mut next: impl FnMut() -> io::Result<Option<Event>>,
) -> io::Result<usize> {
    let start = std::time::Instant::now();
    handle_event(app, first);
    let mut n = 1;
    while !app.quit && start.elapsed() < budget {
        let Some(ev) = next()? else { break };
        handle_event(app, ev);
        n += 1;
    }
    Ok(n)
}

/// A bracketed paste (yaks-f2aa): insert the whole text into the focused text
/// field in one go. Content editors (the comment/body `Edit` overlay, the
/// create/edit form's rows) bulk-insert it literally; the live-filter fields
/// (search, detail find, fuzzy query, drawer text rows) replay it as typed chars
/// in Insert so their per-key preview sync runs, newlines flattened. Anywhere
/// else — the list, pickers, confirms — a paste is dropped: pasted text is
/// never a command stream.
pub(crate) fn handle_paste(app: &mut App, text: &str) {
    if let Some(cmd) = &mut app.cmdline {
        cmd.extend(text.chars().map(|c| if c.is_control() { ' ' } else { c }));
        return;
    }
    let field = match &app.overlay {
        Overlay::Edit(ed) => {
            paste_into(&ed.state, text);
            return;
        }
        Overlay::BlockEdit(be) => {
            paste_into(&be.editor.state, text);
            app.show_block_editor();
            return;
        }
        Overlay::Create(f) => {
            if let Some(e) = f.description_editor() {
                paste_into(e, text);
            } else if let Some(e) = f.line_editor() {
                paste_into(e, text);
            }
            return;
        }
        Overlay::Search(sb) | Overlay::Find(sb) => &sb.query,
        Overlay::Fuzzy(fp) => &fp.query,
        Overlay::Drawer(d) => match d.text_editor() {
            Some(e) => e,
            None => return,
        },
        _ => return,
    };
    field.borrow_mut().mode = EditorMode::Insert;
    for c in text.chars() {
        let c = if matches!(c, '\n' | '\r' | '\t') {
            ' '
        } else {
            c
        };
        if !c.is_control() {
            handle_key_inner(app, KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
    }
}

pub(crate) fn handle_key(app: &mut App, k: KeyEvent) {
    // A notification lasts until the next key (or mouse event).
    app.notification = None;
    let was_editing = app.block_edit().is_some();
    handle_key_inner(app, k);
    // The in-place editor closed (saved or cancelled): its rows give way to the
    // block's again, so pull the scroll back into range.
    if was_editing && app.block_edit().is_none() {
        app.settle_detail_scroll();
    }
    // A no-wrap block's sideways scroll lives only while the cursor is on it.
    app.drop_stale_hscroll();
}

fn handle_key_inner(app: &mut App, k: KeyEvent) {
    // A modal prompt swallows all other input until resolved (including Ctrl-C,
    // which an editor treats as cancel rather than quitting the app).
    if !matches!(app.overlay, Overlay::None) {
        app.handle_overlay_key(k);
        return;
    }
    // Every other surface is "which one, which command, run it": the list,
    // the detail pane or the open document, by the kind of row under the
    // cursor.
    let ctx = app.surface_ctx();
    if let Some(cmd) = keymap::lookup(ctx, app.surface_kind(), &k) {
        app.run_cmd(ctx, cmd);
    }
}
