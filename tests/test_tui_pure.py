"""Pure-function tests for yaktui — no curses involved.

Covers build_tree (ghost ancestors/descendants, filter modes, search)
and build_detail_lines (sections, links, artifacts).
"""

from __future__ import annotations

from pathlib import Path

from conftest import create_task
from yaklib.model import find_task_file, load_task, save_task
from yaktui.detail import build_detail_lines
from yaktui.tree import build_tree


def _yaks(root: Path) -> Path:
    return root / ".yaks"


def test_build_tree_shows_primary_tasks(yak, yak_root):
    a = create_task(yak, "alpha", type="task")
    b = create_task(yak, "beta", type="task")
    flat = build_tree(_yaks(yak_root), status_filter="hairy",
                      filter_mode="all", search_query="")
    ids = [t["id"] for _, t, _, _ in flat]
    assert a in ids and b in ids


def test_build_tree_includes_ghost_descendants(yak, yak_root):
    """A shaving parent should still reveal its hairy child as a ghost."""
    parent = create_task(yak, "parent", type="feature")
    child_out = yak("create", "--title", "kid", "--type", "task",
                    "--parent", parent).stdout
    child_id = child_out.splitlines()[0].split()[1].rstrip(":")

    yak("shave", parent)

    flat = build_tree(_yaks(yak_root), status_filter="shaving",
                      filter_mode="all", search_query="")
    by_id = {t["id"]: (s, ghost) for s, t, _, ghost in flat}
    assert parent in by_id and by_id[parent][1] is False
    assert child_id in by_id and by_id[child_id][1] is True


def test_build_tree_search_matches_across_statuses(yak, yak_root):
    a = create_task(yak, "alpha widget", type="task")
    b = create_task(yak, "beta thing", type="task")
    yak("shave", a)

    flat = build_tree(_yaks(yak_root), status_filter=None,
                      filter_mode="all", search_query="widget")
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
