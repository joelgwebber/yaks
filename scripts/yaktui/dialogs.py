"""Interactive input dialogs for the curses TUI.

Every function here is stateless with respect to the App — it takes
stdscr (and whatever extra context it needs) and blocks until the user
commits or cancels. Returns the entered value (or None / "" on Escape,
depending on the dialog's convention).
"""

from __future__ import annotations

import curses
from pathlib import Path

from yaklib.format import status_emoji
from yaklib.model import STATUSES, all_tasks

from yaktui import vim_edit as _vim_edit
from yaktui.colors import C_SEARCH, C_SELECTED
from yaktui.vim_edit import LineEditor

# Shared row/list navigation keys, used by every multi-row dialog (the fuzzy
# picker and the task form) so Ctrl-N/P, Tab/Shift-Tab, and the arrow keys all
# move selection identically no matter which dialog you're in.
NAV_NEXT_KEYS = (curses.KEY_DOWN, 14, ord("\t"))  # Down / Ctrl-N / Tab
NAV_PREV_KEYS = (curses.KEY_UP, 16, curses.KEY_BTAB)  # Up / Ctrl-P / Shift-Tab


def safe_addstr(stdscr, y, x, text, attr=0):
    """Guarded addnstr: clips to screen, swallows curses errors."""
    h, w = stdscr.getmaxyx()
    if y < 0 or y >= h or x >= w:
        return
    try:
        stdscr.addnstr(y, x, text, w - x, attr)
    except curses.error:
        pass


def line_editor_window(ed: LineEditor, inner_w: int) -> tuple[str, int]:
    """Window a LineEditor to *inner_w* visible columns.

    Returns (visible_text, cursor_col): the buffer slice to draw and the column
    of the caret within that slice. Shared by every single-line input so the
    text-windowing + caret math lives in exactly one place.
    """
    inner_w = max(1, inner_w)
    offset = max(0, ed.pos - inner_w + 1)
    return ed.buf[offset : offset + inner_w], ed.pos - offset


def input_prompt(stdscr, prompt: str, vim: bool = False) -> str:
    """Read a line from the bottom bar. Escape cancels (returns "").
    Empty string on cancel mirrors the pre-vim behavior."""
    r = edit_prompt(stdscr, prompt, "", vim=vim)
    return r or ""


def edit_prompt(stdscr, prompt: str, initial: str = "", vim: bool = False) -> str | None:
    """Read a line pre-populated with *initial*. Returns the string on
    Enter, or None on cancel."""
    h, w = stdscr.getmaxyx()
    y = h - 1
    ed = LineEditor(initial, vim=vim)
    curses.curs_set(1)
    try:
        while True:
            badge = ed.mode_badge()
            badge_w = len(badge) + 1 if badge else 0
            total_prefix = len(prompt) + badge_w
            max_vis = max(1, w - total_prefix - 1)
            visible, cur = line_editor_window(ed, max_vis)
            safe_addstr(stdscr, y, 0, " " * w, 0)
            safe_addstr(stdscr, y, 0, prompt, curses.color_pair(C_SEARCH) | curses.A_BOLD)
            if badge:
                safe_addstr(stdscr, y, len(prompt), badge + " ", curses.A_DIM)
            safe_addstr(stdscr, y, total_prefix, visible, 0)
            try:
                stdscr.move(y, total_prefix + cur)
            except curses.error:
                pass
            stdscr.refresh()

            ch = stdscr.getch()
            if ch == -1:
                continue
            r = ed.step(ch)
            if r == _vim_edit.COMMIT:
                return ed.buf.strip()
            if r == _vim_edit.CANCEL:
                return None
    finally:
        ed.close()
        curses.curs_set(0)


def pick(stdscr, prompt: str, choices: str) -> str | None:
    """Single-key picker at the bottom. Returns the chosen char, or None on Esc.
    *choices* is a case-sensitive string of valid keys.
    """
    h, w = stdscr.getmaxyx()
    y = h - 1
    safe_addstr(stdscr, y, 0, " " * w, 0)
    safe_addstr(stdscr, y, 0, prompt[:w], curses.color_pair(C_SEARCH) | curses.A_BOLD)
    stdscr.refresh()
    while True:
        ch = stdscr.getch()
        if ch == -1:
            continue
        if ch == 27:
            return None
        c = chr(ch) if 0 <= ch < 256 else ""
        if c in choices:
            return c


def confirm(stdscr, prompt: str, default_yes: bool = False) -> bool:
    """yes/no prompt. y/Y = yes, n/N/Esc = no. Enter follows *default_yes*."""
    h, w = stdscr.getmaxyx()
    y = h - 1
    safe_addstr(stdscr, y, 0, " " * w, 0)
    safe_addstr(stdscr, y, 0, prompt[:w], curses.color_pair(C_SEARCH) | curses.A_BOLD)
    stdscr.refresh()
    while True:
        ch = stdscr.getch()
        if ch == -1:
            continue
        if ch in (ord("y"), ord("Y")):
            return True
        if ch in (ord("n"), ord("N"), 27):
            return False
        if ch in (ord("\n"), curses.KEY_ENTER, 10, 13):
            return default_yes


def pick_type_for_create(stdscr) -> str | None:
    """Pick a yak type. Returns 'task'|'bug'|'feature'|'idea', or None on Esc."""
    type_map = {"t": "task", "b": "bug", "f": "feature", "i": "idea"}
    choice = pick(stdscr, "New yak type: t=task b=bug f=feature i=idea  (Esc=cancel)", "tbfi")
    if choice is None:
        return None
    return type_map[choice]


def fuzzy_pick_task(
    stdscr, root: Path, prompt: str, exclude_ids: set[str] | None = None, vim: bool = False
) -> str | None:
    """Interactive fuzzy search over all tasks. Returns task ID or None.

    Shows a floating results list above the prompt that updates as the
    user types. Arrow keys / Ctrl-N/P / Tab cycle, Enter selects.
    When *vim* is true the input line uses vim mode; j/k in normal mode
    also move the list selection.
    """
    exclude = set(exclude_ids or [])
    tasks: list[tuple[str, dict]] = []
    for s in STATUSES:
        for st, t in all_tasks(root, s):
            if t["id"] not in exclude:
                tasks.append((st, t))

    def _match(query: str):
        if not query:
            return tasks[:20]
        q = query.lower()
        scored = []
        for st, t in tasks:
            tid = t["id"].lower()
            title = t.get("title", "").lower()
            if q in tid or q in title:
                score = 0 if tid.startswith(q) else (1 if q in tid else 2)
                scored.append((score, st, t))
        scored.sort(key=lambda x: (x[0], x[2].get("priority", 9), x[2]["id"]))
        return [(s, t) for _, s, t in scored[:20]]

    h, w = stdscr.getmaxyx()
    ed = LineEditor("", vim=vim)
    sel = 0
    max_visible = min(10, h - 4)
    prev_buf = None
    curses.curs_set(1)
    try:
        while True:
            if ed.buf != prev_buf:
                matches = _match(ed.buf)
                sel = 0
                prev_buf = ed.buf
            sel = max(0, min(sel, len(matches) - 1))

            list_y = h - 2 - max_visible
            for i in range(max_visible):
                y = list_y + i
                if y < 1:
                    continue
                safe_addstr(stdscr, y, 0, " " * w, 0)
                if i < len(matches):
                    ms, mt = matches[i]
                    line = f"  {status_emoji(ms)} {mt['id']}  {mt.get('title', '')}"
                    attr = curses.color_pair(C_SELECTED) | curses.A_BOLD if i == sel else 0
                    safe_addstr(stdscr, y, 0, line[:w], attr)

            prompt_y = h - 1
            badge = ed.mode_badge()
            badge_w = len(badge) + 1 if badge else 0
            total_prefix = len(prompt) + badge_w
            max_vis = max(1, w - total_prefix - 1)
            offset = max(0, ed.pos - max_vis + 1)
            visible = ed.buf[offset : offset + max_vis]
            count_str = f" ({len(matches)} matches)" if ed.buf else ""
            safe_addstr(stdscr, prompt_y, 0, " " * w, 0)
            safe_addstr(stdscr, prompt_y, 0, prompt, curses.color_pair(C_SEARCH) | curses.A_BOLD)
            if badge:
                safe_addstr(stdscr, prompt_y, len(prompt), badge + " ", curses.A_DIM)
            safe_addstr(stdscr, prompt_y, total_prefix, visible, 0)
            cs = total_prefix + len(visible)
            if cs + len(count_str) < w:
                safe_addstr(stdscr, prompt_y, cs, count_str, curses.A_DIM)
            try:
                stdscr.move(prompt_y, total_prefix + (ed.pos - offset))
            except curses.error:
                pass
            stdscr.refresh()

            ch = stdscr.getch()
            if ch == -1:
                continue

            # List navigation: always-on keys + vim-normal j/k.
            list_nav = ch in NAV_NEXT_KEYS or ch in NAV_PREV_KEYS
            if vim and ed.mode == "normal" and ch in (ord("j"), ord("k")):
                list_nav = True
            if list_nav:
                if ch in NAV_PREV_KEYS or ch == ord("k"):
                    sel = max(0, sel - 1)
                else:
                    sel = min(len(matches) - 1, sel + 1) if matches else 0
                continue

            r = ed.step(ch)
            if r == _vim_edit.COMMIT:
                if matches and 0 <= sel < len(matches):
                    return matches[sel][1]["id"]
                return None
            if r == _vim_edit.CANCEL:
                return None
    finally:
        ed.close()
        curses.curs_set(0)


def _text_edit(buf: str, pos: int, ch: int) -> tuple[str, int]:
    if ch in (curses.KEY_BACKSPACE, 127, 8):
        if pos > 0:
            buf = buf[: pos - 1] + buf[pos:]
            pos -= 1
    elif ch == curses.KEY_DC:
        if pos < len(buf):
            buf = buf[:pos] + buf[pos + 1 :]
    elif ch == curses.KEY_LEFT:
        pos = max(0, pos - 1)
    elif ch == curses.KEY_RIGHT:
        pos = min(len(buf), pos + 1)
    elif ch in (curses.KEY_HOME, 1):
        pos = 0
    elif ch in (curses.KEY_END, 5):
        pos = len(buf)
    elif ch == 21:  # Ctrl-U
        buf = buf[pos:]
        pos = 0
    elif 32 <= ch < 127:
        buf = buf[:pos] + chr(ch) + buf[pos:]
        pos += 1
    return buf, pos
