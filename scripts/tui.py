# /// script
# requires-python = ">=3.10"
# dependencies = ["pyyaml>=6.0", "prompt-toolkit>=3.0"]
# ///
"""Yaks TUI — curses-based terminal interface for the Yaks task tracker."""

import curses
import hashlib
import json
import os
import sys
import time
from pathlib import Path

_scripts = str(Path(__file__).parent)
if _scripts not in sys.path:
    sys.path.insert(0, _scripts)
from yaklib import artifacts as _artifacts
from yaklib import clipboard as _clipboard
from yaklib import deps as _deps
from yaklib.filter import FilterSpec, collect_all_labels
from yaklib.format import humanize_date, status_char
from yaklib.model import (
    STATUSES,
    find_task_file,
    find_tasks_root,
    load_config,
)
from yaktui import dialogs as _dialogs
from yaktui import keys_detail as _keys_detail
from yaktui import keys_list as _keys_list
from yaktui import mutate as _mutate
from yaktui import render as _render
from yaktui import vim_edit as _vim_edit
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
    init_colors,
)
from yaktui.colors import (
    ghost_badge_attr as _ghost_badge_attr,
)
from yaktui.detail import DetailLine, build_detail_lines
from yaktui.render import TABS
from yaktui.render import tab_counts as _tab_counts
from yaktui.tree import apply_collapse, build_tree
from yaktui.vim_edit import LineEditor


def _ui_state_path_for(root: Path) -> Path:
    """Where per-project UI state (collapsed tree ids) lives on disk."""
    slug = hashlib.sha1(str(root.resolve()).encode("utf-8")).hexdigest()[:12]
    cache_home = os.environ.get("XDG_CACHE_HOME") or str(Path.home() / ".cache")
    return Path(cache_home) / "yaks" / f"{slug}.json"


# ---------------------------------------------------------------------------
# Filter drawer state
# ---------------------------------------------------------------------------

_DRAWER_ROWS = [
    ("status_chips", "status"),
    ("type_chips", "type"),
    ("pri_chips", "priority"),
    ("labels_text", "labels"),
    ("search_text", "search"),
    ("parent_text", "parent"),
    ("deps_chips", "deps"),
]

_DRAWER_STATUS_CHOICES = ["hairy", "shaving", "shorn", "dead"]
_DRAWER_TYPE_CHOICES = ["task", "bug", "feature", "idea"]
_DRAWER_PRI_CHOICES = [1, 2, 3, 4, 5]
_DRAWER_DEPS_CHOICES = ["ready only", "tangled only"]


def _drawer_chip_choices(kind):
    if kind == "status_chips":
        return _DRAWER_STATUS_CHOICES
    if kind == "type_chips":
        return _DRAWER_TYPE_CHOICES
    if kind == "pri_chips":
        return [f"p{p}" for p in _DRAWER_PRI_CHOICES]
    if kind == "deps_chips":
        return _DRAWER_DEPS_CHOICES
    return []


class _DrawerState:
    __slots__ = (
        "saved",
        "statuses",
        "types",
        "priorities",
        "labels",
        "search",
        "parent",
        "ready",
        "tangled",
        "row",
        "chip_idx",
        "vim",
    )

    def __init__(self, spec: FilterSpec, vim: bool = False):
        self.saved = spec  # for revert-on-Esc
        self.statuses = set(spec.statuses)
        self.types = set(spec.types)
        self.priorities = set(spec.priorities)
        self.labels = LineEditor(", ".join(spec.labels), vim=vim)
        self.search = LineEditor(spec.search, vim=vim)
        self.parent = LineEditor(spec.parent, vim=vim)
        self.ready = spec.ready_only
        self.tangled = spec.tangled_only
        self.row = 0
        self.chip_idx = 0
        self.vim = vim

    def build_spec(self) -> FilterSpec:
        lbls = tuple(s.strip() for s in self.labels.buf.split(",") if s.strip())
        return FilterSpec(
            statuses=frozenset(self.statuses),
            types=frozenset(self.types),
            priorities=frozenset(self.priorities),
            labels=lbls,
            search=self.search.buf.strip(),
            ready_only=self.ready,
            tangled_only=self.tangled,
            parent=self.parent.buf.strip(),
        )

    def editor_for(self, kind: str) -> LineEditor | None:
        return {"labels_text": self.labels, "search_text": self.search, "parent_text": self.parent}.get(kind)

    def close(self) -> None:
        self.labels.close()
        self.search.close()
        self.parent.close()

    @property
    def height(self) -> int:
        return len(_DRAWER_ROWS) + 1  # rows + separator line


# ---------------------------------------------------------------------------
# TUI
# ---------------------------------------------------------------------------


class TUI:
    def __init__(self, stdscr, root):
        self.stdscr = stdscr
        self.root = root
        self.config = load_config(root)
        self.vim_mode = bool(self.config.get("vim_mode", False))
        self.show_labels = bool(self.config.get("show_labels", True))

        # List state
        self.tab = 0
        self.cursor = 0
        self.scroll = 0
        self.filter_spec = FilterSpec()
        self.tasks = []
        # Tree collapse: ids whose descendants are hidden from the list view.
        # Loaded from ~/.cache/yaks/<slug>.json; persisted on every toggle.
        self.collapsed_ids: set[str] = set()
        self.collapsed_counts: dict[str, int] = {}
        self._ui_state_path = _ui_state_path_for(root)
        self._load_ui_state()

        # Detail state
        self.focus = "list"  # "list" or "detail"
        self.detail_lines = []
        self.detail_line_cursor = 0  # index into detail_lines
        self.detail_span_cursor = 0  # index into current line's links[]
        self.detail_scroll = 0
        self.detail_search = ""
        self.detail_matches = []  # line indices matching search
        self._detail_build_width = 0  # width lines were wrapped for
        self.detail_select_anchor = None  # line idx when in visual select mode

        # SGR mouse state (for terminals that need SGR rather than X10)
        self._sgr_enabled: bool = False  # set in curses-init block below
        self._sgr_pending: str | None = None  # accumulating an SGR sequence
        self._peek_buf: list[int] = []  # re-queue from SGR prefix peek-ahead

        # Navigation history: list of task IDs
        self._last_click: tuple[float, int, int] = (0.0, -1, -1)  # time, my, mx
        self.nav_history = []
        self.nav_pos = -1

        # UI state
        self.show_help = False
        self.message = ""
        self.notification = ""  # top-right transient notice; clears on next key

        # Filter drawer state (None = closed)
        self._drawer = None  # _DrawerState | None
        self._inline_search = None  # (buf, pos) | None

        # Auto-refresh state
        self._fs_sig = None

        # Dep state
        self.blocked_ids: set[str] = set()
        # Reverse-dep map: task id -> list of (status, task) that depend on it
        self.reverse_deps: dict[str, list[tuple[str, dict]]] = {}
        # Yaks with a pending-sync sidecar (.yaks/.sync-pending/<id>.yaml)
        self.pending_ids: set[str] = set()

        curses.curs_set(0)
        try:
            curses.set_escdelay(25)  # quick Esc; default is ~1s
        except AttributeError:
            pass
        curses.mousemask(curses.ALL_MOUSE_EVENTS)
        # mouseinterval(0): deliver BUTTON_PRESSED immediately without waiting
        # for a release event. The default (~167ms) holds PRESSED until release
        # arrives — scroll-wheel buttons (B4/B5) never send a release, so
        # scroll events were silently dropped. 0 also eliminates click latency.
        curses.mouseinterval(0)
        # SGR extended mouse mode: when ncurses has no XM terminfo capability
        # (macOS ships ncurses 5.x; tigetstr('XM') is None), it only enables
        # X10 tracking.  Terminals like Zed don't implement X10 properly and
        # kitty/Zed both send scroll events only in SGR encoding.  We enable
        # SGR manually and intercept the raw \033[<...M sequences in handle_key
        # rather than relying on KEY_MOUSE (which ncurses won't fire for SGR).
        try:
            self._sgr_enabled = curses.tigetstr("XM") is None
        except Exception:
            self._sgr_enabled = False
        if self._sgr_enabled:
            sys.stdout.write("\033[?1006h")
            sys.stdout.flush()
        self.stdscr.timeout(500)  # poll filesystem every 500ms when idle
        init_colors()
        self._task_cache: list[tuple[str, dict]] | None = None
        self._resolved_cache: set[str] | None = None
        self.reload()
        self._fs_sig = self._scan_fs()

    def reload(self):
        """Full reload — re-reads disk. Use when task files may have changed."""
        self._refresh_task_cache()
        self._rebuild_task_list()
        self._recompute_blocked()
        self._recompute_pending()
        if self.cursor >= len(self.tasks):
            self.cursor = max(0, len(self.tasks) - 1)
        self._fix_scroll()
        self._rebuild_detail()
        self._fs_sig = self._scan_fs()

    def _rebuild_task_list(self):
        """Re-run build_tree for the current tab/filter, then apply collapse."""
        status = TABS[self.tab][0]
        flat = build_tree(
            self.root, status, self.filter_spec, tasks_cache=self._task_cache, resolved_cache=self._resolved_cache
        )
        filter_active = not self.filter_spec.is_empty()
        self.tasks, self.collapsed_counts = apply_collapse(flat, self.collapsed_ids, filter_active)

    def _toggle_collapse(self, tid: str):
        """Toggle collapse at the nearest parent with children. If cursor is
        inside the subtree being collapsed, snap it to the collapsed row."""
        all_ids = {t["id"] for _, t in (self._task_cache or [])}
        target = tid if any(o.startswith(tid + ".") for o in all_ids) else None
        if target is None:
            from yaklib.model import parent_id as _pid

            target = _pid(tid)
            if target is None or target not in all_ids:
                return
        if target in self.collapsed_ids:
            self.collapsed_ids.discard(target)
        else:
            self.collapsed_ids.add(target)
        self._rebuild_task_list()
        # Snap cursor to the target row if present (keeps it visible after
        # collapsing from inside a subtree).
        for i, (_, t, _, _) in enumerate(self.tasks):
            if t["id"] == target:
                self.cursor = i
                break
        else:
            if self.cursor >= len(self.tasks):
                self.cursor = max(0, len(self.tasks) - 1)
        self._fix_scroll()
        self._rebuild_detail()
        self._save_ui_state()

    def _load_ui_state(self):
        try:
            data = json.loads(self._ui_state_path.read_text())
            ids = data.get("collapsed") or []
            if isinstance(ids, list):
                self.collapsed_ids = {s for s in ids if isinstance(s, str)}
        except (FileNotFoundError, json.JSONDecodeError, OSError):
            pass

    def _save_ui_state(self):
        try:
            self._ui_state_path.parent.mkdir(parents=True, exist_ok=True)
            self._ui_state_path.write_text(json.dumps({"collapsed": sorted(self.collapsed_ids)}))
        except OSError:
            pass

    def _recompute_blocked(self):
        """Update blocked_ids and reverse_deps from all tasks on disk."""
        self.blocked_ids, self.reverse_deps = _deps.compute_blocked(self.root)

    def _recompute_pending(self):
        """Update pending_ids from .yaks/.sync-pending/."""
        from yaklib import sync as _sync

        self.pending_ids = set(_sync.list_pending(self.root))

    def _refresh_task_cache(self):
        """Re-read every task from disk into in-memory caches. Expensive —
        call only when the filesystem has changed."""
        from yaklib.model import STATUSES as _STATUSES
        from yaklib.model import all_tasks as _all_tasks

        cache = []
        for s in _STATUSES:
            for st, t in _all_tasks(self.root, s):
                cache.append((st, t))
        self._task_cache = cache
        self._resolved_cache = _deps.resolved_ids(self.root)

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
        self.detail_select_anchor = None
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
        self.detail_lines = build_detail_lines(self.root, task, status, width, reverse_deps=self.reverse_deps)
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
        self.detail_matches = [i for i, dl in enumerate(self.detail_lines) if q in dl.text.lower()]

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

    def _drawer_height(self):
        return self._drawer.height + 1 if self._drawer else 0

    def _list_height(self):
        h, _ = self.stdscr.getmaxyx()
        return max(1, h - 3 - self._drawer_height())

    def _detail_height(self):
        h, _ = self.stdscr.getmaxyx()
        return max(1, h - 2 - self._drawer_height())

    def run(self):
        try:
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
        finally:
            if self._sgr_enabled:
                sys.stdout.write("\033[?1006l")
                sys.stdout.flush()

    # -- Drawing -----------------------------------------------------------

    def draw(self):
        _render.draw(self)

    def _safe_addstr(self, y, x, text, attr=0):
        _dialogs.safe_addstr(self.stdscr, y, x, text, attr)

    # -- Input handling ----------------------------------------------------

    def handle_key(self, key):
        # Clear transient notification on any input
        self.notification = ""

        # --- SGR mouse handling ---
        # Drain peek-ahead buffer (chars re-queued after SGR prefix detection).
        if self._peek_buf:
            if key not in (-1, curses.KEY_MOUSE):
                self._peek_buf.append(key)
            key = self._peek_buf.pop(0)

        # Accumulate subsequent characters of an in-progress SGR sequence.
        if self._sgr_pending is not None:
            if 0 < key < 128:
                self._sgr_pending += chr(key)
                end = self._sgr_pending[-1]
                if end in ("M", "m") or len(self._sgr_pending) > 30:
                    seq, self._sgr_pending = self._sgr_pending, None
                    if seq and seq[-1] in ("M", "m"):
                        self._dispatch_sgr_mouse(seq)
            else:
                self._sgr_pending = None  # abort on unexpected char
            return True

        # Detect SGR sequence start: \033[< arrives as ESC then '[' then '<'.
        # Use a non-blocking peek so we don't block if it's a bare Escape.
        if key == 27 and self._sgr_enabled:
            self.stdscr.timeout(0)
            c1 = self.stdscr.getch()
            c2 = self.stdscr.getch() if c1 == ord("[") else -1
            self.stdscr.timeout(500)
            if c1 == ord("[") and c2 == ord("<"):
                self._sgr_pending = ""  # start accumulating after \033[<
                return True
            # Not an SGR sequence — re-queue the peeked chars.
            if c1 != -1:
                self._peek_buf.append(c1)
            if c2 != -1:
                self._peek_buf.append(c2)
            # Fall through for normal ESC handling.

        # X10 mouse events from ncurses (fires on terminals with native X10).
        if key == curses.KEY_MOUSE:
            try:
                _, mx, my, _, bstate = curses.getmouse()
                self._handle_mouse(bstate, mx, my)
            except curses.error:
                pass
            return True

        # Filter drawer captures all input when open.
        if self._drawer is not None:
            self._handle_drawer_key(key)
            return True

        # Inline search captures all input when active.
        if self._inline_search is not None:
            self._handle_inline_search_key(key)
            return True

        if key == ord("q"):
            return False
        if key == ord("?"):
            self.show_help = True
            return True
        # Refresh: F (re-Fresh) or Ctrl-L (terminal convention for redraw).
        if key == ord("F") or key == 12:
            self.reload()
            return True

        if self.focus == "detail":
            return _keys_detail.handle(self, key)
        return _keys_list.handle(self, key)

    def _handle_drawer_key(self, key):
        d = self._drawer
        kind = _DRAWER_ROWS[d.row][0]

        is_text = kind.endswith("_text")
        if is_text:
            ed = d.editor_for(kind)
            # Row-nav keys always escape the text field — otherwise you
            # can't Tab/arrow out. Tab=9, BTab, Up/Down, Ctrl-N/P (14/16).
            # In vim normal mode, j/k also exit and navigate rows.
            row_nav_down = key in (curses.KEY_DOWN, 9, 14) or (ed.mode == "normal" and key == ord("j"))
            row_nav_up = key in (curses.KEY_UP, curses.KEY_BTAB, 16) or (ed.mode == "normal" and key == ord("k"))
            if row_nav_down:
                d.row = (d.row + 1) % len(_DRAWER_ROWS)
                d.chip_idx = 0
                return
            if row_nav_up:
                d.row = (d.row - 1) % len(_DRAWER_ROWS)
                d.chip_idx = 0
                return

            r = ed.step(key)
            if r == _vim_edit.COMMIT:
                self._close_filter_drawer(commit=True)
                return
            if r == _vim_edit.CANCEL:
                self._close_filter_drawer(commit=False)
                return
            self._drawer_live_preview()
            return

        # Non-text rows: classic drawer-level handling.
        if key in (ord("\n"), curses.KEY_ENTER, 10, 13):
            self._close_filter_drawer(commit=True)
            return
        if key == 27:
            self._close_filter_drawer(commit=False)
            return
        if key == ord("C"):
            d.statuses.clear()
            d.types.clear()
            d.priorities.clear()
            d.labels = LineEditor("", vim=self.vim_mode)
            d.search = LineEditor("", vim=self.vim_mode)
            d.parent = LineEditor("", vim=self.vim_mode)
            d.ready = False
            d.tangled = False
            self._drawer_live_preview()
            return

        # Row navigation. Ctrl-N/P always work; j/k work on chip rows
        # (text rows are already handled above and don't fall through).
        if key in (curses.KEY_DOWN, ord("\t"), 14, ord("j")):
            d.row = (d.row + 1) % len(_DRAWER_ROWS)
            d.chip_idx = 0
            return
        if key in (curses.KEY_UP, curses.KEY_BTAB, 16, ord("k")):
            d.row = (d.row - 1) % len(_DRAWER_ROWS)
            d.chip_idx = 0
            return

        changed = False
        if kind.endswith("_chips"):
            choices = _drawer_chip_choices(kind)
            if key == curses.KEY_LEFT or key == ord("h"):
                d.chip_idx = (d.chip_idx - 1) % len(choices)
            elif key == curses.KEY_RIGHT or key == ord("l"):
                d.chip_idx = (d.chip_idx + 1) % len(choices)
            elif key == ord(" "):
                val = choices[d.chip_idx]
                if kind == "status_chips":
                    d.statuses.symmetric_difference_update({val})
                elif kind == "type_chips":
                    d.types.symmetric_difference_update({val})
                elif kind == "pri_chips":
                    p = int(val[1:])
                    d.priorities.symmetric_difference_update({p})
                elif kind == "deps_chips":
                    if val == "ready only":
                        d.ready = not d.ready
                    else:
                        d.tangled = not d.tangled
                changed = True

        if changed:
            self._drawer_live_preview()

    def _handle_inline_search_key(self, key):
        ed = self._inline_search
        r = ed.step(key)
        if r == _vim_edit.COMMIT:
            self._close_inline_search(commit=True)
            return
        if r == _vim_edit.CANCEL:
            self._close_inline_search(commit=False)
            return
        if r == _vim_edit.ESCALATE:
            # Tab: escalate to full drawer, carrying the typed text forward.
            from dataclasses import replace as _replace

            spec = _replace(self.filter_spec, search=ed.buf.strip())
            ed.close()
            self._inline_search = None
            curses.curs_set(0)
            self.filter_spec = spec
            self._open_filter_drawer()
            self._drawer.row = 4  # search_text
            return
        # Live preview — cache-only, no disk scan.
        from dataclasses import replace as _replace

        self.filter_spec = _replace(self.filter_spec, search=ed.buf.strip())
        self._rebuild_task_list()
        if self.cursor >= len(self.tasks):
            self.cursor = max(0, len(self.tasks) - 1)
        self._fix_scroll()
        self._rebuild_detail()

    # -- Mouse support -------------------------------------------------------

    def _list_y_start(self) -> int:
        """Row index where the list pane begins (mirrors render.py's list_y)."""
        drawer_h = self._drawer.height if self._drawer else 0
        return 1 + drawer_h + 1

    def _scroll_viewport(self, delta: int) -> None:
        """Scroll the list viewport by *delta* rows without moving the cursor.

        The cursor stays on the same task if it remains visible.  If scrolling
        would hide it, the cursor is pulled to the nearest visible edge
        (vim Ctrl-E / Ctrl-Y semantics).
        """
        n = len(self.tasks)
        if n == 0:
            return
        h = self._list_height()
        self.scroll = max(0, min(max(0, n - h), self.scroll + delta))
        changed = False
        if self.cursor < self.scroll:
            self.cursor = self.scroll
            changed = True
        elif self.cursor >= self.scroll + h:
            self.cursor = self.scroll + h - 1
            changed = True
        if changed:
            self._rebuild_detail()

    def _scroll_detail_viewport(self, delta: int) -> None:
        """Scroll the detail viewport by *delta* rows without moving the cursor."""
        n = len(self.detail_lines)
        if n == 0:
            return
        h = self._detail_height()
        if self.detail_search:
            h = max(1, h - 1)
        self.detail_scroll = max(0, min(max(0, n - h), self.detail_scroll + delta))
        if self.detail_line_cursor < self.detail_scroll:
            self.detail_line_cursor = self.detail_scroll
        elif self.detail_line_cursor >= self.detail_scroll + h:
            self.detail_line_cursor = max(0, self.detail_scroll + h - 1)

    def _handle_mouse(self, bstate: int, mx: int, my: int) -> None:
        """Dispatch a mouse event decoded from curses.getmouse()."""
        _, w = self.stdscr.getmaxyx()
        _SCROLL_STEP = 3

        # Scroll wheel up — BUTTON4_PRESSED is reliable across terminals.
        # Scroll wheel down: BUTTON5_PRESSED is absent on older ncurses builds
        # (macOS ships ncurses 5.x where that constant isn't defined).  We use
        # getattr so we get the real constant on systems that have it and fall
        # back gracefully elsewhere (j/k/arrow-down still work).
        _B5 = getattr(curses, "BUTTON5_PRESSED", None)
        if bstate & curses.BUTTON4_PRESSED:
            if self.focus == "detail":
                self._scroll_detail_viewport(-_SCROLL_STEP)
            else:
                self._scroll_viewport(-_SCROLL_STEP)
            return
        if _B5 and (bstate & _B5):
            if self.focus == "detail":
                self._scroll_detail_viewport(+_SCROLL_STEP)
            else:
                self._scroll_viewport(+_SCROLL_STEP)
            return

        # Only handle left-button press from here (mouseinterval=0 means
        # CLICKED is never synthesised; we detect double-click manually).
        if not (bstate & curses.BUTTON1_PRESSED):
            return

        # Manual double-click: two presses on the same spot within 500 ms.
        now = time.monotonic()
        last_t, last_my, last_mx = self._last_click
        double = now - last_t < 0.5 and last_my == my and abs(last_mx - mx) < 4
        self._last_click = (now, my, mx)

        # --- Tab row (y == 0) ---
        if my == 0:
            x = 0
            counts = _tab_counts(self)
            spec_statuses = self.filter_spec.statuses
            for i, (status, label) in enumerate(TABS):
                marker = "*" if (spec_statuses and status in spec_statuses) else ""
                text = f" {label}{marker} ({counts[status]}) "
                if mx < x + len(text):
                    if i != self.tab:
                        self.tab = i
                        self._reset_list()
                    break
                x += len(text) + 1
            return

        list_y = self._list_y_start()
        if my < list_y:
            return  # drawer or gap row — ignore

        # Determine list vs. detail pane geometry.
        detail_x = None
        if self.focus == "detail":
            detail_w = max(w * 2 // 3, 40)
            list_w = w - detail_w - 1
            detail_x = list_w + 1

        in_detail = detail_x is not None and mx >= detail_x

        if in_detail:
            # Click in detail pane: move detail_line_cursor.
            # draw_detail starts at list_y - 1; search bar consumes one row.
            content_y = (list_y - 1) + (1 if self.detail_search else 0)
            dl_idx = self.detail_scroll + (my - content_y)
            if 0 <= dl_idx < len(self.detail_lines):
                self.detail_line_cursor = dl_idx
                self._fix_detail_scroll()
                if double:
                    self._follow_link()
        else:
            # Click in list pane: move cursor.
            idx = self.scroll + (my - list_y)
            if 0 <= idx < len(self.tasks):
                if idx == self.cursor and double and self.detail_lines:
                    self._enter_detail()
                elif idx != self.cursor:
                    self.cursor = idx
                    self._rebuild_detail()

    def _dispatch_sgr_mouse(self, seq: str) -> None:
        """Dispatch an SGR mouse event from the sequence after \\033[<.

        *seq* is the part after the literal prefix, e.g. ``"64;13;7M"``.
        Columns and rows are 1-indexed in the SGR protocol.
        """
        # seq = "button;col;row(M|m)": M = press, m = release.
        released = seq[-1] == "m"
        if released:
            return  # act only on press
        try:
            btn_s, col_s, row_s = seq[:-1].split(";")
            btn = int(btn_s)
            mx = int(col_s) - 1  # 1-indexed → 0-indexed
            my = int(row_s) - 1
        except (ValueError, AttributeError):
            return

        _SCROLL_STEP = 3
        # Strip Shift/Alt/Ctrl modifier bits to get the base button number.
        btn_base = btn & ~(4 | 8 | 16)

        if btn_base == 64:  # scroll up
            if self.focus == "detail":
                self._scroll_detail_viewport(-_SCROLL_STEP)
            else:
                self._scroll_viewport(-_SCROLL_STEP)
        elif btn_base == 65:  # scroll down
            if self.focus == "detail":
                self._scroll_detail_viewport(+_SCROLL_STEP)
            else:
                self._scroll_viewport(+_SCROLL_STEP)
        elif btn_base == 0:  # left button
            self._handle_mouse(curses.BUTTON1_PRESSED, mx, my)

    def _list_page(self, direction, half=False):
        if not self.tasks:
            return
        h = self._list_height()
        step = max(1, h // 2 if half else h - 1)
        self.cursor = max(0, min(len(self.tasks) - 1, self.cursor + direction * step))
        self._fix_scroll()
        self._rebuild_detail()

    def _detail_page(self, direction, half=False):
        if not self.detail_lines:
            return
        h = self._detail_height()
        if self.detail_search:
            h = max(1, h - 1)
        step = max(1, h // 2 if half else h - 1)
        self.detail_line_cursor = max(0, min(len(self.detail_lines) - 1, self.detail_line_cursor + direction * step))
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
        """Open a file or URL using the system's default handler."""
        import platform as _pl
        import subprocess as _sp

        is_url = isinstance(path, str) and (path.startswith("http://") or path.startswith("https://"))
        if not is_url and not Path(path).exists():
            self.notification = f"missing: {path}"
            return
        try:
            opener = "open" if _pl.system() == "Darwin" else "xdg-open"
            _sp.Popen([opener, str(path)], stdout=_sp.DEVNULL, stderr=_sp.DEVNULL)
            label = path if is_url else Path(path).name
            self.notification = f"opened {label}"
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
        if (
            self.nav_pos < 0
            or self.nav_pos >= len(self.nav_history)
            or (current_id and self.nav_history[self.nav_pos] != current_id)
        ):
            if current_id:
                self.nav_history = [current_id]
                self.nav_pos = 0
            else:
                self.nav_history = []
                self.nav_pos = -1
        # Truncate forward, append target
        self.nav_history = self.nav_history[: self.nav_pos + 1]
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

        # Reload with no filters to ensure the task is visible. Also expand
        # any collapsed ancestors so we can actually see the target row.
        self.filter_spec = FilterSpec()
        self.detail_search = ""
        from yaklib.model import parent_id as _pid

        pid = _pid(task_id)
        while pid:
            if pid in self.collapsed_ids:
                self.collapsed_ids.discard(pid)
            pid = _pid(pid)
        self._rebuild_task_list()

        # Find and select the task
        for i, (_, t, _, _) in enumerate(self.tasks):
            if t["id"] == task_id:
                self.cursor = i
                self._fix_scroll()
                break

        self._rebuild_detail()
        self.focus = "detail"

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

    def _quick_adjust_labels(self, tid):
        _mutate.quick_adjust_labels(self, tid)

    def _quick_adjust_state(self, tid):
        _mutate.quick_adjust_state(self, tid)

    def _add_dependency(self, tid):
        _mutate.add_dependency(self, tid)

    def _depends_on_transitively(self, start_id, target_id):
        return _deps.depends_on_transitively(self.root, start_id, target_id)

    def _add_comment(self, tid):
        _mutate.add_comment(self, tid)

    def _attach_file(self, tid):
        _mutate.attach_file(self, tid)

    def _reparent_task(self, tid):
        _mutate.reparent_task(self, tid)

    def _pick_type_for_create(self):
        choice = _dialogs.pick_type_for_create(self.stdscr)
        if choice is None:
            self.notification = "create cancelled"
        return choice

    def _copy_to_clipboard(self, text):
        _mutate.copy_to_clipboard(self, text)

    def _open_sync_review(self, tid):
        if tid not in self.pending_ids:
            self.notification = f"no pending sidecar for {tid}"
            return
        from yaktui import sync_review as _sr

        _sr.open_review(self, tid)

    def _confirm(self, prompt, default_yes=False):
        return _dialogs.confirm(self.stdscr, prompt, default_yes)

    def _fuzzy_pick_task(self, prompt, exclude_ids=None):
        return _dialogs.fuzzy_pick_task(self.stdscr, self.root, prompt, exclude_ids=exclude_ids, vim=self.vim_mode)

    def _edit_prompt(self, prompt, initial=""):
        return _dialogs.edit_prompt(self.stdscr, prompt, initial, vim=self.vim_mode)

    def _input_prompt(self, prompt):
        return _dialogs.input_prompt(self.stdscr, prompt, vim=self.vim_mode)

    def _open_filter_drawer(self):
        self._drawer = _DrawerState(self.filter_spec, vim=self.vim_mode)
        self._available_labels = collect_all_labels(self.root)

    def _close_filter_drawer(self, commit: bool):
        if self._drawer is None:
            return
        if commit:
            self.filter_spec = self._drawer.build_spec()
        else:
            self.filter_spec = self._drawer.saved
        self._drawer.close()
        self._drawer = None
        curses.curs_set(0)
        self._reset_list()

    def _drawer_live_preview(self):
        """Rebuild list from the drawer's working spec (live update).
        Uses the in-memory task cache — no disk scan."""
        if self._drawer is None:
            return
        self.filter_spec = self._drawer.build_spec()
        self._rebuild_task_list()
        if self.cursor >= len(self.tasks):
            self.cursor = max(0, len(self.tasks) - 1)
        self._fix_scroll()
        self._rebuild_detail()

    def _open_inline_search(self):
        self._inline_search = LineEditor(self.filter_spec.search, vim=self.vim_mode, allow_escalate=True)

    def _close_inline_search(self, commit: bool):
        if self._inline_search is None:
            return
        if commit:
            from dataclasses import replace as _replace

            self.filter_spec = _replace(self.filter_spec, search=self._inline_search.buf.strip())
        self._inline_search.close()
        self._inline_search = None
        curses.curs_set(0)
        self._reset_list()


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
