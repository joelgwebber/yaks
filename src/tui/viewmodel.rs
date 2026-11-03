//! `App`'s view-model / read-model layer: the farm-derived row lists (tree +
//! flat + working-set), selection and cursor, per-view family-scope, view/tab
//! management, and the reload paths. Split out of `tui.rs` (yaks-b1cc / b1cc/6)
//! as an `impl App` block. `use super::*` inherits everything `tui.rs` has in
//! scope; the methods are `pub(crate)` so the rest of the crate can call them.

use super::*;

impl App {
    /// Persist the (rebuildable) UI state — collapsed rows and family-scope
    /// overrides — to the per-user cache.
    pub(crate) fn save_ui_state(&self) {
        if let Some(h) = &self.farm {
            cache::save(
                h.root(),
                &cache::UiState {
                    collapsed: self.collapsed.clone(),
                    family: self.family_scope.clone(),
                },
            );
        }
    }

    /// The family scope in effect for `v`: its persisted override, else the global
    /// default. Only meaningful for tree views.
    pub(crate) fn resolved_family_scope(&self, v: &view::View) -> view::FamilyScope {
        self.family_scope
            .get(&v.key)
            .copied()
            .unwrap_or(view::FamilyScope::DEFAULT)
    }

    /// Cycle the active view's family scope (the `h` key): auto -> lone ->
    /// remaining -> all -> auto. Flat/working-set views have no tree, so it's a
    /// no-op there with a hint.
    pub(crate) fn cycle_family_scope(&mut self) {
        let v = self.active_view();
        if v.is_flat() || v.key == "working-set" || v.key == view::SHEDS_KEY {
            self.notification = Some("family scope applies to tree views".into());
            return;
        }
        let key = v.key.clone();
        let next = view::FamilyScope::cycle(self.family_scope.get(&key).copied());
        match next {
            Some(s) => {
                self.family_scope.insert(key, s);
            }
            None => {
                self.family_scope.remove(&key);
            }
        }
        self.save_ui_state();
        self.notification = Some(match next {
            Some(s) => format!("family: {}", s.as_str()),
            None => format!("family: ~{}", view::FamilyScope::DEFAULT.as_str()),
        });
    }

    /// The create-form herd picker's choices: the farm's declared herds unioned
    /// with the id prefixes actually present, sorted and de-duplicated.
    pub(crate) fn herd_choices(&self) -> Vec<String> {
        let mut set: std::collections::BTreeSet<String> =
            self.config_herds.iter().cloned().collect();
        for t in &self.all {
            // A herd is a real id prefix (`prefix-tail`); ids without a `-`
            // (only test fixtures) don't name a herd.
            if let Some((prefix, _)) = t.id.split_once('-') {
                set.insert(prefix.to_string());
            }
        }
        set.into_iter().collect()
    }

    /// True when the farm spans more than one herd (declared or present) — what
    /// activates the per-herd id colour and the create-form herd picker.
    pub(crate) fn is_multi_herd(&self) -> bool {
        self.herd_choices().len() > 1
    }

    /// Re-query the farm view after a mutation and keep the cursor in range.
    pub(crate) fn reload(&mut self) {
        if let Some(h) = &self.farm {
            if let Ok(all) = h.list(FilterSpec::default(), true) {
                self.all = all;
                self.touch();
            }
        }
        self.refresh_farm_sync();
        self.reload_wiki();
        let len = self.row_count();
        self.cursor = if len == 0 {
            0
        } else {
            self.cursor.min(len - 1)
        };
    }

    /// Re-read the farm from disk (external change) while keeping the cursor on
    /// the same task by id. Silent: it must not clobber a mutation's own
    /// notification, and the event loop only calls it while idle (no overlay).
    pub(crate) fn reload_preserving_selection(&mut self) {
        let sel = self.selected_id();
        if let Some(h) = &self.farm {
            if let Ok(all) = h.list(FilterSpec::default(), true) {
                self.all = all;
                self.touch();
            }
        }
        self.refresh_farm_sync();
        self.reload_wiki();
        match sel.and_then(|id| {
            self.rows()
                .iter()
                .position(|r| r.yak().is_some_and(|t| t.id == id))
        }) {
            Some(pos) => self.cursor = pos,
            None => self.clamp_cursor(),
        }
    }

    pub(crate) fn active_view(&self) -> &view::View {
        &self.views[self.view]
    }

    /// Whether a yak passes the query (all of them without one).
    pub(crate) fn query_keeps(&self, t: &Task) -> bool {
        self.query
            .as_deref()
            .is_none_or(|q| tree::query_matches(tree::Item::Yak(t), q))
    }

    /// Count for a view: non-ghost tree rows of its own spec, or working-set
    /// membership, of what the query keeps. Independent of the live facets (a
    /// stable per-view size).
    pub(crate) fn view_count(&self, v: &view::View) -> usize {
        if v.key == "working-set" {
            return self
                .working_set
                .iter()
                .filter_map(|id| self.task(id))
                .filter(|t| self.query_keeps(t))
                .count();
        }
        // The Sheds tab counts busy sheds, not yaks.
        if v.key == view::SHEDS_KEY {
            return self.busy_sheds_count();
        }
        // The Wiki tab counts pages, not yaks.
        if v.key == view::WIKI_KEY {
            return self.wiki_count();
        }
        // Flat views (Recent/Inbox/custom sorted) count via the flat predicate
        // so facets the tree ignores for pruning — notably `needs_only` — are
        // honored (an Inbox tab shows the awaiting-a-human count, not the farm
        // size). Mirrors `flat_rows`: exclude dead unless the spec asks, respect
        // the row cap.
        if v.is_flat() {
            let resolved = filter::resolved_ids(&self.all);
            let want_dead = v.spec.statuses.contains(&Status::Dead);
            let n = self
                .all
                .iter()
                .filter(|t| {
                    (want_dead || t.status != Status::Dead)
                        && v.spec.matches(t, &resolved)
                        && self.query_keeps(t)
                })
                .count();
            let n = v.limit.map_or(n, |l| n.min(l));
            // The All view also holds every document.
            return if v.key == view::ALL_KEY {
                n + self.docs_matching_query().count()
            } else {
                n
            };
        }
        tree::build(
            &self.all,
            &v.spec,
            self.query.as_deref(),
            self.resolved_family_scope(v),
        )
        .iter()
        .filter(|r| !r.ghost)
        .count()
    }

    /// Hairy tasks with at least one unresolved dependency (the blocked set;
    /// Python marks these with a magenta `*`).
    pub(crate) fn blocked_ids(&self) -> HashSet<String> {
        let resolved = filter::resolved_ids(&self.all);
        self.all
            .iter()
            .filter(|t| {
                t.status == Status::Hairy
                    && t.depends_on.iter().any(|d| !resolved.contains(d.as_str()))
            })
            .map(|t| t.id.clone())
            .collect()
    }

    /// Indices of pinned views — exactly the tab strip, in order.
    pub(crate) fn pinned_indices(&self) -> Vec<usize> {
        (0..self.views.len())
            .filter(|&i| self.views[i].pinned)
            .collect()
    }

    /// Rows the starred working set resolves to, in star order (flat).
    pub(crate) fn working_set_rows(&self) -> Vec<tree::Row<'_>> {
        self.working_set
            .iter()
            .filter_map(|id| self.task(id))
            .filter(|t| self.query_keeps(t))
            .map(tree::Row::leaf)
            .collect()
    }

    /// Flat, sorted rows for a sorted view (Recent / custom sorted).
    pub(crate) fn flat_rows(&self) -> Vec<tree::Row<'_>> {
        let v = self.active_view();
        let resolved = filter::resolved_ids(&self.all);
        // Flat views (Recent/custom) span all statuses, so exclude dead unless
        // the live filter explicitly asks for it (fe00): the model now carries
        // dead, but a working list shouldn't surface slaughtered yaks by default.
        let want_dead = self.filter.statuses.contains(&Status::Dead);
        let mut matched: Vec<&Task> = self
            .all
            .iter()
            .filter(|t| {
                (want_dead || t.status != Status::Dead)
                    && self.filter.matches(t, &resolved)
                    && self.query_keeps(t)
            })
            .collect();
        let sort_by = v.sort_by.unwrap_or(view::SortField::Updated);
        matched.sort_by(|a, b| sort_key(a, sort_by).cmp(&sort_key(b, sort_by)));
        if v.sort_dir == view::SortDir::Desc {
            matched.reverse();
        }
        if let Some(lim) = v.limit {
            matched.truncate(lim);
        }
        let mut rows: Vec<tree::Row<'_>> = matched.into_iter().map(tree::Row::leaf).collect();
        // The All view lists the documents after the yaks, unless a facet was
        // narrowed: a document has none of a yak's, so it would not match.
        if v.key == view::ALL_KEY && self.facets_are_the_views() {
            rows.extend(self.wiki_rows().into_iter().map(tree::Row::doc));
        }
        rows
    }

    /// Whether the live filter still has the active view's facets, apart from
    /// its search (which documents match too). The All view shows documents
    /// only then: a type, priority, status or readiness facet describes yaks.
    fn facets_are_the_views(&self) -> bool {
        let (mut live, mut saved) = (self.filter.clone(), self.active_view().spec.clone());
        live.search = None;
        saved.search = None;
        spec_eq(&live, &saved)
            && live.needs_only == saved.needs_only
            && live.run_on == saved.run_on
            && same_set(&live.herds, &saved.herds)
    }

    /// Visible rows for the active view (dispatch: working-set / flat / tree),
    /// memoized (`memo.rs`, yaks-bc30): the tree, filters and shed overlay are
    /// rebuilt (about 1.5 ms on a 450-yak farm) only when an input changed, so
    /// this is cheap enough for a keypress's several asks. Per-row loops should
    /// still hoist it, and ask [`App::row_count`] / [`App::selected`] when that
    /// is all they need.
    pub(crate) fn rows(&self) -> Vec<tree::Row<'_>> {
        self.cached_rows()
            .iter()
            .map(|c| tree::Row {
                item: self.resolve_item(c.loc),
                depth: c.depth,
                ghost: c.ghost,
                has_children: c.has_children,
                collapsed: c.collapsed,
                hidden: c.hidden,
            })
            .collect()
    }

    /// How many rows [`App::rows`] has.
    pub(crate) fn row_count(&self) -> usize {
        self.cached_rows().len()
    }

    /// The uncached [`App::rows`]: the memo's one source of truth.
    pub(super) fn build_rows(&self) -> Vec<tree::Row<'_>> {
        let v = self.active_view();
        if v.key == "working-set" {
            return self.working_set_rows();
        }
        if v.key == view::SHEDS_KEY {
            return tree::apply_collapse(self.shed_rows(), &self.collapsed);
        }
        if v.key == view::WIKI_KEY {
            return self.wiki_rows().into_iter().map(tree::Row::doc).collect();
        }
        if v.is_flat() {
            return self.flat_rows();
        }
        let scope = self.resolved_family_scope(v);
        let flat = tree::build(&self.all, &self.filter, self.query.as_deref(), scope);
        tree::apply_collapse(flat, &self.collapsed)
    }

    /// Keep the cursor within the current row count (after a filter change).
    pub(crate) fn clamp_cursor(&mut self) {
        let len = self.row_count();
        self.cursor = if len == 0 {
            0
        } else {
            self.cursor.min(len - 1)
        };
    }

    /// The row under the cursor, whatever it is.
    pub(crate) fn selected_row(&self) -> Option<tree::Item<'_>> {
        let row = *self.cached_rows().get(self.cursor)?;
        Some(self.resolve_item(row.loc))
    }

    /// The selected yak: `None` on an empty list and on any other kind of row,
    /// so every yak command finds nothing there.
    pub(crate) fn selected(&self) -> Option<&Task> {
        match self.selected_row()? {
            tree::Item::Yak(t) => Some(t),
            _ => None,
        }
    }

    /// The selected document (a wiki page or source), if the row is one.
    pub(crate) fn selected_doc(&self) -> Option<&wiki_tab::WikiEntry> {
        match self.selected_row()? {
            tree::Item::Doc(e) => Some(e),
            _ => None,
        }
    }

    /// The kind of row keys act on: the selected row's, else what the active
    /// view lists (so an empty Wiki still takes `N` and `r`, an empty Sheds `r`).
    pub(crate) fn row_kind(&self) -> tree::RowKind {
        match self.selected_row() {
            Some(item) => item.kind(),
            None if self.on_wiki_tab() => tree::RowKind::Doc,
            None if self.on_sheds_tab() => tree::RowKind::Shed,
            None => tree::RowKind::Yak,
        }
    }

    pub(crate) fn selected_id(&self) -> Option<String> {
        self.selected().map(|t| t.id.clone())
    }

    pub(crate) fn task(&self, id: &str) -> Option<&Task> {
        self.all.iter().find(|t| t.id == id)
    }

    /// Activate view `i`: load its saved spec into the live filter and reset.
    pub(crate) fn set_view(&mut self, i: usize) {
        self.view = i;
        self.filter = clone_spec(&self.views[i].spec);
        self.cursor = 0;
        self.detail_scroll = 0;
        self.focus = Focus::List;
        self.clamp_cursor();
    }

    /// Cycle through the pinned views (the visible tabs) only.
    pub(crate) fn switch_tab(&mut self, delta: i32) {
        let pinned = self.pinned_indices();
        if pinned.is_empty() {
            return;
        }
        let cur = pinned.iter().position(|&i| i == self.view).unwrap_or(0);
        let n = pinned.len() as i32;
        let next = pinned[(((cur as i32 + delta) % n + n) % n) as usize];
        self.set_view(next);
    }

    /// True when the live filter has been edited away from the active view spec.
    pub(crate) fn is_view_modified(&self) -> bool {
        !spec_eq(&self.filter, &self.active_view().spec)
    }

    /// Esc: revert the live filter to the active view's saved spec.
    pub(crate) fn revert_filter_to_view(&mut self) {
        if self.is_view_modified() {
            let i = self.view;
            self.set_view(i);
        }
    }
}

#[cfg(test)]
thread_local! {
    /// How many times [`App::rows`] ran on this thread, so tests can pin the
    /// rebuild count of a keypress or a frame (yaks-48d6).
    pub(crate) static ROWS_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
