"""Golden CLI tests: init, create, list, show, update."""

from __future__ import annotations

from pathlib import Path

from conftest import _make_runner, create_task


def test_init_creates_structure(yak_root: Path):
    assert (yak_root / ".yaks").is_dir()
    assert (yak_root / ".yaks" / "config.yaml").is_file()
    for sub in ("hairy", "shaving", "shorn"):
        assert (yak_root / ".yaks" / sub).is_dir()


def test_init_creates_agents_md_by_default(yak_root: Path):
    """With no existing guidance file, init prefers AGENTS.md (not CLAUDE.md)."""
    agents = yak_root / "AGENTS.md"
    assert agents.is_file()
    assert "Yaks skill" in agents.read_text()
    assert not (yak_root / "CLAUDE.md").exists()


def test_init_appends_to_existing_claude_md(tmp_path: Path):
    """An existing CLAUDE.md is used as-is; no AGENTS.md is created."""
    claude = tmp_path / "CLAUDE.md"
    claude.write_text("# Project\n")
    _make_runner(tmp_path)("init", "--prefix", "test")
    assert "Yaks skill" in claude.read_text()
    assert not (tmp_path / "AGENTS.md").exists()


def test_init_agents_flag_forces_agents_md_over_claude(tmp_path: Path):
    """--agents writes to AGENTS.md even when a CLAUDE.md is present."""
    claude = tmp_path / "CLAUDE.md"
    claude.write_text("# Project\n")
    _make_runner(tmp_path)("init", "--prefix", "test", "--agents")
    assert "Yaks skill" in (tmp_path / "AGENTS.md").read_text()
    assert "Yaks skill" not in claude.read_text()


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
    yak("update", tid, "--title", "renamed", "--priority", "1", "--add-label", "urgent", "core")

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

    src = "---\nid: x-1\n---\n\n### 2026-04-25 Meeting notes\nUser-authored heading; do not migrate.\n"
    assert _migrate_comment_blocks(src) == src


def test_auto_migrate_runs_when_schema_behind(yak, yak_root):
    """When the herd's schema version is behind, the next invocation rewrites
    legacy `### <iso>` comment blocks and re-stamps the schema to current."""
    from yaklib.model import CURRENT_SCHEMA_VERSION

    tid = create_task(yak, "needs migration", type="task")
    path = yak_root / ".yaks" / "hairy" / f"{tid}.md"
    legacy = path.read_text().rstrip() + "\n\n### 2026-04-25T18:25:28Z\nLegacy note.\n"
    path.write_text(legacy)
    # Roll the schema back so the comment-block step (v2) re-runs.
    schema = yak_root / ".yaks" / "schema"
    schema.write_text("1\n")

    yak("list")

    rewritten = path.read_text()
    assert "### 2026-04-25" not in rewritten
    assert "---\n▸ 2026-04-25T18:25:28Z\nLegacy note." in rewritten
    assert schema.read_text().strip() == str(CURRENT_SCHEMA_VERSION)


def test_auto_migrate_skipped_when_schema_current(yak, yak_root):
    """The version gate skips the O(N) scan when the herd is already current,
    so a hand-injected legacy block is left untouched (the intended tradeoff:
    migrations run on version bumps, not on every invocation)."""
    tid = create_task(yak, "already current", type="task")
    path = yak_root / ".yaks" / "hairy" / f"{tid}.md"
    legacy = path.read_text().rstrip() + "\n\n### 2026-04-25T18:25:28Z\nLegacy note.\n"
    path.write_text(legacy)

    yak("list")  # init stamped the current version -> gate skips migration

    assert "### 2026-04-25T18:25:28Z" in path.read_text()


def test_init_stamps_current_schema_version(yak_root):
    from yaklib.model import CURRENT_SCHEMA_VERSION

    schema = yak_root / ".yaks" / "schema"
    assert schema.is_file()
    assert schema.read_text().strip() == str(CURRENT_SCHEMA_VERSION)


def test_migrate_v3_backfills_parent_field(yak, yak_root):
    """A legacy dotted child with no parent field gains one when the schema is
    behind, and the herd is re-stamped to current."""
    import yaml
    from yaklib.model import CURRENT_SCHEMA_VERSION

    parent = create_task(yak, "umbrella", type="feature")
    child_id = f"{parent}.1"
    child_path = yak_root / ".yaks" / "hairy" / f"{child_id}.md"
    child_path.write_text(
        f"---\nid: {child_id}\ntitle: legacy child\ntype: task\npriority: 3\n---\n\nbody\n"
    )
    # Roll the schema back so the v3 step re-runs on the next invocation.
    (yak_root / ".yaks" / "schema").write_text("2\n")

    yak("list")

    fm = yaml.safe_load(child_path.read_text().split("---")[1])
    assert fm["parent"] == parent
    assert (yak_root / ".yaks" / "schema").read_text().strip() == str(CURRENT_SCHEMA_VERSION)
