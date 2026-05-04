"""Pure-function tests for yaktui — no curses involved.

Covers build_tree (ghost ancestors/descendants, filter modes, search)
and build_detail_lines (sections, links, artifacts).
"""

from __future__ import annotations

from pathlib import Path

from conftest import create_task
from yaklib.filter import FilterSpec
from yaklib.model import find_task_file, load_task, save_task
from yaktui.detail import build_detail_lines
from yaktui.keys_detail import dedent_block
from yaktui.mutate import TemplateParseError, parse_template
from yaktui.tree import apply_collapse, build_tree


def _yaks(root: Path) -> Path:
    return root / ".yaks"


def test_build_tree_shows_primary_tasks(yak, yak_root):
    a = create_task(yak, "alpha", type="task")
    b = create_task(yak, "beta", type="task")
    flat = build_tree(_yaks(yak_root), "hairy", FilterSpec())
    ids = [t["id"] for _, t, _, _ in flat]
    assert a in ids and b in ids


def test_build_tree_includes_ghost_descendants(yak, yak_root):
    """A shaving parent should still reveal its hairy child as a ghost."""
    parent = create_task(yak, "parent", type="feature")
    child_out = yak("create", "--title", "kid", "--type", "task",
                    "--parent", parent).stdout
    child_id = child_out.splitlines()[0].split()[1].rstrip(":")

    yak("shave", parent)

    flat = build_tree(_yaks(yak_root), "shaving", FilterSpec())
    by_id = {t["id"]: (s, ghost) for s, t, _, ghost in flat}
    assert parent in by_id and by_id[parent][1] is False
    assert child_id in by_id and by_id[child_id][1] is True


def test_build_tree_search_matches_across_statuses(yak, yak_root):
    a = create_task(yak, "alpha widget", type="task")
    b = create_task(yak, "beta thing", type="task")
    yak("shave", a)

    flat = build_tree(_yaks(yak_root), None,
                      FilterSpec(search="widget"))
    ids = [t["id"] for _, t, _, _ in flat]
    assert a in ids
    assert b not in ids


def test_build_detail_lines_has_sections(yak, yak_root, tmp_path):
    parent = create_task(yak, "parent", type="feature")
    blocker = create_task(yak, "blocker", type="task")
    dep = create_task(yak, "dep", type="task")
    yak("dep", "add", parent, blocker)
    yak("create", "--title", "kid", "--type", "task", "--parent", parent)

    # Attach an artifact to exercise that branch.
    src = tmp_path / "pic.png"
    src.write_bytes(b"x")
    yak("attach", parent, str(src))

    _, path = find_task_file(_yaks(yak_root), parent)
    t = load_task(path)
    lines = build_detail_lines(_yaks(yak_root), t, "hairy", width=100)
    kinds = [l.kind for l in lines]
    texts = [l.text for l in lines]

    assert "header" in kinds
    assert any("Title:" in x for x in texts)
    assert any("Depends on:" in x for x in texts)
    assert any("Children:" in x for x in texts)
    assert any("Artifacts:" in x for x in texts)


def test_build_detail_lines_artifact_is_openable(yak, yak_root, tmp_path):
    tid = create_task(yak, "shot", type="task")
    src = tmp_path / "s.png"
    src.write_bytes(b"x")
    yak("attach", tid, str(src))

    _, path = find_task_file(_yaks(yak_root), tid)
    t = load_task(path)
    lines = build_detail_lines(_yaks(yak_root), t, "hairy", width=100)
    open_lines = [l for l in lines if l.open_path is not None]
    assert len(open_lines) == 1
    assert open_lines[0].open_path.name == "s.png"
    assert open_lines[0].is_link


def _inline_link_ids(lines):
    return [tid for l in lines for _, _, tid in l.links]


def test_build_detail_lines_inline_link_bare_mention(yak, yak_root):
    """A bare yak-ID in the body becomes a navigable inline span."""
    other = yak("create", "--title", "sidecar", "--type", "task").stdout
    other_id = other.splitlines()[0].split()[1].rstrip(":")

    tid = yak("create", "--title", "host", "--type", "task",
              "--description", f"see {other_id} for details").stdout
    tid = tid.splitlines()[0].split()[1].rstrip(":")

    _, path = find_task_file(_yaks(yak_root), tid)
    t = load_task(path)
    lines = build_detail_lines(_yaks(yak_root), t, "hairy", width=100)

    assert other_id in _inline_link_ids(lines)
    # No dedicated References subheader anymore.
    assert not any("References:" in l.text for l in lines)


def test_build_detail_lines_inline_link_wiki_form(yak, yak_root):
    """[[yak-xxxx]] is stripped for display but still resolves to a span."""
    other = yak("create", "--title", "wiki target", "--type", "task").stdout
    other_id = other.splitlines()[0].split()[1].rstrip(":")

    tid = yak("create", "--title", "host", "--type", "task",
              "--description", f"ref: [[{other_id}]] here").stdout
    tid = tid.splitlines()[0].split()[1].rstrip(":")

    _, path = find_task_file(_yaks(yak_root), tid)
    t = load_task(path)
    lines = build_detail_lines(_yaks(yak_root), t, "hairy", width=100)

    # Brackets gone from displayed text.
    assert all("[[" not in l.text for l in lines)
    assert all("]]" not in l.text for l in lines)
    # Link still picked up.
    assert other_id in _inline_link_ids(lines)


def test_build_detail_lines_inline_link_skips_self_and_unknown(yak, yak_root):
    """Self-references and nonexistent IDs do not produce spans."""
    tid = yak("create", "--title", "solo", "--type", "task",
              "--description", "mentions test-ffff (nope) and self below").stdout
    tid = tid.splitlines()[0].split()[1].rstrip(":")

    # Amend the description to include its own id
    from yaklib.model import save_task
    _, path = find_task_file(_yaks(yak_root), tid)
    t = load_task(path)
    t["description"] = f"self {tid} and test-ffff"
    save_task(path, t)

    t = load_task(path)
    lines = build_detail_lines(_yaks(yak_root), t, "hairy", width=100)
    assert _inline_link_ids(lines) == []


def test_build_detail_lines_inline_link_independent_of_explicit_dep(yak, yak_root):
    """An ID that's also an explicit dep still shows up inline in the body —
    the dep is in its own section; we're not suppressing mentions."""
    dep = yak("create", "--title", "dep", "--type", "task").stdout
    dep_id = dep.splitlines()[0].split()[1].rstrip(":")

    tid = yak("create", "--title", "host", "--type", "task",
              "--description", f"blocked on {dep_id}").stdout
    tid = tid.splitlines()[0].split()[1].rstrip(":")
    yak("dep", "add", tid, dep_id)

    _, path = find_task_file(_yaks(yak_root), tid)
    t = load_task(path)
    lines = build_detail_lines(_yaks(yak_root), t, "hairy", width=100)

    assert dep_id in _inline_link_ids(lines)
    # And also appears as a whole-line Depends on row.
    dep_rows = [l for l in lines if l.task_id == dep_id]
    assert len(dep_rows) == 1


def test_build_detail_lines_tags_depends_on_rows(yak, yak_root):
    """Depends-on rows are kind='dep_link' so the B hotkey can distinguish
    them from Parent / Children / Blocks / Artifacts links."""
    dep = yak("create", "--title", "blocker", "--type", "task").stdout
    dep_id = dep.splitlines()[0].split()[1].rstrip(":")
    tid = yak("create", "--title", "waiter", "--type", "task").stdout
    tid = tid.splitlines()[0].split()[1].rstrip(":")
    yak("dep", "add", tid, dep_id)

    _, path = find_task_file(_yaks(yak_root), tid)
    t = load_task(path)
    lines = build_detail_lines(_yaks(yak_root), t, "hairy", width=100)

    dep_rows = [l for l in lines if l.task_id == dep_id]
    assert len(dep_rows) == 1
    assert dep_rows[0].kind == "dep_link"


def test_build_detail_lines_blocks_reverse_deps(yak, yak_root):
    blocker = create_task(yak, "block", type="task")
    waiter = create_task(yak, "wait", type="task")
    yak("dep", "add", waiter, blocker)

    # Render the blocker's detail with reverse_deps populated.
    from yaklib import deps as _deps
    _, reverse = _deps.compute_blocked(_yaks(yak_root))
    _, path = find_task_file(_yaks(yak_root), blocker)
    t = load_task(path)
    lines = build_detail_lines(_yaks(yak_root), t, "hairy", width=100,
                               reverse_deps=reverse)

    assert any("Blocks:" in l.text for l in lines)
    link_ids = [l.task_id for l in lines if l.task_id]
    assert waiter in link_ids


def test_dedent_block_strips_common_indent():
    lines = ["    foo", "    bar", "", "      baz"]
    assert dedent_block(lines) == ["foo", "bar", "", "  baz"]


def test_dedent_block_handles_no_common_indent():
    lines = ["foo", "  bar"]
    assert dedent_block(lines) == ["foo", "  bar"]


def test_dedent_block_ignores_blank_lines():
    lines = ["", "  foo", "", "  bar", ""]
    assert dedent_block(lines) == ["", "foo", "", "bar", ""]


def _row(tid, depth=0):
    return ("hairy", {"id": tid, "title": tid}, depth, False)


def test_apply_collapse_hides_descendants_and_counts():
    flat = [_row("yak-1"), _row("yak-1.1", 1), _row("yak-1.1.2", 2),
            _row("yak-2")]
    visible, counts = apply_collapse(flat, {"yak-1"}, filter_active=False)
    assert [r[1]["id"] for r in visible] == ["yak-1", "yak-2"]
    assert counts == {"yak-1": 2}


def test_apply_collapse_skipped_when_filter_active():
    flat = [_row("yak-1"), _row("yak-1.1", 1), _row("yak-2")]
    visible, counts = apply_collapse(flat, {"yak-1"}, filter_active=True)
    assert [r[1]["id"] for r in visible] == ["yak-1", "yak-1.1", "yak-2"]
    assert counts == {}


def test_apply_collapse_noop_when_empty():
    flat = [_row("yak-1"), _row("yak-1.1", 1)]
    visible, counts = apply_collapse(flat, set(), filter_active=False)
    assert visible == flat and counts == {}


def test_apply_collapse_nested_counts_outer_not_double_inner():
    """When both ancestor and descendant are collapsed, the ancestor's count
    is the whole subtree — we don't hide rows twice or drop the inner."""
    flat = [_row("yak-1"), _row("yak-1.1", 1), _row("yak-1.1.2", 2)]
    visible, counts = apply_collapse(flat, {"yak-1", "yak-1.1"},
                                     filter_active=False)
    assert [r[1]["id"] for r in visible] == ["yak-1"]
    # Outer counts 2 descendants (child + grandchild); inner still logs its 1.
    assert counts["yak-1"] == 2
    assert counts["yak-1.1"] == 1


def test_parse_template_happy_path():
    text = ("---\n"
            "title: plain title\n"
            "type: task\n"
            "priority: 2\n"
            "---\n"
            "body text\n")
    data = parse_template(text)
    assert data["title"] == "plain title"
    assert data["type"] == "task"
    assert data["description"] == "body text"


def test_parse_template_empty_cancelled_returns_none():
    # No fence at all → cancelled, not an error.
    assert parse_template("") is None
    # Truncated: no closing fence.
    assert parse_template("---\ntitle: foo\n") is None


def test_parse_template_raises_on_unquoted_colon_in_title():
    """Regression for yak-7321: an unquoted ':' in the title used to be
    swallowed as 'create cancelled', silently dropping the user's work."""
    text = ("---\n"
            "title: Handle foo: bar edge case\n"
            "type: bug\n"
            "priority: 2\n"
            "---\n")
    try:
        parse_template(text)
    except TemplateParseError as e:
        # Single-line, suitable for a notification.
        assert "\n" not in str(e)
        assert str(e)
    else:
        raise AssertionError("expected TemplateParseError")


def test_parse_template_accepts_quoted_colon():
    text = ("---\n"
            "title: \"Handle foo: bar\"\n"
            "type: bug\n"
            "priority: 2\n"
            "---\n")
    data = parse_template(text)
    assert data["title"] == "Handle foo: bar"


# ---------------------------------------------------------------------------
# Sync-review dialog helpers (bf54.11 Phase 1) — pure, curses-free.
# ---------------------------------------------------------------------------


def test_sync_review_build_rows_unifies_fields_and_buckets():
    from yaktui.sync_review import _build_rows
    sidecar = {
        "fields": [{"name": "title"}, {"name": "priority"}],
        "comments_up": [{"body": "a"}, {"body": "b"}],
        "attachments_down": [{"filename": "x.png"}],
    }
    rows = _build_rows(sidecar)
    kinds = [r.kind for r in rows]
    assert kinds == ["field", "field", "comments_up", "comments_up",
                     "attachments_down"]
    assert rows[2].bucket_idx == 1 and rows[2].bucket_n == 2
    assert rows[3].bucket_idx == 2 and rows[3].bucket_n == 2


def test_sync_review_jump_bucket_skips_to_next_kind():
    from yaktui.sync_review import _build_rows, _jump_bucket
    sidecar = {
        "fields": [{"name": "title"}, {"name": "priority"}],
        "comments_up": [{"body": "a"}, {"body": "b"}],
        "attachments_down": [{"filename": "x.png"}],
    }
    rows = _build_rows(sidecar)
    assert _jump_bucket(rows, 0, +1) == 2
    assert _jump_bucket(rows, 3, +1) == 4
    assert _jump_bucket(rows, 4, +1) == 4
    assert _jump_bucket(rows, 2, -1) == 0


def test_sync_review_next_direction_cycles():
    from yaktui.sync_review import _next_direction
    assert _next_direction("upstream") == "local"
    assert _next_direction("local") == "pending"
    assert _next_direction("pending") == "upstream"
    assert _next_direction("garbage") == "upstream"


def test_sync_review_row_capability_prefers_sidecar_value():
    from yaktui.sync_review import Row, _row_capability
    row = Row(kind="field", item={"name": "description", "capability": "ok"})
    assert _row_capability(row, "jira") == "ok"
    # Missing → fall back to the matrix (jira description = lossy).
    row = Row(kind="field", item={"name": "description"})
    assert _row_capability(row, "jira") == "lossy"
    # Comments_down on any tracker is always "ok" — safe local write.
    row = Row(kind="comments_down", item={"body": "x"})
    assert _row_capability(row, "github") == "ok"


def test_sync_review_row_summary_prefers_merged_value():
    from yaktui.sync_review import Row, _row_summary
    row = Row(kind="field", item={"name": "title", "local": "L", "upstream": "U",
                                  "merged_value": "M"})
    assert _row_summary(row) == "merged: M"
    # When merged_value is None, falls through to local → upstream.
    row = Row(kind="field", item={"name": "title", "local": "L", "upstream": "U"})
    assert _row_summary(row) == "L → U"


def test_sync_review_row_label_marks_merged_with_star():
    from yaktui.sync_review import Row, _row_label
    plain = Row(kind="field", item={"name": "title"})
    edited = Row(kind="field", item={"name": "title", "merged_value": "x"})
    assert _row_label(plain) == "title"
    assert _row_label(edited) == "*title"


def test_sync_review_parse_pending_merge_extracts_post_divider():
    from yaktui.sync_review import _parse_pending_merge
    text = (
        "# === LOCAL (title) ===\n"
        "old title\n"
        "# === UPSTREAM ===\n"
        "raw upstream title\n"
        "# === MERGED (write your reconciliation below) ===\n"
        "reconciled title\n"
        "second line\n"
    )
    out = _parse_pending_merge(text)
    assert out == "reconciled title\nsecond line\n"


def test_sync_review_parse_pending_merge_handles_missing_divider():
    """User stripped the divider — return everything after leading comments."""
    from yaktui.sync_review import _parse_pending_merge
    text = "# leading comment\n# more\nactual content\nmore content\n"
    assert _parse_pending_merge(text) == "actual content\nmore content"


def test_sync_review_deserialize_field_handles_priority_int():
    from yaktui.sync_review import _deserialize_field
    assert _deserialize_field("priority", "2\n", 3) == 2
    # Non-numeric reverts to string (caller can validate).
    assert _deserialize_field("priority", "high", 3) == "high"


def test_sync_review_deserialize_field_handles_labels_list():
    from yaktui.sync_review import _deserialize_field
    assert _deserialize_field("labels", "bug\nurgent\n\n", ["x"]) == ["bug", "urgent"]


def test_sync_review_deserialize_field_default_is_string():
    from yaktui.sync_review import _deserialize_field
    assert _deserialize_field("title", "new title\n", "old") == "new title"
