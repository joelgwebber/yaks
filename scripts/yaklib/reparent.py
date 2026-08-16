"""Reparent a yak by repointing its ``parent`` field.

Since hierarchy lives in the frontmatter rather than the ID (yak-3fd4.6),
reparenting is a single-field rewrite: the yak's ID is stable, its children
keep pointing at it (so the whole subtree rides along for free), and nothing
else needs touching. ``new_parent=None`` promotes the yak to top-level.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from yaklib.model import (
    descendant_ids,
    find_task_file,
    load_task,
    now_iso,
    parent_of,
    save_task,
)


class ReparentError(Exception):
    """Raised when a reparent can't proceed."""


@dataclass
class ReparentResult:
    task_id: str
    old_parent: str | None
    new_parent: str | None


def reparent(root: Path, task_id: str, new_parent: str | None) -> ReparentResult:
    """Repoint *task_id* under *new_parent* (or promote to top-level when None).

    Raises ReparentError for a missing yak, a missing/self/cyclic new parent,
    or a no-op (already there / already top-level).
    """
    res = find_task_file(root, task_id)
    if res is None:
        raise ReparentError(f"task {task_id} not found")
    _, path = res
    task = load_task(path)
    old_parent = parent_of(task)

    if new_parent is not None:
        if new_parent == task_id:
            raise ReparentError("cannot reparent a task under itself")
        if find_task_file(root, new_parent) is None:
            raise ReparentError(f"parent task {new_parent} not found")
        if new_parent in descendant_ids(root, task_id, include_dead=True):
            raise ReparentError("cannot reparent under own descendant")
        if old_parent == new_parent:
            raise ReparentError(f"{task_id} is already a child of {new_parent}")
        task["parent"] = new_parent
    else:
        if not old_parent:
            raise ReparentError(f"{task_id} is already a top-level task")
        task.pop("parent", None)

    task["updated"] = now_iso()
    save_task(path, task)
    return ReparentResult(task_id=task_id, old_parent=old_parent,
                          new_parent=new_parent)
