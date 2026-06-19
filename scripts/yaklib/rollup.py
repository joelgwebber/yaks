"""Roll yaks up to the external issues they point at.

This is the read-only half of the yak -> external *projection* (see
docs/design/sync.md). A yak's ``source:`` records its conceptual home in an
external tracker; many yaks roll up to few external issues. A yak with no own
``source:`` inherits the nearest ancestor's, resolved here at query time —
nothing is ever written into descendants.

No network, no MCP: this only reads local task files.
"""

from __future__ import annotations

import re
from pathlib import Path

from yaklib.filter import FilterSpec, filter_tasks
from yaklib.model import all_tasks, parent_id

# Tracker URL -> (tracker, key) recognizers. First match wins; anything
# unrecognized falls through to ("other", <url>).
_JIRA_RE = re.compile(r"atlassian\.net/browse/([A-Z][A-Z0-9_]*-\d+)")
_LINEAR_RE = re.compile(r"linear\.app/[^/]+/issue/([A-Za-z0-9]+-\d+)")
_GITHUB_RE = re.compile(r"github\.com/([^/\s]+)/([^/\s]+)/issues/(\d+)")


def tracker_and_key(source: str | None) -> tuple[str, str | None]:
    """Classify a ``source:`` URL into (tracker, human key).

    - Jira:   https://x.atlassian.net/browse/SUBTEXT-369 -> ("jira", "SUBTEXT-369")
    - Linear: https://linear.app/team/issue/ROC-5/...     -> ("linear", "ROC-5")
    - GitHub: https://github.com/o/r/issues/123           -> ("github", "o/r#123")
    - other:  anything else                                -> ("other", <url>)
    """
    if not source:
        return ("none", None)
    s = source.strip()
    m = _JIRA_RE.search(s)
    if m:
        return ("jira", m.group(1))
    m = _LINEAR_RE.search(s)
    if m:
        return ("linear", m.group(1).upper())
    m = _GITHUB_RE.search(s)
    if m:
        return ("github", f"{m.group(1)}/{m.group(2)}#{m.group(3)}")
    return ("other", s)


def effective_source(task_id: str, source_by_id: dict[str, str]) -> tuple[str | None, str | None]:
    """Resolve a yak's effective source, walking up the parent chain.

    Returns (source, inherited_from). ``inherited_from`` is None when the yak
    carries its own ``source:``, otherwise the ancestor ID the source came from.
    """
    cur: str | None = task_id
    seen: set[str] = set()
    while cur and cur not in seen:
        seen.add(cur)
        src = source_by_id.get(cur)
        if src:
            return src, (None if cur == task_id else cur)
        cur = parent_id(cur)
    return None, None


def build_rollup(root: Path, spec: FilterSpec) -> tuple[list[dict], int]:
    """Group the filtered yaks by effective source.

    Returns (groups, unsourced_count). Each group is::

        {"source": url, "tracker": str, "key": str | None,
         "yaks": [{"status", "task", "inherited": bool, "inherited_from": str|None}]}

    Groups are sorted by key (falling back to URL); yaks within a group by ID.
    ``unsourced_count`` counts matched yaks with no effective source (omitted
    from the groups — rollup is about external targets).
    """
    # Inheritance must see every visible yak, not just the filtered set: a
    # matched child may inherit from an unmatched ancestor.
    source_by_id = {t["id"]: t["source"] for _s, t in all_tasks(root) if t.get("source")}

    groups: dict[str, dict] = {}
    unsourced = 0
    for status, t in filter_tasks(root, spec):
        src, inherited_from = effective_source(t["id"], source_by_id)
        if not src:
            unsourced += 1
            continue
        g = groups.setdefault(src, {"source": src, "yaks": []})
        g["yaks"].append(
            {
                "status": status,
                "task": t,
                "inherited": inherited_from is not None,
                "inherited_from": inherited_from,
            }
        )

    out: list[dict] = []
    for src, g in groups.items():
        tracker, key = tracker_and_key(src)
        g["tracker"] = tracker
        g["key"] = key
        g["yaks"].sort(key=lambda y: y["task"]["id"])
        out.append(g)
    out.sort(key=lambda g: g["key"] or g["source"])
    return out, unsourced
