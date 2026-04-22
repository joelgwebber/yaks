"""Task-mutating operations invoked from the TUI (create/edit/delete,
quick-adjust, deps, artifacts, comments, editor integration).

All functions take the App instance as their first argument and use it
for the bits that genuinely depend on app state: root, stdscr, reload,
_rebuild_detail, notification. Everything else goes through yaklib.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from datetime import datetime, timezone
from pathlib import Path

import curses
import yaml

from yaklib import artifacts as _artifacts
from yaklib import clipboard as _clipboard
from yaklib import deps as _deps
from yaklib import reparent as _reparent
from yaklib.model import (
    DEAD,
    HAIRY,
    SHAVING,
    SHORN,
    find_children,
    find_task_file,
    generate_id,
    load_config,
    load_task,
    move_task,
    next_child_number,
    now_iso,
    save_task,
)
from yaktui import dialogs as _dialogs


# ---------------------------------------------------------------------------
# Template for `create` flow
# ---------------------------------------------------------------------------

def build_template(root: Path, parent: str | None, yak_type: str = "task") -> str:
    lines = ["---"]
    if parent:
        presult = find_task_file(root, parent)
        ptitle = ""
        if presult:
            pt = load_task(presult[1])
            ptitle = pt.get("title", "")
        lines.append(f"# Child of {parent}: {ptitle}")
    lines.append("# Fill in the title. Save and exit to create, or exit without")
    lines.append("# saving (or leave title blank) to cancel.")
    lines.append("title: ")
    lines.append("# type: task | bug | feature | idea")
    lines.append(f"type: {yak_type}")
    lines.append("# priority: 1 (high) .. 3 (low)")
    lines.append("priority: 2")
    lines.append("# Optional:")
    lines.append("# labels: [foo, bar]")
    lines.append("# depends_on: [yak-xxxx]")
    lines.append("---")
    lines.append("")
    lines.append("")
    return "\n".join(lines)


def parse_template(text: str) -> dict | None:
    """Parse a templated frontmatter + body into a task dict. None on failure."""
    if not text.startswith("---"):
        return None
    rest = text[3:].lstrip("\n")
    end = rest.find("\n---")
    if end < 0:
        return None
    fm = rest[:end]
    body = rest[end + 4:].strip()
    try:
        data = yaml.safe_load(fm) or {}
    except yaml.YAMLError:
        return None
    if not isinstance(data, dict):
        return None
    if body:
        data["description"] = body
    return data


# ---------------------------------------------------------------------------
# Editor integration
# ---------------------------------------------------------------------------

def edit_in_editor(app, initial_content: str) -> str | None:
    """Suspend curses, edit *initial_content* in $EDITOR, return the result."""
    editor = os.environ.get("EDITOR", "vi")
    tmp = tempfile.NamedTemporaryFile(
        mode="w", suffix=".yak.md", delete=False, encoding="utf-8")
    try:
        tmp.write(initial_content)
        tmp.close()

        curses.def_prog_mode()
        curses.endwin()
        try:
            result = subprocess.call([editor, tmp.name])
        except FileNotFoundError:
            curses.reset_prog_mode()
            app.stdscr.refresh()
            app.notification = f"editor '{editor}' not found"
            return None
        curses.reset_prog_mode()
        app.stdscr.refresh()

        if result != 0:
            return None
        with open(tmp.name, "r", encoding="utf-8") as f:
            return f.read()
    finally:
        try:
            os.unlink(tmp.name)
        except OSError:
            pass


def edit_file_in_editor(app, path: Path) -> str | None:
    """Suspend curses and run $EDITOR on an existing file."""
    editor = os.environ.get("EDITOR", "vi")
    curses.def_prog_mode()
    curses.endwin()
    try:
        result = subprocess.call([editor, str(path)])
    except FileNotFoundError:
        curses.reset_prog_mode()
        app.stdscr.refresh()
        app.notification = f"editor '{editor}' not found"
        return None
    curses.reset_prog_mode()
    app.stdscr.refresh()
    if result != 0:
        return None
    return path.read_text()


# ---------------------------------------------------------------------------
# Selection helper shared by create/edit
# ---------------------------------------------------------------------------

def _select_task(app, tid: str) -> None:
    """After reload, move the cursor to *tid* and rebuild detail."""
    for i, (_, t, _, _) in enumerate(app.tasks):
        if t["id"] == tid:
            app.cursor = i
            app._fix_scroll()
            app._rebuild_detail()
            break


# ---------------------------------------------------------------------------
# Create / edit / delete
# ---------------------------------------------------------------------------

def create_task(app, parent: str | None = None, yak_type: str = "task") -> None:
    template = build_template(app.root, parent, yak_type=yak_type)
    edited = edit_in_editor(app, template)
    if edited is None or edited.strip() == template.strip():
        app.notification = "create cancelled"
        return

    data = parse_template(edited)
    if not data or not data.get("title", "").strip():
        app.notification = "create cancelled"
        return

    cfg = load_config(app.root)
    prefix = cfg.get("prefix", "yak")
    if parent:
        if not find_task_file(app.root, parent):
            app.notification = f"parent {parent} not found"
            return
        tid = f"{parent}.{next_child_number(app.root, parent)}"
    else:
        tid = generate_id(app.root, prefix)

    now = now_iso()
    task = {
        "id": tid,
        "title": data["title"].strip(),
        "type": data.get("type") or "task",
        "priority": data.get("priority") if data.get("priority") is not None else 2,
        "created": now,
        "updated": now,
    }
    if data.get("depends_on"):
        task["depends_on"] = data["depends_on"]
    if data.get("labels"):
        task["labels"] = data["labels"]
    if data.get("description"):
        task["description"] = data["description"]

    save_task(app.root / HAIRY / f"{tid}.md", task)

    app.tab = 0
    app.filter_mode = "all"
    app.search_query = ""
    app.reload()
    _select_task(app, tid)
    app.notification = f"created {tid}"


def edit_task(app, tid: str) -> None:
    result = find_task_file(app.root, tid)
    if not result:
        app.notification = f"{tid} not found"
        return

    _, path = result
    original = load_task(path)
    original_id = original.get("id")
    original_created = original.get("created")

    content = path.read_text()
    edited = edit_file_in_editor(app, path)
    if edited is None:
        app.notification = "edit cancelled"
        return
    if edited == content:
        app.notification = "no changes"
        return

    data = parse_template(edited)
    if not data or not data.get("title", "").strip():
        app.notification = "edit cancelled (invalid)"
        return

    data["id"] = original_id
    if original_created:
        data["created"] = original_created
    data["updated"] = now_iso()

    save_task(path, data)
    app.reload()
    _select_task(app, tid)
    app.notification = f"edited {tid}"


def delete_task(app, tid: str) -> None:
    result = find_task_file(app.root, tid)
    if not result:
        app.notification = f"{tid} not found"
        return

    _, path = result
    children = find_children(app.root, tid)
    if children:
        app.notification = f"{tid} has {len(children)} child(ren); delete them first"
        return

    task = load_task(path)
    title = task.get("title", "")[:40]
    if not _dialogs.confirm(app.stdscr, f"Delete {tid} ({title})? (y/N): "):
        app.notification = "delete cancelled"
        return

    try:
        path.unlink()
    except OSError as e:
        app.notification = f"delete failed: {e}"
        return

    app.reload()
    app.notification = f"deleted {tid}"


# ---------------------------------------------------------------------------
# Quick adjusts
# ---------------------------------------------------------------------------

def _load(app, tid: str):
    result = find_task_file(app.root, tid)
    if not result:
        return None, None
    _, path = result
    return path, load_task(path)


def quick_adjust_priority(app, tid: str) -> None:
    path, task = _load(app, tid)
    if path is None:
        return
    choice = _dialogs.pick(app.stdscr,
                           f"Priority for {tid}: 1=high 2=med 3=low  (Esc=cancel)",
                           "123")
    if choice is None:
        app.notification = "priority unchanged"
        return
    new_p = int(choice)
    if task.get("priority") == new_p:
        app.notification = f"{tid} already p{new_p}"
        return
    task["priority"] = new_p
    task["updated"] = now_iso()
    save_task(path, task)
    app.reload()
    app.notification = f"{tid} -> p{new_p}"


def quick_adjust_type(app, tid: str) -> None:
    path, task = _load(app, tid)
    if path is None:
        return
    type_map = {"t": "task", "b": "bug", "f": "feature", "i": "idea"}
    choice = _dialogs.pick(
        app.stdscr,
        f"Type for {tid}: t=task b=bug f=feature i=idea  (Esc=cancel)",
        "tbfi")
    if choice is None:
        app.notification = "type unchanged"
        return
    new_t = type_map[choice]
    if task.get("type") == new_t:
        app.notification = f"{tid} already {new_t}"
        return
    task["type"] = new_t
    task["updated"] = now_iso()
    save_task(path, task)
    app.reload()
    app.notification = f"{tid} -> {new_t}"


_STATE_PICKER = {
    "h": HAIRY,
    "s": SHAVING,
    "n": SHORN,
    "x": DEAD,  # slaughter
}


def quick_adjust_state(app, tid: str) -> None:
    """Move *tid* to a new status via a single-key picker (h/s/n/x)."""
    res = find_task_file(app.root, tid)
    if not res:
        return
    cur_status, _ = res
    choice = _dialogs.pick(
        app.stdscr,
        f"State for {tid}: h=hairy s=shaving n=shorn x=slaughter  (Esc=cancel)",
        "hsnx")
    if choice is None:
        app.notification = "state unchanged"
        return
    dest = _STATE_PICKER[choice]
    if dest == cur_status:
        app.notification = f"{tid} already {dest}"
        return
    ok, msg = move_task(app.root, tid, dest)
    if not ok:
        app.notification = msg
        return
    app.reload()
    app.notification = f"{tid} → {dest}"


def quick_adjust_labels(app, tid: str) -> None:
    path, task = _load(app, tid)
    if path is None:
        return
    current = task.get("labels") or []
    initial = ", ".join(current)
    edited = _dialogs.edit_prompt(app.stdscr, f"Labels for {tid}: ", initial,
                                  vim=app.vim_mode)
    if edited is None:
        app.notification = "labels unchanged"
        return
    new_labels = [l.strip() for l in edited.split(",") if l.strip()] if edited.strip() else []
    if new_labels == current:
        app.notification = "labels unchanged"
        return
    if new_labels:
        task["labels"] = new_labels
    else:
        task.pop("labels", None)
    task["updated"] = now_iso()
    save_task(path, task)
    app.reload()
    app.notification = f"{tid} labels: {', '.join(new_labels) if new_labels else '(none)'}"


# ---------------------------------------------------------------------------
# Dependencies
# ---------------------------------------------------------------------------

def add_dependency(app, tid: str) -> None:
    path, task = _load(app, tid)
    if path is None:
        return
    existing_deps = set(task.get("depends_on") or [])
    target = _dialogs.fuzzy_pick_task(
        app.stdscr, app.root, f"{tid} depends on: ",
        exclude_ids={tid} | existing_deps, vim=app.vim_mode)
    if target is None:
        app.notification = "add dep cancelled"
        return
    # Cycle check: refuse if `target` already transitively depends on `tid`.
    if _deps.depends_on_transitively(app.root, target, tid):
        app.notification = f"refused: would create a cycle ({target} -> {tid})"
        return
    deps = list(existing_deps)
    deps.append(target)
    task["depends_on"] = deps
    task["updated"] = now_iso()
    save_task(path, task)
    app.reload()
    app.notification = f"{tid} -> depends on {target}"


def remove_dependency(app, tid: str, dep_id: str) -> None:
    """Remove *dep_id* from *tid*'s depends_on list. Caller has already
    identified the specific dep to remove (e.g. via the detail cursor)."""
    path, task = _load(app, tid)
    if path is None:
        return
    deps = list(task.get("depends_on") or [])
    if dep_id not in deps:
        app.notification = f"{tid} does not depend on {dep_id}"
        return
    if not _dialogs.confirm(app.stdscr,
                            f"Remove dep {tid} -/-> {dep_id}? (y/N): "):
        app.notification = "remove dep cancelled"
        return
    deps.remove(dep_id)
    if deps:
        task["depends_on"] = deps
    else:
        task.pop("depends_on", None)
    task["updated"] = now_iso()
    save_task(path, task)
    app.reload()
    app.notification = f"{tid} -/-> {dep_id}"


def handle_dep_key(app) -> None:
    """Context-aware dispatcher for the `B` dep hotkey.

    - In the detail pane, if the cursor is sitting on a 'Depends on:' row,
      remove that specific dep.
    - Anywhere else (list pane, or detail with cursor not on a dep row),
      add a new dep to the current task via the fuzzy picker.
    """
    tid = app._current_task_id()
    if not tid:
        return

    if app.focus == "detail" and 0 <= app.detail_line_cursor < len(app.detail_lines):
        dl = app.detail_lines[app.detail_line_cursor]
        if dl.kind == "dep_link" and dl.task_id:
            remove_dependency(app, tid, dl.task_id)
            return

    add_dependency(app, tid)


# ---------------------------------------------------------------------------
# Comments + artifact attach
# ---------------------------------------------------------------------------

def reparent_task(app, tid: str) -> None:
    """Move *tid* under a new parent (or to the top level).

    Flow: pick action (p=pick parent / u=unparent) → fuzzy-pick new parent if
    needed → plan + confirm with a rename summary → apply → reload and move
    the cursor to the renamed yak.
    """
    choice = _dialogs.pick(
        app.stdscr,
        f"Reparent {tid}: p=pick parent, u=unparent  (Esc=cancel)",
        "pu")
    if choice is None:
        app.notification = "reparent cancelled"
        return

    if choice == "u":
        new_parent = None
    else:
        new_parent = _dialogs.fuzzy_pick_task(
            app.stdscr, app.root, f"New parent for {tid}: ",
            exclude_ids={tid}, vim=app.vim_mode)
        if new_parent is None:
            app.notification = "reparent cancelled"
            return

    try:
        plan = _reparent.plan_reparent(app.root, tid, new_parent)
    except _reparent.ReparentError as e:
        app.notification = f"reparent refused: {e}"
        return

    n = len(plan.id_map)
    art = len(plan.artifact_dirs)
    target = new_parent or "top level"
    parts = [f"{n} rename" + ("s" if n != 1 else "")]
    if art:
        parts.append(f"{art} artifact dir" + ("s" if art != 1 else ""))
    summary = f"Reparent {tid} → {target}? ({', '.join(parts)}) (y/N): "
    if not _dialogs.confirm(app.stdscr, summary):
        app.notification = "reparent cancelled"
        return

    try:
        _reparent.apply(plan, app.root)
    except _reparent.ReparentError as e:
        app.notification = f"reparent failed: {e}"
        return

    app.reload()
    # Try to move the cursor onto the renamed yak.
    for i, (_, t, _, _) in enumerate(app.tasks):
        if t["id"] == plan.new_id:
            app.cursor = i
            app._fix_scroll()
            app._rebuild_detail()
            break
    app.notification = f"{plan.old_id} → {plan.new_id}"


def add_comment(app, tid: str) -> None:
    text = _dialogs.input_prompt(app.stdscr, "Comment: ", vim=app.vim_mode)
    if not text:
        app.notification = "comment cancelled"
        return
    path, task = _load(app, tid)
    if path is None:
        return
    now = now_iso()
    # One blank line before the heading, none between heading and body —
    # comments are usually one-liners, keep them tight but separated.
    desc = (task.get("description") or "").rstrip()
    sep = "\n\n" if desc else ""
    task["description"] = f"{desc}{sep}### {now}\n{text}"
    task["updated"] = now
    save_task(path, task)
    app.reload()
    app._rebuild_detail()
    app.notification = f"comment added to {tid}"


def attach_file(app, tid: str) -> None:
    path, _ = _load(app, tid)
    if path is None:
        return

    src_input = _dialogs.input_prompt(app.stdscr, "Attach path (empty = clipboard PNG): ",
                                      vim=app.vim_mode)
    if src_input is None:
        app.notification = "attach cancelled"
        return

    adir = _artifacts.artifacts_dir(app.root, tid)
    adir.mkdir(parents=True, exist_ok=True)

    if src_input.strip() == "":
        data = _clipboard.read_png()
        if not data:
            app.notification = "no PNG image on clipboard"
            return
        name = f"paste-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}.png"
        dest = adir / name
        dest.write_bytes(data)
    else:
        src = Path(src_input.strip()).expanduser()
        if not src.is_file():
            app.notification = f"not a file: {src}"
            return
        name = src.name
        dest = adir / name
        if dest.exists():
            app.notification = f"{name} already attached"
            return
        shutil.copy2(src, dest)

    desc = _dialogs.input_prompt(
        app.stdscr, f"Description for {name} (empty = filename): ",
        vim=app.vim_mode)
    if desc is None:
        desc = ""
    alt = desc.strip() or Path(name).stem
    link = f"![{alt}](artifacts/{tid}/{name})"

    task = load_task(path)
    body = task.get("description", "") or ""
    if body and not body.endswith("\n"):
        body += "\n"
    body += "\n" + link + "\n"
    task["description"] = body
    task["updated"] = now_iso()
    save_task(path, task)
    app.reload()
    app._rebuild_detail()
    app.notification = f"attached {name}"


def copy_to_clipboard(app, text: str) -> None:
    if _clipboard.copy_text(text):
        app.notification = f"copied {text}"
    else:
        app.notification = "clipboard not available"
