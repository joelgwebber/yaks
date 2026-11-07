//! The Doks tab (yaks-ebb2): the farm's doks, saved sources and session notes,
//! one flat list (doks from `doks/` and each `<herd>/doks/`, then `sources/`,
//! then `sessions/`, yaks-85a6), each group sorted by path, with title, kind
//! and tags. Enter opens one in the text viewer ([`App::read_farm_doc`]): a
//! dok's or session note's frontmatter as field rows above its rendered body,
//! a source's `source.md` then its `extracted.md`.
//!
//! The data is the CLI's: [`doks::load_all`], [`sources::list`] and
//! [`sessions::load_all`]. It is
//! read when the tab is shown and again on every farm reload, so an edit
//! made elsewhere appears as soon as the watcher sees it.
//!
//! The tab is a built-in view (`view::doks_view`) whose rows are documents
//! ([`tree::Item::Doc`], from [`App::build_rows`]): the list's own cursor,
//! rendering and key dispatch serve it, and its keys are the `Doc` bindings of
//! `keymap`. Yak keys find no yak there and do nothing. `/` is the yak tabs'
//! inline search: it edits the global query, which [`App::doks_rows`] matches
//! against a document's title, path, kind and tags and its text.

#[cfg(test)]
use super::handlers::DocKind;
use super::*;
use crate::{doks, sessions, sources};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{ListItem, Paragraph};

/// Which of the farm's document collections a row comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DocGroup {
    Dok,
    /// A saved source (`sources/<id>/source.md`).
    Source,
    /// A session note (`sessions/YYYY/MM/...md`).
    Session,
}

/// One row of the list: a dok, a source or a session note.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DoksEntry {
    /// Farm-relative path of the file the viewer opens: the dok, a source's
    /// `source.md`, or the session note.
    pub path: String,
    pub title: String,
    /// The dok's kind (`note`, `topic`, a typed kind), `source` or `session`.
    pub kind: String,
    pub tags: Vec<String>,
    pub group: DocGroup,
    /// What the query searches besides the fields above, lowercased once
    /// here (matching is case-insensitive): a dok's file (frontmatter and
    /// body), a source's `source.md` and extracted text, as `yaks find` reads
    /// them. Held in memory for every document, so a query costs no reads.
    pub text: String,
}

impl DoksEntry {
    /// The path shown in the list: a source by its directory.
    pub(crate) fn shown_path(&self) -> &str {
        if self.group == DocGroup::Source {
            self.path.strip_suffix("/source.md").unwrap_or(&self.path)
        } else {
            &self.path
        }
    }
}

/// The tab's state: the entries last read (the list's rows index them).
#[derive(Default)]
pub(crate) struct DoksTab {
    /// `None` until first read (the tab has not been shown, or there is no
    /// farm: read-only and test constructors fill it directly).
    pub entries: Option<Vec<DoksEntry>>,
    /// The read failed: why.
    pub error: Option<String>,
}

/// Every dok, source and session note of the farm at `root`, in that order,
/// each group sorted by path (so session notes run oldest first).
pub(crate) fn read_entries(root: &Path) -> Result<Vec<DoksEntry>> {
    let mut out: Vec<DoksEntry> = doks::load_all(root)?
        .into_iter()
        .map(|p| DoksEntry {
            title: p.title(),
            kind: p.kind(),
            tags: p.tags(),
            text: p.render().to_lowercase(),
            path: p.path,
            group: DocGroup::Dok,
        })
        .collect();
    out.extend(sources::list(root).into_iter().map(|s| {
        let read =
            |name: &str| std::fs::read_to_string(root.join(&s.dir).join(name)).unwrap_or_default();
        DoksEntry {
            text: format!("{}\n{}", read("source.md"), read("extracted.md")).to_lowercase(),
            path: format!("{}/source.md", s.dir),
            title: s.title,
            kind: "source".into(),
            tags: Vec::new(),
            group: DocGroup::Source,
        }
    }));
    out.extend(sessions::load_all(root)?.into_iter().map(|s| DoksEntry {
        title: s.title(),
        kind: "session".into(),
        tags: s.tags(),
        text: s.render().to_lowercase(),
        path: s.path,
        group: DocGroup::Session,
    }));
    Ok(out)
}

impl App {
    pub(crate) fn on_doks_tab(&self) -> bool {
        self.active_view().key == view::DOKS_KEY
    }

    /// Re-read the farm's doks, sources and session notes (a no-op without a
    /// farm), keeping
    /// the cursor on the same path.
    pub(crate) fn reload_doks(&mut self) {
        let Some(root) = self.farm.as_ref().map(|f| f.root().to_path_buf()) else {
            return;
        };
        match read_entries(&root) {
            Ok(e) => self.set_doks_entries(e),
            Err(e) => self.doks.error = Some(format!("{e:#}")),
        }
    }

    /// Swap in a new list of entries, keeping the cursor on the same path.
    pub(crate) fn set_doks_entries(&mut self, entries: Vec<DoksEntry>) {
        let at = self.selected_doc().map(|e| e.path.clone());
        self.doks.entries = Some(entries);
        self.doks.error = None;
        // The rows index the entries: rebuild them.
        self.touch();
        // Only the Doks view's cursor means a document.
        if self.on_doks_tab() {
            let rows = self.doks_rows();
            self.cursor = at
                .and_then(|p| rows.iter().position(|e| e.path == p))
                .unwrap_or(self.cursor)
                .min(rows.len().saturating_sub(1));
        }
    }

    /// The searches a row must match: the query, and the view's own search
    /// when it has one (the drawer's search row, a saved view's). Documents and
    /// sheds have no other content facet; a yak's own search is `filter.matches`.
    pub(crate) fn searches(&self) -> impl Iterator<Item = &str> {
        [self.query.as_deref(), self.filter.search.as_deref()]
            .into_iter()
            .flatten()
            .filter(|q| !q.is_empty())
    }

    /// Whether `item` (a document or a shed) matches every active search.
    pub(crate) fn search_keeps(&self, item: tree::Item<'_>) -> bool {
        self.searches().all(|q| tree::query_matches(item, q))
    }

    /// The entries the live searches keep, in display order: the Doks view's
    /// rows ([`App::build_rows`] wraps each in a [`tree::Row`]).
    pub(crate) fn doks_rows(&self) -> Vec<&DoksEntry> {
        self.doks
            .entries
            .iter()
            .flatten()
            .filter(|e| self.search_keeps(tree::Item::Doc(e)))
            .collect()
    }

    /// The documents the query keeps (all of them without one), doks first as
    /// read: the part of the All view's count and rows that is not yaks.
    pub(crate) fn docs_matching_query(&self) -> impl Iterator<Item = &DoksEntry> {
        let q = self.query.as_deref();
        self.doks
            .entries
            .iter()
            .flatten()
            .filter(move |e| q.is_none_or(|q| tree::query_matches(tree::Item::Doc(e), q)))
    }

    /// The tab's count: the doks (not sources or session notes) the query
    /// keeps.
    pub(crate) fn doks_count(&self) -> usize {
        self.docs_matching_query()
            .filter(|e| e.group == DocGroup::Dok)
            .count()
    }

    /// Enter / l: open the selected dok or source in the viewer. A dok has
    /// no pane until it is opened, so this is a jump: `o` returns to the row.
    pub(crate) fn open_doks_entry(&mut self) {
        let Some(e) = self.selected_doc().cloned() else {
            return;
        };
        match self.read_farm_doc(&e.path, None) {
            Ok(v) => self.follow_to_doc(DocView::Text(v)),
            Err(msg) => self.notification = Some(msg),
        }
    }

    /// N: a one-line prompt for a new dok's path, seeded with `doks/`.
    pub(crate) fn open_new_dok(&mut self) {
        if self.farm.is_none() {
            return;
        }
        self.new_dok_prompt("doks/");
    }

    fn new_dok_prompt(&mut self, seed: &str) {
        let ed = Editor::new(
            self.editor_vim(),
            true,
            "New dok (doks/<path>.md or <herd>/doks/<path>.md): ".into(),
            seed,
            EditAction::NewDok,
        );
        // Typing goes straight on after the seed, in Insert.
        ed.state.borrow_mut().mode = EditorMode::Insert;
        ed.state.borrow_mut().cursor = edtui::Index2::new(0, seed.chars().count());
        self.overlay = Overlay::Edit(ed);
    }

    /// The new-dok prompt's Enter: create the dok (`Farm::dok_new`: kind
    /// `note`, a title from the file name, any schema defaults; one commit),
    /// select it in the list and open it in the editor for its body. A bad
    /// or taken path keeps the prompt open with the reason.
    pub(crate) fn commit_new_dok(&mut self, text: &str) {
        let mut path = doks::normalize(text.trim());
        if path.is_empty() || path == "doks/" {
            self.notification = Some("no dok created".into());
            return;
        }
        if !path.ends_with(".md") {
            path.push_str(".md");
        }
        let Some(farm) = self.farm.as_ref() else {
            return;
        };
        if let Err(e) = farm.dok_new(&path, None, None, &[], "") {
            self.notification = Some(format!("{e:#}"));
            self.new_dok_prompt(&path);
            return;
        }
        self.reload_doks();
        self.refresh_farm_sync();
        if let Some(i) = self.doks_rows().iter().position(|e| e.path == path) {
            self.cursor = i;
        }
        match self.read_farm_doc(&path, None) {
            Ok(v) => {
                self.follow_to_doc(DocView::Text(v));
                self.open_attachment_edit();
                self.notification = Some(format!("created {path}"));
            }
            Err(e) => self.notification = Some(e),
        }
    }

    /// E on the list: edit the selected dok (open it, then the editor).
    pub(crate) fn edit_doks_entry(&mut self) {
        self.open_doks_entry();
        if self.attach().is_some() {
            self.open_attachment_edit();
        }
    }

    /// y: copy the selected entry's path.
    pub(crate) fn copy_doks_path(&mut self) {
        let Some(path) = self.selected_doc().map(|e| e.shown_path().to_string()) else {
            return;
        };
        self.notification = Some(if crate::clipboard::copy_text(&path) {
            format!("copied {path}")
        } else {
            "clipboard unavailable".into()
        });
    }

    /// toque's state-header digest on the Doks tab.
    pub(crate) fn doks_state_header(&self) -> String {
        let at = self
            .selected_doc()
            .map_or_else(|| "-".into(), |e| e.path.clone());
        format!("doks={} \u{b7} entry={at}", self.doks_rows().len())
    }

    /// `r`: read the doks again.
    pub(crate) fn reread_doks(&mut self) {
        self.reload_doks();
        self.notification = Some("reread the doks".into());
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
    e: &DoksEntry,
    title_w: usize,
    kind_w: usize,
    tags_w: usize,
    width: u16,
) -> Line<'static> {
    let kind_style = match e.group {
        DocGroup::Dok => Style::new().fg(Color::Cyan),
        DocGroup::Source => Style::new().fg(Color::Magenta),
        DocGroup::Session => Style::new().fg(Color::Green),
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

/// What the Doks view's list says instead of rows: why the read failed, or
/// that nothing is there or nothing matches. `None` when there are rows.
pub(crate) fn doks_notice(app: &App, no_rows: bool) -> Option<Paragraph<'static>> {
    if let Some(e) = &app.doks.error {
        return Some(Paragraph::new(Span::styled(
            format!("reading the doks failed: {e}"),
            Style::new().fg(Color::Red),
        )));
    }
    no_rows.then(|| {
        let msg = if app.doks.entries.as_ref().is_some_and(|e| !e.is_empty()) {
            "No dok matches the search."
        } else {
            "No doks yet. `yaks dok new doks/<name>.md` writes one."
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
        let docs: Vec<&DoksEntry> = rows
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
    pub(crate) fn item(&self, e: &DoksEntry, width: u16) -> ListItem<'static> {
        ListItem::new(entry_row(e, self.title, self.kind, self.tags, width))
    }
}

/// A small doks for tests and doc shots: two farm-wide doks in folders,
/// a herd dok, a source and a session note.
#[cfg(test)]
pub(crate) mod fixture {
    use super::*;

    /// Write the fixture's files into the farm at `root`.
    pub(crate) fn write(root: &Path) {
        let files: &[(&str, &str)] = &[
            (
                "doks/topics/vector-search.md",
                "---\ntype: topic\ntitle: Vector search in Postgres\ndescription: pgvector, indexes and when to use them.\ntags: [databases, search]\nsources:\n  - sources/2026-10-22-1a2b3c4d/source.md\n---\n\n# Vector search\n\nStore embeddings in a `vector` column and index them with **HNSW**.\n\n## Indexes\n\n- HNSW: fast queries, slow builds.\n- IVFFlat: quick to build.\n\nSee [full-text search](full-text-search.md) and yaks-0001.\n",
            ),
            (
                "doks/topics/full-text-search.md",
                "---\ntype: topic\ntitle: Full-text search\ntags: [search]\n---\n\nUse `tsvector` and a GIN index.\n",
            ),
            (
                "doks/howto/release.md",
                "---\ntype: howto\ntitle: Cut a release\n---\n\n1. Tag `vX.Y.Z`.\n2. Watch the workflow.\n",
            ),
            (
                "yaks/doks/tui-layout.md",
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
            (
                "sessions/2026/10/herd-2026-10-23-0915-pick-an-index.md",
                "---\ntype: session-review\ntitle: Pick an index\nsource: session-note\nactor: herd\nday: 2026-10-23\ngood: 1\n---\n\n## Summary\n\nChose HNSW for [vector search](/doks/topics/vector-search.md).\n\n## Turns\n\n- good: pick an index\n\n## Lessons\n\n- none\n",
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

    /// A live app on a temp farm holding the fixture doks and one yak,
    /// switched to the Doks tab.
    pub(crate) fn on_doks() -> (PathBuf, App) {
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
            .position(|v| v.key == view::DOKS_KEY)
            .unwrap();
        app.set_view(i);
        (proj, app)
    }

    #[test]
    fn the_doks_tab_sits_after_starred_and_before_sheds() {
        let keys: Vec<String> = view::default_views().into_iter().map(|v| v.key).collect();
        let doks = keys.iter().position(|k| k == view::DOKS_KEY).unwrap();
        assert_eq!(keys[doks - 1], "working-set");
        assert_eq!(keys[doks + 1], view::SHEDS_KEY);
    }

    #[test]
    fn j_moves_the_lists_cursor_and_the_selected_document_follows() {
        let (proj, mut app) = on_doks();
        assert_eq!(app.cursor, 0);
        assert_eq!(app.selected_doc().unwrap().path, "doks/howto/release.md");
        press(&mut app, "j");
        assert_eq!(app.cursor, 1);
        assert_eq!(
            app.selected_doc().unwrap().path,
            "doks/topics/full-text-search.md"
        );
        press(&mut app, "G");
        assert_eq!(app.cursor, 5);
        assert_eq!(app.selected_doc().unwrap().group, DocGroup::Session);
        press(&mut app, "g");
        assert_eq!(app.cursor, 0);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_document_row_is_no_yak_and_yak_keys_do_nothing_on_it() {
        let (proj, mut app) = on_doks();
        assert!(app.selected().is_none());
        assert!(app.selected_doc().is_some());
        assert!(matches!(app.selected_row(), Some(tree::Item::Doc(_))));
        for keys in [
            "S", "P", "T", "L", "X", "D", "M", "a", "Q", "A", "*", "m", "C",
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
        let (proj, mut app) = on_doks();
        enter_key(&mut app);
        let Some(v) = app.attach() else {
            panic!("no viewer")
        };
        assert_eq!(v.target, "doks/howto/release.md");
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn an_empty_doks_view_still_takes_the_document_keys() {
        let (proj, mut app) = on_doks();
        app.filter.search = Some("no such dok".into());
        assert_eq!(app.row_count(), 0);
        press(&mut app, "c");
        assert!(matches!(app.overlay, Overlay::Edit(_)), "c: new dok prompt");
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn it_lists_doks_then_sources_then_sessions_and_counts_doks() {
        let (proj, app) = on_doks();
        let paths: Vec<&str> = app.doks_rows().iter().map(|e| e.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "doks/howto/release.md",
                "doks/topics/full-text-search.md",
                "doks/topics/vector-search.md",
                "yaks/doks/tui-layout.md",
                "sources/2026-10-22-1a2b3c4d/source.md",
                "sessions/2026/10/herd-2026-10-23-0915-pick-an-index.md",
            ]
        );
        assert_eq!(app.doks_count(), 4);
        let v = app.active_view().clone();
        assert_eq!(app.view_count(&v), 4);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn yak_keys_are_inert_on_the_doks_tab() {
        let (proj, mut app) = on_doks();
        for c in ['S', 'X', 'C', 'D', 'M', 'f', '*', 'm', 'a'] {
            press(&mut app, &c.to_string());
            assert!(
                matches!(app.overlay, Overlay::None),
                "{c} opened an overlay on the Doks tab"
            );
        }
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn slash_filters_the_list_and_esc_reverts() {
        let (proj, mut app) = on_doks();
        press(&mut app, "/search");
        enter_key(&mut app);
        let titles: Vec<&str> = app.doks_rows().iter().map(|e| e.title.as_str()).collect();
        assert_eq!(
            titles,
            // The source's extracted text says "search" too, and so does the
            // session note (its link to the vector-search dok).
            vec![
                "Full-text search",
                "Vector search in Postgres",
                "pgvector README",
                "Pick an index"
            ]
        );
        assert_eq!(app.doks_count(), 2, "the count is the doks the query keeps");
        // A tag matches too (Esc first: `/` reopens seeded with the query).
        esc_key(&mut app);
        press(&mut app, "/tui");
        enter_key(&mut app);
        assert_eq!(app.doks_rows().len(), 1);
        esc_key(&mut app);
        assert_eq!(app.doks_rows().len(), 6);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn enter_opens_a_dok_with_its_frontmatter_above_the_body() {
        let (proj, mut app) = on_doks();
        press(&mut app, "jj");
        assert_eq!(
            app.selected_doc().unwrap().title,
            "Vector search in Postgres"
        );
        enter_key(&mut app);
        let Some(v) = app.attach() else {
            panic!("the viewer should be open");
        };
        assert_eq!(v.doc, DocKind::Dok);
        let frame = draw(&app, 140, 30);
        assert!(
            frame.contains("doks/topics/vector-search.md  topic"),
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
        let Some(v) = app.attach() else { panic!() };
        assert!(v.nav.line > 5, "past the frontmatter rows");
        // Esc closes back to the list.
        esc_key(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        assert!(app.on_doks_tab());
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_source_opens_with_its_extracted_text() {
        let (proj, mut app) = on_doks();
        press(&mut app, "G");
        press(&mut app, "k");
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

    /// yaks-85a6: a session note is a row of its own kind, opens like a dok
    /// (frontmatter rows above the rendered body), and a dok's link to one
    /// opens it.
    #[test]
    fn a_session_note_is_listed_and_opens_like_a_dok() {
        let (proj, mut app) = on_doks();
        let list = draw(&app, 140, 30);
        assert!(list.contains("Pick an index"), "{list}");
        assert!(list.contains("session"), "{list}");
        press(&mut app, "G");
        enter_key(&mut app);
        let Some(v) = app.attach() else {
            panic!("the viewer should be open");
        };
        assert_eq!(v.doc, DocKind::Dok);
        let frame = draw(&app, 140, 30);
        assert!(frame.contains("actor:"), "{frame}");
        assert!(
            frame.contains("Summary") && !frame.contains("## Summary"),
            "{frame}"
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn an_edit_on_disk_shows_after_a_reload() {
        let (proj, mut app) = on_doks();
        let root = app.farm.as_ref().unwrap().root().to_path_buf();
        std::fs::write(root.join("doks/new.md"), "---\ntitle: Brand new\n---\n").unwrap();
        app.reload_preserving_selection();
        assert!(app.doks_rows().iter().any(|e| e.title == "Brand new"));
        assert_eq!(app.doks_count(), 5);
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
    fn e_edits_a_dok_as_raw_frontmatter_and_body_and_saves_it() {
        let (proj, mut app) = on_doks();
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
            Some("saved doks/topics/full-text-search.md")
        );
        assert!(app.attach().is_some(), "back in the viewer");
        let on_disk =
            std::fs::read_to_string(root.join("doks/topics/full-text-search.md")).unwrap();
        assert_eq!(
            on_disk,
            "---\ntype: topic\ntitle: Full-text search in Postgres\ntags: [search, postgres]\n---\n\nUse `tsvector`.\n"
        );
        // The list shows the new title and tag.
        let e = app
            .doks_rows()
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
    fn a_dok_without_frontmatter_or_changed_on_disk_is_not_saved() {
        let (proj, mut app) = on_doks();
        let root = app.farm.as_ref().unwrap().root().to_path_buf();
        let file = root.join("doks/howto/release.md");
        let before = std::fs::read_to_string(&file).unwrap();
        enter_key(&mut app); // Cut a release
        press(&mut app, "E");
        set_editor_text(&mut app, "No fence at all.\n");
        ctrl(&mut app, 's');
        assert!(matches!(app.overlay, Overlay::Edit(_)), "still editing");
        assert!(app.notification.as_deref().unwrap().contains("frontmatter"));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
        // Someone else writes the dok meanwhile: the save stops; a second
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
        let (proj, mut app) = on_doks();
        press(&mut app, "G");
        press(&mut app, "k");
        enter_key(&mut app);
        press(&mut app, "E");
        assert!(app.attach().is_some());
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .contains("written once")
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn n_creates_a_dok_then_edits_it() {
        let (proj, mut app) = on_doks();
        let root = app.farm.as_ref().unwrap().root().to_path_buf();
        press(&mut app, "c");
        let Overlay::Edit(ed) = &app.overlay else {
            panic!("the path prompt should be open")
        };
        assert_eq!(ed.text(), "doks/");
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
        assert!(matches!(&app.overlay, Overlay::Edit(ed) if ed.text() == "doks/Bad Name.md"));
        // Fix it: `.md` is added.
        set_editor_text(&mut app, "doks/topics/hnsw-tuning");
        enter_key(&mut app);
        assert_eq!(
            app.notification.as_deref(),
            Some("created doks/topics/hnsw-tuning.md")
        );
        let file = root.join("doks/topics/hnsw-tuning.md");
        // `created:` is now (yaks-c316); the rest is the template.
        let text = std::fs::read_to_string(&file).unwrap();
        let (head, rest) = text.split_once("created: '").unwrap();
        let (stamp, tail) = rest.split_once("'\n").unwrap();
        assert!(
            chrono::DateTime::parse_from_rfc3339(stamp).is_ok(),
            "{stamp}"
        );
        assert_eq!(
            format!("{head}{tail}"),
            "---\ntype: note\ntitle: Hnsw tuning\n---\n"
        );
        assert_eq!(
            app.selected_doc().unwrap().path,
            "doks/topics/hnsw-tuning.md"
        );
        assert_eq!(app.doks_count(), 5);
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
        press(&mut app, "c");
        set_editor_text(&mut app, "doks/topics/hnsw-tuning.md");
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
    /// the fixture doks, committed.
    fn branch_farm() -> (PathBuf, PathBuf) {
        let base = std::env::temp_dir().join(format!(
            "yaks-doks-branch-{}-{}",
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
    fn a_dok_save_is_one_commit_on_the_farm_branch() {
        let (base, root) = branch_farm();
        let Ok(farm) = crate::farm::Farm::at(&root) else {
            panic!("open the branch farm")
        };
        crate::autocommit::set_coalesce(true);
        let mut app = App::with_farm(farm).unwrap();
        let i = app
            .views
            .iter()
            .position(|v| v.key == view::DOKS_KEY)
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
            Some("saved doks/topics/full-text-search.md")
        );
        assert_eq!(count(), before + 1, "one commit for the save");
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["show", "--stat", "--format=%s", "HEAD"])
            .output()
            .unwrap();
        let show = String::from_utf8_lossy(&out.stdout);
        assert!(show.contains("doks/topics/full-text-search.md"), "{show}");
        assert_eq!(show.matches(" | ").count(), 1, "only the dok: {show}");
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn doks_tab_frames() {
        let (proj, mut app) = on_doks();
        insta::assert_snapshot!("doks_tab_list", draw(&app, 120, 12));
        press(&mut app, "jj");
        enter_key(&mut app);
        insta::assert_snapshot!("doks_tab_dok", draw(&app, 150, 26));
        let _ = std::fs::remove_dir_all(proj);
    }
}
