"""Per-view counts + tab-strip text (yak-3fd4.4, yak-5892, yak-1b89).

Counts are a list aligned to app.views (0=Hairy, 1=Shaving, 2=Shorn for the
built-in status Views), computed from each View's OWN spec — independent of the
live filter — and memoized against the data version + the View list.
"""

from __future__ import annotations

from dataclasses import replace
from pathlib import Path

from yaklib.filter import FilterSpec
from yaklib.model import HAIRY, SHAVING, SHORN
from yaktui.render import format_count, view_counts, view_tab_text
from yaktui.view import builtin_status_views, default_views


class _App:
    """Minimal stand-in mirroring what render reads off the real TUI."""

    def __init__(self, cache, filter_spec=None):
        self._task_cache = cache
        self._resolved_cache = set()
        self.root = Path(".")
        self.views = builtin_status_views()  # [Hairy, Shaving, Shorn]
        self.view = 0
        self.filter_spec = (filter_spec if filter_spec is not None
                            else replace(self.views[0].spec))
        self._task_cache_version = 1
        self._counts_memo = None

    def _is_view_modified(self):
        return self.filter_spec != self.views[self.view].spec


def _task(tid, title="t", **f):
    return {"id": tid, "title": title, "type": "task", "priority": 3, **f}


def test_format_count_caps_unbounded():
    assert format_count(0) == "0"
    assert format_count(999) == "999"
    assert format_count(1000) == "999+"
    assert format_count(5, cap=3) == "3+"


def test_counts_are_per_view_by_status():
    cache = [
        (HAIRY, _task("a")), (HAIRY, _task("b")),
        (SHAVING, _task("c")),
        (SHORN, _task("d")), (SHORN, _task("e")), (SHORN, _task("f")),
    ]
    assert view_counts(_App(cache)) == [2, 1, 3]  # aligned to [Hairy, Shaving, Shorn]


def test_counts_ignore_the_live_filter():
    # Counts reflect each View's own spec, NOT the ephemeral live filter.
    cache = [
        (HAIRY, _task("a", title="foo bar")),
        (HAIRY, _task("b", title="baz")),
        (SHORN, _task("c", title="foo again")),
    ]
    app = _App(cache, FilterSpec(search="foo"))  # live filter narrowed to "foo"
    assert view_counts(app) == [2, 0, 1]  # still the full per-status sizes


def test_memoized_on_version_and_views_not_filter():
    cache = [(HAIRY, _task("a")), (SHORN, _task("b"))]
    app = _App(cache)

    first = view_counts(app)
    assert view_counts(app) is first          # stable across re-renders

    app.filter_spec = FilterSpec(search="x")  # editing the filter...
    assert view_counts(app) is first          # ...does NOT recompute counts

    app._task_cache_version += 1              # a data reload does
    assert view_counts(app) is not first


def test_working_set_view_counts_present_starred_ids():
    cache = [(HAIRY, _task("yak-1")), (SHORN, _task("yak-2")), (HAIRY, _task("yak-3"))]
    app = _App(cache)
    app.views = default_views()               # [.., recent, working-set(idx 4)]
    app.working_set = ["yak-2", "ghost", "yak-3"]
    counts = view_counts(app)
    assert counts[4] == 2                       # yak-2 + yak-3 present; ghost ignored


def test_view_tab_text_caps_and_marks_modified_active_view():
    app = _App([(HAIRY, _task("a"))])
    counts = [1000, 0, 0]
    # Cap applies in the rendered width, not the raw number.
    assert "(999+)" in view_tab_text(app, 0, counts)
    # Unmodified active view: no marker.
    assert "*" not in view_tab_text(app, 0, counts)
    # Fork the live filter away from the Hairy view's spec.
    app.filter_spec = FilterSpec(search="z")
    assert "*" in view_tab_text(app, 0, counts)       # active + modified
    assert "*" not in view_tab_text(app, 1, counts)   # non-active view never marked
