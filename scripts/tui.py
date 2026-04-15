# /// script
# requires-python = ">=3.10"
# dependencies = ["pyyaml>=6.0"]
# ///
"""Yaks TUI — curses-based terminal interface for the Yaks task tracker."""

import curses
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from yaklib import artifacts as _artifacts
from yaklib import clipboard as _clipboard
from yaklib import deps as _deps
from yaklib.format import humanize_date, status_char
from yaklib.model import (
    HAIRY,
    SHAVING,
    SHORN,
    STATUSES,
    find_task_file,
    find_tasks_root,
    git_head_short,
    move_task,
)
from yaktui.colors import (
    C_CODE,
    C_GHOST,
    C_GHOST_HAIRY,
    C_GHOST_SHAVING,
    C_GHOST_SHORN,
    C_HEADER,
    C_HELP,
    C_ID,
    C_LABEL,
    C_LINK,
    C_LINK_SEL,
    C_MATCH,
    C_MD_HEADING,
    C_P1,
    C_P2,
    C_P3,
    C_SEARCH,
    C_SELECTED,
    C_TAB_ACTIVE,
    C_TYPE,
    ghost_badge_attr as _ghost_badge_attr,
    init_colors,
)
from yaktui import dialogs as _dialogs
from yaktui import mutate as _mutate
from yaktui import keys_detail as _keys_detail
from yaktui import keys_list as _keys_list
from yaktui import render as _render
from yaktui.detail import DetailLine, build_detail_lines
from yaktui.render import TABS
from yaktui.tree import build_tree


# ---------------------------------------------------------------------------
# TUI
# ---------------------------------------------------------------------------

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
        self.detail_span_cursor = 0  # index into current line's links[]
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
        """Update blocked_ids and reverse_deps from all tasks on disk."""
        self.blocked_ids, self.reverse_deps = _deps.compute_blocked(self.root)

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
        for s in STATUSES:
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
        _render.draw(self)


    def _safe_addstr(self, y, x, text, attr=0):
        _dialogs.safe_addstr(self.stdscr, y, x, text, attr)

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
            return _keys_detail.handle(self, key)
        return _keys_list.handle(self, key)

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

    def _link_targets(self):
        """Flat list of (line_idx, span_idx_or_None) for every navigable
        target in the detail pane. Whole-line task/artifact links contribute
        (line, None); description lines contribute one entry per inline span.
        """
        out = []
        for i, dl in enumerate(self.detail_lines):
            if dl.links:
                for si in range(len(dl.links)):
                    out.append((i, si))
            elif dl.task_id is not None or dl.open_path is not None:
                out.append((i, None))
        return out

    def _current_target_index(self, targets):
        """Return the best index into *targets* for the current cursor.
        If the cursor sits on a target exactly, that one; else the first
        target at or after the current line.
        """
        if not targets:
            return -1
        line = self.detail_line_cursor
        span = self.detail_span_cursor
        dl = self.detail_lines[line] if 0 <= line < len(self.detail_lines) else None
        if dl and dl.links:
            for i, (li, si) in enumerate(targets):
                if li == line and si == span:
                    return i
        elif dl and (dl.task_id is not None or dl.open_path is not None):
            for i, (li, si) in enumerate(targets):
                if li == line and si is None:
                    return i
        for i, (li, _) in enumerate(targets):
            if li >= line:
                return i
        return len(targets) - 1

    def _jump_link(self, direction):
        """Cycle the detail cursor to the next/previous navigable target."""
        targets = self._link_targets()
        if not targets:
            return
        cur = self._current_target_index(targets)
        dl = self.detail_lines[self.detail_line_cursor]
        on_target = bool(dl.links) or (dl.task_id is not None or dl.open_path is not None)
        if on_target:
            nxt = (cur + direction) % len(targets)
        else:
            # Cursor is off-target — the fallback already picked "next";
            # direction > 0 commits it, direction < 0 steps one back.
            nxt = cur if direction > 0 else (cur - 1) % len(targets)
        li, si = targets[nxt]
        self.detail_line_cursor = li
        self.detail_span_cursor = si if si is not None else 0
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
        if dl.links:
            idx = max(0, min(self.detail_span_cursor, len(dl.links) - 1))
            _, _, tid = dl.links[idx]
            self._nav_push(tid)
            self._navigate_to(tid)
        elif dl.task_id:
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
        result = find_task_file(self.root, task_id)
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

        if action == "shave":
            dest = SHAVING
            extra = None
        elif action == "shorn":
            dest = SHORN
            commit = git_head_short()
            extra = {"commit": commit} if commit else None
        else:  # regrow
            dest = HAIRY
            extra = None

        ok, msg = move_task(self.root, tid, dest, extra_fields=extra)
        self.message = f"{action}: {tid}" if ok else msg
        self.reload()

    def _current_task_id(self):
        if not self.tasks or self.cursor >= len(self.tasks):
            return None
        return self.tasks[self.cursor][1]["id"]

    def _create_task(self, parent=None, yak_type="task"):
        _mutate.create_task(self, parent=parent, yak_type=yak_type)

    def _edit_task(self, tid):
        _mutate.edit_task(self, tid)

    def _delete_task(self, tid):
        _mutate.delete_task(self, tid)

    def _pick(self, prompt, choices):
        return _dialogs.pick(self.stdscr, prompt, choices)

    def _quick_adjust_priority(self, tid):
        _mutate.quick_adjust_priority(self, tid)

    def _quick_adjust_type(self, tid):
        _mutate.quick_adjust_type(self, tid)

    def _quick_adjust_title(self, tid):
        _mutate.quick_adjust_title(self, tid)

    def _quick_adjust_labels(self, tid):
        _mutate.quick_adjust_labels(self, tid)

    def _add_dependency(self, tid):
        _mutate.add_dependency(self, tid)

    def _remove_dependency(self, tid):
        _mutate.remove_dependency(self, tid)

    def _depends_on_transitively(self, start_id, target_id):
        return _deps.depends_on_transitively(self.root, start_id, target_id)

    def _add_comment(self, tid):
        _mutate.add_comment(self, tid)

    def _attach_file(self, tid):
        _mutate.attach_file(self, tid)

    def _pick_type_for_create(self):
        choice = _dialogs.pick_type_for_create(self.stdscr)
        if choice is None:
            self.notification = "create cancelled"
        return choice

    def _copy_to_clipboard(self, text):
        _mutate.copy_to_clipboard(self, text)

    def _confirm(self, prompt, default_yes=False):
        return _dialogs.confirm(self.stdscr, prompt, default_yes)

    def _fuzzy_pick_task(self, prompt, exclude_ids=None):
        return _dialogs.fuzzy_pick_task(self.stdscr, self.root, prompt,
                                        exclude_ids=exclude_ids)

    def _edit_prompt(self, prompt, initial=""):
        return _dialogs.edit_prompt(self.stdscr, prompt, initial)

    def _input_prompt(self, prompt):
        return _dialogs.input_prompt(self.stdscr, prompt)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main(stdscr):
    root = find_tasks_root()
    tui = TUI(stdscr, root)
    tui.run()


if __name__ == "__main__":
    try:
        curses.wrapper(main)
    except KeyboardInterrupt:
        pass
