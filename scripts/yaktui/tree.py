"""Task tree building + flattening for the list pane.

build_tree returns a flat list of (status, task, depth, ghost) tuples ready
for the list renderer. Ghost ancestors/descendants keep the tree rooted when
a filter narrows the set; ghosts render dimmed to signal "not the primary
matches, just context."
"""

from __future__ import annotations

from pathlib import Path

from yaklib import deps as _deps
from yaklib.filter import FilterSpec
from yaklib.model import (
    HAIRY,
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
        suffix = tid[dot + 1:]
        if suffix.isdigit():
            return int(suffix)
    return 0


def _child_status_rank(status: str) -> int:
    if status == SHAVING:
        return 0
    if status == SHORN:
        return 2
    return 1


def build_tree(root: Path, tab_status: str | None,
               spec: FilterSpec,
               tasks_cache: list | None = None,
               resolved_cache: set | None = None
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

    # If search is active, flatten globally (no tree context) — matches
    # across all statuses the user allowed.
    if spec.search:
        results = []
        for s, t in all_by_id.values():
            if s not in effective_statuses:
                continue
            # Reuse spec.matches but ignore its own statuses (already scoped).
            if spec.matches(s, t, resolved):
                results.append((s, t, 0, False))
        results.sort(key=lambda x: (x[0] != HAIRY, x[0] != SHAVING,
                                    x[1].get("priority", 9)))
        return results

    # Scope then filter.
    primary = [(s, t) for s, t in all_by_id.values()
               if s in effective_statuses and spec.matches(s, t, resolved)]

    primary_ids = {t["id"] for _, t in primary}

    nodes: dict[str, TaskNode] = {}
    for s, t in primary:
        nodes[t["id"]] = TaskNode(s, t, ghost=False)

    # Ghost ancestors: walk up from primaries to keep tree rooted.
    for tid in list(primary_ids):
        pid = parent_id(tid)
        while pid and pid not in nodes:
            if pid in all_by_id:
                ps, pt = all_by_id[pid]
                nodes[pid] = TaskNode(ps, pt, ghost=True)
            pid = parent_id(pid) if pid else None

    # Ghost descendants: include any descendant of an already-visible node.
    child_prefixes = {tid + "." for tid in nodes}
    for other_id, (os_, ot) in all_by_id.items():
        if other_id in nodes:
            continue
        for prefix in child_prefixes:
            if other_id.startswith(prefix):
                nodes[other_id] = TaskNode(os_, ot, ghost=True)
                break

    roots = []
    for tid, node in nodes.items():
        pid = parent_id(tid)
        if pid and pid in nodes:
            nodes[pid].children.append(node)
        else:
            roots.append(node)

    def sort_children(node: TaskNode):
        node.children.sort(key=lambda n: (
            _child_status_rank(n.status),
            n.task.get("priority", 9),
            _child_sort_key(n.task["id"]),
        ))
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
