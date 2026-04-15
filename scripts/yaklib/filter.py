"""FilterSpec — a single dataclass describing every way a task list can be
narrowed. Predicate evaluation lives here so both the TUI and CLI can share it.

Semantics: AND across fields; OR within a multi-value field. An empty field
matches everything (i.e. "unconstrained"). A filter with no constraints is a
no-op.
"""

from __future__ import annotations

from dataclasses import dataclass, field, replace
from pathlib import Path

from yaklib.model import STATUSES, all_tasks, _ALL_STATUSES


@dataclass
class FilterSpec:
    statuses: frozenset = field(default_factory=frozenset)      # set[str]
    types: frozenset = field(default_factory=frozenset)         # set[str]
    priorities: frozenset = field(default_factory=frozenset)    # set[int]
    labels: tuple = field(default_factory=tuple)                # tuple[str]; OR
    search: str = ""
    ready_only: bool = False
    tangled_only: bool = False
    parent: str = ""

    def is_empty(self) -> bool:
        return not (self.statuses or self.types or self.priorities
                    or self.labels or self.search or self.ready_only
                    or self.tangled_only or self.parent)

    def matches(self, status: str, task: dict, resolved: set) -> bool:
        """Apply every predicate. *resolved* = shorn+dead ids (canonical
        "dep satisfied" set; see yaklib.deps.resolved_ids)."""
        if self.statuses and status not in self.statuses:
            return False
        if self.types and task.get("type") not in self.types:
            return False
        if self.priorities and task.get("priority") not in self.priorities:
            return False
        if self.labels:
            tl = set(task.get("labels") or [])
            if not tl.intersection(self.labels):
                return False
        if self.search:
            q = self.search.lower()
            blob = (task.get("title", "") + " "
                    + (task.get("description") or "") + " "
                    + task.get("id", "")).lower()
            if q not in blob:
                return False
        deps = task.get("depends_on") or []
        unresolved = [d for d in deps if d not in resolved]
        if self.ready_only and unresolved:
            return False
        if self.tangled_only and not unresolved:
            return False
        if self.parent:
            tid = task.get("id", "")
            prefix = self.parent + "."
            if not tid.startswith(prefix):
                return False
        return True

    def summary(self) -> str:
        """One-line chip summary for the status bar. Empty if no constraints."""
        if self.is_empty():
            return ""
        parts = []
        if self.statuses:
            parts.append("status:" + ",".join(sorted(self.statuses)))
        if self.types:
            parts.append("type:" + ",".join(sorted(self.types)))
        if self.priorities:
            parts.append("p:" + ",".join(f"p{p}" for p in sorted(self.priorities)))
        if self.labels:
            parts.append("labels:" + ",".join(self.labels))
        if self.search:
            parts.append(f'"{self.search}"')
        if self.ready_only:
            parts.append("ready")
        if self.tangled_only:
            parts.append("tangled")
        if self.parent:
            parts.append(f"parent:{self.parent}")
        return " ".join(parts)

    def with_statuses(self, statuses) -> "FilterSpec":
        return replace(self, statuses=frozenset(statuses))


def collect_all_labels(root: Path) -> list:
    seen = set()
    for s in STATUSES:
        for _, t in all_tasks(root, s):
            for lab in (t.get("labels") or []):
                seen.add(lab)
    return sorted(seen)


def filter_tasks(root: Path, spec: "FilterSpec",
                 include_dead: bool = False) -> list:
    """Apply *spec* across every visible status. Returns [(status, task), ...].
    Shared filter path for CLI `list`/`search`/`next`/`tangled`.

    Dead tasks are excluded unless *include_dead* is true or the spec's
    statuses field explicitly names DEAD.
    """
    from yaklib.deps import resolved_ids as _resolved
    from yaklib.model import DEAD
    resolved = _resolved(root)

    if spec.statuses:
        statuses = tuple(spec.statuses)
    elif include_dead:
        statuses = _ALL_STATUSES
    else:
        statuses = STATUSES

    out = []
    for s in statuses:
        for st, t in all_tasks(root, s):
            if spec.matches(st, t, resolved):
                out.append((st, t))
    return out
