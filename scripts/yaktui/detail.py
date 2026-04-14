"""Detail pane line model + renderer.

build_detail_lines produces a list[DetailLine] ready for the detail pane
to display. Each line knows its kind (header, subheader, field, code, etc.)
and optionally carries a navigable target — either a task_id to jump to or
an open_path to launch in an external viewer.
"""

from __future__ import annotations

import textwrap
from pathlib import Path

from yaklib import artifacts as _artifacts
from yaklib import links as _links
from yaklib.format import humanize_date, status_emoji
from yaklib.model import find_children, find_task_file, load_task, parent_id


class DetailLine:
    """A single line in the detail pane, optionally a navigable link."""

    __slots__ = ("text", "kind", "task_id", "open_path")

    def __init__(self, text, kind="", task_id=None, open_path=None):
        self.text = text
        self.kind = kind
        self.task_id = task_id
        self.open_path = open_path

    @property
    def is_link(self):
        return self.task_id is not None or self.open_path is not None


def _wrap(text, width) -> list[str]:
    """Wrap a string to width, preserving leading indentation on each line."""
    if width <= 10 or not text:
        return [text]
    if len(text) <= width:
        return [text]
    stripped = text.lstrip(" ")
    lead = text[:len(text) - len(stripped)]
    wrapped = textwrap.wrap(
        stripped, width=max(1, width - len(lead)),
        break_long_words=False, break_on_hyphens=False)
    if not wrapped:
        return [text]
    return [lead + w for w in wrapped]


def build_detail_lines(root, task, status, width=80,
                       reverse_deps=None) -> list[DetailLine]:
    """Build the detail pane content for a task, wrapped to *width*."""
    lines: list[DetailLine] = []

    def emit(text, kind="", task_id=None):
        chunks = _wrap(text, width)
        lines.append(DetailLine(chunks[0], kind, task_id))
        for chunk in chunks[1:]:
            lines.append(DetailLine(chunk, kind))

    emit(f"Task: {task['id']}", "header")
    lines.append(DetailLine(""))

    title = task.get("title", "")
    emit(f"  {'Title:':<12s} {title}", "field")

    fields = [
        ("Status", status.capitalize()),
        ("Type", task.get("type", "-")),
        ("Priority", str(task.get("priority", "-"))),
        ("Created", humanize_date(task.get("created"))),
        ("Updated", humanize_date(task.get("updated"))),
    ]
    if task.get("commit"):
        fields.append(("Commit", task["commit"]))
    if task.get("labels"):
        fields.append(("Labels", ", ".join(task["labels"])))

    for label, value in fields:
        emit(f"  {label + ':':<12s} {value}", "field")

    for dep_id in task.get("depends_on", []):
        dep_result = find_task_file(root, dep_id)
        if dep_result:
            ds, dp = dep_result
            dt = load_task(dp)
            emit(f"  {'Depends on:':<12s} {status_emoji(ds)} {dep_id}  {dt.get('title', '')}",
                 "link", task_id=dep_id)
        else:
            emit(f"  {'Depends on:':<12s} {dep_id} (not found)", "field")

    pid = parent_id(task["id"])
    if pid:
        presult = find_task_file(root, pid)
        if presult:
            ps, pp = presult
            pt = load_task(pp)
            emit(f"  {'Parent:':<12s} {status_emoji(ps)} {pid}  {pt.get('title', '')}",
                 "link", task_id=pid)

    children = find_children(root, task["id"])
    if children:
        lines.append(DetailLine(""))
        lines.append(DetailLine("  Children:", "subheader"))
        for cs, ct in children:
            emit(f"    {status_emoji(cs)} {ct['id']}  {ct.get('title', '')}",
                 "link", task_id=ct["id"])

    if reverse_deps:
        blockers = sorted(reverse_deps.get(task["id"]) or [],
                          key=lambda p: p[1]["id"])
        if blockers:
            lines.append(DetailLine(""))
            lines.append(DetailLine("  Blocks:", "subheader"))
            for bs, bt in blockers:
                emit(f"    {status_emoji(bs)} {bt['id']}  {bt.get('title', '')}",
                     "link", task_id=bt["id"])

    desc = task.get("description", "")

    # Implicit references: yak-ID tokens in the body that aren't already
    # tracked as explicit deps / parent / children / blockers.
    explicit_ids: set[str] = set(task.get("depends_on") or [])
    if pid:
        explicit_ids.add(pid)
    for _, ct in children:
        explicit_ids.add(ct["id"])
    if reverse_deps:
        for _, bt in reverse_deps.get(task["id"]) or []:
            explicit_ids.add(bt["id"])
    refs = _links.resolve_references(root, task, exclude=explicit_ids)
    if refs:
        lines.append(DetailLine(""))
        lines.append(DetailLine("  References:", "subheader"))
        for rs, rt in refs:
            emit(f"    {status_emoji(rs)} {rt['id']}  {rt.get('title', '')}",
                 "link", task_id=rt["id"])

    artifacts = _artifacts.parse_artifacts(desc, task["id"])
    if artifacts:
        lines.append(DetailLine(""))
        lines.append(DetailLine("  Artifacts:", "subheader"))
        for alt, aname in artifacts:
            apath = _artifacts.artifacts_dir(root, task["id"]) / aname
            label = aname if not alt or alt == Path(aname).stem else f"{aname}  ({alt})"
            lines.append(DetailLine(f"    {label}", "link", open_path=apath))

    if desc:
        lines.append(DetailLine(""))
        lines.append(DetailLine("  Description:", "subheader"))
        in_code_block = False
        for dline in desc.split("\n"):
            stripped = dline.strip()
            if stripped.startswith("```"):
                in_code_block = not in_code_block
                for chunk in _wrap(f"    {dline}", width):
                    lines.append(DetailLine(chunk, "code"))
                continue
            if in_code_block:
                for chunk in _wrap(f"    {dline}", width):
                    lines.append(DetailLine(chunk, "code"))
            elif not stripped:
                lines.append(DetailLine("    "))
            elif stripped.startswith("#"):
                for chunk in _wrap(f"    {dline}", width):
                    lines.append(DetailLine(chunk, "md_heading"))
            elif stripped.startswith("> "):
                for chunk in _wrap(f"    {dline}", width):
                    lines.append(DetailLine(chunk, "quote"))
            else:
                for chunk in _wrap(f"    {dline}", width):
                    lines.append(DetailLine(chunk, "desc"))

    # Trailing padding so the last content line can scroll above the bottom.
    lines.append(DetailLine(""))
    lines.append(DetailLine(""))

    return lines
