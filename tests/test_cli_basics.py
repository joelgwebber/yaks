"""Golden CLI tests: init, create, list, show, update."""

from __future__ import annotations

from pathlib import Path

from conftest import create_task


def test_init_creates_structure(yak_root: Path):
    assert (yak_root / ".yaks").is_dir()
    assert (yak_root / ".yaks" / "config.yaml").is_file()
    for sub in ("hairy", "shaving", "shorn"):
        assert (yak_root / ".yaks" / sub).is_dir()


def test_create_and_show(yak, yak_root):
    tid = create_task(yak, "first task", type="task", priority=2)
    assert tid.startswith("test-")
    path = yak_root / ".yaks" / "hairy" / f"{tid}.md"
    assert path.is_file()

    show = yak("show", tid, "--json").json()
    assert show["id"] == tid
    assert show["title"] == "first task"
    assert show["type"] == "task"
    assert show["priority"] == 2


def test_list_filters_by_status(yak):
    a = create_task(yak, "A", type="task")
    b = create_task(yak, "B", type="task")
    yak("shave", a)

    hairy = yak("list", "--status", "hairy", "--json").json()
    shaving = yak("list", "--status", "shaving", "--json").json()

    hairy_ids = {t["id"] for t in hairy}
    shaving_ids = {t["id"] for t in shaving}
    assert a in shaving_ids and a not in hairy_ids
    assert b in hairy_ids and b not in shaving_ids


def test_update_fields(yak):
    tid = create_task(yak, "original", type="task", priority=3)
    yak("update", tid, "--title", "renamed", "--priority", "1",
        "--add-label", "urgent", "core")

    t = yak("show", tid, "--json").json()
    assert t["title"] == "renamed"
    assert t["priority"] == 1
    assert set(t["labels"]) == {"urgent", "core"}


def test_update_note_appends_block(yak):
    tid = create_task(yak, "notes", type="task")
    yak("update", tid, "--note", "first observation")
    yak("update", tid, "--note", "second observation")
    body = yak("show", tid, "--json").json()["description"]
    assert "first observation" in body
    assert "second observation" in body
    # Each note gets its own timestamped heading
    assert body.count("###") == 2


def test_ids_are_unique_across_batch(yak):
    ids = {create_task(yak, f"t{i}", type="task") for i in range(10)}
    assert len(ids) == 10
