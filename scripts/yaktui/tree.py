"""Task tree building + flattening for the list pane.

build_tree returns a flat list of (status, task, depth, ghost) tuples ready
for the list renderer. Ghost ancestors/descendants keep the tree rooted when
a filter narrows the set; ghosts render dimmed to signal "not the primary
matches, just context."
"""

from __future__ import annotations

from dataclasses import replace
from pathlib import Path

from yaklib import deps as _deps
from yaklib.filter import FilterSpec
from yaklib.model import (
    SHAVING,
    SHORN,
    STATUSES,
    all_tasks,
    parent_id,
)


class TaskNode:
    __slots__ = ("status", "task", "children", "ghost")

    def __init__(self, status, task, ghost=False):
        self.status = status
        self.task = task
        self.children = []
        self.ghost = ghost


def _child_sort_key(tid: str) -> int:
    dot = tid.rfind(".")
    if dot >= 0:
        suffix = tid[dot + 1 :]
        if suffix.isdigit():
            return int(suffix)
    return 0


def _child_status_rank(status: str) -> int:
    if status == SHAVING:
        return 0
    if status == SHORN:
        return 2
    return 1


def build_tree(
    root: Path,
    tab_status: str | None,
    spec: FilterSpec,
    tasks_cache: list | None = None,
    resolved_cache: set | None = None,
) -> list[tuple[str, dict, int, bool]]:
    """Flat list of (status, task, depth, ghost) for display.

    *tab_status* is the currently selected tab's status; it scopes the
    primary set unless the spec explicitly overrides with its own statuses.
    *tasks_cache* / *resolved_cache* let the caller avoid rescanning disk
    on every filter keypress.
    """
    all_by_id: dict[str, tuple[str, dict]] = {}
    if tasks_cache is not None:
        for st, t in tasks_cache:
            all_by_id[t["id"]] = (st, t)
    else:
        for s in STATUSES:
            for st, t in all_tasks(root, s):
                all_by_id[t["id"]] = (st, t)

    resolved = resolved_cache if resolved_cache is not None else _deps.resolved_ids(root)

    # Effective status scope: spec.statuses overrides tab; else tab.
    if spec.statuses:
        effective_statuses = set(spec.statuses)
    elif tab_status:
        effective_statuses = {tab_status}
    else:
        effective_statuses = set(STATUSES)

    def ancestors_of(tid: str) -> list[str]:
        out = []
        pid = parent_id(tid)
        while pid:
            if pid in all_by_id:
                out.append(pid)
            pid = parent_id(pid)
        return out

    # The current tab anchors the view: yaks in the effective status scope,
    # plus the family they hold on screen (ancestors walked up, descendants
    # walked down — any status). This `universe` is exactly what the tab shows
    # unfiltered; filtering only re-colors and prunes within it.
    anchor_ids = {tid for tid, (s, _t) in all_by_id.items() if s in effective_statuses}
    universe = set(anchor_ids)
    for tid in anchor_ids:
        universe.update(ancestors_of(tid))
    anchor_prefixes = tuple(tid + "." for tid in anchor_ids)
    if anchor_prefixes:
        for other in all_by_id:
            if other not in universe and other.startswith(anchor_prefixes):
                universe.add(other)

    # Content predicates only — status is the tab's job (above). A match may
    # therefore live in a different status than the tab, as long as the tab's
    # family is already holding it in view (i.e. it's in `universe`).
    content_spec = replace(spec, statuses=frozenset())
    match_active = not content_spec.is_empty()

    if match_active:
        # Matches anywhere in the tab's family light up bright; non-matching
        # ancestors come along dimmed to root them. Everything else is pruned.
        focus = {tid for tid in universe if content_spec.matches(all_by_id[tid][0], all_by_id[tid][1], resolved)}
        members = set(focus)
        for tid in focus:
            members.update(ancestors_of(tid))
    else:
        # No content filter: the tab's own yaks are primary, family is context.
        focus = set(anchor_ids)
        members = set(universe)

    nodes: dict[str, TaskNode] = {}
    for tid in members:
        s, t = all_by_id[tid]
        nodes[tid] = TaskNode(s, t, ghost=(tid not in focus))

    roots = []
    for tid, node in nodes.items():
        pid = parent_id(tid)
        if pid and pid in nodes:
            nodes[pid].children.append(node)
        else:
            roots.append(node)

    def sort_children(node: TaskNode):
        node.children.sort(
            key=lambda n: (
                _child_status_rank(n.status),
                n.task.get("priority", 9),
                _child_sort_key(n.task["id"]),
            )
        )
        for c in node.children:
            sort_children(c)

    if tab_status == SHORN and not spec.statuses:
        roots.sort(key=lambda n: n.task.get("updated", ""), reverse=True)
    else:
        roots.sort(key=lambda n: (n.task.get("priority", 9), n.task["id"]))
    for r in roots:
        sort_children(r)

    flat = []

    def flatten(node: TaskNode, depth: int):
        flat.append((node.status, node.task, depth, node.ghost))
        for c in node.children:
            flatten(c, depth + 1)

    for r in roots:
        flatten(r, 0)
    return flat


def apply_collapse(
    flat: list[tuple[str, dict, int, bool]], collapsed_ids: set[str], filter_active: bool
) -> tuple[list[tuple[str, dict, int, bool]], dict[str, int]]:
    """Drop descendants of collapsed ids and report per-parent hidden counts.

    Returns (visible_rows, counts). When a filter/search is active, collapse
    is ignored — the flattened filter view wins. Counts are keyed only by ids
    that are currently collapsed AND have at least one row hidden underneath
    them; the renderer uses that as the "show chevron + N" signal."""
    if filter_active or not collapsed_ids:
        return flat, {}
    counts: dict[str, int] = {}
    for _, task, _, _ in flat:
        tid = task["id"]
        for cid in collapsed_ids:
            if tid.startswith(cid + "."):
                counts[cid] = counts.get(cid, 0) + 1
    visible = [row for row in flat if not any(row[1]["id"].startswith(cid + ".") for cid in collapsed_ids)]
    return visible, counts
