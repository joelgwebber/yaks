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
from pathlib import Path

import yaml

sys.path.insert(0, str(Path(__file__).parent))
import yak


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

    # Ghost descendants: include children (recursively) of primaries so mixed
    # parent/child states stay visible when filtering by one status.
    child_prefixes = {tid + "." for tid in primary_ids}
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

    def sort_children(node: TaskNode):
        node.children.sort(key=lambda n: _child_sort_key(n.task["id"]))
        for c in node.children:
            sort_children(c)

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
    __slots__ = ("text", "kind", "task_id")

    def __init__(self, text, kind="", task_id=None):
        self.text = text
        self.kind = kind       # header, subheader, field, child, desc, link, ""
        self.task_id = task_id  # non-None means this line is navigable


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


def build_detail_lines(root, task, status, width=80) -> list[DetailLine]:
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
        ("Created", task.get("created", "-")),
        ("Updated", task.get("updated", "-")),
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
            sc = {yak.HAIRY: "H", yak.SHAVING: "S", yak.SHORN: "N"}.get(ds, "?")
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
            sc = {yak.HAIRY: "H", yak.SHAVING: "S", yak.SHORN: "N"}.get(ps, "?")
            emit(f"  {'Parent:':<12s} [{sc}] {pid}  {pt.get('title', '')}",
                 "link", task_id=pid)

    # Children as links
    children = yak.find_children(root, task["id"])
    if children:
        lines.append(DetailLine(""))
        lines.append(DetailLine("  Children:", "subheader"))
        sc = {yak.HAIRY: "H", yak.SHAVING: "S", yak.SHORN: "N"}
        for cs, ct in children:
            ch = sc.get(cs, "?")
            emit(f"    [{ch}] {ct['id']}  {ct.get('title', '')}",
                 "link", task_id=ct["id"])

    # Description
    desc = task.get("description", "")
    if desc:
        lines.append(DetailLine(""))
        lines.append(DetailLine("  Description:", "subheader"))
        for dline in desc.split("\n"):
            if not dline.strip():
                lines.append(DetailLine("    "))
                continue
            for chunk in _wrap(f"    {dline}", width):
                lines.append(DetailLine(chunk, "desc"))

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

        curses.curs_set(0)
        self.stdscr.timeout(500)  # poll filesystem every 500ms when idle
        init_colors()
        self.reload()
        self._fs_sig = self._scan_fs()

    def reload(self):
        status = TABS[self.tab][0]
        self.tasks = build_tree(self.root, status, self.filter_mode,
                                self.search_query)
        if self.cursor >= len(self.tasks):
            self.cursor = max(0, len(self.tasks) - 1)
        self._fix_scroll()
        self._rebuild_detail()
        self._fs_sig = self._scan_fs()

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
        self.detail_lines = build_detail_lines(self.root, task, status, width)
        # Start cursor on the first link, or line 0 if no links
        self.detail_line_cursor = 0
        for i, dl in enumerate(self.detail_lines):
            if dl.task_id:
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

            id_text = f" {indent}{tid}"
            id_text = id_text.ljust(id_col + 1)
            id_attr = base_attr if is_selected else (curses.color_pair(C_ID) | ghost_attr)
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
                sc = {yak.HAIRY: "H", yak.SHAVING: "S", yak.SHORN: "N"}.get(status, "?")
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
                fill_attr = (curses.color_pair(C_LINK_SEL) if dl.task_id
                             else curses.color_pair(C_SELECTED))
                self._safe_addstr(y, x_start, " " * width, fill_attr)
                self._safe_addstr(y, x_start, text, fill_attr | curses.A_BOLD)
            elif dl.task_id:
                self._safe_addstr(y, x_start, text, curses.color_pair(C_LINK))
            elif dl.kind == "header":
                self._safe_addstr(y, x_start, text,
                                  curses.color_pair(C_HEADER) | curses.A_BOLD)
            elif dl.kind == "subheader":
                self._safe_addstr(y, x_start, text,
                                  curses.color_pair(C_HEADER) | curses.A_BOLD)
            elif dl.kind == "desc":
                self._safe_addstr(y, x_start, text, curses.A_DIM)
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
            keys = "h:list  j/k:move  Tab:next link  Enter:follow  Bksp:back  /:search  Esc:clear  q:quit"
        else:
            keys = "Tab:tab  j/k:move  l:detail  c/C:new  e:edit  D:del  s/x/r:shave/shorn/regrow  n/t/a:filter  /:search  ?:help"
        self._safe_addstr(y, 0, " " * w, curses.color_pair(C_HELP))
        self._safe_addstr(y, 0, keys[:w], curses.color_pair(C_HELP))

    def _draw_help_popup(self, h, w):
        sections = [
            ("List pane", [
                "j / k / Up / Down     Move cursor",
                "g / G                 First / last task",
                "Tab / Shift-Tab       Switch status tab",
                "l / Right / Enter     Show detail pane",
                "c / C                 New root / child task",
                "e                     Edit task in $EDITOR",
                "D                     Delete task (confirm)",
                "s / x / r             Shave / shorn / regrow",
                "n / t / a             Next / tangled / all",
                "/                     Search all tasks",
                "Esc                   Clear search",
            ]),
            ("Detail pane", [
                "h / Left              Hide detail pane",
                "j / k / Up / Down     Move line cursor",
                "g / G                 First / last line",
                "Tab / Shift-Tab       Jump between links",
                "Enter                 Follow link",
                "Backspace / Ctrl-O    Navigate back",
                "e                     Edit task in $EDITOR",
                "D                     Delete task (confirm)",
                "/                     Search detail text",
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
                self.focus = "detail"

        # Tab switching
        elif key == ord("\t"):
            self._switch_tab(1)
        elif key == curses.KEY_BTAB:
            self._switch_tab(-1)

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

        # Create
        elif key == ord("c"):
            self._create_task(parent=None)
        elif key == ord("C"):
            parent = self._current_task_id()
            if parent:
                self._create_task(parent=parent)

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

        # Tab/Shift-Tab jump between links
        elif key == ord("\t"):
            self._jump_link(1)
        elif key == curses.KEY_BTAB:
            self._jump_link(-1)

        # Follow link on current line
        elif key in (ord("\n"), curses.KEY_ENTER):
            self._follow_link()

        # Back in history
        elif key in (curses.KEY_BACKSPACE, 127, 8, 15):  # BS, DEL, ^H, ^O
            self._nav_back()

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

    def _jump_link(self, direction):
        """Jump to the next/previous navigable link line."""
        link_lines = [i for i, dl in enumerate(self.detail_lines) if dl.task_id]
        if not link_lines:
            return
        if direction > 0:
            for li in link_lines:
                if li > self.detail_line_cursor:
                    self.detail_line_cursor = li
                    self._fix_detail_scroll()
                    return
            self.detail_line_cursor = link_lines[0]
        else:
            for li in reversed(link_lines):
                if li < self.detail_line_cursor:
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

    def _follow_link(self):
        if not (0 <= self.detail_line_cursor < len(self.detail_lines)):
            return
        target_id = self.detail_lines[self.detail_line_cursor].task_id
        if not target_id:
            return

        # Save current task in history
        if self.tasks and self.cursor < len(self.tasks):
            current_id = self.tasks[self.cursor][1]["id"]
            # Truncate forward history if we're navigating from the middle
            self.nav_history = self.nav_history[:self.nav_pos + 1]
            self.nav_history.append(current_id)
            self.nav_pos = len(self.nav_history) - 1

        self._navigate_to(target_id)

    def _nav_back(self):
        if self.nav_pos < 0 or not self.nav_history:
            self.message = "No history"
            return

        target_id = self.nav_history[self.nav_pos]
        self.nav_pos -= 1
        self._navigate_to(target_id)

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
        if ghost:
            self.message = "Cannot modify ghost tasks"
            return

        tid = task["id"]

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

    def _create_task(self, parent=None):
        """Spawn $EDITOR on a template and create the task on save."""
        template = self._build_template(parent)
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

    def _confirm(self, prompt):
        """Show a yes/no prompt at the bottom. Returns True only on 'y' or 'Y'."""
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
            return ch in (ord("y"), ord("Y"))

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

    def _build_template(self, parent):
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
        lines.append("# type: task | bug | feature")
        lines.append("type: task")
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
