//! One history of locations (yaks-a2c5, phase 5 of
//! /wiki/design/tui-model.md#3-one-history).
//!
//! A [`Location`] is everything needed to be back where you were: the view,
//! its live filter and query, the cursor row (by key, with its index as the
//! fallback), the list's scroll, the focus and the pane's subject and
//! position. [`History`] is two stacks of them, `back` and `fwd`, capped at
//! [`CAP`]; it replaces the three back stacks the TUI had (followed yaks,
//! followed documents, in-page jumps).
//!
//! - **Jumps push, movement doesn't.** A jump is run through [`App::jump`],
//!   which snapshots the location it leaves and pushes it when the jump moved
//!   somewhere else: following a link of any kind (a yak, a document, a
//!   footnote, a `#section`), opening a row in the pane (a yak, a shed or a
//!   page: yaks-aec7), a view switch and `#`. Moving the cursor or a line,
//!   `J`/`K`, closing the pane and committing a search push nothing: the
//!   current location is read off the live state when it is needed, so
//!   nothing has to be kept in step.
//! - **`o`/`i`** ([`App::history_back`], [`App::history_forward`]) move the
//!   current location to the other stack and [restore](App::restore) the one
//!   they pop, in every focus and view.
//! - **Restore clamps.** A view that is gone falls back to the subject's
//!   status view, else the first; a row that left the list leaves the cursor
//!   at its old index and the notification says `<id> moved` (still a yak, but
//!   not in this list) or `<id> gone`; a document is read from disk again and
//!   its cursor and scroll are clamped to what it holds now.
//!
//! The stacks are not persisted.

use super::linenav::LineNav;
use super::*;

/// Most locations each stack keeps; the oldest are dropped past it.
pub(crate) const CAP: usize = 100;

/// A list row by what it is, not where it is: the same row is found again
/// after the list changes.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum RowKey {
    /// A yak, by id.
    Yak(String),
    /// A document (a page or source), by farm-relative path.
    Doc(String),
    /// A shed, by name.
    Shed(String),
}

impl RowKey {
    fn of(item: tree::Item<'_>) -> RowKey {
        match item {
            tree::Item::Yak(t) => RowKey::Yak(t.id.clone()),
            tree::Item::Doc(e) => RowKey::Doc(e.path.clone()),
            tree::Item::Shed(s) => RowKey::Shed(s.name.clone()),
        }
    }

    /// What the notification calls it: the id, the path, the name.
    fn label(&self) -> &str {
        match self {
            RowKey::Yak(s) | RowKey::Doc(s) | RowKey::Shed(s) => s,
        }
    }
}

/// What the pane shows, and where in it.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum PaneAt {
    /// A yak's detail, shown whether or not it is the cursor's row.
    Yak {
        id: String,
        line: usize,
        scroll: u16,
    },
    /// A document open in the pane: an attachment, a page, a source or an
    /// image (whose line and scroll are 0).
    Doc {
        /// The document as the viewer names it: `<id>/<file>` for an
        /// attachment, a farm-relative path otherwise.
        path: String,
        /// The farm commit it was shown at, for a pinned link.
        rev: Option<String>,
        /// Where focus goes when it closes ([`DocPane::from`]).
        from: Focus,
        /// The yak the pane showed under it, when that was not the cursor's
        /// row ([`App::pane_yak`]).
        under: Option<String>,
        line: usize,
        scroll: u16,
    },
    /// The shed under the cursor, in the pane.
    Shed(String),
}

impl PaneAt {
    fn label(&self) -> &str {
        match self {
            PaneAt::Yak { id, .. } => id,
            PaneAt::Doc { path, .. } => path,
            PaneAt::Shed(name) => name,
        }
    }
}

/// Where the user is: see the module docs.
#[derive(Clone, PartialEq)]
pub(crate) struct Location {
    /// The active view, by its stable key (not its index, which reorders).
    pub(crate) view_key: String,
    /// The live facets, unsaved edits included.
    pub(crate) filter: FilterSpec,
    /// The query (part of a location, decision 1 of the design).
    pub(crate) query: Option<String>,
    /// The cursor's row; `None` on an empty list.
    pub(crate) cursor: Option<RowKey>,
    /// Its index: where the cursor goes when the row is not found.
    pub(crate) cursor_idx: usize,
    pub(crate) list_offset: usize,
    pub(crate) focus: Focus,
    /// `None` when the list has the focus and no page is open.
    pub(crate) pane: Option<PaneAt>,
}

impl Location {
    /// The yak this location is about: the pane's, else the cursor's.
    fn yak(&self) -> Option<&str> {
        match (&self.pane, &self.cursor) {
            (Some(PaneAt::Yak { id, .. }), _) => Some(id),
            (_, Some(RowKey::Yak(id))) => Some(id),
            _ => None,
        }
    }

    /// What the notification calls the place: the pane's subject, else the
    /// cursor's row.
    fn subject(&self) -> Option<&str> {
        match (&self.pane, &self.cursor) {
            (Some(pane), _) => Some(pane.label()),
            (None, Some(key)) => Some(key.label()),
            _ => None,
        }
    }
}

/// The two stacks.
#[derive(Default)]
pub(crate) struct History {
    pub(crate) back: Vec<Location>,
    pub(crate) fwd: Vec<Location>,
}

fn push_capped(stack: &mut Vec<Location>, loc: Location) {
    stack.push(loc);
    if stack.len() > CAP {
        stack.remove(0);
    }
}

impl History {
    /// A jump left `from`: it is where `o` goes back to, and nothing is
    /// forward of the new place.
    fn push(&mut self, from: Location) {
        // `l`, `h`, `l` on one row leaves the same stop each time: keep one.
        if self.back.last() != Some(&from) {
            push_capped(&mut self.back, from);
        }
        self.fwd.clear();
    }
}

impl App {
    /// Where the user is now, read off the live state.
    pub(crate) fn current_location(&self) -> Location {
        let rows = self.cached_rows();
        let cursor = rows
            .get(self.cursor)
            .map(|r| RowKey::of(self.resolve_item(r.loc)));
        let pane = match &self.doc {
            Some(DocPane {
                view: DocView::Text(v),
                from,
            }) => Some(PaneAt::Doc {
                path: v.target.clone(),
                rev: v.rev.clone(),
                from: *from,
                under: self.pane_yak.clone(),
                line: v.nav.line,
                scroll: v.nav.scroll,
            }),
            Some(DocPane {
                view: DocView::Image(v),
                from,
            }) => Some(PaneAt::Doc {
                path: v.target.clone(),
                rev: None,
                from: *from,
                under: self.pane_yak.clone(),
                line: 0,
                scroll: 0,
            }),
            None if self.focus == Focus::Detail => {
                if let Some(s) = self.selected_shed() {
                    Some(PaneAt::Shed(s.name.clone()))
                } else {
                    self.selected().map(|t| PaneAt::Yak {
                        id: t.id.clone(),
                        line: self.detail.line,
                        scroll: self.detail.scroll,
                    })
                }
            }
            None => None,
        };
        Location {
            view_key: self.active_view().key.clone(),
            filter: self.filter.clone(),
            query: self.query.clone(),
            cursor,
            cursor_idx: self.cursor,
            list_offset: self.list_offset.get(),
            focus: self.focus,
            pane,
        }
    }

    /// Run `go`, a jump, and remember where it left: the location before is
    /// pushed (the forward stack cleared) unless `go` ended where it began,
    /// as a link to the yak already shown, or one that failed, does.
    pub(crate) fn jump(&mut self, go: impl FnOnce(&mut App)) {
        let before = self.current_location();
        go(self);
        if self.current_location() != before {
            self.history.push(before);
        }
    }

    /// Switch to view `i` as a jump (`Tab`, the picker, a click on a tab).
    pub(crate) fn jump_view(&mut self, i: usize) {
        self.jump(|app| app.set_view(i));
    }

    /// `o`: back to the previous location.
    pub(crate) fn history_back(&mut self) {
        self.history_hop(false);
    }

    /// `i`: forward again after going back.
    pub(crate) fn history_forward(&mut self) {
        self.history_hop(true);
    }

    fn history_hop(&mut self, forward: bool) {
        let here = self.current_location();
        // A stop that is where we already are (a page opened from a row,
        // then closed) is no move: look past it.
        let to = loop {
            let from = if forward {
                &mut self.history.fwd
            } else {
                &mut self.history.back
            };
            match from.pop() {
                Some(l) if l == here => continue,
                Some(l) => break l,
                None => {
                    self.notification = Some(
                        if forward {
                            "no later location"
                        } else {
                            "no earlier location"
                        }
                        .into(),
                    );
                    return;
                }
            }
        };
        let other = if forward {
            &mut self.history.back
        } else {
            &mut self.history.fwd
        };
        push_capped(other, here);
        let at = self.restore(&to);
        let arrow = if forward { "\u{2192}" } else { "\u{2190}" };
        self.notification = Some(format!("{arrow} {at}"));
    }

    /// Put the app where `loc` was, clamping what has changed since, and say
    /// where that is: `Recent \u{b7} yaks-1234`, with ` moved` or ` gone` after
    /// a row that is no longer in the list. A view that is gone is named too.
    fn restore(&mut self, loc: &Location) -> String {
        // The pane the page is in keeps its width through a re-read.
        let doc_width = self.attach().map(|v| v.width.get());
        // What is no longer there: (row or subject, `moved` or `gone`).
        let mut missing: Vec<(String, &'static str)> = Vec::new();
        let mut miss = |id: &str, how: &'static str| {
            if !missing.iter().any(|(m, _)| m == id) {
                missing.push((id.to_string(), how));
            }
        };

        // The view, by key: gone, the subject's status view, else the first.
        let found = self.views.iter().position(|v| v.key == loc.view_key);
        let at = found.unwrap_or_else(|| {
            let status = loc.yak().and_then(|id| self.task(id)).map(|t| t.status);
            status
                .and_then(|st| self.views.iter().position(|v| v.status == Some(st)))
                .unwrap_or(0)
        });
        self.set_view(at);
        let view_gone = found.is_none();
        if !view_gone {
            self.filter = loc.filter.clone();
        }
        self.query = loc.query.clone();

        // The cursor, by key (at its index first: a row can be listed twice,
        // as a yak under two sheds), else clamped to its index.
        let rows = self.cached_rows();
        let len = rows.len();
        let key_at = |i: usize| RowKey::of(self.resolve_item(rows[i].loc));
        let on = loc.cursor.as_ref().and_then(|k| {
            if loc.cursor_idx < len && key_at(loc.cursor_idx) == *k {
                Some(loc.cursor_idx)
            } else {
                (0..len).find(|&i| key_at(i) == *k)
            }
        });
        self.cursor = on.unwrap_or(loc.cursor_idx.min(len.saturating_sub(1)));
        if let (None, Some(key)) = (on, &loc.cursor) {
            // Still a yak that is not shown here (another state, a facet, the
            // query) or a page that is: moved. Else gone.
            let there = match key {
                RowKey::Yak(id) => self.task(id).is_some_and(|t| t.status != Status::Dead),
                RowKey::Doc(path) => self.wiki.entries.iter().flatten().any(|e| e.path == *path),
                RowKey::Shed(_) => false,
            };
            miss(key.label(), if there { "moved" } else { "gone" });
        }
        self.list_offset
            .set(loc.list_offset.min(len.saturating_sub(1)));

        // The pane.
        let mut shown = false;
        match &loc.pane {
            Some(PaneAt::Yak { id, line, scroll }) => {
                if self.task(id).is_none() {
                    miss(id, "gone");
                } else {
                    self.focus = Focus::Detail;
                    self.show_yak_at(id);
                    let last = self.detail_dlines().len().saturating_sub(1);
                    self.detail.line = (*line).min(last);
                    self.detail.scroll = (*scroll).min(self.detail_max_scroll());
                    shown = true;
                }
            }
            Some(PaneAt::Shed(name)) => {
                if self.selected_shed().is_some_and(|s| s.name == *name) {
                    self.detail = LineNav::default();
                    shown = true;
                } else {
                    miss(name, "gone");
                }
            }
            Some(PaneAt::Doc {
                path,
                rev,
                from,
                under,
                line,
                scroll,
            }) => match self.read_doc(path, rev.as_deref()) {
                Ok(mut view) => {
                    if let (DocView::Text(v), Some(w)) = (&mut view, doc_width) {
                        v.width.set(w);
                    }
                    self.doc = Some(DocPane { view, from: *from });
                    self.focus = Focus::Detail;
                    self.pane_yak = under.clone().filter(|id| self.task(id).is_some());
                    self.set_doc_position(*line, *scroll);
                    shown = true;
                }
                Err(_) => miss(path, "gone"),
            },
            None => {}
        }
        self.focus = if shown { loc.focus } else { Focus::List };

        // The tab's name without its icon.
        let view = self
            .active_view()
            .name
            .trim_start_matches(|c: char| !c.is_alphanumeric())
            .to_string();
        let subject = loc
            .subject()
            .map(|s| match missing.iter().find(|(m, _)| m == s) {
                Some((_, how)) => format!("{s} {how}"),
                None => s.to_string(),
            })
            .map(|s| format!(" \u{b7} {s}"))
            .unwrap_or_default();
        let others: String = missing
            .iter()
            .filter(|(m, _)| loc.subject() != Some(m.as_str()))
            .map(|(m, how)| format!(" \u{b7} {m} {how}"))
            .collect();
        let gone = if view_gone {
            format!(" (view {} is gone)", loc.view_key)
        } else {
            String::new()
        };
        format!("{view}{subject}{others}{gone}")
    }

    /// Put the open page's cursor and scroll at `line` and `scroll`, clamped
    /// to the rows it has now (an image has none).
    fn set_doc_position(&mut self, line: usize, scroll: u16) {
        let Some((rows, vp)) = self.attachment_rows() else {
            return;
        };
        if let Some(v) = self.attach_mut() {
            v.nav.line = line.min(rows.len().saturating_sub(1));
            v.nav.scroll = scroll.min(vp.max_scroll(rows.len()) as u16);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_support::*;

    fn goto_view(app: &mut App, key: &str) {
        let i = app.views.iter().position(|v| v.key == key).unwrap();
        app.jump_view(i);
    }

    /// Put the cursor on yak `id` of the active list.
    fn cursor_on(app: &mut App, id: &str) {
        app.cursor = app
            .rows()
            .iter()
            .position(|r| r.yak().is_some_and(|t| t.id == id))
            .unwrap();
    }

    /// a0 links to s1 (shaving) and the list has a1 too.
    fn linked_yaks() -> App {
        let mut a0 = task("a0", "Root A", Status::Hairy, 2, None);
        a0.body = "filler\n\nsee s1 here".into();
        App::new(vec![
            a0,
            task("s1", "Shaving one", Status::Shaving, 3, None),
            task("a1", "Root B", Status::Hairy, 3, None),
        ])
    }

    #[test]
    fn a_link_out_of_a_recent_search_comes_back_with_the_query_cursor_and_line() {
        let mut app = linked_yaks();
        let recent = app.views.iter().position(|v| v.key == "recent").unwrap();
        app.set_view(recent);
        app.query = Some("Root".into());
        cursor_on(&mut app, "a1");
        cursor_on(&mut app, "a0");
        let at = app.cursor;
        enter_key(&mut app);
        draw(&app, 100, 20);
        tab_key(&mut app); // the s1 link line
        let (line, scroll) = (app.detail.line, app.detail.scroll);
        assert!(line > 0);
        enter_key(&mut app); // s1 does not match "Root": not in this list
        assert_eq!(app.pane_yak.as_deref(), Some("s1"));
        assert_eq!(app.query.as_deref(), Some("Root"));
        // A view switch, then the query cleared by hand (not a jump): `o`
        // restores s1 under the query, and `o` again the yak it came from.
        app.jump_view(0);
        app.query = None;
        press(&mut app, "o");
        assert_eq!(app.pane_yak.as_deref(), Some("s1"));
        assert_eq!(app.query.as_deref(), Some("Root"));
        press(&mut app, "o");
        assert_eq!(app.active_view().key, "recent");
        assert_eq!(app.query.as_deref(), Some("Root"), "the query is restored");
        assert_eq!(app.cursor, at);
        assert_eq!(app.pane_yak, None);
        assert_eq!(app.focus, Focus::Detail);
        assert_eq!(app.selected_id().as_deref(), Some("a0"));
        assert_eq!((app.detail.line, app.detail.scroll), (line, scroll));
        assert_eq!(
            app.notification.as_deref(),
            Some("\u{2190} Recent \u{b7} a0")
        );
        // `i` goes to s1 again, from outside the list.
        press(&mut app, "i");
        assert_eq!(app.pane_yak.as_deref(), Some("s1"));
        assert_eq!(
            app.notification.as_deref(),
            Some("\u{2192} Recent \u{b7} s1")
        );
        // The view switch in between is the last stop forward.
        press(&mut app, "i");
        assert_eq!(app.active_view().key, "status:hairy");
        press(&mut app, "i");
        assert_eq!(app.notification.as_deref(), Some("no later location"));
    }

    /// yaks-aec7: open a yak from the Recent list, follow a link out of it,
    /// and back out: `o` returns to the yak, then to Recent with the cursor on
    /// its row and the pane closed, and only then to the view before.
    #[test]
    fn backing_out_of_a_detail_returns_to_the_list_it_was_opened_from() {
        let mut app = linked_yaks();
        goto_view(&mut app, "recent");
        cursor_on(&mut app, "a0");
        let row = app.cursor;
        enter_key(&mut app); // open a0 in the pane
        draw(&app, 100, 20);
        tab_key(&mut app);
        enter_key(&mut app); // follow to s1
        assert_eq!(app.selected_id().as_deref(), Some("s1"));
        press(&mut app, "o");
        assert_eq!(app.selected_id().as_deref(), Some("a0"));
        assert_eq!(app.focus, Focus::Detail);
        press(&mut app, "o");
        assert_eq!(app.active_view().key, "recent", "the list it came from");
        assert_eq!(app.focus, Focus::List);
        assert_eq!(app.cursor, row);
        press(&mut app, "o");
        assert_eq!(app.active_view().key, "status:hairy");
        // Opening and closing the same row many times leaves one stop.
        let mut app = linked_yaks();
        let n = app.history.back.len();
        for _ in 0..5 {
            enter_key(&mut app);
            press(&mut app, "h");
        }
        assert_eq!(app.history.back.len(), n + 1);
    }

    #[test]
    fn o_and_i_work_from_the_list_too() {
        let mut app = linked_yaks();
        goto_view(&mut app, "recent");
        assert_eq!(app.focus, Focus::List);
        press(&mut app, "o");
        assert_eq!(app.active_view().key, "status:hairy");
        press(&mut app, "i");
        assert_eq!(app.active_view().key, "recent");
    }

    #[test]
    fn a_view_switch_is_a_jump_and_o_returns_to_the_previous_view() {
        let mut app = sample();
        app.cursor = 1;
        tab_key(&mut app);
        tab_key(&mut app);
        assert_eq!(app.view, 2);
        press(&mut app, "o");
        assert_eq!(app.view, 1);
        press(&mut app, "o");
        assert_eq!(app.view, 0);
        assert_eq!(app.cursor, 1, "the cursor row is where it was left");
        assert_eq!(app.filter.statuses, vec![Status::Hairy]);
        press(&mut app, "o");
        assert_eq!(app.notification.as_deref(), Some("no earlier location"));
        press(&mut app, "ii");
        assert_eq!(app.view, 2);
        // A new jump drops what was forward.
        press(&mut app, "o");
        tab_key(&mut app);
        assert!(app.history.fwd.is_empty());
        // Moving is not a jump.
        let n = app.history.back.len();
        press(&mut app, "jjkk");
        assert_eq!(app.history.back.len(), n);
    }

    #[test]
    fn o_from_a_page_opened_on_a_wiki_row_returns_to_the_row_then_the_view() {
        let (proj, farm) = temp_farm(&[task("yaks-0001", "Search", Status::Hairy, 2, None)]);
        crate::tui::wiki_tab::fixture::write(farm.root());
        let mut app = App::with_farm(farm).unwrap();
        goto_view(&mut app, view::WIKI_KEY);
        press(&mut app, "j");
        let row = app.cursor;
        enter_key(&mut app);
        assert!(app.attach().is_some());
        press(&mut app, "o");
        assert!(app.doc.is_none(), "the page closes");
        assert_eq!(app.focus, Focus::List);
        assert_eq!(app.active_view().key, view::WIKI_KEY);
        assert_eq!(app.cursor, row, "on the row it was opened from");
        press(&mut app, "o");
        assert_eq!(app.active_view().key, "status:hairy");
        // Esc closing the page forgets nothing: it is a stop `o` skips.
        goto_view(&mut app, view::WIKI_KEY);
        enter_key(&mut app);
        esc_key(&mut app);
        press(&mut app, "o");
        assert_eq!(
            app.active_view().key,
            "status:hairy",
            "past the page just closed"
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_page_is_read_again_and_its_cursor_clamped_on_return() {
        let (proj, farm) = temp_farm(&[task("yaks-0001", "Search", Status::Hairy, 2, None)]);
        crate::tui::wiki_tab::fixture::write(farm.root());
        let mut app = App::with_farm(farm).unwrap();
        goto_view(&mut app, view::WIKI_KEY);
        app.detail_page = 20;
        enter_key(&mut app);
        draw(&app, 120, 24);
        let path = app.attach().unwrap().target.clone();
        app.attach_mut().unwrap().nav.line = 6;
        goto_view(&mut app, "status:hairy");
        let file = app.farm.as_ref().unwrap().root().join(&path);
        std::fs::write(&file, "---\ntitle: T\n---\n\nShort.\n").unwrap();
        press(&mut app, "o");
        let v = app.attach().expect("the page is back");
        assert_eq!(v.target, path);
        assert!(v.text.contains("Short."), "re-read from disk");
        let (rows, _) = app.attachment_rows().unwrap();
        assert!(app.attach().unwrap().nav.line < rows.len());
        // A page deleted meanwhile is gone, and the list is what is left.
        std::fs::remove_file(&file).unwrap();
        goto_view(&mut app, "status:hairy");
        app.history.back.push(Location {
            pane: Some(PaneAt::Doc {
                path: path.clone(),
                rev: None,
                from: Focus::List,
                under: None,
                line: 0,
                scroll: 0,
            }),
            ..app.current_location()
        });
        press(&mut app, "o");
        assert!(app.doc.is_none());
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .ends_with(&format!("{path} gone")),
            "{:?}",
            app.notification
        );
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_yak_link_from_a_page_on_the_wiki_tab_shows_the_yak_outside_the_list() {
        let (proj, farm) = temp_farm(&[task("yaks-0001", "Search", Status::Hairy, 2, None)]);
        crate::tui::wiki_tab::fixture::write(farm.root());
        let mut app = App::with_farm(farm).unwrap();
        goto_view(&mut app, view::WIKI_KEY);
        enter_key(&mut app);
        draw(&app, 120, 24);
        app.goto_task("yaks-0001");
        assert!(app.doc.is_none());
        assert_eq!(app.active_view().key, view::WIKI_KEY, "the list stays");
        assert_eq!(app.pane_yak.as_deref(), Some("yaks-0001"));
        assert_eq!(app.selected_id().as_deref(), Some("yaks-0001"));
        assert_eq!(app.surface_ctx(), keymap::Ctx::Detail);
        assert_eq!(app.surface_kind(), Some(tree::RowKind::Yak));
        let frame = draw(&app, 120, 24);
        assert!(
            frame.contains("Task: yaks-0001  not in this list"),
            "{frame}"
        );
        press(&mut app, "o");
        assert!(app.attach().is_some(), "back to the page");
        assert_eq!(app.pane_yak, None);
        let _ = std::fs::remove_dir_all(proj);
    }

    #[test]
    fn a_yak_slaughtered_meanwhile_is_gone_and_one_that_changed_state_moved() {
        let mut app = linked_yaks();
        let recent = app.views.iter().position(|v| v.key == "recent").unwrap();
        app.set_view(recent);
        cursor_on(&mut app, "a0");
        enter_key(&mut app);
        draw(&app, 100, 20);
        tab_key(&mut app);
        enter_key(&mut app); // on to s1, in this list
        assert_eq!(app.selected_id().as_deref(), Some("s1"));
        // a0 is slaughtered while we are away.
        app.all.iter_mut().find(|t| t.id == "a0").unwrap().status = Status::Dead;
        app.touch();
        press(&mut app, "o");
        let note = app.notification.clone().unwrap();
        assert!(note.contains("a0 gone"), "{note}");
        assert!(app.cursor < app.row_count(), "the cursor is clamped");

        // In Hairy, a yak that is now shaving is no longer listed: moved.
        let mut app = linked_yaks();
        cursor_on(&mut app, "a0");
        enter_key(&mut app);
        draw(&app, 100, 20);
        tab_key(&mut app);
        enter_key(&mut app); // s1 is not Hairy: the pane's own yak
        assert_eq!(app.pane_yak.as_deref(), Some("s1"));
        app.all.iter_mut().find(|t| t.id == "a0").unwrap().status = Status::Shaving;
        app.touch();
        press(&mut app, "o");
        let note = app.notification.clone().unwrap();
        assert!(note.contains("a0 moved"), "{note}");
    }

    #[test]
    fn a_deleted_view_falls_back_to_the_subjects_status_view() {
        let mut app = linked_yaks();
        goto_view(&mut app, "recent");
        app.save_current_view("Mine".into());
        let mine = app.view;
        cursor_on(&mut app, "s1");
        goto_view(&mut app, "status:hairy");
        app.delete_view(mine);
        app.overlay = Overlay::None; // the picker it was done from
        press(&mut app, "o");
        assert_eq!(app.active_view().status, Some(Status::Shaving));
        let note = app.notification.clone().unwrap();
        assert!(note.contains("is gone"), "{note}");
    }

    #[test]
    fn the_history_keeps_the_last_hundred() {
        let mut app = sample();
        for _ in 0..150 {
            app.switch_tab(1);
        }
        assert_eq!(app.history.back.len(), CAP);
        for _ in 0..CAP {
            press(&mut app, "o");
        }
        assert_eq!(app.history.fwd.len(), CAP);
        assert!(app.history.back.is_empty());
    }

    #[test]
    fn a_link_to_the_yak_already_shown_pushes_nothing() {
        let mut app = linked_yaks();
        enter_key(&mut app); // opening the pane is one stop (yaks-aec7)
        let n = app.history.back.len();
        app.goto_task("a0");
        assert_eq!(app.history.back.len(), n);
        app.goto_task("nope");
        assert_eq!(app.history.back.len(), n);
        assert_eq!(app.notification.as_deref(), Some("nope not found"));
    }
}
