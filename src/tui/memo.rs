//! Memoized view-model reads (yaks-bc30): the visible row list and the wrapped
//! detail lines are rebuilt only when an input they read has changed, not on
//! every `App::rows()` / `App::detail_dlines()` call (a keypress asks a dozen
//! times; each rebuild costs 1.5–3.5 ms on a 450-yak farm, see yaks-48d6).
//!
//! # Staleness rule
//!
//! Each cache stores the inputs its value was built from (its *key*) and is
//! used only while the live inputs still equal the key. Most inputs are small
//! `App` fields compared directly (view, live filter, collapsed set, family
//! scope, starred set, detail width, edit slot, the clock's minute), so no
//! mutation point can forget to invalidate them. The two big inputs, the farm
//! tasks (`App::all`) and the sheds snapshot, are compared by a generation
//! counter, `App::data_gen`: **anything that replaces `all` or
//! `sheds.snapshot` must call [`App::touch`]**. Under `cfg(test)` every cache
//! hit is checked against a fresh build, so a missed `touch` fails a test
//! instead of showing stale output.
//!
//! Neither cache holds a borrow: a cached row names its yak by [`Loc`] (an
//! index), so `App::rows` can hand out `Row<'_>` borrowing `self` as before.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::*;

/// Where a cached row's item lives: a yak's index in this farm's tasks, a
/// document's index in `App.wiki.entries`, or a shed's index in the sheds
/// snapshot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Loc {
    Home(usize),
    Doc(usize),
    Shed(usize),
}

/// A `tree::Row` with its `Item` replaced by a [`Loc`].
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct CachedRow {
    pub loc: Loc,
    pub depth: u16,
    pub ghost: bool,
    pub has_children: bool,
    pub collapsed: bool,
    pub hidden: usize,
}

/// Everything `App::build_rows` reads.
pub(super) struct RowsKey {
    data_gen: u64,
    view_key: String,
    sort_by: Option<view::SortField>,
    sort_dir: view::SortDir,
    limit: Option<usize>,
    filter: FilterSpec,
    collapsed: HashSet<String>,
    family_scope: HashMap<String, view::FamilyScope>,
    working_set: Vec<String>,
}

impl RowsKey {
    pub(super) fn of(app: &App) -> RowsKey {
        let v = app.active_view();
        RowsKey {
            data_gen: app.data_gen,
            view_key: v.key.clone(),
            sort_by: v.sort_by,
            sort_dir: v.sort_dir,
            limit: v.limit,
            filter: app.filter.clone(),
            collapsed: app.collapsed.clone(),
            family_scope: app.family_scope.clone(),
            working_set: app.working_set.clone(),
        }
    }

    /// `self == RowsKey::of(app)` without cloning the live inputs.
    pub(super) fn is_current(&self, app: &App) -> bool {
        let v = app.active_view();
        self.data_gen == app.data_gen
            && self.view_key == v.key
            && self.sort_by == v.sort_by
            && self.sort_dir == v.sort_dir
            && self.limit == v.limit
            && self.filter == app.filter
            && self.collapsed == app.collapsed
            && self.family_scope == app.family_scope
            && self.working_set == app.working_set
    }
}

pub(super) struct RowsCache {
    pub key: RowsKey,
    pub rows: Rc<Vec<CachedRow>>,
}

/// Everything `App::build_dlines` reads besides the rows key: which yak is
/// selected, the layout width, the in-place editor's slot and the clock.
#[derive(PartialEq)]
pub(super) struct DlinesKey {
    pub data_gen: u64,
    pub loc: Loc,
    pub width: usize,
    pub slot: Option<detail::EditSlot>,
    /// Inline images: the cell size and tallest image that sized the layout
    /// (`None` when off), so a resize or a new picker rebuilds it.
    pub images: Option<((u16, u16), u16)>,
    /// Relative dates ("5m ago") are good to the minute.
    pub minute: i64,
}

pub(super) struct DlinesCache {
    pub key: DlinesKey,
    pub lines: Rc<Vec<detail::DLine>>,
}

impl App {
    /// Record that `all`, the wiki entries (or the sheds snapshot) were replaced, so the memoized
    /// rows and detail lines are rebuilt. See the module docs.
    pub(crate) fn touch(&mut self) {
        self.data_gen += 1;
    }

    /// Index of `item` in `all`, the wiki entries or the sheds: the inverse of
    /// [`App::resolve_item`]. It must be borrowed from one of them.
    pub(super) fn locate(&self, item: tree::Item<'_>) -> Loc {
        fn within<T>(base: &[T], x: &T) -> Option<usize> {
            let size = std::mem::size_of::<T>();
            let (addr, start) = (x as *const T as usize, base.as_ptr() as usize);
            (addr >= start && addr < start + base.len() * size).then(|| (addr - start) / size)
        }
        match item {
            tree::Item::Yak(t) => match within(&self.all, t) {
                Some(i) => Loc::Home(i),
                None => unreachable!("row task {} is not a task of this farm", t.id),
            },
            tree::Item::Doc(e) => {
                match within(self.wiki.entries.as_deref().unwrap_or_default(), e) {
                    Some(i) => Loc::Doc(i),
                    None => unreachable!("row document {} is not a wiki entry", e.path),
                }
            }
            tree::Item::Shed(s) => {
                let sheds = self.sheds.snapshot.as_ref().map_or(&[][..], |n| n.sheds());
                match within(sheds, s) {
                    Some(i) => Loc::Shed(i),
                    None => unreachable!("row shed {} is not in the snapshot", s.name),
                }
            }
        }
    }

    /// The item at `loc`: the inverse of [`App::locate`].
    pub(super) fn resolve_item(&self, loc: Loc) -> tree::Item<'_> {
        match loc {
            Loc::Home(i) => tree::Item::Yak(&self.all[i]),
            Loc::Doc(i) => tree::Item::Doc(&self.wiki.entries.as_deref().unwrap_or_default()[i]),
            Loc::Shed(i) => {
                let snap = self.sheds.snapshot.as_ref();
                tree::Item::Shed(&snap.expect("a shed row has a snapshot").sheds()[i])
            }
        }
    }

    /// The memoized visible rows (see the module docs).
    pub(super) fn cached_rows(&self) -> Rc<Vec<CachedRow>> {
        if let Some(c) = &*self.rows_cache.borrow() {
            if c.key.is_current(self) {
                #[cfg(test)]
                assert_eq!(*c.rows, self.build_cached_rows(), "stale rows cache");
                return c.rows.clone();
            }
        }
        #[cfg(test)]
        viewmodel::ROWS_CALLS.with(|c| c.set(c.get() + 1));
        let rows = Rc::new(self.build_cached_rows());
        *self.rows_cache.borrow_mut() = Some(RowsCache {
            key: RowsKey::of(self),
            rows: rows.clone(),
        });
        rows
    }

    fn build_cached_rows(&self) -> Vec<CachedRow> {
        self.build_rows()
            .into_iter()
            .map(|r| CachedRow {
                loc: self.locate(r.item),
                depth: r.depth,
                ghost: r.ghost,
                has_children: r.has_children,
                collapsed: r.collapsed,
                hidden: r.hidden,
            })
            .collect()
    }
}

#[cfg(test)]
thread_local! {
    /// How many times the detail lines were actually built on this thread
    /// (cache misses), so tests can pin the rebuild count of a keypress.
    pub(crate) static DLINES_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
