//! The Wiki tab (yaks-ebb2): the farm's pages and saved sources, one flat list
//! sorted by path (pages from `wiki/` and each `<herd>/wiki/`, then
//! `sources/`), with title, kind and tags. Enter opens one in the text
//! viewer ([`App::read_farm_doc`]): a page's frontmatter as field rows
//! above its rendered body, a source's `source.md` then its `extracted.md`.
//!
//! The data is the CLI's: [`pages::load_all`] and [`sources::list`]. It is
//! read when the tab is shown and again on every farm reload, so an edit
//! made elsewhere appears as soon as the watcher sees it.
//!
//! The tab is a built-in view (`view::wiki_view`) whose rows are documents
//! ([`tree::Item::Doc`], from [`App::build_rows`]): the list's own cursor,
//! rendering and key dispatch serve it, and its keys are the `Doc` bindings of
//! `keymap`. Yak keys find no yak there and do nothing. `/` is the yak tabs'
//! inline search: it edits the live filter's `search`, which
//! [`App::wiki_rows`] matches against a row's title, path, kind and tags.

#[cfg(test)]
use super::handlers::DocKind;
use super::*;
use crate::{pages, sources};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{ListItem, Paragraph};

/// One row of the list: a page or a source.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WikiEntry {
    /// Farm-relative path of the file the viewer opens: the page, or a
    /// source's `source.md`.
    pub path: String,
    pub title: String,
    /// The page's kind (`note`, `topic`, a typed kind), or `source`.
    pub kind: String,
    pub tags: Vec<String>,
    pub source: bool,
}

impl WikiEntry {
    /// The path shown in the list: a source by its directory.
    pub(crate) fn shown_path(&self) -> &str {
        if self.source {
            self.path.strip_suffix("/source.md").unwrap_or(&self.path)
        } else {
            &self.path
        }
    }

    fn matches(&self, q: &str) -> bool {
        let q = q.to_lowercase();
        [self.title.as_str(), self.path.as_str(), self.kind.as_str()]
            .into_iter()
            .chain(self.tags.iter().map(String::as_str))
            .any(|s| s.to_lowercase().contains(&q))
    }
}

/// The tab's state: the entries last read (the list's rows index them).
#[derive(Default)]
pub(crate) struct WikiTab {
    /// `None` until first read (the tab has not been shown, or there is no
    /// farm: read-only and test constructors fill it directly).
    pub entries: Option<Vec<WikiEntry>>,
    /// The read failed: why.
    pub error: Option<String>,
}

/// Every page and source of the farm at `root`, pages first, each sorted by
/// path.
pub(crate) fn read_entries(root: &Path) -> Result<Vec<WikiEntry>> {
    let mut out: Vec<WikiEntry> = pages::load_all(root)?
        .into_iter()
        .map(|p| WikiEntry {
            title: p.title(),
            kind: p.kind(),
            tags: p.tags(),
            path: p.path,
            source: false,
        })
        .collect();
    out.extend(sources::list(root).into_iter().map(|s| WikiEntry {
        path: format!("{}/source.md", s.dir),
        title: s.title,
        kind: "source".into(),
        tags: Vec::new(),
        source: true,
    }));
    Ok(out)
}

impl App {
    pub(crate) fn on_wiki_tab(&self) -> bool {
        self.active_view().key == view::WIKI_KEY
    }

    /// Re-read the farm's pages and sources (a no-op without a farm), keeping
    /// the cursor on the same path.
    pub(crate) fn reload_wiki(&mut self) {
        let Some(root) = self.farm.as_ref().map(|f| f.root().to_path_buf()) else {
            return;
        };
        match read_entries(&root) {
            Ok(e) => self.set_wiki_entries(e),
            Err(e) => self.wiki.error = Some(format!("{e:#}")),
        }
    }

    /// Swap in a new list of entries, keeping the cursor on the same path.
    pub(crate) fn set_wiki_entries(&mut self, entries: Vec<WikiEntry>) {
        let at = self.selected_doc().map(|e| e.path.clone());
        self.wiki.entries = Some(entries);
        self.wiki.error = None;
        // The rows index the entries: rebuild them.
        self.touch();
        // Only the Wiki view's cursor means a document.
        if self.on_wiki_tab() {
            let rows = self.wiki_rows();
            self.cursor = at
                .and_then(|p| rows.iter().position(|e| e.path == p))
                .unwrap_or(self.cursor)
                .min(rows.len().saturating_sub(1));
        }
    }

    /// The entries the live search keeps, in display order: the Wiki view's
    /// rows ([`App::build_rows`] wraps each in a [`tree::Row`]).
    pub(crate) fn wiki_rows(&self) -> Vec<&WikiEntry> {
        let q = self.filter.search.as_deref().filter(|q| !q.is_empty());
        self.wiki
            .entries
            .iter()
            .flatten()
            .filter(|e| q.is_none_or(|q| e.matches(q)))
            .collect()
    }

    /// The tab's count: every page (sources not counted), whatever the search.
    pub(crate) fn wiki_count(&self) -> usize {
        self.wiki
            .entries
            .iter()
            .flatten()
            .filter(|e| !e.source)
            .count()
    }

    /// Enter / l: open the selected page or source in the viewer.
    pub(crate) fn open_wiki_entry(&mut self) {
        let Some(e) = self.selected_doc().cloned() else {
            return;
        };
        match self.read_farm_doc(&e.path, None) {
            Ok(v) => {
                self.clear_viewer_history();
                self.overlay = Overlay::Attachment(v);
            }
            Err(msg) => self.notification = Some(msg),
        }
    }

    /// N: a one-line prompt for a new page's path, seeded with `wiki/`.
    pub(crate) fn open_new_page(&mut self) {
        if self.farm.is_none() {
            return;
        }
        self.new_page_prompt("wiki/");
    }

    fn new_page_prompt(&mut self, seed: &str) {
        let ed = Editor::new(
            self.editor_vim,
            true,
            "New page (wiki/<path>.md or <herd>/wiki/<path>.md): ".into(),
            seed,
            EditAction::NewPage,
        );
        // Typing goes straight on after the seed, in Insert.
        ed.state.borrow_mut().mode = EditorMode::Insert;
        ed.state.borrow_mut().cursor = edtui::Index2::new(0, seed.chars().count());
        self.overlay = Overlay::Edit(ed);
    }

    /// The new-page prompt's Enter: create the page (`Farm::page_new`: kind
    /// `note`, a title from the file name, any schema defaults; one commit),
    /// select it in the list and open it in the editor for its body. A bad
    /// or taken path keeps the prompt open with the reason.
    pub(crate) fn commit_new_page(&mut self, text: &str) {
        let mut path = pages::normalize(text.trim());
        if path.is_empty() || path == "wiki/" {
            self.notification = Some("no page created".into());
            return;
        }
        if !path.ends_with(".md") {
            path.push_str(".md");
        }
        let Some(farm) = self.farm.as_ref() else {
            return;
        };
        if let Err(e) = farm.page_new(&path, None, None, &[], "") {
            self.notification = Some(format!("{e:#}"));
            self.new_page_prompt(&path);
            return;
        }
        self.reload_wiki();
        self.refresh_farm_sync();
        if let Some(i) = self.wiki_rows().iter().position(|e| e.path == path) {
            self.cursor = i;
        }
        match self.read_farm_doc(&path, None) {
            Ok(v) => {
                self.clear_viewer_history();
                self.overlay = Overlay::Attachment(v);
                self.open_attachment_edit();
                self.notification = Some(format!("created {path}"));
            }
            Err(e) => self.notification = Some(e),
        }
    }

    /// E on the list: edit the selected page (open it, then the editor).
    pub(crate) fn edit_wiki_entry(&mut self) {
        self.open_wiki_entry();
        if matches!(self.overlay, Overlay::Attachment(_)) {
            self.open_attachment_edit();
        }
    }

    /// y: copy the selected entry's path.
    pub(crate) fn copy_wiki_path(&mut self) {
        let Some(path) = self.selected_doc().map(|e| e.shown_path().to_string()) else {
            return;
        };
        self.notification = Some(if crate::clipboard::copy_text(&path) {
            format!("copied {path}")
        } else {
            "clipboard unavailable".into()
        });
    }

    /// toque's state-header digest on the Wiki tab.
    pub(crate) fn wiki_state_header(&self) -> String {
        let at = self
            .selected_doc()
            .map_or_else(|| "-".into(), |e| e.path.clone());
        format!("wiki={} \u{b7} entry={at}", self.wiki_rows().len())
    }

    /// `r`: read the pages again.
    pub(crate) fn reread_wiki(&mut self) {
        self.reload_wiki();
        self.notification = Some("reread the wiki".into());
    }
}

// -- rendering ---------------------------------------------------------------

fn dim() -> Style {
    Style::new().add_modifier(Modifier::DIM)
}

fn pad(s: &str, w: usize) -> String {
    let t = truncate_disp(s, w);
    let fill = w.saturating_sub(disp_width(&t));
    format!("{t}{}", " ".repeat(fill))
}

/// One list row: title, kind, tags, then the path dimmed, padded to the
/// full width so the cursor highlight spans the row.
fn entry_row(
    e: &WikiEntry,
    title_w: usize,
    kind_w: usize,
    tags_w: usize,
    width: u16,
) -> Line<'static> {
    let kind_style = if e.source {
        Style::new().fg(Color::Magenta)
    } else {
        Style::new().fg(Color::Cyan)
    };
    let tags = e
        .tags
        .iter()
        .map(|t| format!("#{t}"))
        .collect::<Vec<_>>()
        .join(" ");
    let mut spans = vec![
        Span::raw(" "),
        Span::styled(
            pad(&e.title, title_w),
            Style::new().add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(pad(&e.kind, kind_w), kind_style),
        Span::raw("  "),
    ];
    if tags_w > 0 {
        spans.push(Span::styled(
            pad(&tags, tags_w),
            Style::new().fg(Color::Yellow),
        ));
        spans.push(Span::raw("  "));
    }
    let used: usize = spans.iter().map(|sp| disp_width(&sp.content)).sum();
    let room = (width as usize).saturating_sub(used);
    spans.push(Span::styled(pad(e.shown_path(), room), dim()));
    Line::from(spans)
}

/// What the Wiki view's list says instead of rows: why the read failed, or
/// that nothing is there or nothing matches. `None` when there are rows.
pub(crate) fn wiki_notice(app: &App, no_rows: bool) -> Option<Paragraph<'static>> {
    if let Some(e) = &app.wiki.error {
        return Some(Paragraph::new(Span::styled(
            format!("reading the wiki failed: {e}"),
            Style::new().fg(Color::Red),
        )));
    }
    no_rows.then(|| {
        let msg = if app.wiki.entries.as_ref().is_some_and(|e| !e.is_empty()) {
            "No page matches the search."
        } else {
            "No pages yet. `yaks page new wiki/<name>.md` writes one."
        };
        Paragraph::new(Span::styled(msg, dim()))
    })
}

/// The column widths of the document rows among `rows`, sized from them and
/// capped so the path keeps some room.
pub(crate) struct DocCols {
    title: usize,
    kind: usize,
    tags: usize,
}

impl DocCols {
    pub(crate) fn of(rows: &[tree::Row<'_>], width: u16) -> DocCols {
        let docs: Vec<&WikiEntry> = rows
            .iter()
            .filter_map(|r| match r.item {
                tree::Item::Doc(e) => Some(e),
                _ => None,
            })
            .collect();
        let w = width as usize;
        DocCols {
            title: docs
                .iter()
                .map(|e| disp_width(&e.title))
                .max()
                .unwrap_or(5)
                .clamp(5, (w * 2 / 5).max(5)),
            kind: docs
                .iter()
                .map(|e| disp_width(&e.kind))
                .max()
                .unwrap_or(4)
                .clamp(4, 12),
            tags: docs
                .iter()
                .map(|e| e.tags.iter().map(|t| disp_width(t) + 2).sum::<usize>())
                .max()
                .unwrap_or(0)
                .min(w / 4),
        }
    }

    /// The list item of a document row.
    pub(crate) fn item(&self, e: &WikiEntry, width: u16) -> ListItem<'static> {
        ListItem::new(entry_row(e, self.title, self.kind, self.tags, width))
    }
}

/// A small wiki for tests and doc shots: two farm-wide pages in folders,
/// a herd page and a source.
#[cfg(test)]
pub(crate) mod fixture {
    use super::*;

    /// Write the fixture's files into the farm at `root`.
    pub(crate) fn write(root: &Path) {
        let files: &[(&str, &str)] = &[
            (
                "wiki/topics/vector-search.md",
                "---\ntype: topic\ntitle: Vector search in Postgres\ndescription: pgvector, indexes and when to use them.\ntags: [databases, search]\nsources:\n  - sources/2026-10-22-1a2b3c4d/source.md\n---\n\n# Vector search\n\nStore embeddings in a `vector` column and index them with **HNSW**.\n\n## Indexes\n\n- HNSW: fast queries, slow builds.\n- IVFFlat: quick to build.\n\nSee [full-text search](full-text-search.md) and yaks-0001.\n",
            ),
            (
                "wiki/topics/full-text-search.md",
                "---\ntype: topic\ntitle: Full-text search\ntags: [search]\n---\n\nUse `tsvector` and a GIN index.\n",
            ),
            (
                "wiki/howto/release.md",
                "---\ntype: howto\ntitle: Cut a release\n---\n\n1. Tag `vX.Y.Z`.\n2. Watch the workflow.\n",
            ),
            (
                "yaks/wiki/tui-layout.md",
                "---\ntitle: TUI layout\ntags: [tui]\n---\n\nThe list on the left, the detail on the right.\n",
            ),
            (
                "sources/2026-10-22-1a2b3c4d/source.md",
                "---\ntype: source\ntitle: pgvector README\nmedia_type: text/markdown\nsources:\n  - resource: https://github.com/pgvector/pgvector\n---\n\nIts text is in [extracted.md](extracted.md).\n",
            ),
            (
                "sources/2026-10-22-1a2b3c4d/extracted.md",
                "# pgvector\n\nOpen-source vector similarity search for Postgres.\n",
            ),
        ];
        for (rel, text) in files {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_support::*;

    /// A live app on a temp farm holding the fixture wiki and one yak,
    /// switched to the Wiki tab.
    pub(crate) fn on_wiki() -> (PathBuf, App) {
        let (proj, farm) = temp_farm(&[task(
            "yaks-0001",
            "Search",
            crate::model::Status::Hairy,
            2,
            None,
        )]);
        fixture::write(farm.root());
        let mut app = App::with_farm(farm).unwrap();
        let i = app
            .views
            .iter()
            .position(|v| v.key == view::WIKI_KEY)
            .unwrap();
        app.set_view(i);
        (proj, app)
    }

    #[test]
    fn the_wiki_tab_sits_after_starred_and_before_sheds() {
        let keys: Vec<String> = view::default_views().into_iter().map(|v| v.key).collect();
        let wiki = keys.iter().position(|k| k == view::WIKI_KEY).unwrap();
        assert_eq!(keys[wiki - 1], "working-set");
        assert_eq!(keys[wiki + 1], view::SHEDS_KEY);
    }

    #[test]
    fn j_moves_the_lists_cursor_and_the_selected_document_follows() {
        let (proj, mut app) = on_wiki();
        assert_eq!(app.cursor, 0);
        assert_eq!(app.selected_doc().unwrap().path, "wiki/howto/release.md");
        press(&mut app, "j");
        assert_eq!(app.cursor, 1);
        assert_eq!(
            app.selected_doc().unwrap().path,
            "wiki/topics/full-text-search.md"
        );
        press(&mut app, "G");
        assert_eq!(app.cursor, 4);
        assert!(app.selected_doc().unwrap().source);
        press(&mut app, "g");
        assert_eq!(app.cursor, 0);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_document_row_is_no_yak_and_yak_keys_do_nothing_on_it() {
        let (proj, mut app) = on_wiki();
        assert!(app.selected().is_none());
        assert!(app.selected_doc().is_some());
        assert!(matches!(app.selected_row(), Some(tree::Item::Doc(_))));
        for keys in [
            "S", "P", "T", "L", "X", "D", "M", "a", "Q", "A", "*", "m", "c",
        ] {
            press(&mut app, keys);
            assert!(
                matches!(app.overlay, Overlay::None),
                "{keys} opened an overlay on a document row"
            );
        }
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn enter_on_a_document_row_opens_the_viewer() {
        let (proj, mut app) = on_wiki();
        enter_key(&mut app);
        let Overlay::Attachment(v) = &app.overlay else {
            panic!("no viewer")
        };
        assert_eq!(v.target, "wiki/howto/release.md");
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn an_empty_wiki_view_still_takes_the_document_keys() {
        let (proj, mut app) = on_wiki();
        app.filter.search = Some("no such page".into());
        assert_eq!(app.row_count(), 0);
        press(&mut app, "N");
        assert!(
            matches!(app.overlay, Overlay::Edit(_)),
            "N: new page prompt"
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn it_lists_pages_then_sources_and_counts_pages() {
        let (proj, app) = on_wiki();
        let paths: Vec<&str> = app.wiki_rows().iter().map(|e| e.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "wiki/howto/release.md",
                "wiki/topics/full-text-search.md",
                "wiki/topics/vector-search.md",
                "yaks/wiki/tui-layout.md",
                "sources/2026-10-22-1a2b3c4d/source.md",
            ]
        );
        assert_eq!(app.wiki_count(), 4);
        let v = app.active_view().clone();
        assert_eq!(app.view_count(&v), 4);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn yak_keys_are_inert_on_the_wiki_tab() {
        let (proj, mut app) = on_wiki();
        for c in ['S', 'X', 'c', 'D', 'M', 'f', '*', 'm', 'a'] {
            press(&mut app, &c.to_string());
            assert!(
                matches!(app.overlay, Overlay::None),
                "{c} opened an overlay on the Wiki tab"
            );
        }
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn slash_filters_the_list_and_esc_reverts() {
        let (proj, mut app) = on_wiki();
        press(&mut app, "/search");
        enter_key(&mut app);
        let titles: Vec<&str> = app.wiki_rows().iter().map(|e| e.title.as_str()).collect();
        assert_eq!(
            titles,
            vec!["Full-text search", "Vector search in Postgres"]
        );
        assert_eq!(app.wiki_count(), 4, "the count stays the page count");
        // A tag matches too (Esc first: `/` reopens seeded with the query).
        esc_key(&mut app);
        press(&mut app, "/tui");
        enter_key(&mut app);
        assert_eq!(app.wiki_rows().len(), 1);
        esc_key(&mut app);
        assert_eq!(app.wiki_rows().len(), 5);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn enter_opens_a_page_with_its_frontmatter_above_the_body() {
        let (proj, mut app) = on_wiki();
        press(&mut app, "jj");
        assert_eq!(
            app.selected_doc().unwrap().title,
            "Vector search in Postgres"
        );
        enter_key(&mut app);
        let Overlay::Attachment(v) = &app.overlay else {
            panic!("the viewer should be open");
        };
        assert_eq!(v.doc, DocKind::Page);
        let frame = draw(&app, 140, 30);
        assert!(
            frame.contains("wiki/topics/vector-search.md  topic"),
            "{frame}"
        );
        assert!(frame.contains("tags:        databases, search"), "{frame}");
        assert!(
            frame.contains("sources:     sources/2026-10-22-1a2b3c4d/source.md"),
            "{frame}"
        );
        // The body is rendered: heading markers hidden, bold drawn.
        assert!(
            frame.contains("Vector search") && !frame.contains("# Vector"),
            "{frame}"
        );
        // The detail pane's navigation: Tab finds the body's links.
        tab_key(&mut app);
        let Overlay::Attachment(v) = &app.overlay else {
            panic!()
        };
        assert!(v.nav.line > 5, "past the frontmatter rows");
        // Esc closes back to the list.
        esc_key(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        assert!(app.on_wiki_tab());
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_source_opens_with_its_extracted_text() {
        let (proj, mut app) = on_wiki();
        press(&mut app, "G");
        enter_key(&mut app);
        let frame = draw(&app, 140, 30);
        assert!(
            frame.contains("sources/2026-10-22-1a2b3c4d/source.md  source"),
            "{frame}"
        );
        assert!(frame.contains("── extracted.md ──"), "{frame}");
        assert!(
            frame.contains("Open-source vector similarity search"),
            "{frame}"
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn an_edit_on_disk_shows_after_a_reload() {
        let (proj, mut app) = on_wiki();
        let root = app.farm.as_ref().unwrap().root().to_path_buf();
        std::fs::write(root.join("wiki/new.md"), "---\ntitle: Brand new\n---\n").unwrap();
        app.reload_preserving_selection();
        assert!(app.wiki_rows().iter().any(|e| e.title == "Brand new"));
        assert_eq!(app.wiki_count(), 5);
        let _ = std::fs::remove_dir_all(proj);
    }

    fn ctrl(app: &mut App, c: char) {
        crate::tui::handle_key(app, KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
    }

    /// The open editor's text (the `E` overlay).
    fn editor_text(app: &App) -> String {
        match &app.overlay {
            Overlay::Edit(ed) => ed.text(),
            _ => panic!("the editor should be open"),
        }
    }

    /// Replace the open editor's text, as a user selecting all and typing.
    fn set_editor_text(app: &mut App, text: &str) {
        let Overlay::Edit(ed) = &mut app.overlay else {
            panic!("the editor should be open");
        };
        *ed.state.borrow_mut() = edtui::EditorState::new(edtui::Lines::from(text));
    }

    #[test]
    fn e_edits_a_page_as_raw_frontmatter_and_body_and_saves_it() {
        let (proj, mut app) = on_wiki();
        let root = app.farm.as_ref().unwrap().root().to_path_buf();
        press(&mut app, "j"); // Full-text search
        enter_key(&mut app);
        press(&mut app, "E");
        let text = editor_text(&app);
        assert!(
            text.starts_with("---\ntype: topic\ntitle: Full-text search\n"),
            "{text}"
        );
        set_editor_text(
            &mut app,
            "---\ntype: topic\ntitle: Full-text search in Postgres\ntags: [search, postgres]\n---\n\nUse `tsvector`.\n",
        );
        ctrl(&mut app, 's');
        assert_eq!(
            app.notification.as_deref(),
            Some("saved wiki/topics/full-text-search.md")
        );
        assert!(
            matches!(app.overlay, Overlay::Attachment(_)),
            "back in the viewer"
        );
        let on_disk =
            std::fs::read_to_string(root.join("wiki/topics/full-text-search.md")).unwrap();
        assert_eq!(
            on_disk,
            "---\ntype: topic\ntitle: Full-text search in Postgres\ntags: [search, postgres]\n---\n\nUse `tsvector`.\n"
        );
        // The list shows the new title and tag.
        let e = app
            .wiki_rows()
            .into_iter()
            .find(|e| e.path.ends_with("full-text-search.md"))
            .unwrap();
        assert_eq!(e.title, "Full-text search in Postgres");
        assert_eq!(e.tags, vec!["search", "postgres"]);
        // E on the list goes straight to the editor.
        esc_key(&mut app);
        press(&mut app, "E");
        assert!(editor_text(&app).contains("in Postgres"));
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_page_without_frontmatter_or_changed_on_disk_is_not_saved() {
        let (proj, mut app) = on_wiki();
        let root = app.farm.as_ref().unwrap().root().to_path_buf();
        let file = root.join("wiki/howto/release.md");
        let before = std::fs::read_to_string(&file).unwrap();
        enter_key(&mut app); // Cut a release
        press(&mut app, "E");
        set_editor_text(&mut app, "No fence at all.\n");
        ctrl(&mut app, 's');
        assert!(matches!(app.overlay, Overlay::Edit(_)), "still editing");
        assert!(app.notification.as_deref().unwrap().contains("frontmatter"));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
        // Someone else writes the page meanwhile: the save stops; a second
        // Ctrl-S overwrites.
        std::fs::write(&file, "---\ntitle: Theirs\n---\n").unwrap();
        set_editor_text(&mut app, "---\ntitle: Mine\n---\n");
        ctrl(&mut app, 's');
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .contains("changed on disk")
        );
        assert!(std::fs::read_to_string(&file).unwrap().contains("Theirs"));
        ctrl(&mut app, 's');
        assert!(std::fs::read_to_string(&file).unwrap().contains("Mine"));
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_source_is_read_only() {
        let (proj, mut app) = on_wiki();
        press(&mut app, "G");
        enter_key(&mut app);
        press(&mut app, "E");
        assert!(matches!(app.overlay, Overlay::Attachment(_)));
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .contains("written once")
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn n_creates_a_page_then_edits_it() {
        let (proj, mut app) = on_wiki();
        let root = app.farm.as_ref().unwrap().root().to_path_buf();
        press(&mut app, "N");
        let Overlay::Edit(ed) = &app.overlay else {
            panic!("the path prompt should be open")
        };
        assert_eq!(ed.text(), "wiki/");
        // A bad name keeps the prompt, with the reason.
        press(&mut app, "Bad Name");
        enter_key(&mut app);
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .contains("lowercase-kebab"),
            "{:?}",
            app.notification
        );
        assert!(matches!(&app.overlay, Overlay::Edit(ed) if ed.text() == "wiki/Bad Name.md"));
        // Fix it: `.md` is added.
        set_editor_text(&mut app, "wiki/topics/hnsw-tuning");
        enter_key(&mut app);
        assert_eq!(
            app.notification.as_deref(),
            Some("created wiki/topics/hnsw-tuning.md")
        );
        let file = root.join("wiki/topics/hnsw-tuning.md");
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "---\ntype: note\ntitle: Hnsw tuning\n---\n"
        );
        assert_eq!(
            app.selected_doc().unwrap().path,
            "wiki/topics/hnsw-tuning.md"
        );
        assert_eq!(app.wiki_count(), 5);
        // The editor is open on it; type a body and save.
        let text = editor_text(&app);
        set_editor_text(&mut app, &format!("{text}\nRaise ef_search for recall.\n"));
        ctrl(&mut app, 's');
        assert!(
            std::fs::read_to_string(&file)
                .unwrap()
                .ends_with("Raise ef_search for recall.\n")
        );
        // A path that exists is refused.
        esc_key(&mut app);
        press(&mut app, "N");
        set_editor_text(&mut app, "wiki/topics/hnsw-tuning.md");
        enter_key(&mut app);
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .contains("already exists")
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    /// A farm on its own `yaks` branch (so writes commit, yaks-a838) holding
    /// the fixture wiki, committed.
    fn branch_farm() -> (PathBuf, PathBuf) {
        let base = std::env::temp_dir().join(format!(
            "yaks-wiki-branch-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let git = |dir: &Path, args: &[&str]| {
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(dir)
                .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
                .args(["-c", "commit.gpgsign=false"])
                .args(args)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        };
        let (code, wt) = (base.join("code"), base.join("farm"));
        std::fs::create_dir_all(&code).unwrap();
        git(&code, &["init", "-q", "-b", "main"]);
        git(&code, &["config", "user.name", "t"]);
        git(&code, &["config", "user.email", "t@example.com"]);
        std::fs::write(code.join("code.rs"), "fn main() {}").unwrap();
        git(&code, &["add", "."]);
        git(&code, &["commit", "-q", "-m", "code"]);
        git(
            &code,
            &["worktree", "add", "-q", "--detach", wt.to_str().unwrap()],
        );
        git(&wt, &["checkout", "-q", "--orphan", "yaks"]);
        git(&wt, &["rm", "-rfq", "."]);
        let root = wt.join(".yaks");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("schema"), crate::store::SCHEMA.to_string()).unwrap();
        std::fs::write(root.join("config.yaml"), "herd: yak\n").unwrap();
        std::fs::write(root.join(".gitignore"), ".lock\n").unwrap();
        fixture::write(&root);
        git(&wt, &["add", "."]);
        git(&wt, &["commit", "-q", "-m", "farm"]);
        (base, root)
    }

    #[test]
    fn a_page_save_is_one_commit_on_the_farm_branch() {
        let (base, root) = branch_farm();
        let Ok(farm) = crate::farm::Farm::at(&root) else {
            panic!("open the branch farm")
        };
        crate::autocommit::set_coalesce(true);
        let mut app = App::with_farm(farm).unwrap();
        let i = app
            .views
            .iter()
            .position(|v| v.key == view::WIKI_KEY)
            .unwrap();
        app.set_view(i);
        let count = || {
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(["rev-list", "--count", "HEAD"])
                .output()
                .unwrap();
            let n = String::from_utf8_lossy(&out.stdout).trim().to_string();
            n.parse::<u32>().unwrap_or_else(|_| {
                panic!("rev-list: {n:?} {}", String::from_utf8_lossy(&out.stderr))
            })
        };
        let before = count();
        press(&mut app, "j");
        press(&mut app, "E");
        set_editor_text(
            &mut app,
            "---\ntype: topic\ntitle: Full-text search\n---\n\nEdited in the TUI.\n",
        );
        ctrl(&mut app, 's');
        assert_eq!(
            app.notification.as_deref(),
            Some("saved wiki/topics/full-text-search.md")
        );
        assert_eq!(count(), before + 1, "one commit for the save");
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["show", "--stat", "--format=%s", "HEAD"])
            .output()
            .unwrap();
        let show = String::from_utf8_lossy(&out.stdout);
        assert!(show.contains("wiki/topics/full-text-search.md"), "{show}");
        assert_eq!(show.matches(" | ").count(), 1, "only the page: {show}");
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn wiki_tab_frames() {
        let (proj, mut app) = on_wiki();
        insta::assert_snapshot!("wiki_tab_list", draw(&app, 120, 12));
        press(&mut app, "jj");
        enter_key(&mut app);
        insta::assert_snapshot!("wiki_tab_page", draw(&app, 150, 26));
        let _ = std::fs::remove_dir_all(proj);
    }
}
