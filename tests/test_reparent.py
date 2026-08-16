"""Tests for yaklib.reparent — parent-field repointing (yak-3fd4.6).

Reparenting no longer changes IDs or cascades: it rewrites a single ``parent``
field, so children ride along for free and there are no link/artifact rewrites.
"""

from __future__ import annotations

from pathlib import Path

import pytest
from conftest import create_task
from yaklib import reparent as rp
from yaklib.model import find_task_file, load_task


def _yaks(root: Path) -> Path:
    return root / ".yaks"


def _child(yak, parent: str) -> str:
    out = yak("create", "--title", "c", "--type", "task", "--parent", parent).stdout
    return out.splitlines()[0].split()[1].rstrip(":")


def _parent_field(root: Path, tid: str) -> str | None:
    _, path = find_task_file(root, tid)
    return load_task(path).get("parent")


def test_reparent_promote_clears_parent_keeps_id(yak, yak_root):
    parent = create_task(yak, "p", type="feature")
    child = _child(yak, parent)
    assert _parent_field(_yaks(yak_root), child) == parent

    res = rp.reparent(_yaks(yak_root), child, None)
    assert res.old_parent == parent and res.new_parent is None
    # ID is unchanged; the yak is still found at the same id, now top-level.
    assert find_task_file(_yaks(yak_root), child) is not None
    assert _parent_field(_yaks(yak_root), child) is None


def test_reparent_moves_under_new_parent(yak, yak_root):
    p1 = create_task(yak, "p1", type="feature")
    p2 = create_task(yak, "p2", type="feature")
    child = _child(yak, p1)

    rp.reparent(_yaks(yak_root), child, p2)
    assert _parent_field(_yaks(yak_root), child) == p2


def test_reparent_children_ride_along(yak, yak_root):
    root_p = create_task(yak, "root", type="feature")
    mid = _child(yak, root_p)
    leaf = _child(yak, mid)

    # Promote mid to top-level; leaf still points at mid (unchanged), so the
    # subtree moves with it for free — no cascade, no ID churn.
    rp.reparent(_yaks(yak_root), mid, None)
    assert _parent_field(_yaks(yak_root), mid) is None
    assert _parent_field(_yaks(yak_root), leaf) == mid


def test_reparent_refuses_cycle(yak, yak_root):
    root_p = create_task(yak, "root", type="feature")
    mid = _child(yak, root_p)
    with pytest.raises(rp.ReparentError, match="own descendant"):
        rp.reparent(_yaks(yak_root), root_p, mid)


def test_reparent_refuses_self(yak, yak_root):
    t = create_task(yak, "t", type="task")
    with pytest.raises(rp.ReparentError, match="under itself"):
        rp.reparent(_yaks(yak_root), t, t)


def test_reparent_refuses_unknown_new_parent(yak, yak_root):
    t = create_task(yak, "t", type="task")
    with pytest.raises(rp.ReparentError, match="not found"):
        rp.reparent(_yaks(yak_root), t, "nope-0000")


def test_reparent_refuses_already_toplevel_unparent(yak, yak_root):
    t = create_task(yak, "t", type="task")
    with pytest.raises(rp.ReparentError, match="already a top-level"):
        rp.reparent(_yaks(yak_root), t, None)
