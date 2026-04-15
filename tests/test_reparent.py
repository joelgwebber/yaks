"""Tests for yaklib.reparent — focus on link-integrity rewrites that
the pre-refactor implementation missed (inline body mentions, artifact
directories, [[wiki]] form, self-references, collision detection)."""

from __future__ import annotations

import shutil
from pathlib import Path

import pytest

from conftest import create_task
from yaklib import reparent as rp
from yaklib.model import find_task_file, load_task, save_task


def _yaks(root: Path) -> Path:
    return root / ".yaks"


def _set_desc(root: Path, tid: str, desc: str) -> None:
    _, path = find_task_file(root, tid)
    t = load_task(path)
    t["description"] = desc
    save_task(path, t)


def _get_desc(root: Path, tid: str) -> str:
    _, path = find_task_file(root, tid)
    return load_task(path).get("description", "") or ""


def test_reparent_simple_promote(yak, yak_root):
    parent = create_task(yak, "p", type="feature")
    child_out = yak("create", "--title", "c", "--type", "task",
                    "--parent", parent).stdout
    child = child_out.splitlines()[0].split()[1].rstrip(":")
    assert child.startswith(parent + ".")

    plan = rp.reparent(_yaks(yak_root), child, None)
    assert plan.new_id != child
    assert "." not in plan.new_id[plan.new_id.index("-") + 1:]
    assert find_task_file(_yaks(yak_root), child) is None
    assert find_task_file(_yaks(yak_root), plan.new_id) is not None


def test_reparent_rewrites_third_party_dep(yak, yak_root):
    parent = create_task(yak, "p", type="feature")
    child_out = yak("create", "--title", "c", "--type", "task",
                    "--parent", parent).stdout
    child = child_out.splitlines()[0].split()[1].rstrip(":")

    other = create_task(yak, "o", type="task")
    yak("dep", "add", other, child)

    plan = rp.reparent(_yaks(yak_root), child, None)

    _, path = find_task_file(_yaks(yak_root), other)
    t = load_task(path)
    assert t["depends_on"] == [plan.new_id]


def test_reparent_rewrites_bare_body_mentions(yak, yak_root):
    parent = create_task(yak, "p", type="feature")
    child_out = yak("create", "--title", "c", "--type", "task",
                    "--parent", parent).stdout
    child = child_out.splitlines()[0].split()[1].rstrip(":")

    referrer = create_task(yak, "r", type="task")
    _set_desc(_yaks(yak_root), referrer, f"see {child} for details")

    plan = rp.reparent(_yaks(yak_root), child, None)

    desc = _get_desc(_yaks(yak_root), referrer)
    assert child not in desc
    assert plan.new_id in desc


def test_reparent_rewrites_wiki_form_mentions(yak, yak_root):
    parent = create_task(yak, "p", type="feature")
    child_out = yak("create", "--title", "c", "--type", "task",
                    "--parent", parent).stdout
    child = child_out.splitlines()[0].split()[1].rstrip(":")

    referrer = create_task(yak, "r", type="task")
    _set_desc(_yaks(yak_root), referrer, f"per [[{child}]] spec")

    plan = rp.reparent(_yaks(yak_root), child, None)

    desc = _get_desc(_yaks(yak_root), referrer)
    assert f"[[{plan.new_id}]]" in desc
    assert child not in desc


def test_reparent_rewrites_self_reference(yak, yak_root):
    parent = create_task(yak, "p", type="feature")
    child_out = yak("create", "--title", "c", "--type", "task",
                    "--parent", parent).stdout
    child = child_out.splitlines()[0].split()[1].rstrip(":")

    _set_desc(_yaks(yak_root), child, f"I am {child}")

    plan = rp.reparent(_yaks(yak_root), child, None)

    desc = _get_desc(_yaks(yak_root), plan.new_id)
    assert child not in desc
    assert plan.new_id in desc


def test_reparent_renames_artifact_dir_and_body_refs(yak, yak_root, tmp_path):
    parent = create_task(yak, "p", type="feature")
    child_out = yak("create", "--title", "c", "--type", "task",
                    "--parent", parent).stdout
    child = child_out.splitlines()[0].split()[1].rstrip(":")

    src = tmp_path / "shot.png"
    src.write_bytes(b"x")
    yak("attach", child, str(src))

    art_old = _yaks(yak_root) / "artifacts" / child
    assert art_old.is_dir()

    plan = rp.reparent(_yaks(yak_root), child, None)

    art_new = _yaks(yak_root) / "artifacts" / plan.new_id
    assert art_new.is_dir()
    assert (art_new / "shot.png").is_file()
    assert not art_old.exists()

    desc = _get_desc(_yaks(yak_root), plan.new_id)
    assert f"artifacts/{plan.new_id}/shot.png" in desc
    assert f"artifacts/{child}/" not in desc


def test_reparent_deep_subtree(yak, yak_root):
    root_p = create_task(yak, "root", type="feature")
    mid_out = yak("create", "--title", "mid", "--type", "task",
                  "--parent", root_p).stdout
    mid = mid_out.splitlines()[0].split()[1].rstrip(":")
    leaf_out = yak("create", "--title", "leaf", "--type", "task",
                   "--parent", mid).stdout
    leaf = leaf_out.splitlines()[0].split()[1].rstrip(":")

    # Reparent mid to be a child of root_p still... no, promote it.
    plan = rp.reparent(_yaks(yak_root), mid, None)
    # leaf's new id should share the new mid id as its prefix.
    leaf_new = plan.id_map[leaf]
    assert leaf_new.startswith(plan.new_id + ".")
    assert find_task_file(_yaks(yak_root), leaf_new) is not None
    assert find_task_file(_yaks(yak_root), leaf) is None


def test_reparent_refuses_cycle(yak, yak_root):
    root_p = create_task(yak, "root", type="feature")
    mid_out = yak("create", "--title", "mid", "--type", "task",
                  "--parent", root_p).stdout
    mid = mid_out.splitlines()[0].split()[1].rstrip(":")

    with pytest.raises(rp.ReparentError, match="own descendant"):
        rp.reparent(_yaks(yak_root), root_p, mid)


def test_reparent_refuses_unknown_new_parent(yak, yak_root):
    t = create_task(yak, "t", type="task")
    with pytest.raises(rp.ReparentError, match="not found"):
        rp.reparent(_yaks(yak_root), t, "nope-0000")


def test_reparent_refuses_already_toplevel_unparent(yak, yak_root):
    t = create_task(yak, "t", type="task")
    with pytest.raises(rp.ReparentError, match="already a top-level"):
        rp.reparent(_yaks(yak_root), t, None)


def test_reparent_detects_artifact_dir_collision(yak, yak_root, tmp_path):
    """If someone has a stray artifacts/{new_id}/ lying around (shouldn't
    happen in practice, but the check guards the common foot-gun), the
    operation refuses rather than overwriting."""
    parent = create_task(yak, "p", type="feature")
    child_out = yak("create", "--title", "c", "--type", "task",
                    "--parent", parent).stdout
    child = child_out.splitlines()[0].split()[1].rstrip(":")

    src = tmp_path / "shot.png"
    src.write_bytes(b"x")
    yak("attach", child, str(src))

    # Precompute what new_id would be and plant a colliding dir there.
    plan = rp.plan_reparent(_yaks(yak_root), child, None)
    (_yaks(yak_root) / "artifacts" / plan.new_id).mkdir(parents=True)

    with pytest.raises(rp.ReparentError, match="already exists"):
        rp.apply(plan, _yaks(yak_root))

    # Also sanity-check the original state was not mutated (file still at
    # old id, artifact dir still owned by old id).
    assert find_task_file(_yaks(yak_root), child) is not None
    assert (_yaks(yak_root) / "artifacts" / child).is_dir()
