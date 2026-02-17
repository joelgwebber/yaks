# /// script
# requires-python = ">=3.10"
# dependencies = ["pyyaml>=6.0"]
# ///
"""Filesystem-native task tracker. Plain YAML files, no database, no daemon."""

import argparse
import json
import random
import string
import sys
from datetime import datetime, timezone
from pathlib import Path

import yaml

# ---------------------------------------------------------------------------
# Status constants
# ---------------------------------------------------------------------------

HAIRY = "hairy"
SHAVING = "shaving"
SHORN = "shorn"
STATUSES = (HAIRY, SHAVING, SHORN)

# Aliases (old names → canonical)
_STATUS_ALIASES = {
    "open": HAIRY, "working": SHAVING, "closed": SHORN,
    HAIRY: HAIRY, SHAVING: SHAVING, SHORN: SHORN,
}


def _resolve_status(name: str) -> str:
    return _STATUS_ALIASES.get(name, name)


# ---------------------------------------------------------------------------
# YAML helpers
# ---------------------------------------------------------------------------

class _BlockScalarDumper(yaml.SafeDumper):
    """Dumper that uses block scalars for multiline strings."""


def _str_representer(dumper: _BlockScalarDumper, data: str):
    if "\n" in data:
        return dumper.represent_scalar("tag:yaml.org,2002:str", data, style="|")
    return dumper.represent_scalar("tag:yaml.org,2002:str", data)


_BlockScalarDumper.add_representer(str, _str_representer)


def dump_yaml(data: dict) -> str:
    return yaml.dump(data, Dumper=_BlockScalarDumper, default_flow_style=False, sort_keys=False)


# ---------------------------------------------------------------------------
# Filesystem helpers
# ---------------------------------------------------------------------------

def find_tasks_root(start: Path | None = None) -> Path:
    """Walk up from *start* (default cwd) looking for a `.yaks/` directory."""
    p = (start or Path.cwd()).resolve()
    while True:
        candidate = p / ".yaks"
        if candidate.is_dir():
            return candidate
        if p.parent == p:
            break
        p = p.parent
    print("error: no .yaks/ directory found (run /yaks:init first)", file=sys.stderr)
    sys.exit(1)


def load_config(root: Path) -> dict:
    cfg_path = root / "config.yaml"
    if cfg_path.exists():
        return yaml.safe_load(cfg_path.read_text()) or {}
    return {}


def load_task(path: Path) -> dict:
    return yaml.safe_load(path.read_text()) or {}


def save_task(path: Path, task: dict) -> None:
    path.write_text(dump_yaml(task))


def all_tasks(root: Path, status: str | None = None) -> list[tuple[str, dict]]:
    """Return list of (status, task_dict) for tasks in the given status dir(s)."""
    dirs = []
    for s in STATUSES:
        if status is None or status == s:
            dirs.append((s, root / s))
    results = []
    for s, d in dirs:
        if not d.exists():
            continue
        for f in sorted(d.glob("*.yaml")):
            task = load_task(f)
            if task:
                results.append((s, task))
    return results


def find_task_file(root: Path, task_id: str) -> tuple[str, Path] | None:
    """Locate a task file by ID, searching all dirs. Returns (status, path)."""
    for status_dir in STATUSES:
        p = root / status_dir / f"{task_id}.yaml"
        if p.exists():
            return status_dir, p
    return None


def generate_id(root: Path, prefix: str) -> str:
    """Generate a collision-free task ID."""
    existing = set()
    for d in (root / s for s in STATUSES):
        if d.exists():
            for f in d.glob("*.yaml"):
                existing.add(f.stem)
    for _ in range(100):
        suffix = "".join(random.choices(string.hexdigits[:16], k=4))
        tid = f"{prefix}-{suffix}"
        if tid not in existing:
            return tid
    print("error: could not generate unique ID after 100 attempts", file=sys.stderr)
    sys.exit(1)


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


# ---------------------------------------------------------------------------
# Subcommands
# ---------------------------------------------------------------------------

def cmd_init(args):
    target = Path.cwd() / ".yaks"
    if target.exists():
        print(f".yaks/ already exists at {target}")
        return
    prefix = args.prefix or Path.cwd().name.lower()
    target.mkdir()
    for s in STATUSES:
        (target / s).mkdir()
    config = {"prefix": prefix}
    (target / "config.yaml").write_text(dump_yaml(config))
    print(f"Initialized .yaks/ in {Path.cwd()} (prefix: {prefix})")


def cmd_create(args):
    root = find_tasks_root()
    cfg = load_config(root)
    prefix = cfg.get("prefix", "yak")

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

    path = root / HAIRY / f"{tid}.yaml"
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

    _status_char = {HAIRY: "H", SHAVING: "S", SHORN: "N"}
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
        print(json.dumps({"status": status, **task}, indent=2))
        return

    print(f"Status: {status}")
    print(dump_yaml(task), end="")


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

    if changed:
        task["updated"] = now_iso()
        save_task(path, task)
        print(f"Updated {args.id}")
    else:
        print("No changes specified.")


def _move_task(args, dest_status: str, already_msg: str, done_msg: str):
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
    dest = root / dest_status / path.name
    path.rename(dest)
    task = load_task(dest)
    task["updated"] = now_iso()
    save_task(dest, task)
    print(f"{done_msg} {args.id}")


def cmd_shave(args):
    _move_task(args, SHAVING, "already being shaved", "Shaving")


def cmd_shorn(args):
    _move_task(args, SHORN, "already shorn", "Shorn!")


def cmd_regrow(args):
    _move_task(args, HAIRY, "already hairy", "Regrown:")


def cmd_next(args):
    root = find_tasks_root()
    hairy_tasks = all_tasks(root, HAIRY)
    shorn_ids = {t["id"] for _, t in all_tasks(root, SHORN)}

    ready = []
    for _, task in hairy_tasks:
        deps = task.get("depends_on", [])
        if not deps or all(d in shorn_ids for d in deps):
            ready.append(task)

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
    hairy_tasks = all_tasks(root, HAIRY)
    shorn_ids = {t["id"] for _, t in all_tasks(root, SHORN)}

    tangled = []
    for _, task in hairy_tasks:
        deps = task.get("depends_on", [])
        unshorn = [d for d in deps if d not in shorn_ids]
        if unshorn:
            tangled.append({**task, "_unshorn_deps": unshorn})

    if args.json:
        out = [{"unshorn_deps": t.pop("_unshorn_deps"), **t} for t in tangled]
        print(json.dumps(out, indent=2))
        return

    if not tangled:
        print("No tangled yaks.")
        return

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
            for f in d.glob("*.yaml"):
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

        # Description
        if bead.get("description"):
            task["description"] = bead["description"]

        if args.dry_run:
            print(f"  [dry-run] {yak_dir}/{bead_id}.yaml  {task.get('title', '')}")
        else:
            dest = root / yak_dir / f"{bead_id}.yaml"
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

    # create
    sp = sub.add_parser("create", help="Create a new task")
    sp.add_argument("--title", required=True, help="Task title")
    sp.add_argument("--type", help="Task type (bug, feature, task, etc.)")
    sp.add_argument("--priority", type=int, help="Priority (1=highest)")
    sp.add_argument("--description", help="Task description")
    sp.add_argument("--labels", nargs="+", help="Labels")
    sp.add_argument("--depends-on", nargs="+", help="Dependency task IDs")

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

    # shave (+ alias: work)
    for name in ("shave", "work"):
        sp = sub.add_parser(name, help="Start shaving a yak")
        sp.add_argument("id", help="Task ID")

    # shorn (+ alias: close)
    for name in ("shorn", "close"):
        sp = sub.add_parser(name, help="Mark a yak as shorn")
        sp.add_argument("id", help="Task ID")

    # regrow (+ alias: reopen)
    for name in ("regrow", "reopen"):
        sp = sub.add_parser(name, help="Regrow a shorn yak")
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

    # stats
    sp = sub.add_parser("stats", help="Show task statistics")
    sp.add_argument("--json", action="store_true", help="JSON output")

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
        "next": cmd_next,
        "ready": cmd_next,
        "tangled": cmd_tangled,
        "blocked": cmd_tangled,
        "dep": cmd_dep,
        "stats": cmd_stats,
        "import-beads": cmd_import_beads,
    }
    commands[args.command](args)


if __name__ == "__main__":
    main()
