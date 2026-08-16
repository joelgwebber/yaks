"""Tab-count memoization + capped display (yak-3fd4.4)."""

from __future__ import annotations

from pathlib import Path

from yaklib.filter import FilterSpec
from yaklib.model import HAIRY, SHAVING, SHORN
from yaktui.render import format_count, tab_counts


class _App:
    """Minimal stand-in exposing what tab_counts reads."""

    def __init__(self, cache, spec=None):
        self._task_cache = cache
        self._resolved_cache = set()
        self.filter_spec = spec or FilterSpec()
        self.root = Path(".")
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


def test_empty_spec_counts_from_cache_by_status():
    cache = [
        (HAIRY, _task("a")), (HAIRY, _task("b")),
        (SHAVING, _task("c")),
        (SHORN, _task("d")), (SHORN, _task("e")), (SHORN, _task("f")),
    ]
    counts = tab_counts(_App(cache))
    assert counts == {HAIRY: 2, SHAVING: 1, SHORN: 3}


def test_memoized_until_version_or_spec_changes():
    cache = [(HAIRY, _task("a")), (SHORN, _task("b"))]
    app = _App(cache)

    first = tab_counts(app)
    assert tab_counts(app) is first  # same version + spec => memo hit

    app._task_cache_version += 1      # data reloaded
    after_reload = tab_counts(app)
    assert after_reload is not first

    app.filter_spec = FilterSpec(search="nope")  # filter changed
    after_filter = tab_counts(app)
    assert after_filter is not after_reload


def test_active_spec_counts_only_matches():
    cache = [
        (HAIRY, _task("a", title="foo bar")),
        (HAIRY, _task("b", title="baz")),
        (SHORN, _task("c", title="foo again")),
    ]
    counts = tab_counts(_App(cache, FilterSpec(search="foo")))
    assert counts[HAIRY] == 1   # only "foo bar" matches; "baz" pruned
    assert counts[SHORN] == 1   # "foo again" matches
