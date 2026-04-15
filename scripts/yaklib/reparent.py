"""Reparent a yak (and its descendants) with full link-integrity rewrite.

Reparenting changes a yak's ID, which cascades to:
- its file name
- its descendants' IDs + file names
- every `depends_on` list that referenced any of the renamed IDs
- every inline yak-ID mention in any body (bare or [[wiki]] form)
- the renamed yaks' artifact directories (.yaks/artifacts/{id}/)
- markdown `![](artifacts/{id}/...)` refs inside bodies

Implementation is split into plan_reparent (pure: validate + build id_map)
and apply (impure: collision check, renames, rewrites). Callers should
surface ReparentError messages to users; apply is still best-effort and
will leave the tree in an inconsistent state if the OS errors mid-rename.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

from yaklib.links import BARE_LINK_RE, EXPLICIT_LINK_RE
from yaklib.model import (
    _ALL_STATUSES,
    find_descendants,
    find_task_file,
    generate_id,
    load_config,
    load_task,
    next_child_number,
    now_iso,
    parent_id,
    save_task,
)


class ReparentError(Exception):
    """Raised by plan_reparent / apply when the operation can't proceed."""


@dataclass
class ReparentPlan:
    old_id: str
    new_id: str
    id_map: dict[str, str]
    artifact_dirs: list[tuple[str, str]] = field(default_factory=list)


def plan_reparent(root: Path, old_id: str,
                  new_parent: str | None) -> ReparentPlan:
    """Validate and compute the id_map for a reparent operation.

    new_parent=None means "unparent" (promote to a fresh top-level ID).
    """
    if find_task_file(root, old_id) is None:
        raise ReparentError(f"task {old_id} not found")

    if new_parent is not None:
        if new_parent == old_id or new_parent.startswith(old_id + "."):
            raise ReparentError("cannot reparent under own descendant")
        if parent_id(old_id) == new_parent:
            raise ReparentError(f"{old_id} is already a child of {new_parent}")
        if find_task_file(root, new_parent) is None:
            raise ReparentError(f"parent task {new_parent} not found")
        new_id = f"{new_parent}.{next_child_number(root, new_parent)}"
    else:
        if parent_id(old_id) is None:
            raise ReparentError(f"{old_id} is already a top-level task")
        cfg = load_config(root)
        prefix = cfg.get("prefix", "yak")
        new_id = generate_id(root, prefix)

    id_map = {old_id: new_id}
    for _, p in find_descendants(root, old_id):
        desc_old = p.stem
        desc_new = new_id + desc_old[len(old_id):]
        id_map[desc_old] = desc_new

    art_dirs = [
        (old, new) for old, new in id_map.items()
        if (root / "artifacts" / old).is_dir()
    ]

    return ReparentPlan(old_id=old_id, new_id=new_id, id_map=id_map,
                        artifact_dirs=art_dirs)


def _rewrite_ids_in_text(text: str, id_map: dict[str, str]) -> str:
    """Rewrite [[old]] and bare `old` occurrences (including artifact-path
    fragments like `artifacts/old/foo.png`) through *id_map*."""
    def _sub_wiki(m):
        tid = m.group(1)
        return f"[[{id_map.get(tid, tid)}]]"

    def _sub_bare(m):
        tid = m.group(1)
        return id_map.get(tid, tid)

    text = EXPLICIT_LINK_RE.sub(_sub_wiki, text)
    text = BARE_LINK_RE.sub(_sub_bare, text)
    return text


def _check_collisions(root: Path, plan: ReparentPlan) -> None:
    """Fail fast if any destination file or artifact dir already exists."""
    for old, new in plan.id_map.items():
        res = find_task_file(root, old)
        if not res:
            continue
        _, src = res
        dst = src.parent / f"{new}.md"
        if dst.exists():
            raise ReparentError(f"destination {dst.relative_to(root)} already exists")
    for old, new in plan.artifact_dirs:
        if (root / "artifacts" / new).exists():
            raise ReparentError(
                f"artifact dir {('artifacts/' + new)} already exists")


def apply(plan: ReparentPlan, root: Path) -> None:
    """Execute the plan: rename files + artifact dirs, rewrite all bodies.

    Raises ReparentError before touching anything if a collision is detected.
    After that, OS failures (disk full, permissions) can leave a partial state.
    """
    _check_collisions(root, plan)
    now = now_iso()

    # Phase 1: rename the reparented subtree's files and bump their id field.
    for old, new in plan.id_map.items():
        res = find_task_file(root, old)
        if not res:
            continue
        _, path = res
        task = load_task(path)
        task["id"] = new
        task["updated"] = now
        save_task(path.parent / f"{new}.md", task)
        path.unlink()

    # Phase 2: rename artifact directories whose owner id changed.
    for old, new in plan.artifact_dirs:
        src = root / "artifacts" / old
        if src.exists():
            src.rename(root / "artifacts" / new)

    # Phase 3: rewrite every yak's depends_on + description (including dead,
    # so slaughtered-yak references also get fixed).
    for s in _ALL_STATUSES:
        d = root / s
        if not d.exists():
            continue
        for f in d.glob("*.md"):
            task = load_task(f)
            changed = False

            deps = task.get("depends_on", [])
            if deps:
                new_deps = [plan.id_map.get(dep, dep) for dep in deps]
                if new_deps != deps:
                    task["depends_on"] = new_deps
                    changed = True

            desc = task.get("description")
            if desc:
                new_desc = _rewrite_ids_in_text(desc, plan.id_map)
                if new_desc != desc:
                    task["description"] = new_desc
                    changed = True

            if changed:
                task["updated"] = now
                save_task(f, task)


def reparent(root: Path, old_id: str, new_parent: str | None) -> ReparentPlan:
    """Convenience one-shot: plan + apply. Returns the plan for reporting."""
    plan = plan_reparent(root, old_id, new_parent)
    apply(plan, root)
    return plan
