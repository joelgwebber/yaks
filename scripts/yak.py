# /// script
# requires-python = ">=3.10"
# dependencies = ["pyyaml>=6.0"]
# ///
"""Filesystem-native task tracker. Markdown files with YAML frontmatter, no database, no daemon."""

import argparse
import json
import re
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

# Make scripts/ importable so we can pull in yaklib.
sys.path.insert(0, str(Path(__file__).parent))

# Core model (status consts, YAML I/O, fs layout, ID arithmetic, git).
# Re-exported as `yak.X` for back-compat with tui.py until the final step.
from yaklib import deps as deps_mod  # noqa: E402
from yaklib.artifacts import artifacts_dir, parse_artifacts  # noqa: E402
from yaklib.clipboard import read_png as read_clipboard_png  # noqa: E402
from yaklib.model import (  # noqa: E402
    ALL_STATUS_NAMES,
    DEAD,
    HAIRY,
    SHAVING,
    SHORN,
    STATUSES,
    _ALL_STATUSES,
    _BlockScalarDumper,
    _STATUS_ALIASES,
    _auto_migrate,
    all_tasks,
    dump_yaml,
    find_children,
    find_descendants,
    find_task_file,
    find_tasks_root,
    generate_id,
    git_head_short,
    load_config,
    load_task,
    next_child_number,
    now_iso,
    parent_id,
    resolve_status,
    save_task,
)

# Legacy alias retained so callers (including in-tree cmd_* functions below)
# keep working while the extraction proceeds.
_resolve_status = resolve_status


# ---------------------------------------------------------------------------
# Subcommands
# ---------------------------------------------------------------------------

_YAKS_MANDATE = """\

## Task tracking

This project uses Yaks. The Yaks skill has the full workflow.

1. Never start coding without a shaving yak. No exceptions.
2. Shorn immediately after committing, before anything else.
3. Check existing yaks before creating new ones.
4. Append progress notes to yak descriptions as you work.
5. When unsure what's next, run `/yaks:next` — don't freelance.
"""


def _inject_mandate(force_agents: bool = False):
    """Append the yaks mandate block to CLAUDE.md or AGENTS.md.

    Precedence:
    - --agents flag → always use AGENTS.md (create if needed)
    - AGENTS.md exists → append there
    - CLAUDE.md exists → append there
    - Neither → create CLAUDE.md
    """
    cwd = Path.cwd()
    agents = cwd / "AGENTS.md"
    claude = cwd / "CLAUDE.md"

    if force_agents:
        target = agents
    elif agents.exists():
        target = agents
    elif claude.exists():
        target = claude
    else:
        target = claude

    # Don't inject twice
    if target.exists():
        content = target.read_text()
        if "Yaks skill" in content or "yaks:next" in content:
            print(f"Yaks guidance already present in {target.name}, skipping")
            return
        target.write_text(content.rstrip() + "\n" + _YAKS_MANDATE)
        print(f"Appended yaks guidance to {target.name}")
    else:
        target.write_text(_YAKS_MANDATE.lstrip())
        print(f"Created {target.name} with yaks guidance")


def cmd_init(args):
    target = Path.cwd() / ".yaks"
    if target.exists():
        print(f".yaks/ already exists at {target}")
        return
    prefix = args.prefix or Path.cwd().name.lower()
    if "." in prefix:
        print("error: prefix must not contain dots (dots are used for parent/child IDs)", file=sys.stderr)
        sys.exit(1)
    target.mkdir()
    for s in _ALL_STATUSES:
        (target / s).mkdir()
    config = {"prefix": prefix}
    (target / "config.yaml").write_text(dump_yaml(config))
    print(f"Initialized .yaks/ in {Path.cwd()} (prefix: {prefix})")
    _inject_mandate(force_agents=getattr(args, "agents", False))


def cmd_create(args):
    root = find_tasks_root()
    cfg = load_config(root)
    prefix = cfg.get("prefix", "yak")

    parent = getattr(args, "parent", None)
    if parent:
        # Verify parent exists
        if not find_task_file(root, parent):
            print(f"error: parent task {parent} not found", file=sys.stderr)
            sys.exit(1)
        tid = f"{parent}.{next_child_number(root, parent)}"
    else:
        tid = generate_id(root, prefix)

    now = now_iso()
    task = {
        "id": tid,
        "title": args.title,
        "type": args.type or cfg.get("default_type", "task"),
        "priority": args.priority if args.priority is not None else cfg.get("default_priority", 2),
        "created": now,
        "updated": now,
    }
    if args.depends_on:
        task["depends_on"] = args.depends_on
    if args.labels:
        task["labels"] = args.labels
    if args.description:
        task["description"] = args.description

    path = root / HAIRY / f"{tid}.md"
    save_task(path, task)
    print(f"Created {tid}: {args.title}")


def cmd_list(args):
    root = find_tasks_root()
    status_filter = _resolve_status(args.status) if args.status else None
    tasks = all_tasks(root, status_filter)

    # Apply filters
    if args.type:
        tasks = [(s, t) for s, t in tasks if t.get("type") == args.type]
    if args.priority is not None:
        tasks = [(s, t) for s, t in tasks if t.get("priority") == args.priority]
    if args.label:
        tasks = [(s, t) for s, t in tasks if args.label in t.get("labels", [])]

    if args.json:
        out = [{"status": s, **t} for s, t in tasks]
        print(json.dumps(out, indent=2))
        return

    if not tasks:
        print("No tasks found.")
        return

    _status_char = {HAIRY: "H", SHAVING: "S", SHORN: "N", DEAD: "X"}
    for status, t in tasks:
        pri = t.get("priority", "-")
        ttype = t.get("type", "-")
        labels = ",".join(t.get("labels", []))
        deps = t.get("depends_on", [])
        dep_str = f" (deps: {','.join(deps)})" if deps else ""
        label_str = f" [{labels}]" if labels else ""
        ch = _status_char.get(status, status[0].upper())
        print(f"  [{ch}] {t['id']}  p{pri} {ttype:8s} {t.get('title', '')}{label_str}{dep_str}")


def cmd_show(args):
    root = find_tasks_root()
    result = find_task_file(root, args.id)
    if not result:
        print(f"error: task {args.id} not found", file=sys.stderr)
        sys.exit(1)
    status, path = result
    task = load_task(path)

    if args.json:
        out = {"status": status, **task}
        pid = parent_id(args.id)
        if pid and find_task_file(root, pid):
            out["parent"] = pid
        children = find_children(root, args.id)
        if children:
            out["children"] = [{"id": t["id"], "status": s, "title": t.get("title", "")} for s, t in children]
        print(json.dumps(out, indent=2))
        return

    print(f"Status: {status}")
    print(dump_yaml(task), end="")

    _status_char = {HAIRY: "H", SHAVING: "S", SHORN: "N", DEAD: "X"}
    pid = parent_id(args.id)
    parent_result = find_task_file(root, pid) if pid else None
    if parent_result:
        ps, pp = parent_result
        pt = load_task(pp)
        ch = _status_char.get(ps, ps[0].upper())
        print(f"\nParent:\n  [{ch}] {pid}  {pt.get('title', '')}")

    children = find_children(root, args.id)
    if children:
        print(f"\nChildren:")
        for cs, ct in children:
            ch = _status_char.get(cs, cs[0].upper())
            print(f"  [{ch}] {ct['id']}  {ct.get('title', '')}")


def cmd_update(args):
    root = find_tasks_root()
    result = find_task_file(root, args.id)
    if not result:
        print(f"error: task {args.id} not found", file=sys.stderr)
        sys.exit(1)
    _, path = result
    task = load_task(path)

    changed = False
    if args.title is not None:
        task["title"] = args.title
        changed = True
    if args.type is not None:
        task["type"] = args.type
        changed = True
    if args.priority is not None:
        task["priority"] = args.priority
        changed = True
    if args.description is not None:
        task["description"] = args.description
        changed = True
    if args.add_label:
        labels = task.get("labels", [])
        for lbl in args.add_label:
            if lbl not in labels:
                labels.append(lbl)
        task["labels"] = labels
        changed = True
    if args.remove_label:
        labels = task.get("labels", [])
        for lbl in args.remove_label:
            if lbl in labels:
                labels.remove(lbl)
        task["labels"] = labels if labels else []
        if not task["labels"]:
            del task["labels"]
        changed = True
    if getattr(args, "note", None):
        # Append a timestamped note to the markdown body
        ts = now_iso()
        heading = f"### {ts}"
        note_block = f"\n\n{heading}\n{args.note}\n"
        desc = task.get("description", "") or ""
        task["description"] = desc + note_block
        changed = True

    if changed:
        task["updated"] = now_iso()
        save_task(path, task)
        print(f"Updated {args.id}")
    else:
        print("No changes specified.")


def _move_task(args, dest_status: str, already_msg: str, done_msg: str,
               extra_fields: dict | None = None):
    """Shared logic for shave/shorn/regrow."""
    root = find_tasks_root()
    result = find_task_file(root, args.id)
    if not result:
        print(f"error: task {args.id} not found", file=sys.stderr)
        sys.exit(1)
    status, path = result
    if status == dest_status:
        print(f"{args.id} is {already_msg}")
        return
    dest_dir = root / dest_status
    dest_dir.mkdir(exist_ok=True)
    dest = dest_dir / path.name
    path.rename(dest)
    task = load_task(dest)
    task["updated"] = now_iso()
    if extra_fields:
        task.update(extra_fields)
    save_task(dest, task)
    print(f"{done_msg} {args.id}")


def cmd_shave(args):
    _move_task(args, SHAVING, "already being shaved", "Shaving")


def cmd_shorn(args):
    commit = getattr(args, "commit", None)
    if commit is None:
        commit = git_head_short()
        if commit:
            print(f"(using HEAD commit: {commit})")
    extra = {"commit": commit} if commit else {}
    _move_task(args, SHORN, "already shorn", "Shorn!", extra_fields=extra)


def cmd_regrow(args):
    _move_task(args, HAIRY, "already hairy", "Regrown:")


def cmd_slaughter(args):
    """Move a yak to the hidden 'dead' state.

    Dead yaks stay on disk so history is preserved, but they are excluded
    from every default query and don't appear in the TUI. Use this for
    ideas you won't pursue and tasks that have been obviated.
    """
    _move_task(args, DEAD, "already dead", "Slaughtered:")


def cmd_revive(args):
    """Bring a dead yak back to the hairy state."""
    _move_task(args, HAIRY, "already hairy", "Revived:")


def cmd_next(args):
    root = find_tasks_root()
    ready = deps_mod.ready_tasks(root)

    if args.json:
        print(json.dumps(ready, indent=2))
        return

    if not ready:
        print("No yaks ready to shave.")
        return

    print("Ready to shave (all dependencies met):")
    for t in ready:
        pri = t.get("priority", "-")
        print(f"  {t['id']}  p{pri} {t.get('type', '-'):8s} {t.get('title', '')}")


def cmd_tangled(args):
    root = find_tasks_root()
    tangled = deps_mod.tangled_tasks(root)

    if args.json:
        out = [{"unshorn_deps": unshorn, **t} for t, unshorn in tangled]
        print(json.dumps(out, indent=2))
        return

    if not tangled:
        print("No tangled yaks.")
        return

    # Unpack to the list-of-dict form legacy text output used.
    tangled = [{**t, "_unshorn_deps": unshorn} for t, unshorn in tangled]
    print("Tangled yaks:")
    for t in tangled:
        unshorn = t.pop("_unshorn_deps")
        print(f"  {t['id']}  {t.get('title', '')}  (waiting on: {', '.join(unshorn)})")


def cmd_dep(args):
    root = find_tasks_root()
    result = find_task_file(root, args.id)
    if not result:
        print(f"error: task {args.id} not found", file=sys.stderr)
        sys.exit(1)
    _, path = result
    task = load_task(path)

    if args.action == "add":
        # Verify dep exists
        if not find_task_file(root, args.dep_id):
            print(f"error: dependency task {args.dep_id} not found", file=sys.stderr)
            sys.exit(1)
        deps = task.get("depends_on", [])
        if args.dep_id in deps:
            print(f"{args.dep_id} is already a dependency of {args.id}")
            return
        deps.append(args.dep_id)
        task["depends_on"] = deps
        task["updated"] = now_iso()
        save_task(path, task)
        print(f"Added dependency: {args.id} -> {args.dep_id}")

    elif args.action == "remove":
        deps = task.get("depends_on", [])
        if args.dep_id not in deps:
            print(f"{args.dep_id} is not a dependency of {args.id}")
            return
        deps.remove(args.dep_id)
        if deps:
            task["depends_on"] = deps
        else:
            task.pop("depends_on", None)
        task["updated"] = now_iso()
        save_task(path, task)
        print(f"Removed dependency: {args.id} -> {args.dep_id}")


def cmd_reparent(args):
    root = find_tasks_root()
    old_id = args.id

    result = find_task_file(root, old_id)
    if not result:
        print(f"error: task {old_id} not found", file=sys.stderr)
        sys.exit(1)

    new_parent = getattr(args, "parent", None)
    unparent = getattr(args, "unparent", False)

    if new_parent:
        # Can't reparent under self or own descendant
        if new_parent == old_id or new_parent.startswith(old_id + "."):
            print(f"error: cannot reparent under own descendant", file=sys.stderr)
            sys.exit(1)
        # Already a child of this parent?
        if parent_id(old_id) == new_parent:
            print(f"{old_id} is already a child of {new_parent}")
            return
        if not find_task_file(root, new_parent):
            print(f"error: parent task {new_parent} not found", file=sys.stderr)
            sys.exit(1)
        new_id = f"{new_parent}.{next_child_number(root, new_parent)}"
    elif unparent:
        if parent_id(old_id) is None:
            print(f"error: {old_id} is already a top-level task", file=sys.stderr)
            sys.exit(1)
        cfg = load_config(root)
        prefix = cfg.get("prefix", "yak")
        new_id = generate_id(root, prefix)
    else:
        print("error: specify --parent TASK_ID or --unparent", file=sys.stderr)
        sys.exit(1)

    # Build old→new ID mapping for target + all descendants
    id_map = {old_id: new_id}
    for _, path in find_descendants(root, old_id):
        desc_old = path.stem
        desc_new = new_id + desc_old[len(old_id):]
        id_map[desc_old] = desc_new

    # Rename files, update id fields and internal deps
    now = now_iso()
    for old, new in id_map.items():
        res = find_task_file(root, old)
        if not res:
            continue
        _, path = res
        task = load_task(path)
        task["id"] = new
        task["updated"] = now
        deps = task.get("depends_on", [])
        if deps:
            task["depends_on"] = [id_map.get(d, d) for d in deps]
        save_task(path.parent / f"{new}.md", task)
        path.unlink()

    # Scan all remaining tasks for depends_on references to renamed IDs
    renamed_stems = set(id_map.values())
    for s in STATUSES:
        d = root / s
        if not d.exists():
            continue
        for f in d.glob("*.md"):
            if f.stem in renamed_stems:
                continue
            task = load_task(f)
            deps = task.get("depends_on", [])
            if not deps:
                continue
            new_deps = [id_map.get(d, d) for d in deps]
            if new_deps != deps:
                task["depends_on"] = new_deps
                task["updated"] = now
                save_task(f, task)
                print(f"  updated dep in {task['id']}")

    print(f"Reparented {old_id} → {new_id}")
    if len(id_map) > 1:
        for old, new in sorted(id_map.items()):
            if old != old_id:
                print(f"  {old} → {new}")


def cmd_attach(args):
    root = find_tasks_root()
    loc = find_task_file(root, args.id)
    if loc is None:
        print(f"error: task {args.id} not found", file=sys.stderr)
        sys.exit(1)
    status, path = loc
    task = load_task(path)

    adir = artifacts_dir(root, args.id)
    adir.mkdir(parents=True, exist_ok=True)

    if args.paste:
        data = read_clipboard_png()
        if not data:
            print("error: no PNG image found on clipboard", file=sys.stderr)
            sys.exit(1)
        name = args.name or f"paste-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}.png"
        dest = adir / name
        if dest.exists() and not args.force:
            print(f"error: {dest} exists (use --force to overwrite)", file=sys.stderr)
            sys.exit(1)
        dest.write_bytes(data)
    else:
        if not args.path:
            print("error: provide <path> or --paste", file=sys.stderr)
            sys.exit(1)
        src = Path(args.path).expanduser()
        if not src.is_file():
            print(f"error: {src} is not a file", file=sys.stderr)
            sys.exit(1)
        name = args.name or src.name
        dest = adir / name
        if dest.exists() and not args.force:
            print(f"error: {dest} exists (use --force to overwrite)", file=sys.stderr)
            sys.exit(1)
        shutil.copy2(src, dest)

    desc = args.desc or Path(name).stem
    link = f"![{desc}](artifacts/{args.id}/{name})"
    body = task.get("description", "")
    if body and not body.endswith("\n"):
        body += "\n"
    body += "\n" + link + "\n"
    task["description"] = body
    task["updated"] = now_iso()
    save_task(path, task)
    print(f"Attached {dest.relative_to(root)} to {args.id}")


def cmd_detach(args):
    root = find_tasks_root()
    loc = find_task_file(root, args.id)
    if loc is None:
        print(f"error: task {args.id} not found", file=sys.stderr)
        sys.exit(1)
    _, path = loc
    task = load_task(path)
    body = task.get("description", "")

    target = args.name
    pat = re.compile(
        r"[ \t]*!\[[^\]]*\]\(artifacts/" + re.escape(args.id) + "/" + re.escape(target) + r"\)[ \t]*\n?"
    )
    new_body, n = pat.subn("", body)
    if n == 0:
        print(f"warning: no reference to {target} found in description", file=sys.stderr)

    afile = artifacts_dir(root, args.id) / target
    if afile.exists():
        afile.unlink()
        print(f"Removed {afile.relative_to(root)}")
    else:
        print(f"warning: {afile} did not exist", file=sys.stderr)

    task["description"] = new_body
    task["updated"] = now_iso()
    save_task(path, task)

    adir = artifacts_dir(root, args.id)
    if adir.exists() and not any(adir.iterdir()):
        adir.rmdir()


def cmd_search(args):
    root = find_tasks_root()
    status_filter = _resolve_status(args.status) if args.status else None
    tasks = all_tasks(root, status_filter)

    query = args.query.lower()
    matches = []
    for s, t in tasks:
        title = t.get("title", "").lower()
        desc = t.get("description", "").lower()
        if query in title or query in desc:
            matches.append((s, t))

    if args.json:
        out = [{"status": s, **t} for s, t in matches]
        print(json.dumps(out, indent=2))
        return

    if not matches:
        print("No tasks found.")
        return

    _status_char = {HAIRY: "H", SHAVING: "S", SHORN: "N", DEAD: "X"}
    for status, t in matches:
        pri = t.get("priority", "-")
        ttype = t.get("type", "-")
        labels = ",".join(t.get("labels", []))
        deps = t.get("depends_on", [])
        dep_str = f" (deps: {','.join(deps)})" if deps else ""
        label_str = f" [{labels}]" if labels else ""
        ch = _status_char.get(status, status[0].upper())
        print(f"  [{ch}] {t['id']}  p{pri} {ttype:8s} {t.get('title', '')}{label_str}{dep_str}")


def cmd_stats(args):
    root = find_tasks_root()
    tasks = all_tasks(root)

    hairy_count = sum(1 for s, _ in tasks if s == HAIRY)
    shaving_count = sum(1 for s, _ in tasks if s == SHAVING)
    shorn_count = sum(1 for s, _ in tasks if s == SHORN)

    by_type: dict[str, int] = {}
    by_priority: dict[int, int] = {}
    for _, t in tasks:
        ttype = t.get("type", "unknown")
        by_type[ttype] = by_type.get(ttype, 0) + 1
        pri = t.get("priority", 0)
        by_priority[pri] = by_priority.get(pri, 0) + 1

    if args.json:
        print(json.dumps({
            "total": len(tasks),
            "hairy": hairy_count,
            "shaving": shaving_count,
            "shorn": shorn_count,
            "by_type": by_type,
            "by_priority": dict(sorted(by_priority.items())),
        }, indent=2))
        return

    print(f"Total: {len(tasks)}  Hairy: {hairy_count}  Shaving: {shaving_count}  Shorn: {shorn_count}")
    if by_type:
        print("By type:")
        for k, v in sorted(by_type.items()):
            print(f"  {k}: {v}")
    if by_priority:
        print("By priority:")
        for k, v in sorted(by_priority.items()):
            print(f"  p{k}: {v}")


def cmd_import_beads(args):
    root = find_tasks_root()

    # Locate the beads JSONL file
    if args.file:
        jsonl_path = Path(args.file)
    else:
        p = Path.cwd().resolve()
        jsonl_path = None
        while True:
            candidate = p / ".beads" / "issues.jsonl"
            if candidate.is_file():
                jsonl_path = candidate
                break
            if p.parent == p:
                break
            p = p.parent
        if not jsonl_path:
            print("error: no .beads/issues.jsonl found (use --file to specify)", file=sys.stderr)
            sys.exit(1)

    if not jsonl_path.is_file():
        print(f"error: {jsonl_path} not found", file=sys.stderr)
        sys.exit(1)

    # Collect existing task IDs so we can skip duplicates
    existing_ids: set[str] = set()
    for d in (root / s for s in STATUSES):
        if d.exists():
            for f in d.glob("*.md"):
                existing_ids.add(f.stem)

    skip_types = {"message", "molecule", "merge-request"}
    skip_statuses = {"tombstone", "pinned"}
    priority_map = {0: 1, 1: 1, 2: 2, 3: 3, 4: 3}
    type_map = {"bug": "bug", "feature": "feature"}

    # Map beads status → yaks directory
    _bead_status_map = {
        "in_progress": SHAVING,
        "closed": SHORN,
    }

    created = {s: 0 for s in STATUSES}
    skipped = 0

    for line in jsonl_path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        bead = json.loads(line)

        # Skip non-task types and soft-deleted/pinned
        if bead.get("issue_type") in skip_types:
            skipped += 1
            continue
        if bead.get("status") in skip_statuses:
            skipped += 1
            continue

        bead_id = bead.get("id", "")
        if not bead_id:
            skipped += 1
            continue

        # Idempotent: skip if already imported
        if bead_id in existing_ids:
            skipped += 1
            continue

        # Map status to yaks directory
        yak_dir = _bead_status_map.get(bead.get("status", ""), HAIRY)

        # Build task
        task: dict = {"id": bead_id}
        if bead.get("title"):
            task["title"] = bead["title"]
        task["type"] = type_map.get(bead.get("issue_type", ""), "task")
        task["priority"] = priority_map.get(bead.get("priority", 2), 2)

        # Timestamps
        if bead.get("created_at"):
            task["created"] = bead["created_at"]
        else:
            task["created"] = now_iso()
        if bead.get("updated_at"):
            task["updated"] = bead["updated_at"]
        else:
            task["updated"] = task["created"]

        # Dependencies — only "blocks" type
        deps = bead.get("dependencies", [])
        if deps:
            dep_ids = [d["depends_on_id"] for d in deps if d.get("type") == "blocks" and d.get("depends_on_id")]
            if dep_ids:
                task["depends_on"] = dep_ids

        # Labels
        if bead.get("labels"):
            task["labels"] = bead["labels"]

        # Description — unescape literal \n sequences left by beads export,
        # and strip trailing whitespace so the YAML dumper uses block scalars.
        if bead.get("description"):
            desc = bead["description"]
            desc = desc.replace("\\n", "\n").replace("\\t", "\t")
            desc = "\n".join(line.rstrip() for line in desc.split("\n"))
            task["description"] = desc

        if args.dry_run:
            print(f"  [dry-run] {yak_dir}/{bead_id}.md  {task.get('title', '')}")
        else:
            dest = root / yak_dir / f"{bead_id}.md"
            save_task(dest, task)

        created[yak_dir] += 1

    total = sum(created.values())
    prefix = "[dry-run] " if args.dry_run else ""
    print(f"{prefix}Imported {total} tasks (hairy: {created[HAIRY]}, shaving: {created[SHAVING]}, shorn: {created[SHORN]}), skipped {skipped}")


# ---------------------------------------------------------------------------
# Argument parser
# ---------------------------------------------------------------------------

_ALL_STATUS_NAMES = sorted(_STATUS_ALIASES.keys())


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="yaks", description="Filesystem-native task tracker")
    sub = p.add_subparsers(dest="command")

    # init
    sp = sub.add_parser("init", help="Initialize .yaks/ in the current directory")
    sp.add_argument("--prefix", help="Task ID prefix (default: directory name)")
    sp.add_argument("--agents", action="store_true",
                    help="Write guidance to AGENTS.md instead of CLAUDE.md")

    # create
    sp = sub.add_parser("create", help="Create a new task")
    sp.add_argument("--title", required=True, help="Task title")
    sp.add_argument("--type", help="Task type (bug, feature, task, idea, etc.)")
    sp.add_argument("--priority", type=int, help="Priority (1=highest)")
    sp.add_argument("--description", help="Task description")
    sp.add_argument("--labels", nargs="+", help="Labels")
    sp.add_argument("--depends-on", nargs="+", help="Dependency task IDs")
    sp.add_argument("--parent", help="Parent task ID (creates a child task)")

    # list
    sp = sub.add_parser("list", help="List tasks")
    sp.add_argument("--status", choices=_ALL_STATUS_NAMES, help="Filter by status")
    sp.add_argument("--type", help="Filter by type")
    sp.add_argument("--priority", type=int, help="Filter by priority")
    sp.add_argument("--label", help="Filter by label")
    sp.add_argument("--json", action="store_true", help="JSON output")

    # show
    sp = sub.add_parser("show", help="Show a task")
    sp.add_argument("id", help="Task ID")
    sp.add_argument("--json", action="store_true", help="JSON output")

    # update
    sp = sub.add_parser("update", help="Update a task")
    sp.add_argument("id", help="Task ID")
    sp.add_argument("--title", help="New title")
    sp.add_argument("--type", help="New type")
    sp.add_argument("--priority", type=int, help="New priority")
    sp.add_argument("--description", help="New description")
    sp.add_argument("--add-label", nargs="+", help="Add labels")
    sp.add_argument("--remove-label", nargs="+", help="Remove labels")
    sp.add_argument("--note", help="Append a timestamped progress note to the description")

    # shave (+ alias: work)
    for name in ("shave", "work"):
        sp = sub.add_parser(name, help="Start shaving a yak")
        sp.add_argument("id", help="Task ID")

    # shorn (+ alias: close)
    for name in ("shorn", "close"):
        sp = sub.add_parser(name, help="Mark a yak as shorn")
        sp.add_argument("id", help="Task ID")
        sp.add_argument("--commit", help="Commit hash (default: git HEAD)")

    # regrow (+ alias: reopen)
    for name in ("regrow", "reopen"):
        sp = sub.add_parser(name, help="Regrow a shorn yak")
        sp.add_argument("id", help="Task ID")

    # slaughter: move a yak to the hidden 'dead' state
    sp = sub.add_parser("slaughter",
                        help="Slaughter a yak (move to hidden 'dead' state)")
    sp.add_argument("id", help="Task ID")

    # revive: bring a dead yak back
    sp = sub.add_parser("revive", help="Revive a dead yak (back to hairy)")
    sp.add_argument("id", help="Task ID")

    # next (+ alias: ready)
    for name in ("next", "ready"):
        sp = sub.add_parser(name, help="Show yaks ready to shave")
        sp.add_argument("--json", action="store_true", help="JSON output")

    # tangled (+ alias: blocked)
    for name in ("tangled", "blocked"):
        sp = sub.add_parser(name, help="Show tangled yaks")
        sp.add_argument("--json", action="store_true", help="JSON output")

    # dep
    sp = sub.add_parser("dep", help="Manage dependencies")
    sp.add_argument("action", choices=["add", "remove"], help="Add or remove dependency")
    sp.add_argument("id", help="Task ID")
    sp.add_argument("dep_id", help="Dependency task ID")

    # reparent
    sp = sub.add_parser("reparent", help="Move a task to a new parent or to top-level")
    sp.add_argument("id", help="Task ID to reparent")
    group = sp.add_mutually_exclusive_group(required=True)
    group.add_argument("--parent", help="New parent task ID")
    group.add_argument("--unparent", action="store_true", help="Promote to top-level task")

    # attach
    sp = sub.add_parser("attach", help="Attach a file (or clipboard image) to a yak")
    sp.add_argument("id", help="Task ID")
    sp.add_argument("path", nargs="?", help="Path to file (omit with --paste)")
    sp.add_argument("--paste", action="store_true", help="Read PNG image from clipboard")
    sp.add_argument("--name", help="Override stored filename")
    sp.add_argument("--desc", help="Alt text / description for the markdown link")
    sp.add_argument("--force", action="store_true", help="Overwrite if file already exists")

    # detach
    sp = sub.add_parser("detach", help="Detach an artifact from a yak")
    sp.add_argument("id", help="Task ID")
    sp.add_argument("name", help="Artifact filename")

    # search
    sp = sub.add_parser("search", help="Search tasks by keyword")
    sp.add_argument("query", help="Search term")
    sp.add_argument("--status", choices=_ALL_STATUS_NAMES, help="Filter by status")
    sp.add_argument("--json", action="store_true", help="JSON output")

    # stats
    sp = sub.add_parser("stats", help="Show task statistics")
    sp.add_argument("--json", action="store_true", help="JSON output")

    # tui
    sub.add_parser("tui", help="Open interactive TUI")

    # import-beads
    sp = sub.add_parser("import-beads", help="Import tasks from a beads issues.jsonl file")
    sp.add_argument("--file", help="Path to issues.jsonl (default: auto-detect .beads/issues.jsonl)")
    sp.add_argument("--dry-run", action="store_true", help="Print what would be created without writing")

    return p


def main():
    parser = build_parser()
    args = parser.parse_args()
    if not args.command:
        parser.print_help()
        sys.exit(1)

    if args.command == "tui":
        import subprocess as _sp
        _sp.execvp(sys.executable, [sys.executable, str(Path(__file__).parent / "tui.py")])

    commands = {
        "init": cmd_init,
        "create": cmd_create,
        "list": cmd_list,
        "show": cmd_show,
        "update": cmd_update,
        "shave": cmd_shave,
        "work": cmd_shave,
        "shorn": cmd_shorn,
        "close": cmd_shorn,
        "regrow": cmd_regrow,
        "reopen": cmd_regrow,
        "slaughter": cmd_slaughter,
        "revive": cmd_revive,
        "next": cmd_next,
        "ready": cmd_next,
        "tangled": cmd_tangled,
        "blocked": cmd_tangled,
        "dep": cmd_dep,
        "reparent": cmd_reparent,
        "attach": cmd_attach,
        "detach": cmd_detach,
        "search": cmd_search,
        "stats": cmd_stats,
        "import-beads": cmd_import_beads,
    }
    commands[args.command](args)


if __name__ == "__main__":
    main()
