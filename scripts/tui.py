# /// script
# requires-python = ">=3.10"
# dependencies = ["pyyaml>=6.0"]
# ///
"""Yaks TUI — curses-based terminal interface for the Yaks task tracker."""

import curses
import os
import subprocess
import sys
import tempfile
import textwrap
from datetime import datetime, timezone
from pathlib import Path

import yaml

sys.path.insert(0, str(Path(__file__).parent))
import yak
from yaklib.format import humanize_date, status_char


# ---------------------------------------------------------------------------
# Color pairs
# ---------------------------------------------------------------------------

C_ID = 1
C_P1 = 2
C_P2 = 3
C_P3 = 4
C_TAB_ACTIVE = 5
C_GHOST = 6
C_TYPE = 7
C_SELECTED = 8
C_LABEL = 9
C_HEADER = 10
C_HELP = 11
C_SEARCH = 12
C_LINK = 13
C_LINK_SEL = 14
C_MATCH = 15
C_GHOST_HAIRY = 16
C_GHOST_SHAVING = 17
C_GHOST_SHORN = 18
C_CODE = 19
C_MD_HEADING = 20


def init_colors():
    curses.use_default_colors()
    curses.init_pair(C_ID, curses.COLOR_BLUE, -1)
    curses.init_pair(C_P1, curses.COLOR_RED, -1)
    curses.init_pair(C_P2, curses.COLOR_YELLOW, -1)
    curses.init_pair(C_P3, curses.COLOR_GREEN, -1)
    curses.init_pair(C_TAB_ACTIVE, curses.COLOR_BLACK, curses.COLOR_WHITE)
    curses.init_pair(C_GHOST, 8, -1)
    curses.init_pair(C_TYPE, curses.COLOR_CYAN, -1)
    # Selection: subtle dark gray bg on 256-color terminals, else plain blue
    if curses.COLORS >= 256:
        curses.init_pair(C_SELECTED, -1, 237)  # default fg on dark gray
        curses.init_pair(C_LINK_SEL, curses.COLOR_BLUE, 237)
    else:
        curses.init_pair(C_SELECTED, curses.COLOR_WHITE, curses.COLOR_BLUE)
        curses.init_pair(C_LINK_SEL, curses.COLOR_CYAN, curses.COLOR_BLUE)
    curses.init_pair(C_LABEL, curses.COLOR_MAGENTA, -1)
    curses.init_pair(C_HEADER, curses.COLOR_WHITE, -1)
    curses.init_pair(C_HELP, curses.COLOR_BLACK, curses.COLOR_WHITE)
    curses.init_pair(C_SEARCH, curses.COLOR_YELLOW, -1)
    curses.init_pair(C_LINK, curses.COLOR_BLUE, -1)
    curses.init_pair(C_MATCH, curses.COLOR_BLACK, curses.COLOR_YELLOW)
    # Ghost state badges — distinct prominence by completion state
    curses.init_pair(C_GHOST_HAIRY, curses.COLOR_YELLOW, -1)   # attention: undone
    curses.init_pair(C_GHOST_SHAVING, 8, -1)                    # faded: in progress
    curses.init_pair(C_GHOST_SHORN, curses.COLOR_GREEN, -1)     # done
    curses.init_pair(C_CODE, curses.COLOR_CYAN, -1)
    curses.init_pair(C_MD_HEADING, curses.COLOR_WHITE, -1)


# ---------------------------------------------------------------------------
# Tree building
# ---------------------------------------------------------------------------

class TaskNode:
    __slots__ = ("status", "task", "children", "ghost")

    def __init__(self, status, task, ghost=False):
        self.status = status
        self.task = task
        self.children = []
        self.ghost = ghost


def build_tree(root: Path, status_filter: str | None, filter_mode: str,
               search_query: str) -> list[tuple[str, dict, int, bool]]:
    """Return flat list of (status, task, depth, ghost) for display."""
    if search_query:
        return _build_search_results(root, search_query)

    all_by_id: dict[str, tuple[str, dict]] = {}
    for s in yak.STATUSES:
        for st, t in yak.all_tasks(root, s):
            all_by_id[t["id"]] = (st, t)

    if status_filter and filter_mode in ("all", "next", "tangled"):
        primary = [(s, t) for s, t in all_by_id.values() if s == status_filter]
    else:
        primary = list(all_by_id.values())

    if filter_mode == "next" and status_filter == yak.HAIRY:
        shorn_ids = {t["id"] for s, t in all_by_id.values() if s == yak.SHORN}
        primary = [(s, t) for s, t in primary
                   if not t.get("depends_on") or
                   all(d in shorn_ids for d in t.get("depends_on", []))]
    elif filter_mode == "tangled" and status_filter == yak.HAIRY:
        shorn_ids = {t["id"] for s, t in all_by_id.values() if s == yak.SHORN}
        primary = [(s, t) for s, t in primary
                   if any(d not in shorn_ids for d in t.get("depends_on", []))]

    primary_ids = {t["id"] for _, t in primary}

    nodes: dict[str, TaskNode] = {}
    for s, t in primary:
        nodes[t["id"]] = TaskNode(s, t, ghost=False)

    # Ghost ancestors: walk up from primaries to keep tree rooted
    for tid in list(primary_ids):
        pid = yak.parent_id(tid)
        while pid and pid not in nodes:
            if pid in all_by_id:
                ps, pt = all_by_id[pid]
                nodes[pid] = TaskNode(ps, pt, ghost=True)
            pid = yak.parent_id(pid) if pid else None

    # Ghost descendants: include children (recursively) of all visible nodes
    # (primaries + ghost ancestors) so mixed parent/child states stay visible.
    child_prefixes = {tid + "." for tid in nodes}
    for other_id, (os_, ot) in all_by_id.items():
        if other_id in nodes:
            continue
        # If any prefix is an ancestor of other_id, include it as a ghost
        for prefix in child_prefixes:
            if other_id.startswith(prefix):
                nodes[other_id] = TaskNode(os_, ot, ghost=True)
                break

    roots = []
    for tid, node in nodes.items():
        pid = yak.parent_id(tid)
        if pid and pid in nodes:
            nodes[pid].children.append(node)
        else:
            roots.append(node)

    def _child_status_rank(status):
        if status == yak.SHAVING:
            return 0
        if status == yak.SHORN:
            return 2
        return 1

    def sort_children(node: TaskNode):
        node.children.sort(key=lambda n: (_child_status_rank(n.status), n.task.get("priority", 9), _child_sort_key(n.task["id"])))
        for c in node.children:
            sort_children(c)

    if status_filter == yak.SHORN:
        roots.sort(key=lambda n: n.task.get("updated", ""), reverse=True)
    else:
        roots.sort(key=lambda n: (n.task.get("priority", 9), n.task["id"]))
    for r in roots:
        sort_children(r)

    flat = []
    def flatten(node: TaskNode, depth: int):
        flat.append((node.status, node.task, depth, node.ghost))
        for c in node.children:
            flatten(c, depth + 1)
    for r in roots:
        flatten(r, 0)
    return flat


def _build_search_results(root, query):
    q = query.lower()
    results = []
    for s in yak.STATUSES:
        for st, t in yak.all_tasks(root, s):
            title = t.get("title", "").lower()
            desc = t.get("description", "").lower()
            if q in title or q in desc or q in t.get("id", "").lower():
                results.append((st, t, 0, False))
    results.sort(key=lambda x: (x[0] != yak.HAIRY, x[0] != yak.SHAVING,
                                x[1].get("priority", 9)))
    return results


def _child_sort_key(tid):
    dot = tid.rfind(".")
    if dot >= 0:
        suffix = tid[dot + 1:]
        if suffix.isdigit():
            return int(suffix)
    return 0


def _ghost_badge_attr(status):
    """Return the color pair + attributes for a ghost badge of the given state."""
    if status == yak.HAIRY:
        return curses.color_pair(C_GHOST_HAIRY) | curses.A_BOLD
    if status == yak.SHORN:
        return curses.color_pair(C_GHOST_SHORN)
    return curses.color_pair(C_GHOST_SHAVING) | curses.A_DIM


# ---------------------------------------------------------------------------
# Detail line model
# ---------------------------------------------------------------------------

class DetailLine:
    """A single line in the detail pane, optionally a navigable link."""
    __slots__ = ("text", "kind", "task_id", "open_path")

    def __init__(self, text, kind="", task_id=None, open_path=None):
        self.text = text
        self.kind = kind       # header, subheader, field, child, desc, link, ""
        self.task_id = task_id  # non-None means this line is navigable
        self.open_path = open_path  # non-None means Enter opens this file externally

    @property
    def is_link(self):
        return self.task_id is not None or self.open_path is not None


def _wrap(text, width) -> list[str]:
    """Wrap a string to the given width, preserving leading indentation.
    Continuation lines use the same lead as the original."""
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
    """Build the detail pane content for a task, wrapped to `width`."""
    def emit(text, kind="", task_id=None):
        """Append a line, wrapping text to width. Only the first chunk is a link."""
        chunks = _wrap(text, width)
        lines.append(DetailLine(chunks[0], kind, task_id))
        for chunk in chunks[1:]:
            lines.append(DetailLine(chunk, kind))  # continuation, not a link

    lines = []
    emit(f"Task: {task['id']}", "header")
    lines.append(DetailLine(""))

    # Title gets wrapped on its own line so long titles are readable
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

    # Dependencies as links
    for dep_id in task.get("depends_on", []):
        dep_result = yak.find_task_file(root, dep_id)
        if dep_result:
            ds, dp = dep_result
            dt = yak.load_task(dp)
            sc = status_char(ds)
            emit(f"  {'Depends on:':<12s} [{sc}] {dep_id}  {dt.get('title', '')}",
                 "link", task_id=dep_id)
        else:
            emit(f"  {'Depends on:':<12s} {dep_id} (not found)", "field")

    # Parent as link
    pid = yak.parent_id(task["id"])
    if pid:
        presult = yak.find_task_file(root, pid)
        if presult:
            ps, pp = presult
            pt = yak.load_task(pp)
            sc = status_char(ps)
            emit(f"  {'Parent:':<12s} [{sc}] {pid}  {pt.get('title', '')}",
                 "link", task_id=pid)

    # Children as links
    children = yak.find_children(root, task["id"])
    if children:
        lines.append(DetailLine(""))
        lines.append(DetailLine("  Children:", "subheader"))
        for cs, ct in children:
            emit(f"    [{status_char(cs)}] {ct['id']}  {ct.get('title', '')}",
                 "link", task_id=ct["id"])

    # Reverse deps: tasks that depend on this one ("Blocks:")
    if reverse_deps:
        blockers = reverse_deps.get(task["id"]) or []
        # Sort by id for stable rendering
        blockers = sorted(blockers, key=lambda p: p[1]["id"])
        if blockers:
            lines.append(DetailLine(""))
            lines.append(DetailLine("  Blocks:", "subheader"))
            for bs, bt in blockers:
                emit(f"    [{status_char(bs)}] {bt['id']}  {bt.get('title', '')}",
                     "link", task_id=bt["id"])

    # Artifacts as openable links
    desc = task.get("description", "")
    artifacts = yak.parse_artifacts(desc, task["id"])
    if artifacts:
        lines.append(DetailLine(""))
        lines.append(DetailLine("  Artifacts:", "subheader"))
        for alt, aname in artifacts:
            apath = yak.artifacts_dir(root, task["id"]) / aname
            label = f"{aname}" if not alt or alt == Path(aname).stem else f"{aname}  ({alt})"
            lines.append(DetailLine(f"    {label}", "link", open_path=apath))

    # Description (with basic markdown styling)
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


# ---------------------------------------------------------------------------
# TUI
# ---------------------------------------------------------------------------

TABS = [
    (yak.HAIRY, "\U0001f9ac Hairy"),
    (yak.SHAVING, "\u2702\ufe0f  Shaving"),
    (yak.SHORN, "\U0001f411 Shorn"),
]


class TUI:
    def __init__(self, stdscr, root):
        self.stdscr = stdscr
        self.root = root

        # List state
        self.tab = 0
        self.cursor = 0
        self.scroll = 0
        self.filter_mode = "all"
        self.search_query = ""
        self.tasks = []

        # Detail state
        self.focus = "list"  # "list" or "detail"
        self.detail_lines = []
        self.detail_line_cursor = 0  # index into detail_lines
        self.detail_scroll = 0
        self.detail_search = ""
        self.detail_matches = []  # line indices matching search
        self._detail_build_width = 0  # width lines were wrapped for

        # Navigation history: list of task IDs
        self.nav_history = []
        self.nav_pos = -1

        # UI state
        self.show_help = False
        self.message = ""
        self.notification = ""  # top-right transient notice; clears on next key

        # Auto-refresh state
        self._fs_sig = None

        # Dep state
        self.blocked_ids: set[str] = set()
        # Reverse-dep map: task id -> list of (status, task) that depend on it
        self.reverse_deps: dict[str, list[tuple[str, dict]]] = {}

        curses.curs_set(0)
        self.stdscr.timeout(500)  # poll filesystem every 500ms when idle
        init_colors()
        self.reload()
        self._fs_sig = self._scan_fs()

    def reload(self):
        status = TABS[self.tab][0]
        self.tasks = build_tree(self.root, status, self.filter_mode,
                                self.search_query)
        self._recompute_blocked()
        if self.cursor >= len(self.tasks):
            self.cursor = max(0, len(self.tasks) - 1)
        self._fix_scroll()
        self._rebuild_detail()
        self._fs_sig = self._scan_fs()

    def _recompute_blocked(self):
        """Update blocked_ids and reverse_deps from all tasks on disk.
        Dead deps are treated as resolved (slaughtering a dep unblocks its
        dependents).
        """
        visible = yak.all_tasks(self.root)
        dead = yak.all_tasks(self.root, yak.DEAD)
        resolved_ids = (
            {t["id"] for s, t in visible if s == yak.SHORN}
            | {t["id"] for _, t in dead}
        )
        blocked = set()
        reverse: dict[str, list[tuple[str, dict]]] = {}
        for s, t in visible:
            deps = t.get("depends_on") or []
            if s == yak.HAIRY and any(d not in resolved_ids for d in deps):
                blocked.add(t["id"])
            for d in deps:
                reverse.setdefault(d, []).append((s, t))
        self.blocked_ids = blocked
        self.reverse_deps = reverse

    def _reload_preserving_position(self):
        """Reload while keeping the cursor on the same task id if possible."""
        current_id = None
        if self.tasks and 0 <= self.cursor < len(self.tasks):
            current_id = self.tasks[self.cursor][1]["id"]
        self.reload()
        if current_id:
            for i, (_, t, _, _) in enumerate(self.tasks):
                if t["id"] == current_id:
                    self.cursor = i
                    self._fix_scroll()
                    self._rebuild_detail()
                    break

    def _scan_fs(self):
        """Return a signature that changes when task files change."""
        total_mtime = 0.0
        count = 0
        for s in yak.STATUSES:
            d = self.root / s
            if not d.exists():
                continue
            try:
                total_mtime += d.stat().st_mtime
            except OSError:
                pass
            for f in d.glob("*.md"):
                try:
                    total_mtime += f.stat().st_mtime
                    count += 1
                except OSError:
                    pass
        return (count, total_mtime)

    def _check_fs_changes(self):
        """Called on idle poll. Reloads if filesystem changed."""
        sig = self._scan_fs()
        if sig != self._fs_sig:
            self._fs_sig = sig
            self._reload_preserving_position()

    def _rebuild_detail(self, width=None):
        if not self.tasks or self.cursor >= len(self.tasks):
            self.detail_lines = []
            self.detail_line_cursor = 0
            self.detail_scroll = 0
            self._detail_build_width = width or 0
            return

        if width is None:
            width = self._detail_build_width or 80
        self._detail_build_width = width

        status, task, _, _ = self.tasks[self.cursor]
        self.detail_lines = build_detail_lines(
            self.root, task, status, width, reverse_deps=self.reverse_deps)
        # Start cursor on the first link, or line 0 if no links
        self.detail_line_cursor = 0
        for i, dl in enumerate(self.detail_lines):
            if dl.is_link:
                self.detail_line_cursor = i
                break
        self.detail_scroll = 0
        self._apply_detail_search()

    def _apply_detail_search(self):
        if not self.detail_search:
            self.detail_matches = []
            return
        q = self.detail_search.lower()
        self.detail_matches = [i for i, dl in enumerate(self.detail_lines)
                               if q in dl.text.lower()]

    def _fix_scroll(self):
        h = self._list_height()
        if self.cursor < self.scroll:
            self.scroll = self.cursor
        elif self.cursor >= self.scroll + h:
            self.scroll = self.cursor - h + 1
        if self.scroll < 0:
            self.scroll = 0

    def _fix_detail_scroll(self):
        h = self._detail_height()
        if self.detail_search:
            h -= 1  # account for match-info line
        if self.detail_line_cursor < self.detail_scroll:
            self.detail_scroll = self.detail_line_cursor
        elif self.detail_line_cursor >= self.detail_scroll + h:
            self.detail_scroll = self.detail_line_cursor - h + 1
        if self.detail_scroll < 0:
            self.detail_scroll = 0

    def _list_height(self):
        h, _ = self.stdscr.getmaxyx()
        return max(1, h - 3)

    def _detail_height(self):
        h, _ = self.stdscr.getmaxyx()
        return max(1, h - 2)  # tab line + help bar

    def run(self):
        while True:
            self.draw()
            key = self.stdscr.getch()
            if key == -1:
                # Idle timeout — poll for filesystem changes
                self._check_fs_changes()
                continue
            if key == curses.KEY_RESIZE:
                continue
            if self.show_help:
                self.show_help = False
                continue
            if not self.handle_key(key):
                break

    # -- Drawing -----------------------------------------------------------

    def draw(self):
        self.stdscr.erase()
        h, w = self.stdscr.getmaxyx()
        if h < 5 or w < 40:
            self.stdscr.addstr(0, 0, "Terminal too small")
            self.stdscr.refresh()
            return

        self._draw_tabs(0, w)

        if self.focus == "detail":
            # Detail pane takes ~2/3, list takes ~1/3
            detail_w = max(w * 2 // 3, 40)
            list_w = w - detail_w - 1
            detail_x = list_w + 1

            # If the cached detail was built for a different width, rebuild
            if self._detail_build_width != detail_w:
                self._rebuild_detail(detail_w)

            self._draw_list(2, 0, h - 3, list_w)
            self._draw_separator(1, list_w, h - 2)
            self._draw_detail(1, detail_x, h - 2, detail_w)
        else:
            # List takes the full width
            self._draw_list(2, 0, h - 3, w)

        self._draw_help_bar(h - 1, w)

        if self.show_help:
            self._draw_help_popup(h, w)

        self.stdscr.refresh()

    def _draw_tabs(self, y, w):
        x = 0
        counts = self._tab_counts()
        for i, (status, label) in enumerate(TABS):
            text = f" {label} ({counts[status]}) "
            attr = curses.color_pair(C_TAB_ACTIVE) | curses.A_BOLD if i == self.tab else curses.A_DIM
            try:
                self.stdscr.addstr(y, x, text, attr)
            except curses.error:
                pass
            x += len(text) + 1

        if self.search_query:
            indicator = f'  search: "{self.search_query}"'
            self._safe_addstr(y, x, indicator, curses.color_pair(C_SEARCH))
        elif self.filter_mode != "all":
            indicator = f"  {self.filter_mode}"
            self._safe_addstr(y, x, indicator, curses.color_pair(C_SEARCH))

        # Top-right notification (transient)
        if self.notification:
            nx = max(x + 2, w - len(self.notification) - 2)
            self._safe_addstr(y, nx, self.notification,
                              curses.color_pair(C_SEARCH) | curses.A_BOLD)

    def _draw_list(self, y_start, x_start, height, width):
        if not self.tasks:
            self._safe_addstr(y_start + 1, x_start + 2, "No tasks.", curses.A_DIM)
            return

        max_id_len = 4
        for _, t, depth, _ in self.tasks:
            id_len = len(t["id"]) + depth * 2
            max_id_len = max(max_id_len, id_len)
        id_col = max_id_len + 1

        for i in range(height):
            idx = self.scroll + i
            if idx >= len(self.tasks):
                break
            status, task, depth, ghost = self.tasks[idx]
            y = y_start + i
            is_selected = idx == self.cursor

            if is_selected:
                self._safe_addstr(y, x_start, " " * width, curses.color_pair(C_SELECTED))

            indent = "  " * depth
            tid = task["id"]
            pri = task.get("priority", "-")
            ttype = task.get("type", "-")
            title = task.get("title", "")

            x = x_start
            ghost_attr = curses.A_DIM if ghost else 0

            base_attr = curses.color_pair(C_SELECTED) if is_selected else 0

            # Blocked marker: replace the leading space with '*' for any
            # hairy task whose deps aren't all shorn.
            blocked = tid in self.blocked_ids and status == yak.HAIRY
            lead = "*" if blocked else " "
            id_text = f"{lead}{indent}{tid}"
            id_text = id_text.ljust(id_col + 1)
            id_attr = base_attr if is_selected else (curses.color_pair(C_ID) | ghost_attr)
            if blocked and not is_selected:
                # Highlight just the leading '*' in the warning color
                block_attr = curses.color_pair(C_P2) | curses.A_BOLD
                self._safe_addstr(y, x, lead, block_attr)
                self._safe_addstr(y, x + 1, id_text[1:], id_attr)
            else:
                self._safe_addstr(y, x, id_text, id_attr)
            x += len(id_text)

            pri_text = f"p{pri} "
            if is_selected:
                pri_attr = base_attr
            elif pri == 1:
                pri_attr = curses.color_pair(C_P1) | curses.A_BOLD | ghost_attr
            elif pri == 3:
                pri_attr = curses.color_pair(C_P3) | ghost_attr
            else:
                pri_attr = curses.color_pair(C_P2) | ghost_attr
            self._safe_addstr(y, x, pri_text, pri_attr)
            x += len(pri_text)

            type_text = f"{ttype:8s} "
            type_attr = base_attr if is_selected else (curses.color_pair(C_TYPE) | ghost_attr)
            self._safe_addstr(y, x, type_text, type_attr)
            x += len(type_text)

            remaining = width - (x - x_start) - 1
            if remaining > 0:
                display_title = title[:remaining]
                title_attr = base_attr | ghost_attr
                if self.search_query:
                    sc = {"hairy": "[H]", "shaving": "[S]", "shorn": "[N]"}.get(status, "")
                    display_title = f"{sc} {display_title}"
                    display_title = display_title[:remaining]
                self._safe_addstr(y, x, display_title, title_attr)

            if ghost and not self.search_query:
                sc = status_char(status)
                badge = f" [{sc}]"
                bx = x_start + width - len(badge) - 1
                if bx > x:
                    badge_attr = _ghost_badge_attr(status) | base_attr
                    self._safe_addstr(y, bx, badge, badge_attr)

    def _draw_separator(self, y_start, x, height):
        for y in range(y_start, y_start + height):
            try:
                self.stdscr.addch(y, x, curses.ACS_VLINE, curses.A_DIM)
            except curses.error:
                pass

    def _draw_detail(self, y_start, x_start, height, width):
        if not self.detail_lines:
            return

        detail_focused = self.focus == "detail"

        # Detail search indicator
        if self.detail_search:
            match_info = f"  /{self.detail_search}  ({len(self.detail_matches)} matches)"
            self._safe_addstr(y_start, x_start, match_info[:width],
                              curses.color_pair(C_SEARCH))
            y_start += 1
            height -= 1

        for i in range(height):
            line_idx = self.detail_scroll + i
            if line_idx >= len(self.detail_lines):
                break
            dl = self.detail_lines[line_idx]
            y = y_start + i
            text = dl.text[:width]

            is_cursor = detail_focused and line_idx == self.detail_line_cursor
            is_match = line_idx in self.detail_matches

            if is_cursor:
                # Fill line with cursor color
                fill_attr = (curses.color_pair(C_LINK_SEL) if dl.is_link
                             else curses.color_pair(C_SELECTED))
                self._safe_addstr(y, x_start, " " * width, fill_attr)
                self._safe_addstr(y, x_start, text, fill_attr | curses.A_BOLD)
            elif dl.is_link:
                self._safe_addstr(y, x_start, text, curses.color_pair(C_LINK))
            elif dl.kind == "header":
                self._safe_addstr(y, x_start, text,
                                  curses.color_pair(C_HEADER) | curses.A_BOLD)
            elif dl.kind == "subheader":
                self._safe_addstr(y, x_start, text,
                                  curses.color_pair(C_HEADER) | curses.A_BOLD)
            elif dl.kind == "desc":
                self._safe_addstr(y, x_start, text, curses.A_DIM)
            elif dl.kind == "code":
                self._safe_addstr(y, x_start, text, curses.color_pair(C_CODE))
            elif dl.kind == "md_heading":
                self._safe_addstr(y, x_start, text,
                                  curses.color_pair(C_MD_HEADING) | curses.A_BOLD)
            elif dl.kind == "quote":
                self._safe_addstr(y, x_start, text, curses.A_DIM | curses.A_ITALIC)
            else:
                self._safe_addstr(y, x_start, text, 0)

            if is_match and not is_cursor:
                self._highlight_matches(y, x_start, text, width)

    def _highlight_matches(self, y, x_start, text, width):
        """Overlay search match highlights on an already-drawn line."""
        if not self.detail_search:
            return
        q = self.detail_search.lower()
        tl = text.lower()
        pos = 0
        while True:
            idx = tl.find(q, pos)
            if idx < 0 or idx >= width:
                break
            end = min(idx + len(q), width)
            self._safe_addstr(y, x_start + idx, text[idx:end],
                              curses.color_pair(C_MATCH) | curses.A_BOLD)
            pos = idx + 1

    def _draw_help_bar(self, y, w):
        if self.message:
            self._safe_addstr(y, 0, self.message[:w], curses.color_pair(C_SEARCH))
            self.message = ""
            return
        if self.focus == "detail":
            keys = "h:list  j/k:move  Tab:next link  Enter:follow  i/o:fwd/back  /:search  Esc:clear  q:quit"
        else:
            keys = "Tab:tab  j/k:move  l:detail  c/C:new  e:edit  D:del  s/x/r:shave/shorn/regrow  n/t/a:filter  /:search  ?:help"
        self._safe_addstr(y, 0, " " * w, curses.color_pair(C_HELP))
        self._safe_addstr(y, 0, keys[:w], curses.color_pair(C_HELP))

    def _draw_help_popup(self, h, w):
        sections = [
            ("List pane", [
                "j / k / Up / Down     Move cursor",
                "d / u                 Half-page down / up",
                "PgDn / PgUp           Full-page down / up",
                "g / G                 First / last task",
                "Tab / Shift-Tab       Switch status tab",
                "[ / ]                 Previous / next tab",
                "l / Right / Enter     Show detail pane",
                "c / C                 New root / child task (picks type)",
                "y                     Copy yak ID to clipboard",
                "m                     Add comment/note",
                "A                     Attach file / clipboard image",
                "e                     Edit task in $EDITOR",
                "D                     Delete task (confirm)",
                "s / x / r             Shave / shorn / regrow",
                "P / T / N / L         Adjust priority / type / title / labels",
                "b / B                 Add / remove dependency (fuzzy search)",
                "n / t / a             Next / tangled / all",
                "/                     Search all tasks",
                "Esc                   Clear search",
            ]),
            ("Detail pane", [
                "h / Left              Hide detail pane",
                "j / k / Up / Down     Move line cursor",
                "d / u                 Half-page down / up",
                "PgDn / PgUp           Full-page down / up",
                "g / G                 First / last line",
                "Tab / ] / Shift-Tab / [   Cycle between links",
                "Enter                 Follow link / open artifact",
                "O                     Open artifact externally",
                "J / K                 Next / prev task in list",
                "i                     Nav forward in jumplist",
                "o / Backspace         Nav back in jumplist",
                "y                     Copy yak ID to clipboard",
                "m                     Add comment/note",
                "A                     Attach file / clipboard image",
                "e                     Edit task in $EDITOR",
                "D                     Delete task (confirm)",
                "/                     Search detail text",
                "n / N                 Next / prev match",
                "Esc                   Clear search / back",
            ]),
            ("General", [
                "R                     Refresh",
                "?                     Toggle this help",
                "q                     Quit",
            ]),
        ]

        title = "Yaks TUI - Keyboard shortcuts"
        footer = "Press any key to close"

        # Decide layout: vertical if it fits, horizontal otherwise
        vertical_h = 4 + sum(len(lines) + 2 for _, lines in sections)
        max_line = max(max(len(l) for l in lines) for _, lines in sections)
        vertical_w = max_line + 4

        if h >= vertical_h + 2:
            self._draw_help_vertical(h, w, title, footer, sections, max_line)
        else:
            self._draw_help_horizontal(h, w, title, footer, sections, max_line)

    def _draw_help_vertical(self, h, w, title, footer, sections, max_line):
        lines = [title, ""]
        for name, entries in sections:
            lines.append(name)
            for e in entries:
                lines.append("  " + e)
            lines.append("")
        lines.append(footer)
        box_w = max(len(l) for l in lines) + 4
        box_h = len(lines) + 2
        self._render_popup(h, w, box_w, box_h, lines, bold_lines={0} | {
            i for i, l in enumerate(lines)
            if l in {name for name, _ in sections}
        })

    def _draw_help_horizontal(self, h, w, title, footer, sections, max_line):
        col_w = max_line + 4
        col_lines_count = max(len(entries) + 1 for _, entries in sections)
        total_w = col_w * len(sections) + 2

        # Build row-wise content
        # Row 0: title (full width)
        # Rows 1..col_lines_count: section columns
        # Last row: footer
        box_h = col_lines_count + 4  # title + blank + sections + blank + footer
        box_w = min(total_w, w - 2)

        start_y = max(0, (h - box_h) // 2)
        start_x = max(0, (w - box_w) // 2)

        # Draw background + border
        for i in range(box_h):
            y = start_y + i
            if y >= h:
                break
            self._safe_addstr(y, start_x, " " * box_w, curses.color_pair(C_TAB_ACTIVE))
            if i == 0 or i == box_h - 1:
                border = "\u2500" * (box_w - 2)
                corner = "\u250c" if i == 0 else "\u2514"
                end = "\u2510" if i == 0 else "\u2518"
                self._safe_addstr(y, start_x, corner + border + end,
                                  curses.color_pair(C_TAB_ACTIVE))
            else:
                self._safe_addstr(y, start_x, "\u2502", curses.color_pair(C_TAB_ACTIVE))
                self._safe_addstr(y, start_x + box_w - 1, "\u2502",
                                  curses.color_pair(C_TAB_ACTIVE))

        # Title
        self._safe_addstr(start_y + 1, start_x + 2, title[:box_w - 4],
                          curses.color_pair(C_TAB_ACTIVE) | curses.A_BOLD)

        # Sections in columns
        for si, (name, entries) in enumerate(sections):
            col_x = start_x + 2 + si * col_w
            if col_x + col_w > start_x + box_w - 1:
                break
            self._safe_addstr(start_y + 3, col_x, name,
                              curses.color_pair(C_TAB_ACTIVE) | curses.A_BOLD)
            for ei, entry in enumerate(entries):
                self._safe_addstr(start_y + 4 + ei, col_x, entry[:col_w - 2],
                                  curses.color_pair(C_TAB_ACTIVE))

        # Footer
        self._safe_addstr(start_y + box_h - 2, start_x + 2, footer[:box_w - 4],
                          curses.color_pair(C_TAB_ACTIVE) | curses.A_DIM)

    def _render_popup(self, h, w, box_w, box_h, lines, bold_lines=None):
        bold_lines = bold_lines or set()
        start_y = max(0, (h - box_h) // 2)
        start_x = max(0, (w - box_w) // 2)

        for i in range(box_h):
            y = start_y + i
            if y >= h:
                break
            self._safe_addstr(y, start_x, " " * box_w, curses.color_pair(C_TAB_ACTIVE))
            if i == 0 or i == box_h - 1:
                border = "\u2500" * (box_w - 2)
                corner = "\u250c" if i == 0 else "\u2514"
                end = "\u2510" if i == 0 else "\u2518"
                self._safe_addstr(y, start_x, corner + border + end,
                                  curses.color_pair(C_TAB_ACTIVE))
            elif i - 1 < len(lines):
                line = lines[i - 1]
                attr = curses.color_pair(C_TAB_ACTIVE)
                if (i - 1) in bold_lines:
                    attr |= curses.A_BOLD
                self._safe_addstr(y, start_x, "\u2502", curses.color_pair(C_TAB_ACTIVE))
                self._safe_addstr(y, start_x + 2, line[:box_w - 4], attr)
                self._safe_addstr(y, start_x + box_w - 1, "\u2502",
                                  curses.color_pair(C_TAB_ACTIVE))

    def _tab_counts(self):
        counts = {}
        for status, _ in TABS:
            d = self.root / status
            counts[status] = len(list(d.glob("*.md"))) if d.exists() else 0
        return counts

    def _safe_addstr(self, y, x, text, attr=0):
        h, w = self.stdscr.getmaxyx()
        if y < 0 or y >= h or x >= w:
            return
        try:
            self.stdscr.addnstr(y, x, text, w - x, attr)
        except curses.error:
            pass

    # -- Input handling ----------------------------------------------------

    def handle_key(self, key):
        # Clear transient notification on any input
        self.notification = ""

        if key == ord("q"):
            return False
        if key == ord("?"):
            self.show_help = True
            return True
        if key == ord("R"):
            self.reload()
            return True

        if self.focus == "detail":
            return self._handle_detail_key(key)
        return self._handle_list_key(key)

    def _handle_list_key(self, key):
        # Navigation
        if key in (ord("j"), curses.KEY_DOWN):
            if self.cursor < len(self.tasks) - 1:
                self.cursor += 1
                self._fix_scroll()
                self._rebuild_detail()
        elif key in (ord("k"), curses.KEY_UP):
            if self.cursor > 0:
                self.cursor -= 1
                self._fix_scroll()
                self._rebuild_detail()
        elif key == ord("g"):
            self.cursor = 0
            self._fix_scroll()
            self._rebuild_detail()
        elif key == ord("G"):
            self.cursor = max(0, len(self.tasks) - 1)
            self._fix_scroll()
            self._rebuild_detail()

        # Focus detail
        elif key in (ord("l"), curses.KEY_RIGHT, ord("\n"), curses.KEY_ENTER):
            if self.detail_lines:
                self._enter_detail()

        # Tab switching
        elif key == ord("\t") or key == ord("]"):
            self._switch_tab(1)
        elif key == curses.KEY_BTAB or key == ord("["):
            self._switch_tab(-1)

        # Page scrolling in list. 'd'/'u' are the documented keys;
        # Ctrl-D/Ctrl-U are undocumented vim aliases.
        elif key in (curses.KEY_NPAGE, ord("d"), 4):
            self._list_page(+1, half=(key in (ord("d"), 4)))
        elif key in (curses.KEY_PPAGE, ord("u"), 21):
            self._list_page(-1, half=(key in (ord("u"), 21)))

        # Filters
        elif key == ord("n"):
            if self.tab == 0:
                self.filter_mode = "next" if self.filter_mode != "next" else "all"
                self._reset_list()
        elif key == ord("t"):
            if self.tab == 0:
                self.filter_mode = "tangled" if self.filter_mode != "tangled" else "all"
                self._reset_list()
        elif key == ord("a"):
            self.filter_mode = "all"
            self.search_query = ""
            self._reset_list()

        # Search
        elif key == ord("/"):
            query = self._input_prompt("Search: ")
            if query:
                self.search_query = query
                self.filter_mode = "all"
                self._reset_list()
        elif key == 27:  # Escape
            if self.search_query:
                self.search_query = ""
                self._reset_list()

        # Status changes
        elif key == ord("s"):
            self._move_current("shave")
        elif key == ord("x"):
            self._move_current("shorn")
        elif key == ord("r"):
            self._move_current("regrow")

        # Create (with type picker)
        elif key == ord("c"):
            yak_type = self._pick_type_for_create()
            if yak_type:
                self._create_task(parent=None, yak_type=yak_type)
        elif key == ord("C"):
            parent = self._current_task_id()
            if parent:
                yak_type = self._pick_type_for_create()
                if yak_type:
                    self._create_task(parent=parent, yak_type=yak_type)

        # Edit
        elif key == ord("e"):
            tid = self._current_task_id()
            if tid:
                self._edit_task(tid)

        # Delete
        elif key == ord("D"):
            tid = self._current_task_id()
            if tid:
                self._delete_task(tid)

        # Quick adjust: priority
        elif key == ord("P"):
            tid = self._current_task_id()
            if tid:
                self._quick_adjust_priority(tid)

        # Quick adjust: type
        elif key == ord("T"):
            tid = self._current_task_id()
            if tid:
                self._quick_adjust_type(tid)

        # Quick adjust: title
        elif key == ord("N"):
            tid = self._current_task_id()
            if tid:
                self._quick_adjust_title(tid)

        # Quick adjust: labels
        elif key == ord("L"):
            tid = self._current_task_id()
            if tid:
                self._quick_adjust_labels(tid)

        # Add / remove dependency
        elif key == ord("b"):
            tid = self._current_task_id()
            if tid:
                self._add_dependency(tid)
        elif key == ord("B"):
            tid = self._current_task_id()
            if tid:
                self._remove_dependency(tid)

        # Copy yak ID to clipboard
        elif key == ord("y"):
            tid = self._current_task_id()
            if tid:
                self._copy_to_clipboard(tid)

        # Add comment/note
        elif key == ord("m"):
            tid = self._current_task_id()
            if tid:
                self._add_comment(tid)

        # Attach artifact (file or clipboard image)
        elif key == ord("A"):
            tid = self._current_task_id()
            if tid:
                self._attach_file(tid)

        return True

    def _handle_detail_key(self, key):
        # Escape: clear search first, else back to list
        if key == 27:
            if self.detail_search:
                self.detail_search = ""
                self.detail_matches = []
            else:
                self.focus = "list"
            return True

        # Back to list
        if key in (ord("h"), curses.KEY_LEFT):
            self.focus = "list"
            return True

        # Line cursor movement
        if key in (ord("j"), curses.KEY_DOWN):
            if self.detail_line_cursor < len(self.detail_lines) - 1:
                self.detail_line_cursor += 1
                self._fix_detail_scroll()
        elif key in (ord("k"), curses.KEY_UP):
            if self.detail_line_cursor > 0:
                self.detail_line_cursor -= 1
                self._fix_detail_scroll()
        elif key == ord("g"):
            self.detail_line_cursor = 0
            self._fix_detail_scroll()
        elif key == ord("G"):
            self.detail_line_cursor = max(0, len(self.detail_lines) - 1)
            self._fix_detail_scroll()

        # Cycle between link lines
        elif key in (ord("\t"), ord("]")):
            self._jump_link(+1)
        elif key in (curses.KEY_BTAB, ord("[")):
            self._jump_link(-1)

        # Nav forward / back in the jumplist. 'i'/'o' mirror vim, Backspace
        # is the browser-style back alias.
        elif key == ord("i"):
            self._nav_forward()
        elif key in (ord("o"), curses.KEY_BACKSPACE, 127, 8):
            self._nav_back()

        # Page scrolling in detail. 'd'/'u' are the documented keys;
        # Ctrl-D/Ctrl-U are undocumented vim aliases.
        elif key in (curses.KEY_NPAGE, ord("d"), 4):
            self._detail_page(+1, half=(key in (ord("d"), 4)))
        elif key in (curses.KEY_PPAGE, ord("u"), 21):
            self._detail_page(-1, half=(key in (ord("u"), 21)))

        # Find next / prev match
        elif key == ord("n"):
            if self.detail_matches:
                self._jump_match(+1)
        elif key == ord("N"):
            if self.detail_matches:
                self._jump_match(-1)

        # Follow link on current line
        elif key in (ord("\n"), curses.KEY_ENTER):
            self._follow_link()

        # Open artifact externally (also available via Enter)
        elif key == ord("O"):
            if 0 <= self.detail_line_cursor < len(self.detail_lines):
                dl = self.detail_lines[self.detail_line_cursor]
                if dl.open_path:
                    self._open_externally(dl.open_path)

        # Edit the task being displayed
        elif key == ord("e"):
            tid = self._current_task_id()
            if tid:
                self._edit_task(tid)

        # Delete the task being displayed
        elif key == ord("D"):
            tid = self._current_task_id()
            if tid:
                self._delete_task(tid)

        # Next / prev task in list
        elif key == ord("J"):
            self._detail_next_task(+1)
        elif key == ord("K"):
            self._detail_next_task(-1)

        # Copy yak ID to clipboard
        elif key == ord("y"):
            tid = self._current_task_id()
            if tid:
                self._copy_to_clipboard(tid)

        # Add comment/note
        elif key == ord("m"):
            tid = self._current_task_id()
            if tid:
                self._add_comment(tid)

        # Attach artifact
        elif key == ord("A"):
            tid = self._current_task_id()
            if tid:
                self._attach_file(tid)

        # Detail search
        elif key == ord("/"):
            query = self._input_prompt("Detail search: ")
            if query:
                self.detail_search = query
                self._apply_detail_search()
                # Jump to first match
                if self.detail_matches:
                    self.detail_line_cursor = self.detail_matches[0]
                    self._fix_detail_scroll()

        return True

    def _list_page(self, direction, half=False):
        if not self.tasks:
            return
        h = self._list_height()
        step = max(1, h // 2 if half else h - 1)
        self.cursor = max(0, min(len(self.tasks) - 1,
                                 self.cursor + direction * step))
        self._fix_scroll()
        self._rebuild_detail()

    def _detail_page(self, direction, half=False):
        if not self.detail_lines:
            return
        h = self._detail_height()
        if self.detail_search:
            h = max(1, h - 1)
        step = max(1, h // 2 if half else h - 1)
        self.detail_line_cursor = max(
            0, min(len(self.detail_lines) - 1,
                   self.detail_line_cursor + direction * step))
        self._fix_detail_scroll()

    def _jump_match(self, direction):
        """Jump to the next/previous search match in the detail pane."""
        if not self.detail_matches:
            return
        cur = self.detail_line_cursor
        if direction > 0:
            for m in self.detail_matches:
                if m > cur:
                    self.detail_line_cursor = m
                    self._fix_detail_scroll()
                    return
            self.detail_line_cursor = self.detail_matches[0]
        else:
            for m in reversed(self.detail_matches):
                if m < cur:
                    self.detail_line_cursor = m
                    self._fix_detail_scroll()
                    return
            self.detail_line_cursor = self.detail_matches[-1]
        self._fix_detail_scroll()

    def _jump_link(self, direction):
        """Cycle the detail cursor to the next/previous navigable link line."""
        link_lines = [i for i, dl in enumerate(self.detail_lines) if dl.is_link]
        if not link_lines:
            return
        cur = self.detail_line_cursor
        if direction > 0:
            for li in link_lines:
                if li > cur:
                    self.detail_line_cursor = li
                    self._fix_detail_scroll()
                    return
            self.detail_line_cursor = link_lines[0]
        else:
            for li in reversed(link_lines):
                if li < cur:
                    self.detail_line_cursor = li
                    self._fix_detail_scroll()
                    return
            self.detail_line_cursor = link_lines[-1]
        self._fix_detail_scroll()

    def _switch_tab(self, direction):
        self.tab = (self.tab + direction) % len(TABS)
        self.filter_mode = "all"
        self.search_query = ""
        self._reset_list()

    def _reset_list(self):
        self.cursor = 0
        self.scroll = 0
        self.detail_search = ""
        self.reload()

    def _detail_next_task(self, direction):
        """Move to the next/prev task in the list while staying in detail view."""
        if not self.tasks:
            return
        new = self.cursor + direction
        if new < 0 or new >= len(self.tasks):
            self.notification = "no more tasks"
            return
        self.cursor = new
        self._fix_scroll()
        self._rebuild_detail()
        self.nav_history = []
        self.nav_pos = -1

    def _enter_detail(self):
        """Focus the detail pane and reset its nav stack.
        The stack is scoped to a single detail context, so every fresh entry
        starts empty.
        """
        self.nav_history = []
        self.nav_pos = -1
        self.focus = "detail"

    def _follow_link(self):
        if not (0 <= self.detail_line_cursor < len(self.detail_lines)):
            return
        dl = self.detail_lines[self.detail_line_cursor]
        if dl.task_id:
            self._nav_push(dl.task_id)
            self._navigate_to(dl.task_id)
        elif dl.open_path:
            self._open_externally(dl.open_path)

    def _open_externally(self, path):
        """Open a file using the system's default handler."""
        import subprocess as _sp
        import platform as _pl
        if not Path(path).exists():
            self.notification = f"missing: {path}"
            return
        try:
            opener = "open" if _pl.system() == "Darwin" else "xdg-open"
            _sp.Popen([opener, str(path)],
                      stdout=_sp.DEVNULL, stderr=_sp.DEVNULL)
            self.notification = f"opened {Path(path).name}"
        except FileNotFoundError:
            self.notification = "no system opener available"

    def _nav_push(self, target_id):
        """Push a new entry onto the nav stack, truncating any forward
        history. Ensures the current task is recorded as the prior entry
        so 'back' returns to where we came from.
        """
        current_id = self._current_task_id()
        # Re-sync history with current task if it's drifted (cold start,
        # user scrolled, etc.)
        if (self.nav_pos < 0 or
                self.nav_pos >= len(self.nav_history) or
                (current_id and self.nav_history[self.nav_pos] != current_id)):
            if current_id:
                self.nav_history = [current_id]
                self.nav_pos = 0
            else:
                self.nav_history = []
                self.nav_pos = -1
        # Truncate forward, append target
        self.nav_history = self.nav_history[:self.nav_pos + 1]
        self.nav_history.append(target_id)
        self.nav_pos = len(self.nav_history) - 1

    def _nav_back(self):
        if self.nav_pos <= 0:
            self.notification = "no previous task"
            return
        self.nav_pos -= 1
        self._navigate_to(self.nav_history[self.nav_pos])

    def _nav_forward(self):
        if self.nav_pos < 0 or self.nav_pos >= len(self.nav_history) - 1:
            self.notification = "no next task"
            return
        self.nav_pos += 1
        self._navigate_to(self.nav_history[self.nav_pos])

    def _navigate_to(self, task_id):
        """Navigate to a task by ID — find it in any tab."""
        result = yak.find_task_file(self.root, task_id)
        if not result:
            self.message = f"Task {task_id} not found"
            return

        target_status, _ = result

        # Switch to the right tab
        for i, (status, _) in enumerate(TABS):
            if status == target_status:
                self.tab = i
                break

        # Reload with no filters to ensure the task is visible
        self.filter_mode = "all"
        self.search_query = ""
        self.detail_search = ""
        self.tasks = build_tree(self.root, target_status, "all", "")

        # Find and select the task
        for i, (_, t, _, _) in enumerate(self.tasks):
            if t["id"] == task_id:
                self.cursor = i
                self._fix_scroll()
                break

        self._rebuild_detail()
        self.focus = "detail"

    def _move_current(self, action):
        if not self.tasks or self.cursor >= len(self.tasks):
            return
        _, task, _, ghost = self.tasks[self.cursor]
        tid = task["id"]
        title = task.get("title", "")[:40]
        verb = {"shave": "Shave", "shorn": "Shorn", "regrow": "Regrow"}[action]
        prompt = f"{verb} {tid} ({title})? (Y/n): "
        if not self._confirm(prompt, default_yes=True):
            self.notification = f"{action} cancelled"
            return

        class FakeArgs:
            pass

        args = FakeArgs()
        args.id = tid

        try:
            if action == "shave":
                yak._move_task(args, yak.SHAVING, "already being shaved", "Shaving")
            elif action == "shorn":
                commit = yak.git_head_short()
                extra = {"commit": commit} if commit else {}
                yak._move_task(args, yak.SHORN, "already shorn", "Shorn!",
                               extra_fields=extra)
            elif action == "regrow":
                yak._move_task(args, yak.HAIRY, "already hairy", "Regrown:")
            self.message = f"{action}: {tid}"
        except SystemExit:
            self.message = f"Error: could not {action} {tid}"

        self.reload()

    def _current_task_id(self):
        if not self.tasks or self.cursor >= len(self.tasks):
            return None
        return self.tasks[self.cursor][1]["id"]

    def _create_task(self, parent=None, yak_type="task"):
        """Spawn $EDITOR on a template and create the task on save."""
        template = self._build_template(parent, yak_type=yak_type)
        edited = self._edit_in_editor(template)
        if edited is None or edited.strip() == template.strip():
            self.notification = "create cancelled"
            return

        data = self._parse_template(edited)
        if not data or not data.get("title", "").strip():
            self.notification = "create cancelled"
            return

        # Create the task
        cfg = yak.load_config(self.root)
        prefix = cfg.get("prefix", "yak")
        if parent:
            if not yak.find_task_file(self.root, parent):
                self.notification = f"parent {parent} not found"
                return
            tid = f"{parent}.{yak.next_child_number(self.root, parent)}"
        else:
            tid = yak.generate_id(self.root, prefix)

        now = yak.now_iso()
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

        path = self.root / yak.HAIRY / f"{tid}.md"
        yak.save_task(path, task)

        # Switch to Hairy tab and select the new task
        self.tab = 0
        self.filter_mode = "all"
        self.search_query = ""
        self.reload()
        for i, (_, t, _, _) in enumerate(self.tasks):
            if t["id"] == tid:
                self.cursor = i
                self._fix_scroll()
                self._rebuild_detail()
                break
        self.notification = f"created {tid}"

    def _edit_task(self, tid):
        """Open the task's file in $EDITOR, reload on save."""
        result = yak.find_task_file(self.root, tid)
        if not result:
            self.notification = f"{tid} not found"
            return

        status, path = result
        original = yak.load_task(path)
        original_id = original.get("id")
        original_created = original.get("created")

        # Read the file content as-is for editing
        content = path.read_text()
        edited = self._edit_file_in_editor(path)
        if edited is None:
            self.notification = "edit cancelled"
            return
        if edited == content:
            self.notification = "no changes"
            return

        # Re-parse and normalize: preserve id + created, bump updated
        data = self._parse_template(edited)
        if not data or not data.get("title", "").strip():
            self.notification = "edit cancelled (invalid)"
            return

        data["id"] = original_id
        if original_created:
            data["created"] = original_created
        data["updated"] = yak.now_iso()

        # Re-serialize cleanly
        yak.save_task(path, data)
        self.reload()
        # Re-select the edited task
        for i, (_, t, _, _) in enumerate(self.tasks):
            if t["id"] == tid:
                self.cursor = i
                self._fix_scroll()
                self._rebuild_detail()
                break
        self.notification = f"edited {tid}"

    def _delete_task(self, tid):
        """Delete a task file with confirmation. Refuses if task has children."""
        result = yak.find_task_file(self.root, tid)
        if not result:
            self.notification = f"{tid} not found"
            return

        _, path = result
        children = yak.find_children(self.root, tid)
        if children:
            self.notification = f"{tid} has {len(children)} child(ren); delete them first"
            return

        task = yak.load_task(path)
        title = task.get("title", "")[:40]
        prompt = f"Delete {tid} ({title})? (y/N): "
        if not self._confirm(prompt):
            self.notification = "delete cancelled"
            return

        try:
            path.unlink()
        except OSError as e:
            self.notification = f"delete failed: {e}"
            return

        self.reload()
        self.notification = f"deleted {tid}"

    def _pick(self, prompt, choices):
        """Show a single-key picker at the bottom. Returns the chosen char
        or None on Esc. `choices` is a string of valid chars (case-sensitive).
        """
        h, w = self.stdscr.getmaxyx()
        y = h - 1
        self._safe_addstr(y, 0, " " * w, 0)
        self._safe_addstr(y, 0, prompt[:w],
                          curses.color_pair(C_SEARCH) | curses.A_BOLD)
        self.stdscr.refresh()
        while True:
            ch = self.stdscr.getch()
            if ch == -1:
                continue
            if ch == 27:
                return None
            c = chr(ch) if 0 <= ch < 256 else ""
            if c in choices:
                return c

    def _quick_adjust_priority(self, tid):
        result = yak.find_task_file(self.root, tid)
        if not result:
            return
        _, path = result
        choice = self._pick(
            f"Priority for {tid}: 1=high 2=med 3=low  (Esc=cancel)", "123")
        if choice is None:
            self.notification = "priority unchanged"
            return
        task = yak.load_task(path)
        new_p = int(choice)
        if task.get("priority") == new_p:
            self.notification = f"{tid} already p{new_p}"
            return
        task["priority"] = new_p
        task["updated"] = yak.now_iso()
        yak.save_task(path, task)
        self.reload()
        self.notification = f"{tid} -> p{new_p}"

    def _add_dependency(self, tid):
        result = yak.find_task_file(self.root, tid)
        if not result:
            return
        _, path = result
        task = yak.load_task(path)
        existing_deps = set(task.get("depends_on") or [])
        target = self._fuzzy_pick_task(
            f"{tid} depends on: ", exclude_ids={tid} | existing_deps)
        if target is None:
            self.notification = "add dep cancelled"
            return
        # Cycle check: refuse if `target` already (transitively) depends on `tid`
        if self._depends_on_transitively(target, tid):
            self.notification = f"refused: would create a cycle ({target} -> {tid})"
            return
        deps = list(existing_deps)
        deps.append(target)
        task["depends_on"] = deps
        task["updated"] = yak.now_iso()
        yak.save_task(path, task)
        self.reload()
        self.notification = f"{tid} -> depends on {target}"

    def _remove_dependency(self, tid):
        result = yak.find_task_file(self.root, tid)
        if not result:
            return
        _, path = result
        task = yak.load_task(path)
        deps = list(task.get("depends_on") or [])
        if not deps:
            self.notification = f"{tid} has no deps"
            return
        # Build a digit-keyed picker (up to 9 deps)
        display = deps[:9]
        picker = "  ".join(f"({i + 1}){d}" for i, d in enumerate(display))
        prompt = f"Remove dep: {picker}  (Esc=cancel)"
        choices = "".join(str(i + 1) for i in range(len(display)))
        choice = self._pick(prompt, choices)
        if choice is None:
            self.notification = "remove dep cancelled"
            return
        idx = int(choice) - 1
        removed = display[idx]
        deps.remove(removed)
        if deps:
            task["depends_on"] = deps
        else:
            task.pop("depends_on", None)
        task["updated"] = yak.now_iso()
        yak.save_task(path, task)
        self.reload()
        self.notification = f"{tid} -/-> {removed}"

    def _depends_on_transitively(self, start_id, target_id):
        """True if start_id depends (directly or transitively) on target_id."""
        seen = set()
        stack = [start_id]
        while stack:
            cur = stack.pop()
            if cur in seen:
                continue
            seen.add(cur)
            result = yak.find_task_file(self.root, cur)
            if not result:
                continue
            t = yak.load_task(result[1])
            for d in t.get("depends_on") or []:
                if d == target_id:
                    return True
                stack.append(d)
        return False

    def _quick_adjust_title(self, tid):
        result = yak.find_task_file(self.root, tid)
        if not result:
            return
        _, path = result
        task = yak.load_task(path)
        current = task.get("title", "")
        new_title = self._edit_prompt(f"Title ({tid}): ", initial=current)
        if new_title is None:
            self.notification = "title unchanged"
            return
        if not new_title:
            self.notification = "title cannot be empty"
            return
        if new_title == current:
            return
        task["title"] = new_title
        task["updated"] = yak.now_iso()
        yak.save_task(path, task)
        self.reload()
        self.notification = f"{tid} title updated"

    def _quick_adjust_labels(self, tid):
        result = yak.find_task_file(self.root, tid)
        if not result:
            return
        _, path = result
        task = yak.load_task(path)
        current = task.get("labels") or []
        initial = ", ".join(current)
        edited = self._edit_prompt(f"Labels for {tid}: ", initial)
        if edited is None:
            self.notification = "labels unchanged"
            return
        if edited.strip():
            new_labels = [l.strip() for l in edited.split(",") if l.strip()]
        else:
            new_labels = []
        if new_labels == current:
            self.notification = "labels unchanged"
            return
        if new_labels:
            task["labels"] = new_labels
        else:
            task.pop("labels", None)
        task["updated"] = yak.now_iso()
        yak.save_task(path, task)
        self.reload()
        self.notification = f"{tid} labels: {', '.join(new_labels) if new_labels else '(none)'}"

    def _quick_adjust_type(self, tid):
        result = yak.find_task_file(self.root, tid)
        if not result:
            return
        _, path = result
        type_map = {"t": "task", "b": "bug", "f": "feature", "i": "idea"}
        choice = self._pick(
            f"Type for {tid}: t=task b=bug f=feature i=idea  (Esc=cancel)",
            "tbfi")
        if choice is None:
            self.notification = "type unchanged"
            return
        task = yak.load_task(path)
        new_t = type_map[choice]
        if task.get("type") == new_t:
            self.notification = f"{tid} already {new_t}"
            return
        task["type"] = new_t
        task["updated"] = yak.now_iso()
        yak.save_task(path, task)
        self.reload()
        self.notification = f"{tid} -> {new_t}"

    def _add_comment(self, tid):
        """Append a timestamped comment to the task's description."""
        text = self._input_prompt("Comment: ")
        if not text:
            self.notification = "comment cancelled"
            return
        result = yak.find_task_file(self.root, tid)
        if not result:
            return
        _, path = result
        task = yak.load_task(path)
        now = yak.now_iso()
        note_block = f"\n### {now}\n\n{text}"
        desc = task.get("description", "") or ""
        task["description"] = desc + note_block
        task["updated"] = now
        yak.save_task(path, task)
        self.reload()
        self._rebuild_detail()
        self.notification = f"comment added to {tid}"

    def _attach_file(self, tid):
        """Attach a file or clipboard image to a task."""
        result = yak.find_task_file(self.root, tid)
        if not result:
            return
        _, path = result

        src_input = self._input_prompt("Attach path (empty = clipboard PNG): ")
        if src_input is None:
            self.notification = "attach cancelled"
            return

        adir = yak.artifacts_dir(self.root, tid)
        adir.mkdir(parents=True, exist_ok=True)

        if src_input.strip() == "":
            data = yak.read_clipboard_png()
            if not data:
                self.notification = "no PNG image on clipboard"
                return
            name = f"paste-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}.png"
            dest = adir / name
            dest.write_bytes(data)
        else:
            import shutil as _sh
            src = Path(src_input.strip()).expanduser()
            if not src.is_file():
                self.notification = f"not a file: {src}"
                return
            name = src.name
            dest = adir / name
            if dest.exists():
                self.notification = f"{name} already attached"
                return
            _sh.copy2(src, dest)

        desc = self._input_prompt(f"Description for {name} (empty = filename): ")
        if desc is None:
            desc = ""
        alt = desc.strip() or Path(name).stem
        link = f"![{alt}](artifacts/{tid}/{name})"

        task = yak.load_task(path)
        body = task.get("description", "") or ""
        if body and not body.endswith("\n"):
            body += "\n"
        body += "\n" + link + "\n"
        task["description"] = body
        task["updated"] = yak.now_iso()
        yak.save_task(path, task)
        self.reload()
        self._rebuild_detail()
        self.notification = f"attached {name}"

    def _pick_type_for_create(self):
        """Pick a yak type before creation. Returns type string or None."""
        type_map = {"t": "task", "b": "bug", "f": "feature", "i": "idea"}
        choice = self._pick(
            "New yak type: t=task b=bug f=feature i=idea  (Esc=cancel)",
            "tbfi")
        if choice is None:
            self.notification = "create cancelled"
            return None
        return type_map[choice]

    def _copy_to_clipboard(self, text):
        """Copy text to system clipboard."""
        import subprocess as _sp
        import platform as _pl
        try:
            if _pl.system() == "Darwin":
                proc = _sp.Popen(["pbcopy"], stdin=_sp.PIPE)
            else:
                proc = _sp.Popen(["xclip", "-selection", "clipboard"], stdin=_sp.PIPE)
            proc.communicate(text.encode())
            self.notification = f"copied {text}"
        except FileNotFoundError:
            self.notification = "clipboard not available"

    def _confirm(self, prompt, default_yes=False):
        """Show a yes/no prompt at the bottom.
        y/Y = yes, n/N/Esc = no. Enter follows `default_yes`.
        Unrecognized keys keep waiting.
        """
        h, w = self.stdscr.getmaxyx()
        y = h - 1
        self._safe_addstr(y, 0, " " * w, 0)
        self._safe_addstr(y, 0, prompt[:w],
                          curses.color_pair(C_SEARCH) | curses.A_BOLD)
        self.stdscr.refresh()
        while True:
            ch = self.stdscr.getch()
            if ch == -1:
                continue  # idle timeout, keep waiting
            if ch in (ord("y"), ord("Y")):
                return True
            if ch in (ord("n"), ord("N"), 27):
                return False
            if ch in (ord("\n"), curses.KEY_ENTER, 10, 13):
                return default_yes

    def _edit_file_in_editor(self, path):
        """Suspend curses, run $EDITOR directly on an existing file."""
        editor = os.environ.get("EDITOR", "vi")
        curses.def_prog_mode()
        curses.endwin()
        try:
            result = subprocess.call([editor, str(path)])
        except FileNotFoundError:
            curses.reset_prog_mode()
            self.stdscr.refresh()
            self.notification = f"editor '{editor}' not found"
            return None
        curses.reset_prog_mode()
        self.stdscr.refresh()
        if result != 0:
            return None
        return path.read_text()

    def _build_template(self, parent, yak_type="task"):
        lines = ["---"]
        if parent:
            presult = yak.find_task_file(self.root, parent)
            ptitle = ""
            if presult:
                pt = yak.load_task(presult[1])
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

    def _parse_template(self, text):
        """Parse the edited template into a task dict. Returns None on failure."""
        if not text.startswith("---"):
            return None
        # Skip opening ---
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

    def _edit_in_editor(self, initial_content):
        """Suspend curses, run $EDITOR on a temp file, return edited content."""
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
                self.stdscr.refresh()
                self.notification = f"editor '{editor}' not found"
                return None
            curses.reset_prog_mode()
            self.stdscr.refresh()

            if result != 0:
                return None
            with open(tmp.name, "r", encoding="utf-8") as f:
                return f.read()
        finally:
            try:
                os.unlink(tmp.name)
            except OSError:
                pass

    def _fuzzy_pick_task(self, prompt, exclude_ids=None):
        """Interactive fuzzy search over all tasks. Returns task ID or None.
        Shows a floating results list that updates as you type.
        """
        exclude = set(exclude_ids or [])
        # Load all tasks once
        all_tasks = []
        for s in yak.STATUSES:
            for st, t in yak.all_tasks(self.root, s):
                if t["id"] not in exclude:
                    all_tasks.append((st, t))

        def _match(query, tasks):
            if not query:
                return tasks[:20]
            q = query.lower()
            scored = []
            for st, t in tasks:
                tid = t["id"].lower()
                title = t.get("title", "").lower()
                if q in tid or q in title:
                    # Prefer ID prefix matches, then title matches
                    score = 0 if tid.startswith(q) else (1 if q in tid else 2)
                    scored.append((score, st, t))
            scored.sort(key=lambda x: (x[0], x[2].get("priority", 9), x[2]["id"]))
            return [(s, t) for _, s, t in scored[:20]]

        h, w = self.stdscr.getmaxyx()
        buf = ""
        pos = 0
        sel = 0
        max_visible = min(10, h - 4)
        curses.curs_set(1)
        try:
            while True:
                matches = _match(buf, all_tasks)
                sel = max(0, min(sel, len(matches) - 1))

                # Draw matches above the prompt
                list_y = h - 2 - max_visible
                for i in range(max_visible):
                    y = list_y + i
                    if y < 1:
                        continue
                    self._safe_addstr(y, 0, " " * w, 0)
                    if i < len(matches):
                        ms, mt = matches[i]
                        badge = status_char(ms)
                        line = f"  [{badge}] {mt['id']}  {mt.get('title', '')}"
                        attr = curses.color_pair(C_SELECTED) | curses.A_BOLD if i == sel else 0
                        self._safe_addstr(y, 0, line[:w], attr)

                # Draw prompt
                prompt_y = h - 1
                max_vis = max(1, w - len(prompt) - 1)
                offset = max(0, pos - max_vis + 1)
                visible = buf[offset:offset + max_vis]
                count_str = f" ({len(matches)} matches)" if buf else ""
                self._safe_addstr(prompt_y, 0, " " * w, 0)
                self._safe_addstr(prompt_y, 0, prompt,
                                  curses.color_pair(C_SEARCH) | curses.A_BOLD)
                self._safe_addstr(prompt_y, len(prompt), visible, 0)
                cs = len(prompt) + len(visible)
                if cs + len(count_str) < w:
                    self._safe_addstr(prompt_y, cs, count_str, curses.A_DIM)
                try:
                    self.stdscr.move(prompt_y, len(prompt) + (pos - offset))
                except curses.error:
                    pass
                self.stdscr.refresh()

                ch = self.stdscr.getch()
                if ch == -1:
                    continue
                if ch == 27:
                    return None
                if ch in (ord("\n"), curses.KEY_ENTER, 10, 13):
                    if matches and 0 <= sel < len(matches):
                        return matches[sel][1]["id"]
                    return None
                if ch in (curses.KEY_UP, 16):  # Up, Ctrl-P
                    sel = max(0, sel - 1)
                elif ch in (curses.KEY_DOWN, 14):  # Down, Ctrl-N
                    sel = min(len(matches) - 1, sel + 1) if matches else 0
                elif ch == 9:  # Tab — also move down
                    sel = min(len(matches) - 1, sel + 1) if matches else 0
                elif ch in (curses.KEY_BACKSPACE, 127, 8):
                    if pos > 0:
                        buf = buf[:pos - 1] + buf[pos:]
                        pos -= 1
                        sel = 0
                elif ch == 21:  # Ctrl-U
                    buf = ""
                    pos = 0
                    sel = 0
                elif ch == 23:  # Ctrl-W
                    i = pos
                    while i > 0 and buf[i - 1] == " ":
                        i -= 1
                    while i > 0 and buf[i - 1] != " ":
                        i -= 1
                    buf = buf[:i] + buf[pos:]
                    pos = i
                    sel = 0
                elif 32 <= ch < 127:
                    buf = buf[:pos] + chr(ch) + buf[pos:]
                    pos += 1
                    sel = 0
        finally:
            curses.curs_set(0)

    def _edit_prompt(self, prompt, initial=""):
        """Read a line from the bottom bar, pre-populated with `initial`.
        Returns the edited string on Enter, or None on Escape.
        Supports Home/End/arrows/Ctrl-A/Ctrl-E/Ctrl-K/Ctrl-U.
        """
        h, w = self.stdscr.getmaxyx()
        y = h - 1
        buf = initial
        pos = len(buf)
        curses.curs_set(1)
        try:
            while True:
                max_vis = max(1, w - len(prompt) - 1)
                # Simple scroll: if pos exceeds visible window, shift
                offset = max(0, pos - max_vis + 1)
                visible = buf[offset:offset + max_vis]
                self._safe_addstr(y, 0, " " * w, 0)
                self._safe_addstr(y, 0, prompt,
                                  curses.color_pair(C_SEARCH) | curses.A_BOLD)
                self._safe_addstr(y, len(prompt), visible, 0)
                try:
                    self.stdscr.move(y, len(prompt) + (pos - offset))
                except curses.error:
                    pass
                self.stdscr.refresh()

                ch = self.stdscr.getch()
                if ch == -1:
                    continue
                if ch == 27:
                    return None
                if ch in (ord("\n"), curses.KEY_ENTER, 10, 13):
                    return buf.strip()
                if ch in (curses.KEY_BACKSPACE, 127, 8):
                    if pos > 0:
                        buf = buf[:pos - 1] + buf[pos:]
                        pos -= 1
                elif ch == curses.KEY_DC:  # Delete
                    if pos < len(buf):
                        buf = buf[:pos] + buf[pos + 1:]
                elif ch == curses.KEY_LEFT:
                    pos = max(0, pos - 1)
                elif ch == curses.KEY_RIGHT:
                    pos = min(len(buf), pos + 1)
                elif ch in (curses.KEY_HOME, 1):  # Home, Ctrl-A
                    pos = 0
                elif ch in (curses.KEY_END, 5):   # End, Ctrl-E
                    pos = len(buf)
                elif ch == 11:  # Ctrl-K: kill to end of line
                    buf = buf[:pos]
                elif ch == 21:  # Ctrl-U: kill to start of line
                    buf = buf[pos:]
                    pos = 0
                elif ch == 23:  # Ctrl-W: delete previous word
                    # Strip trailing spaces, then non-spaces
                    i = pos
                    while i > 0 and buf[i - 1] == " ":
                        i -= 1
                    while i > 0 and buf[i - 1] != " ":
                        i -= 1
                    buf = buf[:i] + buf[pos:]
                    pos = i
                elif 32 <= ch < 127:
                    buf = buf[:pos] + chr(ch) + buf[pos:]
                    pos += 1
        finally:
            curses.curs_set(0)

    def _input_prompt(self, prompt):
        """Read a line from the bottom bar. Escape cancels (returns "")."""
        h, w = self.stdscr.getmaxyx()
        y = h - 1
        buf = ""
        curses.curs_set(1)
        try:
            while True:
                self._safe_addstr(y, 0, " " * w, 0)
                self._safe_addstr(y, 0, prompt, curses.color_pair(C_SEARCH) | curses.A_BOLD)
                self._safe_addstr(y, len(prompt), buf, 0)
                try:
                    self.stdscr.move(y, min(len(prompt) + len(buf), w - 1))
                except curses.error:
                    pass
                self.stdscr.refresh()

                ch = self.stdscr.getch()
                if ch == -1:
                    continue  # idle timeout, just redraw
                if ch == 27:  # Escape
                    return ""
                elif ch in (ord("\n"), curses.KEY_ENTER, 10, 13):
                    return buf.strip()
                elif ch in (curses.KEY_BACKSPACE, 127, 8):
                    buf = buf[:-1]
                elif 32 <= ch < 127:
                    if len(prompt) + len(buf) < w - 1:
                        buf += chr(ch)
        finally:
            curses.curs_set(0)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main(stdscr):
    root = yak.find_tasks_root()
    tui = TUI(stdscr, root)
    tui.run()


if __name__ == "__main__":
    try:
        curses.wrapper(main)
    except KeyboardInterrupt:
        pass
