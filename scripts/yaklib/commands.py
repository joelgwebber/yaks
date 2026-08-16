"""CLI subcommand implementations. Each cmd_* takes parsed argparse `args`."""

from __future__ import annotations

import json
import re
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

from yaklib import deps as deps_mod
from yaklib import reparent as reparent_mod
from yaklib.artifacts import artifacts_dir
from yaklib.clipboard import read_png as read_clipboard_png
from yaklib.filter import FilterSpec, filter_tasks
from yaklib.model import (
    CURRENT_SCHEMA_VERSION,
    DEAD,
    HAIRY,
    SHAVING,
    SHORN,
    STATUSES,
    all_tasks,
    dump_yaml,
    find_children,
    find_task_file,
    find_tasks_root,
    generate_id,
    load_config,
    load_task,
    move_task,
    now_iso,
    parent_of,
    resolve_status,
    save_task,
    write_schema_version,
)

_STATUS_CHAR = {HAIRY: "H", SHAVING: "S", SHORN: "N", DEAD: "X"}


def _split_labels(tokens: list[str] | None) -> list[str]:
    """Flatten label tokens so users can type `--labels foo bar` or
    `--labels foo,bar` or `--labels "foo bar,baz"` and get the obvious thing."""
    if not tokens:
        return []
    out: list[str] = []
    for t in tokens:
        for piece in re.split(r"[,\s]+", t):
            if piece and piece not in out:
                out.append(piece)
    return out


# ---------------------------------------------------------------------------
# Mandate injection
# ---------------------------------------------------------------------------

_YAKS_MANDATE = """\

## Task tracking

This project uses Yaks. The Yaks skill has the full workflow.

1. Never start coding without a shaving yak. No exceptions.
2. Shear a yak as soon as its work is done. If the project commits its yaks (`.yaks/` is tracked by git), commit the shorn yak alongside the code that completed it; if `.yaks/` is gitignored, keep yak files — and their IDs — out of commits, PRs, and anything external.
3. Check existing yaks before creating new ones.
4. Append progress notes to yak descriptions as you work.
5. When unsure what's next, run `yaks next` — don't freelance.
"""


def _inject_mandate(force_agents: bool = False):
    cwd = Path.cwd()
    agents = cwd / "AGENTS.md"
    claude = cwd / "CLAUDE.md"

    if force_agents or agents.exists():
        target = agents
    elif claude.exists():
        target = claude
    else:
        target = agents

    if target.exists():
        content = target.read_text()
        if "Yaks skill" in content or "yaks next" in content:
            print(f"Yaks guidance already present in {target.name}, skipping")
            return
        target.write_text(content.rstrip() + "\n" + _YAKS_MANDATE)
        print(f"Appended yaks guidance to {target.name}")
    else:
        target.write_text(_YAKS_MANDATE.lstrip())
        print(f"Created {target.name} with yaks guidance")


# ---------------------------------------------------------------------------
# Init / create
# ---------------------------------------------------------------------------


def cmd_init(args):
    from yaklib.model import _ALL_STATUSES  # avoid leaking into module API

    target = Path.cwd() / ".yaks"
    if target.exists():
        print(f".yaks/ already exists at {target}")
    else:
        prefix = args.prefix or Path.cwd().name.lower()
        if "." in prefix:
            print("error: prefix must not contain dots (dots are used for parent/child IDs)", file=sys.stderr)
            sys.exit(1)
        target.mkdir()
        for s in _ALL_STATUSES:
            (target / s).mkdir()
        config = {"prefix": prefix}
        (target / "config.yaml").write_text(dump_yaml(config))
        write_schema_version(target, CURRENT_SCHEMA_VERSION)
        print(f"Initialized .yaks/ in {Path.cwd()} (prefix: {prefix})")
        _inject_mandate(force_agents=getattr(args, "agents", False))

    # Seed the user-global config with documented defaults on first run.
    user_cfg = Path.home() / ".config" / "yaks" / "config.yaml"
    if not user_cfg.exists():
        user_cfg.parent.mkdir(parents=True, exist_ok=True)
        user_cfg.write_text(
            "# yaks user-global config. Per-project .yaks/config.yaml\n"
            "# keys override anything set here.\n"
            "\n"
            "# Vim-style line editing in all text inputs (insert/normal\n"
            "# modes, double-Esc to cancel). Set to true if you want it.\n"
            "vim_mode: false\n"
        )
        print(f"Wrote user-global config: {user_cfg}")


def cmd_create(args):
    root = find_tasks_root()
    cfg = load_config(root)
    prefix = cfg.get("prefix", "yak")

    parent = getattr(args, "parent", None)
    if parent and not find_task_file(root, parent):
        print(f"error: parent task {parent} not found", file=sys.stderr)
        sys.exit(1)
    tid = generate_id(root, prefix)

    now = now_iso()
    task = {
        "id": tid,
        "title": args.title,
        "type": args.type or cfg.get("default_type", "task"),
        "priority": args.priority if args.priority is not None else cfg.get("default_priority", 3),
        "created": now,
        "updated": now,
    }
    if parent:
        task["parent"] = parent
    if args.depends_on:
        task["depends_on"] = args.depends_on
    labels = _split_labels(args.labels)
    if labels:
        task["labels"] = labels
    if args.description:
        task["description"] = args.description
    if getattr(args, "source", None):
        task["source"] = args.source

    path = root / HAIRY / f"{tid}.md"
    save_task(path, task)
    print(f"Created {tid}: {args.title}")


# ---------------------------------------------------------------------------
# List / show / update
# ---------------------------------------------------------------------------


def _fmt_task_row(status, t):
    pri = t.get("priority", "-")
    ttype = t.get("type", "-")
    labels = ",".join(t.get("labels", []))
    deps = t.get("depends_on", [])
    dep_str = f" (deps: {','.join(deps)})" if deps else ""
    label_str = f" [{labels}]" if labels else ""
    ch = _STATUS_CHAR.get(status, status[0].upper())
    return f"  [{ch}] {t['id']}  p{pri} {ttype:8s} {t.get('title', '')}{label_str}{dep_str}"


def _spec_from_args(args, defaults: dict | None = None) -> FilterSpec:
    """Build a FilterSpec from argparse args populated by _add_filter_flags.
    *defaults* lets a command inject baseline constraints (e.g. cmd_next
    hard-codes ready_only=True)."""
    d = defaults or {}

    def _get(name, default=None):
        return getattr(args, name, None) if hasattr(args, name) else default

    statuses = _get("status") or []
    statuses = [resolve_status(s) for s in statuses]
    types = _get("type") or []
    priorities = _get("priority") or []
    labels = _split_labels(_get("label") or [])

    return FilterSpec(
        statuses=frozenset(d.get("statuses", statuses)),
        types=frozenset(types),
        priorities=frozenset(priorities),
        labels=tuple(labels),
        search=_get("search") or d.get("search", "") or "",
        ready_only=d.get("ready_only", bool(_get("ready"))),
        tangled_only=d.get("tangled_only", bool(_get("tangled"))),
        parent=_get("parent_of") or d.get("parent", "") or "",
    )


def cmd_list(args):
    root = find_tasks_root()
    spec = _spec_from_args(args)
    tasks = filter_tasks(root, spec)

    if args.json:
        out = [{"status": s, **t} for s, t in tasks]
        print(json.dumps(out, indent=2))
        return

    if not tasks:
        print("No tasks found.")
        return

    for status, t in tasks:
        print(_fmt_task_row(status, t))


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
        pid = parent_of(task)
        if pid:
            out["parent"] = pid
        children = find_children(root, args.id)
        if children:
            out["children"] = [{"id": t["id"], "status": s, "title": t.get("title", "")} for s, t in children]
        print(json.dumps(out, indent=2))
        return

    print(f"Status: {status}")
    print(dump_yaml(task), end="")

    pid = parent_of(task)
    parent_result = find_task_file(root, pid) if pid else None
    if parent_result:
        ps, pp = parent_result
        pt = load_task(pp)
        ch = _STATUS_CHAR.get(ps, ps[0].upper())
        print(f"\nParent:\n  [{ch}] {pid}  {pt.get('title', '')}")

    children = find_children(root, args.id)
    if children:
        print("\nChildren:")
        for cs, ct in children:
            ch = _STATUS_CHAR.get(cs, cs[0].upper())
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
    add = _split_labels(args.add_label)
    if add:
        labels = task.get("labels", [])
        for lbl in add:
            if lbl not in labels:
                labels.append(lbl)
        task["labels"] = labels
        changed = True
    remove = _split_labels(args.remove_label)
    if remove:
        labels = task.get("labels", [])
        for lbl in remove:
            if lbl in labels:
                labels.remove(lbl)
        task["labels"] = labels if labels else []
        if not task["labels"]:
            del task["labels"]
        changed = True
    if getattr(args, "source", None):
        task["source"] = args.source
        changed = True
    if getattr(args, "note", None):
        ts = now_iso()
        desc = (task.get("description") or "").rstrip()
        sep = "\n\n" if desc else ""
        task["description"] = f"{desc}{sep}---\n▸ {ts}\n{args.note}"
        changed = True

    if changed:
        task["updated"] = now_iso()
        save_task(path, task)
        print(f"Updated {args.id}")
    else:
        print("No changes specified.")


# ---------------------------------------------------------------------------
# Status transitions
# ---------------------------------------------------------------------------


def _move_task(args, dest_status: str, already_msg: str, done_msg: str, extra_fields: dict | None = None):
    root = find_tasks_root()
    if not find_task_file(root, args.id):
        print(f"error: task {args.id} not found", file=sys.stderr)
        sys.exit(1)
    ok, _ = move_task(root, args.id, dest_status, extra_fields=extra_fields)
    if not ok:
        print(f"{args.id} is {already_msg}")
        return
    print(f"{done_msg} {args.id}")


def cmd_shave(args):
    _move_task(args, SHAVING, "already being shaved", "Shaving")


def cmd_shorn(args):
    _move_task(args, SHORN, "already shorn", "Shorn!")


def cmd_regrow(args):
    _move_task(args, HAIRY, "already hairy", "Regrown:")


def cmd_slaughter(args):
    _move_task(args, DEAD, "already dead", "Slaughtered:")


def cmd_revive(args):
    _move_task(args, HAIRY, "already hairy", "Revived:")


# ---------------------------------------------------------------------------
# Queries: next, tangled
# ---------------------------------------------------------------------------


def cmd_next(args):
    root = find_tasks_root()
    spec = _spec_from_args(
        args,
        defaults={
            "statuses": [HAIRY],
            "ready_only": True,
        },
    )
    tasks = [t for _, t in filter_tasks(root, spec)]

    if args.json:
        print(json.dumps(tasks, indent=2))
        return
    if not tasks:
        print("No yaks ready to shave.")
        return

    print("Ready to shave (all dependencies met):")
    for t in tasks:
        pri = t.get("priority", "-")
        print(f"  {t['id']}  p{pri} {t.get('type', '-'):8s} {t.get('title', '')}")


def cmd_tangled(args):
    root = find_tasks_root()
    spec = _spec_from_args(
        args,
        defaults={
            "statuses": [HAIRY],
            "tangled_only": True,
        },
    )
    tasks = [t for _, t in filter_tasks(root, spec)]
    resolved = deps_mod.resolved_ids(root)

    if args.json:
        out = [{"unshorn_deps": deps_mod.unresolved_deps(t, resolved), **t} for t in tasks]
        print(json.dumps(out, indent=2))
        return
    if not tasks:
        print("No tangled yaks.")
        return

    print("Tangled yaks:")
    for t in tasks:
        unshorn = deps_mod.unresolved_deps(t, resolved)
        print(f"  {t['id']}  {t.get('title', '')}  (waiting on: {', '.join(unshorn)})")


# ---------------------------------------------------------------------------
# Dependency + reparent
# ---------------------------------------------------------------------------


def cmd_dep(args):
    root = find_tasks_root()
    result = find_task_file(root, args.id)
    if not result:
        print(f"error: task {args.id} not found", file=sys.stderr)
        sys.exit(1)
    _, path = result
    task = load_task(path)

    if args.action == "add":
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
    new_parent = getattr(args, "parent", None)
    if getattr(args, "unparent", False):
        new_parent = None
    elif not new_parent:
        print("error: specify --parent TASK_ID or --unparent", file=sys.stderr)
        sys.exit(1)

    try:
        result = reparent_mod.reparent(root, args.id, new_parent)
    except reparent_mod.ReparentError as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)

    if result.new_parent is None:
        print(f"Promoted {result.task_id} to top-level")
    else:
        print(f"Reparented {result.task_id} under {result.new_parent}")


# ---------------------------------------------------------------------------
# Artifact attach / detach
# ---------------------------------------------------------------------------


def cmd_attach(args):
    root = find_tasks_root()
    loc = find_task_file(root, args.id)
    if loc is None:
        print(f"error: task {args.id} not found", file=sys.stderr)
        sys.exit(1)
    _, path = loc
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
    pat = re.compile(r"[ \t]*!\[[^\]]*\]\(artifacts/" + re.escape(args.id) + "/" + re.escape(target) + r"\)[ \t]*\n?")
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


# ---------------------------------------------------------------------------
# Search / stats
# ---------------------------------------------------------------------------


def cmd_search(args):
    root = find_tasks_root()
    spec = _spec_from_args(args, defaults={"search": args.query})
    matches = filter_tasks(root, spec)

    if args.json:
        out = [{"status": s, **t} for s, t in matches]
        print(json.dumps(out, indent=2))
        return
    if not matches:
        print("No tasks found.")
        return

    for status, t in matches:
        print(_fmt_task_row(status, t))


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
        print(
            json.dumps(
                {
                    "total": len(tasks),
                    "hairy": hairy_count,
                    "shaving": shaving_count,
                    "shorn": shorn_count,
                    "by_type": by_type,
                    "by_priority": dict(sorted(by_priority.items())),
                },
                indent=2,
            )
        )
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


# ---------------------------------------------------------------------------
# Beads import
# ---------------------------------------------------------------------------


def cmd_import_beads(args):
    root = find_tasks_root()

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

    existing_ids: set[str] = set()
    for d in (root / s for s in STATUSES):
        if d.exists():
            for f in d.glob("*.md"):
                existing_ids.add(f.stem)

    skip_types = {"message", "molecule", "merge-request"}
    skip_statuses = {"tombstone", "pinned"}
    priority_map = {0: 1, 1: 2, 2: 3, 3: 4, 4: 5}
    type_map = {"bug": "bug", "feature": "feature"}
    bead_status_map = {"in_progress": SHAVING, "closed": SHORN}

    created = {s: 0 for s in STATUSES}
    skipped = 0

    for line in jsonl_path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        bead = json.loads(line)

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

        if bead_id in existing_ids:
            skipped += 1
            continue

        yak_dir = bead_status_map.get(bead.get("status", ""), HAIRY)

        task: dict = {"id": bead_id}
        if bead.get("title"):
            task["title"] = bead["title"]
        task["type"] = type_map.get(bead.get("issue_type", ""), "task")
        task["priority"] = priority_map.get(bead.get("priority", 2), 3)

        task["created"] = bead.get("created_at") or now_iso()
        task["updated"] = bead.get("updated_at") or task["created"]

        deps = bead.get("dependencies", [])
        if deps:
            dep_ids = [d["depends_on_id"] for d in deps if d.get("type") == "blocks" and d.get("depends_on_id")]
            if dep_ids:
                task["depends_on"] = dep_ids

        if bead.get("labels"):
            task["labels"] = bead["labels"]

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
    print(
        f"{prefix}Imported {total} tasks (hairy: {created[HAIRY]}, "
        f"shaving: {created[SHAVING]}, shorn: {created[SHORN]}), skipped {skipped}"
    )


# ---------------------------------------------------------------------------
# Rollup: project yaks onto the external issues they point at
# ---------------------------------------------------------------------------


def cmd_rollup(args):
    from yaklib import rollup as _rollup

    root = find_tasks_root()
    spec = _spec_from_args(args)
    groups, unsourced = _rollup.build_rollup(root, spec)

    if getattr(args, "keys", False):
        keys = []
        seen = set()
        for g in groups:
            k = g["key"] or g["source"]
            if k not in seen:
                seen.add(k)
                keys.append(k)
        if args.json:
            print(json.dumps(keys, indent=2))
        else:
            for k in keys:
                print(k)
        return

    if args.json:
        out = [
            {
                "source": g["source"],
                "tracker": g["tracker"],
                "key": g["key"],
                "yaks": [
                    {
                        "id": y["task"]["id"],
                        "status": y["status"],
                        "title": y["task"].get("title", ""),
                        "inherited": y["inherited"],
                        "inherited_from": y["inherited_from"],
                    }
                    for y in g["yaks"]
                ],
            }
            for g in groups
        ]
        print(json.dumps(out, indent=2))
        return

    if not groups:
        print("No yaks with an external source.")
        return

    for g in groups:
        head = g["key"] or g["source"]
        print(f"{head}  ({g['tracker']})  {g['source']}")
        for y in g["yaks"]:
            row = _fmt_task_row(y["status"], y["task"])
            if y["inherited"]:
                row += f"  (via {y['inherited_from']})"
            print(row)
        print()
    if unsourced:
        noun = "yak" if unsourced == 1 else "yaks"
        print(f"{unsourced} {noun} in scope with no external source (omitted).")


# ---------------------------------------------------------------------------
# Dispatch table (used by the CLI entry point)
# ---------------------------------------------------------------------------

COMMANDS = {
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
    "rollup": cmd_rollup,
    "import-beads": cmd_import_beads,
}
