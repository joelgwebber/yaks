"""Golden CLI tests: status transitions, deps, next/tangled, slaughter/revive."""

from __future__ import annotations

from pathlib import Path

from conftest import create_task


def test_shave_shorn_regrow_moves_files(yak, yak_root):
    tid = create_task(yak, "flow", type="task")
    base = yak_root / ".yaks"

    assert (base / "hairy" / f"{tid}.md").exists()
    yak("shave", tid)
    assert (base / "shaving" / f"{tid}.md").exists()
    assert not (base / "hairy" / f"{tid}.md").exists()

    yak("shorn", tid)
    assert (base / "shorn" / f"{tid}.md").exists()

    yak("regrow", tid)
    assert (base / "hairy" / f"{tid}.md").exists()


def test_slaughter_hides_from_default_queries(yak):
    a = create_task(yak, "doomed", type="task")
    b = create_task(yak, "kept", type="task")
    yak("slaughter", a)

    ids = {t["id"] for t in yak("list", "--json").json()}
    assert a not in ids
    assert b in ids

    # But dead status is reachable explicitly
    dead = {t["id"] for t in yak("list", "--status", "dead", "--json").json()}
    assert a in dead


def test_revive_brings_back_to_hairy(yak):
    tid = create_task(yak, "zombie", type="task")
    yak("slaughter", tid)
    yak("revive", tid)
    hairy = {t["id"] for t in yak("list", "--status", "hairy", "--json").json()}
    assert tid in hairy


def test_dep_add_blocks_next(yak):
    a = create_task(yak, "blocker", type="task")
    b = create_task(yak, "blocked", type="task")
    yak("dep", "add", b, a)

    ready = {t["id"] for t in yak("next", "--json").json()}
    assert a in ready
    assert b not in ready

    tangled = {t["id"] for t in yak("tangled", "--json").json()}
    assert b in tangled


def test_shorn_dep_unblocks_dependent(yak):
    a = create_task(yak, "blocker", type="task")
    b = create_task(yak, "waits", type="task")
    yak("dep", "add", b, a)
    yak("shave", a)
    yak("shorn", a)

    ready = {t["id"] for t in yak("next", "--json").json()}
    assert b in ready


def test_slaughtered_dep_also_unblocks(yak):
    a = create_task(yak, "will-die", type="task")
    b = create_task(yak, "dependent", type="task")
    yak("dep", "add", b, a)
    yak("slaughter", a)

    ready = {t["id"] for t in yak("next", "--json").json()}
    assert b in ready


def test_dep_remove(yak):
    a = create_task(yak, "blocker", type="task")
    b = create_task(yak, "dependent", type="task")
    yak("dep", "add", b, a)
    yak("dep", "remove", b, a)

    ready = {t["id"] for t in yak("next", "--json").json()}
    assert b in ready


def test_child_task_ids_use_parent_prefix(yak):
    parent = create_task(yak, "parent", type="feature")
    child_out = yak("create", "--title", "kid", "--type", "task",
                    "--parent", parent).stdout
    child_id = child_out.splitlines()[0].split()[1].rstrip(":")
    assert child_id.startswith(parent + ".")
    assert child_id == f"{parent}.1"

    # Second child increments
    child2 = yak("create", "--title", "kid2", "--type", "task",
                 "--parent", parent).stdout
    child2_id = child2.splitlines()[0].split()[1].rstrip(":")
    assert child2_id == f"{parent}.2"


def test_search_finds_by_title_and_description(yak):
    create_task(yak, "alpha widget", type="task")
    create_task(yak, "beta gadget", type="task",
                description="has alpha in body")

    hits = yak("search", "alpha", "--json").json()
    titles = {t["title"] for t in hits}
    assert "alpha widget" in titles
    assert "beta gadget" in titles
