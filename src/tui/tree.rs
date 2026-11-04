//! Task tree building + collapse for the list pane.
//!
//! `build` returns a flat, pre-ordered list of rows for one tab. The tab's
//! status set is the *anchor*; the tab also pulls in each anchor's family
//! (ancestors walked up, descendants walked down — any status) so the tree
//! stays rooted. Non-anchor family rows are marked `ghost` (rendered dimmed).
//! Content-filter re-coloring (bright matches vs dim context) is deferred to
//! the filter slice; this is the no-filter path.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::filter::{self, FilterSpec};
use crate::model::{Status, Task};
use crate::sheds::Shed;
use crate::tui::doks_tab::DoksEntry;
use crate::tui::view::FamilyScope;

/// What a list row is (the design's "rows": /doks/design/tui-model.md#1-rows-and-views).
/// A yak, a document (a dok or a saved source) or a shed (an actor on
/// the Sheds view, with the yaks it touched below it).
#[derive(Clone, Copy)]
pub enum Item<'a> {
    Yak(&'a Task),
    Doc(&'a DoksEntry),
    Shed(&'a Shed),
}

/// An [`Item`]'s kind, without the data: what a key binding names to apply to
/// one kind of row (`keymap::Binding::kind`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RowKind {
    Yak,
    Doc,
    Shed,
}

impl Item<'_> {
    pub fn kind(&self) -> RowKind {
        match self {
            Item::Yak(_) => RowKind::Yak,
            Item::Doc(_) => RowKind::Doc,
            Item::Shed(_) => RowKind::Shed,
        }
    }
}

/// Whether `item` matches the query `q` (case-insensitive substring), the one
/// rule every view applies (design decision 6: what `yaks find` searches):
///
/// - a yak: its id, title and body (the notes are in the body);
/// - a document: its title, path, kind and tags, and its text (a dok's file,
///   a source's `source.md` and extracted text: [`DoksEntry::text`]);
/// - a shed: its name. A shed also stays when a yak of it matches; that is the
///   tree rule, applied where the shed's rows are built.
pub fn query_matches(item: Item<'_>, q: &str) -> bool {
    let q = q.to_lowercase();
    let has = |s: &str| s.to_lowercase().contains(&q);
    match item {
        Item::Yak(t) => has(&t.id) || has(&t.title) || has(&t.body),
        Item::Doc(e) => {
            has(&e.title)
                || has(&e.path)
                || has(&e.kind)
                || e.tags.iter().any(|t| has(t))
                || e.text.contains(&q)
        }
        Item::Shed(s) => has(&s.name),
    }
}

/// The key of a shed in the `collapsed` set: it cannot collide with a yak id
/// (ids have no colon).
pub fn shed_collapse_key(name: &str) -> String {
    format!("shed:{name}")
}

/// The key of a quiet shed (idle, nothing asked) in the `collapsed` set. A
/// quiet shed starts collapsed (yaks-58f0), so for it the key in the set means
/// *expanded*; a key of its own, so a saved choice keeps its meaning when the
/// shed turns active or quiet again.
pub fn shed_open_key(name: &str) -> String {
    format!("shed-open:{name}")
}

pub struct Row<'a> {
    pub item: Item<'a>,
    pub depth: u16,
    pub ghost: bool,
    pub has_children: bool,
    /// Set by `apply_collapse` when this row is a collapsed parent.
    pub collapsed: bool,
    /// Number of descendant rows hidden beneath a collapsed parent.
    pub hidden: usize,
}

impl<'a> Row<'a> {
    /// The yak this row is, or `None` for any other kind of row.
    pub fn yak(&self) -> Option<&'a Task> {
        match self.item {
            Item::Yak(t) => Some(t),
            _ => None,
        }
    }

    /// What this row is called in the `collapsed` set, if it can collapse: a
    /// yak by its id, a shed by [`shed_collapse_key`]; a document never does.
    pub fn collapse_key(&self) -> Option<String> {
        match self.item {
            Item::Yak(t) => Some(t.id.clone()),
            Item::Shed(s) if s.quiet() => Some(shed_open_key(&s.name)),
            Item::Shed(s) => Some(shed_collapse_key(&s.name)),
            Item::Doc(_) => None,
        }
    }

    /// Whether this row is collapsed when its key is *not* in the set: a
    /// quiet shed. Toggling flips the key's presence either way.
    pub fn collapsed_by_default(&self) -> bool {
        matches!(self.item, Item::Shed(s) if s.quiet())
    }

    /// A shed row, with `has_children` set when it has yaks below it.
    pub fn shed(shed: &'a Shed, has_children: bool) -> Row<'a> {
        Row {
            has_children,
            ..Row::of(Item::Shed(shed))
        }
    }

    /// A non-ghost, childless yak row below a shed. Depth 2, not 1: a shed's
    /// name starts three columns in (after its `⏳` mark), so one level of
    /// indent would line its yaks' ids up under the names (yaks-efc0).
    pub fn shed_yak(task: &'a Task) -> Row<'a> {
        Row {
            depth: 2,
            ..Row::of(Item::Yak(task))
        }
    }

    /// A depth-0, non-ghost, childless row (for flat / working-set views).
    pub fn leaf(task: &'a Task) -> Row<'a> {
        Row::of(Item::Yak(task))
    }

    /// A depth-0, non-ghost, childless row for a document.
    pub fn doc(entry: &'a DoksEntry) -> Row<'a> {
        Row::of(Item::Doc(entry))
    }

    fn of(item: Item<'a>) -> Row<'a> {
        Row {
            item,
            depth: 0,
            ghost: false,
            has_children: false,
            collapsed: false,
            hidden: 0,
        }
    }
}

fn child_rank(s: Status) -> u8 {
    match s {
        Status::Shaving => 0,
        Status::Shorn => 2,
        _ => 1,
    }
}

fn cmp_child(a: &Task, b: &Task) -> Ordering {
    child_rank(a.status)
        .cmp(&child_rank(b.status))
        .then(a.priority.cmp(&b.priority))
        .then(
            a.created
                .as_deref()
                .unwrap_or("")
                .cmp(b.created.as_deref().unwrap_or("")),
        )
        .then(a.id.cmp(&b.id))
}

/// Build the pre-ordered tree rows for `tab`'s scope over `all`, honoring the
/// live `spec`. With no content filter, the tab's own tasks are the focus and
/// their family is dimmed context. With a content filter, matches anywhere in
/// that family become the focus and non-matching ancestors are dimmed to root
/// them; everything else is pruned. Mirrors Python `tree.build_tree`.
/// The global `query`, when set, is one more content filter: a yak must match
/// it as well as the spec.
pub fn build<'a>(
    all: &'a [Task],
    spec: &FilterSpec,
    query: Option<&str>,
    farm: FamilyScope,
) -> Vec<Row<'a>> {
    let by_id: HashMap<&str, &Task> = all.iter().map(|t| (t.id.as_str(), t)).collect();

    let mut children_of: HashMap<&str, Vec<&str>> = HashMap::new();
    for t in all {
        if let Some(p) = &t.parent {
            if by_id.contains_key(p.as_str()) {
                children_of
                    .entry(p.as_str())
                    .or_default()
                    .push(t.id.as_str());
            }
        }
    }

    // Effective status scope: the spec's statuses, else all non-dead statuses
    // (a bare/custom view with no status axis spans the whole farm).
    let eff: Vec<Status> = if spec.statuses.is_empty() {
        vec![Status::Hairy, Status::Shaving, Status::Shorn]
    } else {
        spec.statuses.clone()
    };
    let anchors: HashSet<&str> = all
        .iter()
        .filter(|t| eff.contains(&t.status))
        .map(|t| t.id.as_str())
        .collect();

    let ancestors_of = |start: &str| -> Vec<&'a str> {
        let mut out = Vec::new();
        let mut seen: HashSet<&str> = HashSet::new();
        let mut pid = by_id.get(start).and_then(|t| t.parent.as_deref());
        while let Some(p) = pid {
            if !seen.insert(p) {
                break;
            }
            if let Some(t) = by_id.get(p) {
                out.push(t.id.as_str());
            }
            pid = by_id.get(p).and_then(|t| t.parent.as_deref());
        }
        out
    };

    // Seeds (the bright focus): the anchors, narrowed to the content matches
    // (spec and global query) when a filter is active. A match must also be in
    // the status scope: a herd or a search on the Hairy view must not bring
    // shorn and dead descendants in bright (yaks-b608); family scope below
    // decides which descendants come along, dimmed.
    let focus: HashSet<&str> = if spec.content_active() || query.is_some() {
        let resolved = filter::resolved_ids(all);
        anchors
            .iter()
            .copied()
            .filter(|&tid| {
                spec.matches(by_id[tid], &resolved)
                    && query.is_none_or(|q| query_matches(Item::Yak(by_id[tid]), q))
            })
            .collect()
    } else {
        anchors.clone()
    };

    // members = seeds + ancestors (always, to root the chain) + family-scoped
    // descendants. One rule for both the filtered and unfiltered paths, so
    // turning on a filter no longer silently drops descendant context.
    let mut members: HashSet<&str> = focus.clone();
    for &s in &focus {
        members.extend(ancestors_of(s));
    }
    members.extend(family_descendants(&focus, farm, &children_of, &by_id));

    // roots = members whose parent is not itself a member.
    let mut roots: Vec<&str> = members
        .iter()
        .copied()
        .filter(|id| match by_id[id].parent.as_deref() {
            Some(p) => !members.contains(p),
            None => true,
        })
        .collect();
    if eff == [Status::Shorn] {
        // Recency-first for the (unbounded) shorn view.
        roots.sort_by(|&a, &b| {
            by_id[b]
                .updated
                .as_deref()
                .unwrap_or("")
                .cmp(by_id[a].updated.as_deref().unwrap_or(""))
                .then(a.cmp(b))
        });
    } else {
        roots.sort_by(|&a, &b| by_id[a].priority.cmp(&by_id[b].priority).then(a.cmp(b)));
    }

    let mut out = Vec::new();
    for r in roots {
        flatten(r, 0, &by_id, &children_of, &members, &focus, &mut out);
    }
    out
}

/// The descendant rows a tree view pulls in under its `seeds`, per family scope.
/// `Lone` yields nothing; `All` yields the full descendant closure; `Remaining`
/// keeps descendants with open work (hairy/shaving) plus the completed nodes
/// that connect them back to a seed, dropping fully-shorn subtrees.
fn family_descendants<'a>(
    seeds: &HashSet<&'a str>,
    farm: FamilyScope,
    children_of: &HashMap<&'a str, Vec<&'a str>>,
    by_id: &HashMap<&'a str, &'a Task>,
) -> HashSet<&'a str> {
    if farm == FamilyScope::Lone {
        return HashSet::new();
    }
    // Full descendant closure of the seeds (any status).
    let mut full: HashSet<&str> = HashSet::new();
    let mut stack: Vec<&str> = seeds.iter().copied().collect();
    while let Some(cur) = stack.pop() {
        if let Some(kids) = children_of.get(cur) {
            for &c in kids {
                if !seeds.contains(c) && full.insert(c) {
                    stack.push(c);
                }
            }
        }
    }
    if farm == FamilyScope::All {
        return full;
    }
    // Remaining: start from the open descendants, then walk each one up toward
    // its seed, keeping the completed nodes in between as (dim) connectors.
    let is_open = |id: &str| matches!(by_id[id].status, Status::Hairy | Status::Shaving);
    let mut keep: HashSet<&str> = full.iter().copied().filter(|&d| is_open(d)).collect();
    let open: Vec<&str> = keep.iter().copied().collect();
    for od in open {
        let mut pid = by_id[od].parent.as_deref();
        while let Some(p) = pid {
            if seeds.contains(p) || !full.contains(p) || !keep.insert(p) {
                break;
            }
            pid = by_id[p].parent.as_deref();
        }
    }
    keep
}

fn flatten<'a>(
    id: &'a str,
    depth: u16,
    by_id: &HashMap<&'a str, &'a Task>,
    children_of: &HashMap<&'a str, Vec<&'a str>>,
    members: &HashSet<&'a str>,
    focus: &HashSet<&str>,
    out: &mut Vec<Row<'a>>,
) {
    let mut kids: Vec<&str> = children_of
        .get(id)
        .map(|v| v.iter().copied().filter(|c| members.contains(c)).collect())
        .unwrap_or_default();
    kids.sort_by(|&a, &b| cmp_child(by_id[a], by_id[b]));
    out.push(Row {
        item: Item::Yak(by_id[id]),
        depth,
        ghost: !focus.contains(id),
        has_children: !kids.is_empty(),
        collapsed: false,
        hidden: 0,
    });
    for c in kids {
        flatten(c, depth + 1, by_id, children_of, members, focus, out);
    }
}

/// Drop descendants of collapsed ids; annotate collapsed parents with their
/// hidden-row count. Empty `collapsed` returns the input unchanged.
pub fn apply_collapse<'a>(flat: Vec<Row<'a>>, collapsed: &HashSet<String>) -> Vec<Row<'a>> {
    if collapsed.is_empty() && !flat.iter().any(|r| r.collapsed_by_default()) {
        return flat;
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut visible: Vec<Row<'a>> = Vec::new();
    let mut hide_stack: Vec<(String, u16)> = Vec::new();
    for mut row in flat {
        let depth = row.depth;
        // A row with no key (a document) never nests.
        let key = row.collapse_key();
        let in_set = key.as_ref().is_some_and(|k| collapsed.contains(k));
        let is_collapsed = row.has_children && (in_set != row.collapsed_by_default());
        while hide_stack.last().is_some_and(|(_, d)| depth <= *d) {
            hide_stack.pop();
        }
        if hide_stack.is_empty() {
            row.collapsed = is_collapsed;
            visible.push(row);
        } else {
            for (cid, _) in &hide_stack {
                *counts.entry(cid.clone()).or_insert(0) += 1;
            }
        }
        if is_collapsed {
            hide_stack.push((key.unwrap_or_default(), depth));
        }
    }
    for row in &mut visible {
        if row.collapsed {
            let key = row.collapse_key().unwrap_or_default();
            row.hidden = counts.get(&key).copied().unwrap_or(0);
        }
    }
    visible
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(id: &str, status: Status, parent: Option<&str>) -> Task {
        Task {
            id: id.into(),
            title: id.into(),
            kind: "task".into(),
            priority: 3,
            status,
            created: None,
            updated: None,
            parent: parent.map(String::from),
            tags: vec![],
            depends_on: vec![],
            source: None,
            prs: Vec::new(),
            needs: None,
            verify: None,
            extra: Default::default(),
            body: String::new(),
        }
    }

    #[test]
    fn collapse_hides_descendants_and_counts() {
        let all = vec![
            t("a", Status::Hairy, None),
            t("a1", Status::Hairy, Some("a")),
            t("a2", Status::Hairy, Some("a")),
        ];
        let spec = FilterSpec {
            statuses: vec![Status::Hairy],
            ..Default::default()
        };
        let flat = build(&all, &spec, None, FamilyScope::All);
        assert_eq!(flat.len(), 3);
        let collapsed: HashSet<String> = ["a".to_string()].into_iter().collect();
        let vis = apply_collapse(flat, &collapsed);
        assert_eq!(vis.len(), 1);
        assert!(vis[0].collapsed);
        assert_eq!(vis[0].hidden, 2);
    }

    #[test]
    fn content_filter_focuses_matches_and_dims_ancestors() {
        // a > a1 > a2 ; only a2's title matches. a2 is focus (bright), a + a1
        // come along as dimmed ancestors, and unrelated b is pruned.
        let mut a = t("a", Status::Hairy, None);
        a.title = "root alpha".into();
        let mut a1 = t("a1", Status::Hairy, Some("a"));
        a1.title = "mid".into();
        let mut a2 = t("a2", Status::Hairy, Some("a1"));
        a2.title = "needle here".into();
        let b = t("b", Status::Hairy, None);
        let all = vec![a, a1, a2, b];

        let spec = FilterSpec {
            statuses: vec![Status::Hairy],
            search: Some("needle".into()),
            ..Default::default()
        };
        let flat = build(&all, &spec, None, FamilyScope::All);
        let ids: Vec<&str> = flat.iter().map(|r| r.yak().unwrap().id.as_str()).collect();
        assert_eq!(ids, vec!["a", "a1", "a2"]); // b pruned
        let ghost = |id: &str| {
            flat.iter()
                .find(|r| r.yak().unwrap().id == id)
                .unwrap()
                .ghost
        };
        assert!(ghost("a") && ghost("a1"), "non-matching ancestors dim");
        assert!(!ghost("a2"), "the match is the focus");
    }

    #[test]
    fn family_scope_governs_descendants() {
        // a(hairy anchor) with a shorn leaf b and a shaving child c. On the
        // Hairy view b and c are non-anchor descendants, governed by the scope.
        let all = vec![
            t("a", Status::Hairy, None),
            t("b", Status::Shorn, Some("a")),
            t("c", Status::Shaving, Some("a")),
        ];
        let spec = FilterSpec {
            statuses: vec![Status::Hairy],
            ..Default::default()
        };
        let ids = |scope| {
            let mut v: Vec<String> = build(&all, &spec, None, scope)
                .iter()
                .map(|r| r.yak().unwrap().id.clone())
                .collect();
            v.sort();
            v
        };
        assert_eq!(ids(FamilyScope::Lone), vec!["a"]);
        assert_eq!(ids(FamilyScope::Remaining), vec!["a", "c"]); // shorn leaf b dropped
        assert_eq!(ids(FamilyScope::All), vec!["a", "b", "c"]);
    }

    #[test]
    fn remaining_keeps_completed_connectors_to_open_work() {
        // a(hairy) > b(shorn) > c(shaving): b is completed but connects open c,
        // so "remaining" must keep it (never show a child without its chain).
        let all = vec![
            t("a", Status::Hairy, None),
            t("b", Status::Shorn, Some("a")),
            t("c", Status::Shaving, Some("b")),
        ];
        let spec = FilterSpec {
            statuses: vec![Status::Hairy],
            ..Default::default()
        };
        let flat = build(&all, &spec, None, FamilyScope::Remaining);
        let mut ids: Vec<String> = flat.iter().map(|r| r.yak().unwrap().id.clone()).collect();
        ids.sort();
        assert_eq!(ids, vec!["a", "b", "c"]);
        let ghost = |id: &str| {
            flat.iter()
                .find(|r| r.yak().unwrap().id == id)
                .unwrap()
                .ghost
        };
        assert!(
            ghost("b") && ghost("c"),
            "connector + open descendant are dim"
        );
        assert!(!ghost("a"), "the anchor is bright");
    }

    /// yaks-b608: a content filter (a herd here) on the Hairy view keeps the
    /// status axis. A shorn or dead child of a hairy match is not focus, and
    /// `remaining` drops it as it does with no filter.
    #[test]
    fn a_content_filter_keeps_the_status_scope() {
        let all = vec![
            t("h-a", Status::Hairy, None),
            t("h-b", Status::Shorn, Some("h-a")),
            t("h-c", Status::Dead, Some("h-a")),
            t("h-d", Status::Hairy, Some("h-a")),
            t("h-e", Status::Shorn, None),
        ];
        let spec = FilterSpec {
            statuses: vec![Status::Hairy],
            herds: vec!["h".into()],
            ..Default::default()
        };
        let flat = build(&all, &spec, None, FamilyScope::Remaining);
        let mut ids: Vec<&str> = flat.iter().map(|r| r.yak().unwrap().id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["h-a", "h-d"]);
        // The global query is a content filter too: same rule.
        let flat = build(
            &all,
            &FilterSpec {
                statuses: vec![Status::Hairy],
                ..Default::default()
            },
            Some("h-"),
            FamilyScope::Remaining,
        );
        assert!(
            flat.iter()
                .all(|r| r.yak().unwrap().status == Status::Hairy)
        );
        assert!(flat.iter().all(|r| !r.ghost), "both are hairy matches");
    }

    #[test]
    fn content_filter_includes_descendants_of_matches() {
        // a matches; its open child a1 now comes along as descendant context.
        // (The old filter path dropped descendants of a match — yaksrs-3331.)
        let mut a = t("a", Status::Hairy, None);
        a.title = "needle root".into();
        let a1 = t("a1", Status::Hairy, Some("a"));
        let all = vec![a, a1];
        let spec = FilterSpec {
            statuses: vec![Status::Hairy],
            search: Some("needle".into()),
            ..Default::default()
        };
        let flat = build(&all, &spec, None, FamilyScope::Remaining);
        let ids: Vec<&str> = flat.iter().map(|r| r.yak().unwrap().id.as_str()).collect();
        assert_eq!(ids, vec!["a", "a1"]);
        let ghost = |id: &str| {
            flat.iter()
                .find(|r| r.yak().unwrap().id == id)
                .unwrap()
                .ghost
        };
        assert!(!ghost("a"), "the match is bright");
        assert!(ghost("a1"), "its descendant is dim context");
    }
}
