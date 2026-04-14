"""Implicit inter-yak link detection in task bodies.

Scans free-form text for tokens that look like yak IDs ({prefix}-{4 hex},
optionally with .N child suffixes) and verifies each against the filesystem.
Used by the TUI to render a "References:" section in the detail pane.
"""

from __future__ import annotations

import re
from pathlib import Path

from yaklib.model import find_task_file

# A yak ID: lowercase prefix (letters/digits/dashes, starts with letter),
# dash, 4 hex chars, optional .N[.N...] child suffix. Word-boundaried so
# we don't match mid-identifier.
_ID_RE = re.compile(
    r"(?<![\w-])([a-z][a-z0-9-]*-[0-9a-f]{4}(?:\.\d+)*)(?![\w])"
)


def find_references(body: str) -> list[str]:
    """Return the ordered, deduplicated list of yak-ID-shaped tokens in *body*.

    Does not verify existence — callers filter by find_task_file.
    """
    if not body:
        return []
    seen: set[str] = set()
    out: list[str] = []
    for m in _ID_RE.finditer(body):
        tid = m.group(1)
        if tid not in seen:
            seen.add(tid)
            out.append(tid)
    return out


def resolve_references(root: Path, task: dict,
                       exclude: set[str] | None = None) -> list[tuple[str, dict]]:
    """Return [(status, task)] for each ID referenced in task['description']
    that exists on disk and isn't already in *exclude* (plus self, parent,
    children, explicit deps — callers pass these in *exclude*).
    """
    candidates = find_references(task.get("description") or "")
    exclude = set(exclude or ())
    exclude.add(task["id"])

    resolved: list[tuple[str, dict]] = []
    for tid in candidates:
        if tid in exclude:
            continue
        hit = find_task_file(root, tid)
        if not hit:
            continue
        status, path = hit
        from yaklib.model import load_task
        resolved.append((status, load_task(path)))
    return resolved
