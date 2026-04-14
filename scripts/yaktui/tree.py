"""Task tree building + flattening for the list pane.

build_tree returns a flat list of (status, task, depth, ghost) tuples ready
for the list renderer. Ghost ancestors/descendants keep the tree rooted when
a filter narrows the set; ghosts render dimmed to signal "not the primary
matches, just context."
"""

from __future__ import annotations

from pathlib import Path

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


def _build_search_results(root: Path, query: str):
    q = query.lower()
    results = []
    for s in STATUSES:
        for st, t in all_tasks(root, s):
            title = t.get("title", "").lower()
            desc = (t.get("description") or "").lower()
            if q in title or q in desc or q in t.get("id", "").lower():
                results.append((st, t, 0, False))
    results.sort(key=lambda x: (x[0] != HAIRY, x[0] != SHAVING,
                                x[1].get("priority", 9)))
    return results


def build_tree(root: Path, status_filter: str | None, filter_mode: str,
               search_query: str) -> list[tuple[str, dict, int, bool]]:
    """Flat list of (status, task, depth, ghost) for display."""
    if search_query:
        return _build_search_results(root, search_query)

    all_by_id: dict[str, tuple[str, dict]] = {}
    for s in STATUSES:
        for st, t in all_tasks(root, s):
            all_by_id[t["id"]] = (st, t)

    if status_filter and filter_mode in ("all", "next", "tangled"):
        primary = [(s, t) for s, t in all_by_id.values() if s == status_filter]
    else:
        primary = list(all_by_id.values())

    if filter_mode == "next" and status_filter == HAIRY:
        shorn_ids = {t["id"] for s, t in all_by_id.values() if s == SHORN}
        primary = [(s, t) for s, t in primary
                   if not t.get("depends_on") or
                   all(d in shorn_ids for d in t.get("depends_on", []))]
    elif filter_mode == "tangled" and status_filter == HAIRY:
        shorn_ids = {t["id"] for s, t in all_by_id.values() if s == SHORN}
        primary = [(s, t) for s, t in primary
                   if any(d not in shorn_ids for d in t.get("depends_on", []))]

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

    if status_filter == SHORN:
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
