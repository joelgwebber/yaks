"""Dependency resolution: ready/tangled/blocked computations.

Consolidates logic that was duplicated across cmd_next, cmd_tangled
(in yak.py) and _recompute_blocked / _depends_on_transitively (in tui.py).

Canonical rule: a dep counts as "resolved" if the task is shorn OR dead —
slaughtering a blocker unblocks its dependents.
"""

from __future__ import annotations

from pathlib import Path

from yaklib.model import (
    DEAD,
    HAIRY,
    SHORN,
    all_tasks,
    find_task_file,
    load_task,
)


def resolved_ids(root: Path) -> set[str]:
    """IDs that count as "dep satisfied" — shorn + dead."""
    return (
        {t["id"] for _, t in all_tasks(root, SHORN)}
        | {t["id"] for _, t in all_tasks(root, DEAD)}
    )


def is_ready(task: dict, resolved: set[str]) -> bool:
    deps = task.get("depends_on") or []
    return not deps or all(d in resolved for d in deps)


def unresolved_deps(task: dict, resolved: set[str]) -> list[str]:
    return [d for d in (task.get("depends_on") or []) if d not in resolved]


def ready_tasks(root: Path) -> list[dict]:
    """Hairy tasks with all deps resolved."""
    resolved = resolved_ids(root)
    return [t for _, t in all_tasks(root, HAIRY) if is_ready(t, resolved)]


def tangled_tasks(root: Path) -> list[tuple[dict, list[str]]]:
    """Hairy tasks with at least one unresolved dep, paired with that list."""
    resolved = resolved_ids(root)
    out = []
    for _, t in all_tasks(root, HAIRY):
        unshorn = unresolved_deps(t, resolved)
        if unshorn:
            out.append((t, unshorn))
    return out


def compute_blocked(root: Path) -> tuple[set[str], dict[str, list[tuple[str, dict]]]]:
    """Return (blocked_ids, reverse_deps) for TUI overlays.

    - blocked_ids: hairy task IDs whose deps aren't all resolved.
    - reverse_deps: {dep_id: [(status, task), ...]} — tasks that depend on dep_id.
      Includes blockers from every visible status, used to render "Blocks:" lists.
    """
    visible = all_tasks(root)
    resolved = resolved_ids(root)
    blocked: set[str] = set()
    reverse: dict[str, list[tuple[str, dict]]] = {}
    for s, t in visible:
        deps = t.get("depends_on") or []
        if s == HAIRY and any(d not in resolved for d in deps):
            blocked.add(t["id"])
        for d in deps:
            reverse.setdefault(d, []).append((s, t))
    return blocked, reverse


def depends_on_transitively(root: Path, start_id: str, target_id: str) -> bool:
    """True if start_id depends on target_id directly or via any chain.

    Walks depends_on links from start_id outward. Used to reject cyclic
    dependency edits.
    """
    seen: set[str] = set()
    stack = [start_id]
    while stack:
        cur = stack.pop()
        if cur in seen:
            continue
        seen.add(cur)
        result = find_task_file(root, cur)
        if not result:
            continue
        t = load_task(result[1])
        for d in t.get("depends_on") or []:
            if d == target_id:
                return True
            stack.append(d)
    return False
