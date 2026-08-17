"""Flat sorted views + the Recent built-in (yak-b601)."""

from __future__ import annotations

from pathlib import Path

from yaklib.filter import FilterSpec
from yaklib.model import HAIRY, SHORN
from yaktui.tree import build_flat
from yaktui.view import default_views, recent_view


def _task(tid, **f):
    return {"id": tid, "title": f.pop("title", tid), "type": "task",
            "priority": f.pop("priority", 3), **f}


def _ids(rows):
    return [t["id"] for _s, t, _d, _g in rows]


def test_build_flat_sorts_by_updated_desc_and_limits():
    cache = [
        (HAIRY, _task("a", updated="2026-08-10T00:00:00Z")),
        (SHORN, _task("b", updated="2026-08-16T00:00:00Z")),
        (HAIRY, _task("c", updated="2026-08-13T00:00:00Z")),
    ]
    rows = build_flat(Path("."), FilterSpec(), "updated", "desc", limit=2,
                      tasks_cache=cache, resolved_cache=set())
    assert _ids(rows) == ["b", "c"]              # newest first, capped at 2
    assert all(depth == 0 and not ghost for _s, _t, depth, ghost in rows)


def test_build_flat_priority_ascending():
    cache = [
        (HAIRY, _task("hi", priority=1)),
        (HAIRY, _task("lo", priority=5)),
        (HAIRY, _task("mid", priority=3)),
    ]
    rows = build_flat(Path("."), FilterSpec(), "priority", "asc",
                      tasks_cache=cache, resolved_cache=set())
    assert _ids(rows) == ["hi", "mid", "lo"]


def test_build_flat_respects_filter_spec():
    cache = [
        (HAIRY, _task("h1", title="alpha")),
        (SHORN, _task("s1", title="alpha")),
        (HAIRY, _task("h2", title="beta")),
    ]
    # Status scope from the spec.
    rows = build_flat(Path("."), FilterSpec(statuses=frozenset({HAIRY})),
                      "id", "asc", tasks_cache=cache, resolved_cache=set())
    assert _ids(rows) == ["h1", "h2"]
    # Content search.
    rows = build_flat(Path("."), FilterSpec(search="beta"),
                      "id", "asc", tasks_cache=cache, resolved_cache=set())
    assert _ids(rows) == ["h2"]


def test_build_flat_parent_scope():
    cache = [
        (HAIRY, _task("root")),
        (HAIRY, _task("kid", parent="root")),
        (HAIRY, _task("grandkid", parent="kid")),
        (HAIRY, _task("outsider")),
    ]
    rows = build_flat(Path("."), FilterSpec(parent="root"), "id", "asc",
                      tasks_cache=cache, resolved_cache=set())
    assert _ids(rows) == ["grandkid", "kid"]  # descendants only, not root/outsider


def test_recent_view_shape():
    rv = recent_view()
    assert rv.status is None and rv.is_flat and rv.builtin
    assert rv.sort_by == "updated" and rv.sort_dir == "desc" and rv.limit
    assert rv.spec == FilterSpec()  # all tasks


def test_default_views_is_status_tabs_plus_recent():
    views = default_views()
    assert [v.status for v in views] == [HAIRY, "shaving", SHORN, None]
    assert not any(v.is_flat for v in views[:3])  # status views stay tree
    assert views[3].is_flat                       # Recent is flat
