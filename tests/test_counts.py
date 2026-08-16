"""Per-view count memoization + capped display (yak-3fd4.4, yak-5892).

Counts are a list aligned to app.views (index 0=Hairy, 1=Shaving, 2=Shorn for
the built-in status Views), memoized against the data version, the filter, and
the View list.
"""

from __future__ import annotations

from pathlib import Path

from yaklib.filter import FilterSpec
from yaklib.model import HAIRY, SHAVING, SHORN
from yaktui.render import format_count, view_counts, view_tab_text
from yaktui.view import builtin_status_views


class _App:
    """Minimal stand-in exposing what view_counts / view_tab_text read."""

    def __init__(self, cache, spec=None):
        self._task_cache = cache
        self._resolved_cache = set()
        self.filter_spec = spec or FilterSpec()
        self.root = Path(".")
        self.views = builtin_status_views()  # [Hairy, Shaving, Shorn]
        self.view = 0
        self._task_cache_version = 1
        self._counts_memo = None


def _task(tid, title="t", **f):
    return {"id": tid, "title": title, "type": "task", "priority": 3, **f}


def test_format_count_caps_unbounded():
    assert format_count(0) == "0"
    assert format_count(999) == "999"
    assert format_count(1000) == "999+"
    assert format_count(50000) == "999+"
    assert format_count(5, cap=3) == "3+"


def test_empty_spec_counts_from_cache_by_view():
    cache = [
        (HAIRY, _task("a")), (HAIRY, _task("b")),
        (SHAVING, _task("c")),
        (SHORN, _task("d")), (SHORN, _task("e")), (SHORN, _task("f")),
    ]
    # Aligned to views: [Hairy, Shaving, Shorn].
    assert view_counts(_App(cache)) == [2, 1, 3]


def test_memoized_until_version_or_spec_changes():
    cache = [(HAIRY, _task("a")), (SHORN, _task("b"))]
    app = _App(cache)

    first = view_counts(app)
    assert view_counts(app) is first  # same version + spec + views => memo hit

    app._task_cache_version += 1      # data reloaded
    after_reload = view_counts(app)
    assert after_reload is not first

    app.filter_spec = FilterSpec(search="nope")  # filter changed
    after_filter = view_counts(app)
    assert after_filter is not after_reload


def test_active_spec_counts_only_matches():
    cache = [
        (HAIRY, _task("a", title="foo bar")),
        (HAIRY, _task("b", title="baz")),
        (SHORN, _task("c", title="foo again")),
    ]
    counts = view_counts(_App(cache, FilterSpec(search="foo")))
    assert counts[0] == 1   # Hairy: only "foo bar" matches; "baz" pruned
    assert counts[2] == 1   # Shorn: "foo again" matches


def test_view_tab_text_is_capped_and_marked():
    app = _App([(HAIRY, _task("a"))])
    counts = [1000, 0, 0]
    # Cap applies in the rendered width (not the raw number).
    assert "(999+)" in view_tab_text(app, 0, counts, frozenset())
    # A status override adds the * marker.
    assert "*" in view_tab_text(app, 0, counts, frozenset({HAIRY}))
    assert "*" not in view_tab_text(app, 1, counts, frozenset({HAIRY}))
