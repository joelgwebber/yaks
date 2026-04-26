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


def test_update_last_synced_explicit_iso(yak):
    tid = create_task(yak, "synced-task", type="task")
    stamp = "2026-04-25T12:00:00Z"
    yak("update", tid, "--last-synced", stamp)
    t = yak("show", tid, "--json").json()
    assert t["last_synced"] == stamp


def test_update_last_synced_now_uses_current_time(yak):
    tid = create_task(yak, "synced-now", type="task")
    yak("update", tid, "--last-synced", "now")
    t = yak("show", tid, "--json").json()
    assert t["last_synced"]
    # Looks like an ISO timestamp.
    assert t["last_synced"].startswith("20")
    assert "T" in t["last_synced"]


def test_update_preserves_last_synced_across_routine_edits(yak):
    """Routine field edits (--note, --add-label, etc.) must not trample
    last_synced — only an explicit --last-synced rewrites it."""
    tid = create_task(yak, "preserve", type="task")
    yak("update", tid, "--last-synced", "2026-04-25T12:00:00Z")
    yak("update", tid, "--note", "did some work")
    yak("update", tid, "--add-label", "urgent")
    t = yak("show", tid, "--json").json()
    assert t["last_synced"] == "2026-04-25T12:00:00Z"


def test_update_note_appends_block(yak):
    tid = create_task(yak, "notes", type="task")
    yak("update", tid, "--note", "first observation")
    yak("update", tid, "--note", "second observation")
    body = yak("show", tid, "--json").json()["description"]
    assert "first observation" in body
    assert "second observation" in body
    # Each note gets its own sigil-marked block (preceded by a thematic break).
    assert body.count("---\n▸ ") == 2


def test_ids_are_unique_across_batch(yak):
    ids = {create_task(yak, f"t{i}", type="task") for i in range(10)}
    assert len(ids) == 10


def test_migrate_comment_blocks_pure_helper():
    from yaklib.model import _migrate_comment_blocks
    src = (
        "---\n"
        "id: x-1\n"
        "title: t\n"
        "---\n"
        "\n"
        "Body paragraph.\n"
        "\n"
        "### 2026-04-25T18:25:28Z\n"
        "Local note.\n"
        "\n"
        "### 2026-04-25T18:30:00Z @alice (from linear:ROC-9)\n"
        "Ferried comment.\n"
    )
    out = _migrate_comment_blocks(src)
    assert "### 2026-04-25" not in out
    assert "---\n▸ 2026-04-25T18:25:28Z\n" in out
    assert "---\n▸ 2026-04-25T18:30:00Z @alice (from linear:ROC-9)\n" in out
    # Idempotent: running again is a no-op.
    assert _migrate_comment_blocks(out) == out


def test_migrate_comment_blocks_leaves_date_only_headings_alone():
    """A bare `### 2026-04-25` (date, no time) is not a comment marker."""
    from yaklib.model import _migrate_comment_blocks
    src = (
        "---\n"
        "id: x-1\n"
        "---\n"
        "\n"
        "### 2026-04-25 Meeting notes\n"
        "User-authored heading; do not migrate.\n"
    )
    assert _migrate_comment_blocks(src) == src


def test_auto_migrate_rewrites_md_files_on_load(yak, yak_root):
    """find_tasks_root → _auto_migrate must rewrite legacy `### <iso>` blocks
    in any .md file under the .yaks/ directory."""
    tid = create_task(yak, "needs migration", type="task")
    path = yak_root / ".yaks" / "hairy" / f"{tid}.md"
    text = path.read_text()
    # Hand-write a legacy comment block (simulating an old yak file).
    legacy = text.rstrip() + "\n\n### 2026-04-25T18:25:28Z\nLegacy note.\n"
    path.write_text(legacy)
    # Any subsequent yak invocation triggers find_tasks_root → _auto_migrate.
    yak("list")
    rewritten = path.read_text()
    assert "### 2026-04-25" not in rewritten
    assert "---\n▸ 2026-04-25T18:25:28Z\nLegacy note." in rewritten
