"""Unit tests for yaklib.deps (cycle detection, ready/tangled)."""

from __future__ import annotations

from pathlib import Path

from conftest import create_task
from yaklib import deps
from yaklib.model import find_task_file, load_task, save_task


def _add_dep(root: Path, task_id: str, dep_id: str) -> None:
    _, path = find_task_file(root, task_id)
    t = load_task(path)
    t.setdefault("depends_on", []).append(dep_id)
    save_task(path, t)


def test_depends_on_transitively_direct(yak, yak_root):
    a = create_task(yak, "a", type="task")
    b = create_task(yak, "b", type="task")
    _add_dep(yak_root / ".yaks", b, a)
    assert deps.depends_on_transitively(yak_root / ".yaks", b, a)
    assert not deps.depends_on_transitively(yak_root / ".yaks", a, b)


def test_depends_on_transitively_chain(yak, yak_root):
    a = create_task(yak, "a", type="task")
    b = create_task(yak, "b", type="task")
    c = create_task(yak, "c", type="task")
    root = yak_root / ".yaks"
    _add_dep(root, b, a)
    _add_dep(root, c, b)
    assert deps.depends_on_transitively(root, c, a)
    assert deps.depends_on_transitively(root, c, b)
    assert not deps.depends_on_transitively(root, a, c)


def test_depends_on_transitively_handles_cycles(yak, yak_root):
    a = create_task(yak, "a", type="task")
    b = create_task(yak, "b", type="task")
    root = yak_root / ".yaks"
    # Forge a cycle by hand (the CLI rejects these, but the traversal must
    # still terminate if one exists).
    _add_dep(root, a, b)
    _add_dep(root, b, a)
    assert deps.depends_on_transitively(root, a, b)
    # Unrelated id should return False without looping forever.
    assert not deps.depends_on_transitively(root, a, "nonexistent")


def test_ready_and_tangled(yak, yak_root):
    a = create_task(yak, "blocker", type="task")
    b = create_task(yak, "dependent", type="task")
    c = create_task(yak, "free", type="task")
    root = yak_root / ".yaks"
    _add_dep(root, b, a)

    ready = {t["id"] for t in deps.ready_tasks(root)}
    assert a in ready and c in ready
    assert b not in ready

    tangled = {t["id"] for t, _ in deps.tangled_tasks(root)}
    assert tangled == {b}


def test_resolved_ids_includes_dead(yak, yak_root):
    a = create_task(yak, "slaughterable", type="task")
    b = create_task(yak, "dependent", type="task")
    root = yak_root / ".yaks"
    _add_dep(root, b, a)

    yak("slaughter", a)

    resolved = deps.resolved_ids(root)
    assert a in resolved
    ready = {t["id"] for t in deps.ready_tasks(root)}
    assert b in ready
