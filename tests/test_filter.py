"""Unit tests for FilterSpec predicate + build_tree integration."""

from __future__ import annotations

from pathlib import Path

from conftest import create_task
from yaklib.filter import FilterSpec, filter_tasks
from yaktui.tree import build_tree


def _yaks(root: Path) -> Path:
    return root / ".yaks"


def _task(id="x", type="task", priority=2, labels=None, depends_on=None,
          title="t", description=""):
    return {
        "id": id, "type": type, "priority": priority,
        "labels": labels or [], "depends_on": depends_on or [],
        "title": title, "description": description,
    }


def test_is_empty():
    assert FilterSpec().is_empty()
    assert not FilterSpec(search="x").is_empty()
    assert not FilterSpec(types=frozenset({"bug"})).is_empty()


def test_matches_type():
    spec = FilterSpec(types=frozenset({"bug", "feature"}))
    assert spec.matches("hairy", _task(type="bug"), set())
    assert spec.matches("hairy", _task(type="feature"), set())
    assert not spec.matches("hairy", _task(type="task"), set())


def test_matches_priority():
    spec = FilterSpec(priorities=frozenset({1, 2}))
    assert spec.matches("hairy", _task(priority=1), set())
    assert not spec.matches("hairy", _task(priority=3), set())


def test_matches_labels_or():
    spec = FilterSpec(labels=("urgent", "core"))
    assert spec.matches("hairy", _task(labels=["urgent"]), set())
    assert spec.matches("hairy", _task(labels=["core", "other"]), set())
    assert not spec.matches("hairy", _task(labels=["other"]), set())


def test_matches_search_title_and_desc():
    spec = FilterSpec(search="retry")
    assert spec.matches("hairy", _task(title="add retry logic"), set())
    assert spec.matches("hairy", _task(description="on retry..."), set())
    assert not spec.matches("hairy", _task(title="nope"), set())


def test_matches_ready_only():
    spec = FilterSpec(ready_only=True)
    assert spec.matches("hairy", _task(depends_on=[]), set())
    assert spec.matches("hairy", _task(depends_on=["a"]), {"a"})
    assert not spec.matches("hairy", _task(depends_on=["a"]), set())


def test_matches_tangled_only():
    spec = FilterSpec(tangled_only=True)
    assert not spec.matches("hairy", _task(depends_on=[]), set())
    assert spec.matches("hairy", _task(depends_on=["a"]), set())


def test_matches_parent_descendants():
    spec = FilterSpec(parent="yak-abcd")
    assert spec.matches("hairy", _task(id="yak-abcd.1"), set())
    assert spec.matches("hairy", _task(id="yak-abcd.1.2"), set())
    assert not spec.matches("hairy", _task(id="yak-abcd"), set())
    assert not spec.matches("hairy", _task(id="yak-beef"), set())


def test_matches_combination_is_and():
    spec = FilterSpec(types=frozenset({"bug"}),
                      priorities=frozenset({1}))
    assert spec.matches("hairy", _task(type="bug", priority=1), set())
    assert not spec.matches("hairy", _task(type="bug", priority=2), set())
    assert not spec.matches("hairy", _task(type="task", priority=1), set())


def test_summary_includes_constraints():
    spec = FilterSpec(types=frozenset({"bug"}),
                      priorities=frozenset({1, 2}),
                      search="retry")
    s = spec.summary()
    assert "type:bug" in s
    assert "p:p1,p2" in s
    assert '"retry"' in s


def test_build_tree_filters_by_type(yak, yak_root):
    yak("create", "--title", "bug1", "--type", "bug")
    yak("create", "--title", "task1", "--type", "task")
    spec = FilterSpec(types=frozenset({"bug"}))
    flat = build_tree(_yaks(yak_root), "hairy", spec)
    titles = [t["title"] for _, t, _, _ in flat]
    assert "bug1" in titles
    assert "task1" not in titles


def test_filter_tasks_shared_with_cli(yak, yak_root):
    """filter_tasks (CLI path) applies the same spec as build_tree."""
    yak("create", "--title", "bug1", "--type", "bug", "--priority", "1")
    yak("create", "--title", "task1", "--type", "task", "--priority", "2")
    spec = FilterSpec(priorities=frozenset({1}))
    res = filter_tasks(_yaks(yak_root), spec)
    titles = [t["title"] for _, t in res]
    assert titles == ["bug1"]


def test_cli_list_repeatable_flags(yak):
    """--type is repeatable and OR-within."""
    yak("create", "--title", "bug1", "--type", "bug")
    yak("create", "--title", "feat1", "--type", "feature")
    yak("create", "--title", "task1", "--type", "task")
    out = yak("list", "--type", "bug", "--type", "feature").stdout
    assert "bug1" in out and "feat1" in out
    assert "task1" not in out


def test_cli_next_honors_filter(yak):
    """`next` accepts type/priority filters on top of ready_only."""
    yak("create", "--title", "b", "--type", "bug")
    yak("create", "--title", "t", "--type", "task")
    out = yak("next", "--type", "bug").stdout
    assert "b" in out and "  t " not in out


def test_cli_list_ready_and_tangled(yak):
    a = yak("create", "--title", "dep", "--type", "task").stdout
    a_id = a.splitlines()[0].split()[1].rstrip(":")
    b = yak("create", "--title", "waiter", "--type", "task").stdout
    b_id = b.splitlines()[0].split()[1].rstrip(":")
    yak("dep", "add", b_id, a_id)

    ready = yak("list", "--ready").stdout
    assert a_id in ready and b_id not in ready

    tangled = yak("list", "--tangled").stdout
    rows = [l for l in tangled.splitlines() if l.strip().startswith("[H]")]
    tangled_ids = {l.split()[1] for l in rows}
    assert tangled_ids == {b_id}


def test_build_tree_status_override(yak, yak_root):
    a = create_task(yak, "alpha", type="task")
    yak("shave", a)
    # Request shorn via spec even though tab is hairy.
    spec = FilterSpec(statuses=frozenset({"shaving"}))
    flat = build_tree(_yaks(yak_root), "hairy", spec)
    ids = [t["id"] for _, t, _, ghost in flat if not ghost]
    assert a in ids
