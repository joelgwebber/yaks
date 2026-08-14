"""Backend-agnostic task access for the shared view-model builders.

``tree.build_tree`` and ``detail.build_detail_lines`` describe *what* the list
and detail panes show. To keep that logic in one place — shared by the curses
TUI and the demo renderer — the detail builder reads related tasks (deps,
parent, children, link targets, artifacts) through this ``TaskRepo`` protocol
instead of touching the filesystem directly.

``FsTaskRepo`` is the production implementation over a ``.yaks/`` root. Other
front-ends (e.g. the docs demo) provide their own in-memory repo.
"""

from __future__ import annotations

from pathlib import Path
from typing import Protocol, runtime_checkable

from yaklib import artifacts as _artifacts
from yaklib import deps as _deps
from yaklib import links as _links
from yaklib.model import all_tasks, find_children, find_task_file, load_task


@runtime_checkable
class TaskRepo(Protocol):
    """The task-access surface the shared view-model builders depend on."""

    def all_tasks(self) -> list[tuple[str, dict]]:
        """(status, task) for every visible task."""

    def resolved_ids(self) -> set[str]:
        """IDs that count as 'dependency satisfied' (shorn + dead)."""

    def find(self, task_id: str) -> tuple[str, dict] | None:
        """(status, task) for a single id, or None."""

    def children(self, task_id: str) -> list[tuple[str, dict]]:
        """(status, task) for each direct child, display-ordered."""

    def resolve_link_spans(self, text: str, self_id: str) -> list[tuple[int, int, str]]:
        """(start, end, id) for bare yak-id mentions in *text* that exist."""

    def artifacts(self, task_id: str, body: str) -> list[tuple[str, object]]:
        """(label, open_target) for each attachment referenced in *body*."""


class FsTaskRepo:
    """``TaskRepo`` backed by a ``.yaks/`` directory on disk."""

    def __init__(self, root: Path):
        self.root = root

    def all_tasks(self) -> list[tuple[str, dict]]:
        return all_tasks(self.root)

    def resolved_ids(self) -> set[str]:
        return _deps.resolved_ids(self.root)

    def find(self, task_id: str) -> tuple[str, dict] | None:
        res = find_task_file(self.root, task_id)
        if res is None:
            return None
        status, path = res
        return status, load_task(path)

    def children(self, task_id: str) -> list[tuple[str, dict]]:
        return find_children(self.root, task_id)

    def resolve_link_spans(self, text: str, self_id: str) -> list[tuple[int, int, str]]:
        return _links.resolve_spans(self.root, text, self_id)

    def artifacts(self, task_id: str, body: str) -> list[tuple[str, object]]:
        out: list[tuple[str, object]] = []
        for alt, aname in _artifacts.parse_artifacts(body, task_id):
            apath = _artifacts.artifacts_dir(self.root, task_id) / aname
            label = aname if not alt or alt == Path(aname).stem else f"{aname}  ({alt})"
            out.append((label, apath))
        return out
