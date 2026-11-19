//! `App`'s view actions and overlay openers: star/collapse/mark toggles, save
//! and pick views, cursor + `state_header`, and the `open_*` methods that raise
//! the create/edit form, pickers, drawer, help, attach, comment, and ask/answer
//! overlays. Split out of `tui.rs` (yaks-b1cc / b1cc/7) as an `impl App` block;
//! `use super::*` inherits `tui.rs`'s scope and the methods are `pub(crate)`.

use super::*;

/// What a star in the working set names now (see [`App::resolve_star`]).
pub(crate) enum Star<'a> {
    Yak(&'a Task),
    /// A dok, a source or a session note.
    Doc(&'a doks_tab::DoksEntry),
    /// Anything else the pane can show, by its link destination.
    Place(&'a str),
}

/// A starred place's list row ([`App::place_label`]).
pub(crate) struct PlaceLabel {
    pub icon: &'static str,
    pub name: String,
    pub whence: String,
}

impl App {
    /// A star's one name, its canonical link destination
    /// ([`views_store::star_key`]).
    pub(crate) fn star_key(&self, star: &str) -> String {
        views_store::star_key(star, |id| self.task(id).is_some())
    }

    /// Replace the working set with `stars`, each as [`App::star_key`] names
    /// it (a stored `doks/x.md` is the star `/doks/x.md`; a star listed twice
    /// is one). Every read of the set compares canonical keys.
    pub(crate) fn set_working_set(&mut self, stars: Vec<String>) {
        self.working_set = views_store::canonical_set(stars, |id| self.task(id).is_some());
    }

    /// What `*` stars, as a star key: what the pane shows when it shows a
    /// document (a file, an image, a directory, a commit: its link
    /// destination), else the selected yak, else the selected row (a dok, a
    /// source or session note, a starred place). `Err` says why there is
    /// nothing to star; "nothing to star" is not worth a notification.
    fn star_target(&self) -> Result<String, &'static str> {
        if let Some(pane) = &self.doc {
            return self
                .doc_link(&pane.view)
                .and_then(|l| crate::links::dest(&l))
                .map(|d| self.star_key(&d))
                .ok_or("this document has no name to star");
        }
        if let Some(id) = self.selected_id() {
            return Ok(id);
        }
        match self.selected_row() {
            Some(tree::Item::Doc(e)) => {
                let link = crate::links::Link::Farm {
                    path: e.path.clone(),
                    rev: None,
                    anchor: None,
                };
                let dest = crate::links::dest(&link).unwrap_or_else(|| e.path.clone());
                Ok(self.star_key(&dest))
            }
            Some(tree::Item::Place(dest)) => Ok(dest.to_string()),
            Some(tree::Item::Yak(_) | tree::Item::Shed(_)) | None => Err("nothing to star"),
        }
    }

    /// `*`: star or unstar the pane's subject, or the selected row's, in the
    /// one working set the Starred view lists. A star is a link destination
    /// (a yak is its bare id).
    pub(crate) fn toggle_star(&mut self) {
        let key = match self.star_target() {
            Ok(key) => key,
            Err(why) => {
                // A yak command with no yak under the cursor stays silent
                // (the Doks tab's yak keys do nothing).
                if why != "nothing to star" {
                    self.notification = Some(why.into());
                }
                return;
            }
        };
        let was = self.is_starred(&key);
        self.working_set = views_store::toggle_working_set(&self.working_set, &key);
        if let Some(h) = &self.farm {
            views_store::save_working_set(h.root(), &self.working_set);
        }
        if self.active_view().key == "working-set" {
            self.clamp_cursor();
        }
        self.notification = Some(if was {
            format!("unstarred {key}")
        } else {
            format!("starred {key}")
        });
    }

    /// Whether `star` (a yak id or a link destination, in any spelling
    /// [`App::star_key`] reads) is starred.
    pub(crate) fn is_starred(&self, star: &str) -> bool {
        let key = self.star_key(star);
        self.working_set.iter().any(|s| *s == key)
    }

    /// What star `star` names now, for the Starred list; `None` when it no
    /// longer resolves (a gone yak, a deleted file or attachment). A commit
    /// and a code file are not looked up (that is a git call per frame): they
    /// stay listed, and opening one says what is missing.
    pub(crate) fn resolve_star<'a>(&'a self, star: &'a str) -> Option<Star<'a>> {
        use crate::links::{self, Link};
        use crate::locate::{Place, Space};
        if let Some(t) = self.task(star) {
            return Some(Star::Yak(t));
        }
        // A bare token that is no yak is a yak that is gone.
        if !star.contains(['/', ':']) {
            return None;
        }
        let link = links::parse(star, links::From::Cli, |s| self.task(s).is_some());
        if let Link::Farm {
            path, rev: None, ..
        } = &link
        {
            let rel = crate::locate::farm_file(path);
            let entry = self.doks.entries.iter().flatten().find(|e| e.path == *rel);
            if let Some(e) = entry {
                return Some(Star::Doc(e));
            }
        }
        let root = self.farm.as_ref()?.root();
        let place = Space::new(root, None).resolve(&link);
        let there = match &place {
            // Resolving a directory looked at the disk.
            Place::FarmDir { .. } | Place::CodeFile { .. } | Place::Commit { .. } => true,
            Place::FarmFile { rel, rev: None, .. } => root.join(rel).is_file(),
            Place::FarmFile { rev: Some(_), .. } => true,
            Place::Attachment { id, name, .. } => {
                crate::farm::attachment_path(root, id, name).is_some_and(|p| p.is_file())
            }
            Place::Yak { .. } | Place::Url(_) | Place::Missing(_) => false,
        };
        there.then_some(Star::Place(star))
    }

    /// The starred place under the cursor (a Starred row of a file,
    /// directory, code file or commit), by its link destination.
    pub(crate) fn selected_place(&self) -> Option<String> {
        match self.selected_row()? {
            tree::Item::Place(dest) => Some(dest.to_string()),
            _ => None,
        }
    }

    /// Enter on a starred place: open what it names, a jump (`o` comes back).
    pub(crate) fn open_selected_place(&mut self) {
        if let Some(dest) = self.selected_place() {
            let place = self.place_of_dest(&dest);
            self.open_place(&place);
        }
    }

    /// How a starred place reads in the list: an icon for its kind, its
    /// name, and where it lives.
    pub(crate) fn place_label(&self, dest: &str) -> PlaceLabel {
        use crate::links::{self, Link};
        use crate::locate::{Place, Space};
        const FILE: &str = "\u{1f4c4}";
        const PAPERCLIP: &str = "\u{1f4ce}";
        const IMAGE: &str = "\u{1f5bc}";
        const FOLDER: &str = "\u{1f4c1}";
        const COMMIT: &str = "\u{1f500}";
        let is_yak = |s: &str| self.task(s).is_some();
        let unknown = || PlaceLabel {
            icon: PAPERCLIP,
            name: dest.to_string(),
            whence: String::new(),
        };
        let Some(root) = self.farm.as_ref().map(|f| f.root()) else {
            return unknown();
        };
        // `dir/name` as (`dir`, `name`).
        let split = |p: &str| match p.trim_end_matches('/').rsplit_once('/') {
            Some((d, n)) => (d.to_string(), n.to_string()),
            None => (String::new(), p.trim_end_matches('/').to_string()),
        };
        // Something inside yak `id`'s attachments: `<id>/<inner>`, then the
        // yak's title.
        let in_yak = |id: &str, inner: &str, dir: bool, icon: &'static str| {
            let full = if inner.is_empty() {
                id.to_string()
            } else {
                format!("{id}/{inner}")
            };
            let (parent, name) = split(&full);
            let title = self.task(id).map(|t| t.title.as_str()).unwrap_or("");
            let whence = match (parent.is_empty(), title.is_empty()) {
                (true, _) => title.to_string(),
                (false, true) => parent,
                (false, false) => format!("{parent}  {title}"),
            };
            PlaceLabel {
                icon,
                name: if dir { format!("{name}/") } else { name },
                whence,
            }
        };
        let file_icon = |name: &str, other: &'static str| {
            if crate::store::is_image_name(name) {
                IMAGE
            } else {
                other
            }
        };
        let place = Space::new(root, None).resolve(&links::parse(dest, links::From::Cli, is_yak));
        match place {
            Place::Attachment { id, name, .. } => {
                // A file of the yak itself is an attachment (📎); one inside an
                // attached directory is a file (📄).
                let icon = file_icon(&name, if name.contains('/') { FILE } else { PAPERCLIP });
                in_yak(&id, &name, false, icon)
            }
            Place::FarmDir { rel } => {
                // A yak's attachment directory, or a directory in it.
                match links::parse(&format!("/{rel}"), links::From::Cli, is_yak) {
                    Link::Attachment { id, name, .. } => {
                        in_yak(&id, name.trim_end_matches('/'), true, FOLDER)
                    }
                    // The yak's whole attachment directory, `<herd>/yaks/<id>`.
                    _ if rel
                        .split('/')
                        .nth(2)
                        .is_some_and(|id| rel.split('/').count() == 3 && is_yak(id)) =>
                    {
                        let id = rel.split('/').nth(2).unwrap_or_default();
                        in_yak(id, "", true, FOLDER)
                    }
                    _ => {
                        let (parent, name) = split(&rel);
                        PlaceLabel {
                            icon: FOLDER,
                            name: format!("{name}/"),
                            whence: if parent.is_empty() {
                                "/".into()
                            } else {
                                parent
                            },
                        }
                    }
                }
            }
            Place::FarmFile { rel, rev, .. } => {
                let (parent, name) = split(&rel);
                let pin = rev.map(|r| format!("@{}", &r[..r.len().min(7)]));
                PlaceLabel {
                    icon: file_icon(&name, FILE),
                    whence: format!(
                        "{}{}",
                        if parent.is_empty() { "/" } else { &parent },
                        pin.unwrap_or_default()
                    ),
                    name,
                }
            }
            Place::CodeFile { rel, rev, .. } => {
                let (parent, name) = split(&rel);
                let pin = rev.map(|r| format!("@{}", &r[..r.len().min(7)]));
                PlaceLabel {
                    icon: file_icon(&name, FILE),
                    whence: format!(
                        "code:{}{}{}",
                        parent,
                        if parent.is_empty() { "" } else { "/" },
                        pin.unwrap_or_default()
                    ),
                    name,
                }
            }
            Place::Commit { sha } => PlaceLabel {
                icon: COMMIT,
                name: sha[..sha.len().min(7)].to_string(),
                whence: "commit:".into(),
            },
            Place::Yak { .. } | Place::Url(_) | Place::Missing(_) => unknown(),
        }
    }

    pub(crate) fn save_current_view(&mut self, name: String) {
        if name.trim().is_empty() {
            self.notification = Some("save view cancelled".into());
            return;
        }
        let active = self.active_view();
        let (sort_by, sort_dir, limit) = (active.sort_by, active.sort_dir, active.limit);
        let seed = views_store::custom_key_seed(&name, &self.views);
        let v = view::custom_view(
            name.clone(),
            clone_spec(&self.filter),
            sort_by,
            sort_dir,
            limit,
            &seed,
        );
        self.views.push(v);
        if let Some(h) = &self.farm {
            views_store::save_views(h.root(), &self.views);
        }
        self.set_view(self.views.len() - 1);
        self.notification = Some(format!("saved view: {name}"));
    }

    pub(crate) fn persist_views(&self) {
        if let Some(h) = &self.farm {
            views_store::save_views(h.root(), &self.views);
        }
    }

    pub(crate) fn open_view_picker(&mut self) {
        self.overlay = Overlay::ViewPicker(self.view.min(self.views.len().saturating_sub(1)));
    }

    pub(crate) fn handle_view_picker_key(&mut self, k: KeyEvent) {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let sel = match &self.overlay {
            Overlay::ViewPicker(s) => *s,
            _ => return,
        };
        let n = self.views.len();
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => self.overlay = Overlay::None,
            KeyCode::Down => self.overlay = Overlay::ViewPicker((sel + 1).min(n - 1)),
            KeyCode::Up => self.overlay = Overlay::ViewPicker(sel.saturating_sub(1)),
            KeyCode::Char('j') if self.editor_vim() => {
                self.overlay = Overlay::ViewPicker((sel + 1).min(n - 1))
            }
            KeyCode::Char('k') if self.editor_vim() => {
                self.overlay = Overlay::ViewPicker(sel.saturating_sub(1))
            }
            KeyCode::Enter => {
                self.jump_view(sel);
                self.overlay = Overlay::None;
            }
            KeyCode::Char('p') | KeyCode::Char(' ') => {
                if self.views[sel].pinned && !views_store::can_unpin(&self.views, sel) {
                    self.notification = Some("can't unpin the last tab".into());
                } else {
                    self.views[sel].pinned = !self.views[sel].pinned;
                    self.persist_views();
                }
            }
            KeyCode::Char('J') => self.reorder_view(sel, 1),
            KeyCode::Char('K') => self.reorder_view(sel, -1),
            KeyCode::Char('r') => {
                let name = self.views[sel].name.clone();
                self.overlay = Overlay::Edit(Editor::new(
                    self.editor_vim(),
                    true,
                    "Rename view: ".into(),
                    &name,
                    EditAction::RenameView { index: sel },
                ));
            }
            KeyCode::Char('d') => self.delete_view(sel),
            _ if ctrl && k.code == KeyCode::Char('n') => {
                self.overlay = Overlay::ViewPicker((sel + 1).min(n - 1))
            }
            _ if ctrl && k.code == KeyCode::Char('p') => {
                self.overlay = Overlay::ViewPicker(sel.saturating_sub(1))
            }
            _ => {}
        }
    }

    /// Move view `sel` by `dir`, keeping the active view pointing at its own
    /// entry and the picker selection following the moved row.
    pub(crate) fn reorder_view(&mut self, sel: usize, dir: i32) {
        let active_key = self.views[self.view].key.clone();
        let ns = views_store::move_view(&mut self.views, sel, dir);
        self.view = self
            .views
            .iter()
            .position(|v| v.key == active_key)
            .unwrap_or(self.view);
        self.persist_views();
        self.overlay = Overlay::ViewPicker(ns);
    }

    pub(crate) fn delete_view(&mut self, sel: usize) {
        if self.views[sel].builtin {
            self.notification = Some("can't delete a built-in view".into());
            return;
        }
        let active_key = self.views[self.view].key.clone();
        self.views.remove(sel);
        self.persist_views();
        // Keep the active view valid; if it was the deleted one, clamp.
        self.view = self
            .views
            .iter()
            .position(|v| v.key == active_key)
            .unwrap_or_else(|| self.view.min(self.views.len() - 1));
        let ns = sel.min(self.views.len() - 1);
        self.overlay = Overlay::ViewPicker(ns);
        self.clamp_cursor();
    }

    pub(crate) fn move_cursor(&mut self, delta: i32) {
        let len = self.rows().len() as i32;
        self.pane_yak = None;
        if len == 0 {
            self.cursor = 0;
            return;
        }
        self.cursor = (self.cursor as i32 + delta).clamp(0, len - 1) as usize;
    }

    /// One-line summary of internal state for the headless snapshot header.
    /// This is the optional, developer-fillable debug facility (Encoding "B"):
    /// it surfaces app-state facts that colour/layout alone can hide, so
    /// internal-state bugs show up directly in a snapshot. Keep it one line.
    pub(crate) fn state_header(&self) -> String {
        let focus = match self.focus {
            Focus::List => "list",
            Focus::Detail => "detail",
        };
        let sel = self.selected_id().unwrap_or_else(|| "-".into());
        let mut blocked: Vec<String> = self.blocked_ids().into_iter().collect();
        blocked.sort();
        let blocked = if blocked.is_empty() {
            "-".to_string()
        } else {
            format!("[{}]", blocked.join(","))
        };
        let filter = {
            let s = filter_summary(&self.filter);
            if s.is_empty() { "-".to_string() } else { s }
        };
        let base = format!(
            "focus={focus} · view={} · cursor={} · rows={} · sel={sel} · blocked={blocked} · collapsed={} · filter={filter} · overlay={}",
            self.active_view().name,
            self.cursor,
            self.rows().len(),
            self.collapsed.len(),
            overlay_name(&self.overlay),
        );
        // The `?` popup is no overlay, so say when it is up (a headless driver
        // has no frame to look at otherwise).
        let base = if self.keys_popup {
            format!("{base} · popup=keys")
        } else {
            base
        };
        let base = match &self.doc {
            Some(DocPane {
                view: DocView::Text(_),
                ..
            }) => format!("{base} · doc=text"),
            Some(DocPane {
                view: DocView::Image(_),
                ..
            }) => format!("{base} · doc=image"),
            Some(DocPane {
                view: DocView::Dir(_),
                ..
            }) => format!("{base} · doc=dir"),
            Some(DocPane {
                view: DocView::Commit(_),
                ..
            }) => format!("{base} · doc=commit"),
            None => base,
        };
        // A yak shown from outside the list: not the selected row's.
        let base = match self.pane_yak_id() {
            Some(id) => format!("{base} · pane_yak={id}"),
            None => base,
        };
        if self.on_sheds_tab() {
            return format!("{base} · {}", self.sheds_state_header());
        }
        if self.on_doks_tab() {
            return format!("{base} · {}", self.doks_state_header());
        }

        // In the detail pane, expose the scroll offset and whether the sticky
        // header is showing (the frontmatter top has scrolled off).
        if self.focus == Focus::Detail {
            format!(
                "{base} · detail_scroll={} · pinned={}",
                self.detail.scroll,
                if self.detail.scroll > 0 { "yes" } else { "no" }
            )
        } else {
            base
        }
    }

    /// `Space`: collapse or expand the row under the cursor (a yak's
    /// children, a shed's yaks).
    pub(crate) fn toggle_collapse(&mut self) {
        let rows = self.rows();
        let Some(key) = rows
            .get(self.cursor)
            .filter(|row| row.has_children)
            .and_then(|row| row.collapse_key())
        else {
            return;
        };
        drop(rows);
        if !self.collapsed.remove(&key) {
            self.collapsed.insert(key);
        }
        self.save_ui_state();
    }

    /// `h` on a list row: collapse it when it is an expanded parent, else go to
    /// its parent (the nearest row above it that is less indented). Nothing on
    /// a root that has nothing to collapse, in a flat view, or on a document.
    pub(crate) fn tree_left(&mut self) {
        let rows = self.rows();
        let Some(row) = rows.get(self.cursor) else {
            return;
        };
        if row.has_children && !row.collapsed {
            let Some(key) = row.collapse_key() else {
                return;
            };
            drop(rows);
            // Flip, as Space does: for a quiet shed the key means expanded.
            if !self.collapsed.remove(&key) {
                self.collapsed.insert(key);
            }
            self.save_ui_state();
            return;
        }
        let depth = row.depth;
        let parent = rows[..self.cursor].iter().rposition(|r| r.depth < depth);
        drop(rows);
        if let Some(parent) = parent {
            self.cursor = parent;
        }
    }

    /// `l` on a list row: expand it when it is a collapsed parent, else go to
    /// its first child. Nothing on a row without children, in a flat view, or
    /// on a document (yaks-7c6e).
    pub(crate) fn tree_right(&mut self) {
        let rows = self.rows();
        let Some(row) = rows.get(self.cursor) else {
            return;
        };
        if !row.has_children {
            return;
        }
        if row.collapsed {
            let Some(key) = row.collapse_key() else {
                return;
            };
            drop(rows);
            // Flip, as Space does: for a quiet shed the key means expanded.
            if !self.collapsed.remove(&key) {
                self.collapsed.insert(key);
            }
            self.save_ui_state();
            return;
        }
        let depth = row.depth;
        let child = rows.get(self.cursor + 1).is_some_and(|r| r.depth > depth);
        drop(rows);
        if child {
            self.cursor += 1;
        }
    }

    // -- overlay openers --------------------------------------------------

    /// Toggle the cursor's yak in the multi-select set. `m` is the toggle key:
    /// the natural `Space` is already collapse/expand (and `v` is visual select).
    pub(crate) fn toggle_selected(&mut self) {
        let Some(id) = self.selected_id() else { return };
        if self.selected.remove(&id) {
            self.notification = Some(format!("unmarked {id} ({} marked)", self.selected.len()));
        } else {
            self.selected.insert(id.clone());
            self.notification = Some(format!("marked {id} ({} marked)", self.selected.len()));
        }
    }

    /// Open the h/s/n/x picker as a bulk action over the multi-select set. The
    /// resolved pick loops every marked id through `farm.transition` and then
    /// clears the set (see [`App::resolve_pick`]).
    pub(crate) fn open_bulk_state_picker(&mut self) {
        let ids: Vec<String> = self.selected.iter().cloned().collect();
        if ids.is_empty() {
            return;
        }
        let n = ids.len();
        self.overlay = Overlay::Pick {
            prompt: format!(
                "State for {n} marked: h=hairy s=shaving n=shorn x=slaughter  (Esc=cancel)"
            ),
            keys: "hsnx".into(),
            action: PickAction::BulkState(ids),
        };
    }

    pub(crate) fn open_state_picker(&mut self) {
        if let Some(id) = self.selected_id() {
            self.overlay = Overlay::Pick {
                prompt: format!(
                    "State for {id}: h=hairy s=shaving n=shorn x=slaughter  (Esc=cancel)"
                ),
                keys: "hsnx".into(),
                action: PickAction::State(id),
            };
        }
    }

    pub(crate) fn open_priority_picker(&mut self) {
        if let Some(id) = self.selected_id() {
            self.overlay = Overlay::Pick {
                prompt: format!(
                    "Priority for {id}: 1=urgent 2=high 3=med 4=low 5=lowest  (Esc=cancel)"
                ),
                keys: "12345".into(),
                action: PickAction::Priority(id),
            };
        }
    }

    pub(crate) fn open_type_picker(&mut self) {
        if let Some(id) = self.selected_id() {
            self.overlay = Overlay::Pick {
                prompt: format!("Type for {id}: t=task b=bug f=feature i=idea  (Esc=cancel)"),
                keys: "tbfi".into(),
                action: PickAction::Type(id),
            };
        }
    }

    /// Open a single-key picker of the *other* herds to move the selected yak
    /// into (yaks-71d1). A herd is an id prefix, so the move is a rename from
    /// `<oldherd>-<tail>` to `<newherd>-<tail>` (see [`App::resolve_pick`]).
    /// Herds only exist in a multi-herd farm, so single-herd farms no-op with a
    /// hint. The candidate herds are digit-indexed (1..=9), reusing the same
    /// `Overlay::Pick` machinery as the type/state pickers.
    pub(crate) fn open_herd_picker(&mut self) {
        let Some(id) = self.selected_id() else { return };
        if !self.is_multi_herd() {
            self.notification = Some("herds need a multi-herd farm".into());
            return;
        }
        // A herd is the id prefix (`prefix-tail`); an id with no `-` names no
        // herd, so it can't be moved between herds.
        let Some((current, _tail)) = id.split_once('-') else {
            self.notification = Some(format!("{id} has no herd"));
            return;
        };
        let current = current.to_string();
        // Offer every known herd except the yak's current one, capped at 9 so
        // each maps to a single digit key.
        let candidates: Vec<String> = self
            .herd_choices()
            .into_iter()
            .filter(|h| *h != current)
            .take(9)
            .collect();
        if candidates.is_empty() {
            self.notification = Some("no other herd".into());
            return;
        }
        let menu = candidates
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{}={h}", i + 1))
            .collect::<Vec<_>>()
            .join(" ");
        let keys: String = (1..=candidates.len())
            .map(|i| char::from(b'0' + i as u8))
            .collect();
        self.overlay = Overlay::Pick {
            prompt: format!("Herd for {id}: {menu}  (Esc=cancel)"),
            keys,
            action: PickAction::Herd(id, candidates),
        };
    }

    pub(crate) fn open_slaughter_confirm(&mut self) {
        let Some(id) = self.selected_id() else { return };
        let title: String = self
            .task(&id)
            .map(|t| t.title.chars().take(40).collect())
            .unwrap_or_default();
        // Slaughtering alone would leave open descendants under a dead yak, so
        // offer to take them too, naming how many go with it (yaks-05da).
        let kin = crate::farm::open_descendants(&self.all, &id).len();
        if kin > 0 {
            let noun = if kin == 1 {
                "descendant"
            } else {
                "descendants"
            };
            self.overlay = Overlay::Confirm {
                prompt: format!(
                    "Slaughter {id} ({title}) AND its {kin} unfinished {noun}? (y/N): "
                ),
                action: ConfirmAction::SlaughterFamily(id),
            };
            return;
        }
        self.overlay = Overlay::Confirm {
            prompt: format!("Slaughter {id} ({title})? (y/N): "),
            action: ConfirmAction::Slaughter(id),
        };
    }

    pub(crate) fn open_tags(&mut self) {
        if let Some(id) = self.selected_id() {
            let initial = self
                .task(&id)
                .map(|t| t.tags.join(", "))
                .unwrap_or_default();
            self.overlay = Overlay::Edit(Editor::new(
                self.editor_vim(),
                true,
                format!("Tags for {id}: "),
                &initial,
                EditAction::Labels(id),
            ));
        }
    }

    pub(crate) fn open_create(&mut self, child: bool) {
        let parent = if child { self.selected_id() } else { None };
        if child && parent.is_none() {
            return;
        }
        let mut form = self.create_form_for(parent);
        self.restore_form_draft(&mut form);
        self.overlay = Overlay::Create(form);
    }

    /// A fresh create form for a child of `parent` (or a root yak), without
    /// any saved draft ([`App::restore_form_draft`] adds that).
    pub(crate) fn create_form_for(&self, parent: Option<String>) -> CreateForm {
        // In a multi-herd farm, offer a herd picker seeded to the reference
        // yak's herd (the parent for a child create, else the selected row).
        let choices = if self.is_multi_herd() {
            self.herd_choices()
        } else {
            Vec::new()
        };
        let seed = parent
            .clone()
            .or_else(|| self.selected_id())
            .and_then(|id| id.split('-').next().map(String::from));
        let herd_idx = seed
            .as_deref()
            .and_then(|p| choices.iter().position(|c| c == p))
            .unwrap_or(0);
        CreateForm::new(self.editor_vim(), parent, choices, herd_idx)
    }

    /// Open the shared form seeded from the selected task, for editing (E).
    pub(crate) fn open_edit(&mut self) {
        self.open_edit_focus(None);
    }

    /// `E` from the detail pane: on a header field open the edit form focused on
    /// it; on a body row (description or a comment) edit that block in place;
    /// Status routes to its own picker (it isn't a form field).
    pub(crate) fn open_edit_at_cursor(&mut self) {
        match self.edit_target_at(self.detail.line) {
            Some(EditTarget::Status) => self.open_state_picker(),
            // A body row edits that block in place; header rows use the form.
            Some(EditTarget::Content(b)) => self.open_block_edit(Some(b)),
            other => self.open_edit_focus(other),
        }
    }

    /// Open the edit form, optionally pre-focusing a row derived from `target`.
    pub(crate) fn open_edit_focus(&mut self, target: Option<EditTarget>) {
        let Some(id) = self.selected_id() else { return };
        let Some(mut form) = self.edit_form_for(&id) else {
            return;
        };
        if let Some(t) = target {
            form.row = match t {
                EditTarget::Title => 0,
                EditTarget::Type => 1,
                EditTarget::Priority => 2,
                EditTarget::Labels => 3,
                EditTarget::Source => 4,
                EditTarget::Due => 5,
                EditTarget::Every => 6,
                EditTarget::Wake => 7,
                EditTarget::Herd => HEADER_ROWS,
                // Status and body rows never reach the form: `open_edit_at_cursor`
                // routes them to the picker / the in-place block editor.
                EditTarget::Status | EditTarget::Content(_) => 0,
            };
        }
        self.restore_form_draft(&mut form);
        self.overlay = Overlay::Create(form);
    }

    /// The edit form for `id`, seeded from the yak as it is on disk (no draft).
    pub(crate) fn edit_form_for(&self, id: &str) -> Option<CreateForm> {
        let mut form = CreateForm::for_edit(self.editor_vim(), self.task(id)?);
        // On a multi-herd farm, seed the herd chip row (like create) so the herd
        // is editable in the edit form too (yaks-71d1); committing a change
        // renames the id prefix. The row shows only when there's >1 herd.
        if self.is_multi_herd() {
            let choices = self.herd_choices();
            form.herd_idx = id
                .split_once('-')
                .and_then(|(cur, _)| choices.iter().position(|c| c == cur))
                .unwrap_or(0);
            form.herds = choices;
        }
        Some(form)
    }

    /// Map a detail line to what `E` should edit there.
    pub(crate) fn edit_target_at(&self, line: usize) -> Option<EditTarget> {
        let lines = self.detail_dlines();
        let dl = lines.get(line)?;
        if dl.kind == detail::Kind::Field {
            let t = dl.text.as_str();
            for (label, target) in [
                ("Title:", EditTarget::Title),
                ("Type:", EditTarget::Type),
                ("Priority:", EditTarget::Priority),
                ("Tags:", EditTarget::Labels),
                ("Source:", EditTarget::Source),
                ("Due:", EditTarget::Due),
                ("Every:", EditTarget::Every),
                ("Wakes:", EditTarget::Wake),
                ("Herd:", EditTarget::Herd),
                ("Status:", EditTarget::Status),
            ] {
                if t.starts_with(label) {
                    return Some(target);
                }
            }
        }
        if dl.kind == detail::Kind::Body {
            if let Some(Some(b)) = detail::block_index_per_line(&lines).get(line) {
                return Some(EditTarget::Content(*b));
            }
        }
        None
    }

    /// The detail rows where each content block starts (description + comments),
    /// for `Ctrl-N/P` block navigation in the detail pane.
    pub(crate) fn block_starts(&self) -> Vec<usize> {
        let lines = self.detail_dlines();
        let mut starts = Vec::new();
        let mut last: Option<usize> = None;
        for (i, b) in detail::block_index_per_line(&lines).into_iter().enumerate() {
            if let Some(b) = b {
                if Some(b) != last {
                    starts.push(i);
                    last = Some(b);
                }
            }
        }
        starts
    }

    /// Ctrl-N/P: move the line cursor to a content block's header.
    ///
    /// Next goes to the next block's header and scrolls so the whole block is
    /// visible; a block taller than the viewport puts its header at the top
    /// instead. Previous goes to the header of the block the cursor is inside,
    /// or, when the cursor is already on that header, of the one before it.
    pub(crate) fn jump_block(&mut self, delta: i32) {
        let starts = self.block_starts();
        if starts.is_empty() {
            return;
        }
        let cur = starts
            .iter()
            .rposition(|&s| s <= self.detail.line)
            .unwrap_or(0);
        let next = if delta < 0 && self.detail.line > starts[cur] {
            cur
        } else {
            (cur as i32 + delta).clamp(0, starts.len() as i32 - 1) as usize
        };
        let start = starts[next];
        self.detail.line = start;
        if delta < 0 {
            self.scroll_line_into_view(start as u16);
            return;
        }
        // The block's last row: its own rows run to the last one that belongs
        // to it (a trailing blank separator is not part of the comment).
        let end = detail::block_index_per_line(&self.detail_dlines())
            .iter()
            .rposition(|b| *b == Some(self.block_of_row(start)))
            .unwrap_or(start);
        self.scroll_line_into_view(end as u16);
        if start < self.detail.scroll as usize {
            // Too tall to show whole: header at the top.
            self.detail.scroll = (start as u16).min(self.detail_max_scroll());
        }
    }

    /// The content block (0 = description, 1.. = comments) the detail row
    /// belongs to; the row must be a block row.
    fn block_of_row(&self, row: usize) -> usize {
        detail::block_index_per_line(&self.detail_dlines())[row].unwrap_or(0)
    }
}
