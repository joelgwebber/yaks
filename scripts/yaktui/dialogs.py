"""Interactive input dialogs for the curses TUI.

Every function here is stateless with respect to the App — it takes
stdscr (and whatever extra context it needs) and blocks until the user
commits or cancels. Returns the entered value (or None / "" on Escape,
depending on the dialog's convention).
"""

from __future__ import annotations

from pathlib import Path

import curses

from yaklib.format import status_char
from yaklib.model import STATUSES, all_tasks
from yaktui.colors import C_SEARCH, C_SELECTED


def safe_addstr(stdscr, y, x, text, attr=0):
    """Guarded addnstr: clips to screen, swallows curses errors."""
    h, w = stdscr.getmaxyx()
    if y < 0 or y >= h or x >= w:
        return
    try:
        stdscr.addnstr(y, x, text, w - x, attr)
    except curses.error:
        pass


def input_prompt(stdscr, prompt: str) -> str:
    """Read a line from the bottom bar. Escape cancels (returns "")."""
    h, w = stdscr.getmaxyx()
    y = h - 1
    buf = ""
    curses.curs_set(1)
    try:
        while True:
            safe_addstr(stdscr, y, 0, " " * w, 0)
            safe_addstr(stdscr, y, 0, prompt,
                        curses.color_pair(C_SEARCH) | curses.A_BOLD)
            safe_addstr(stdscr, y, len(prompt), buf, 0)
            try:
                stdscr.move(y, min(len(prompt) + len(buf), w - 1))
            except curses.error:
                pass
            stdscr.refresh()

            ch = stdscr.getch()
            if ch == -1:
                continue
            if ch == 27:
                return ""
            if ch in (ord("\n"), curses.KEY_ENTER, 10, 13):
                return buf.strip()
            if ch in (curses.KEY_BACKSPACE, 127, 8):
                buf = buf[:-1]
            elif 32 <= ch < 127:
                if len(prompt) + len(buf) < w - 1:
                    buf += chr(ch)
    finally:
        curses.curs_set(0)


def edit_prompt(stdscr, prompt: str, initial: str = "") -> str | None:
    """Read a line pre-populated with *initial*. Returns the string on
    Enter, or None on Escape. Supports Home/End/arrows/Ctrl-A/E/K/U/W.
    """
    h, w = stdscr.getmaxyx()
    y = h - 1
    buf = initial
    pos = len(buf)
    curses.curs_set(1)
    try:
        while True:
            max_vis = max(1, w - len(prompt) - 1)
            offset = max(0, pos - max_vis + 1)
            visible = buf[offset:offset + max_vis]
            safe_addstr(stdscr, y, 0, " " * w, 0)
            safe_addstr(stdscr, y, 0, prompt,
                        curses.color_pair(C_SEARCH) | curses.A_BOLD)
            safe_addstr(stdscr, y, len(prompt), visible, 0)
            try:
                stdscr.move(y, len(prompt) + (pos - offset))
            except curses.error:
                pass
            stdscr.refresh()

            ch = stdscr.getch()
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
            elif ch == curses.KEY_DC:
                if pos < len(buf):
                    buf = buf[:pos] + buf[pos + 1:]
            elif ch == curses.KEY_LEFT:
                pos = max(0, pos - 1)
            elif ch == curses.KEY_RIGHT:
                pos = min(len(buf), pos + 1)
            elif ch in (curses.KEY_HOME, 1):
                pos = 0
            elif ch in (curses.KEY_END, 5):
                pos = len(buf)
            elif ch == 11:  # Ctrl-K
                buf = buf[:pos]
            elif ch == 21:  # Ctrl-U
                buf = buf[pos:]
                pos = 0
            elif ch == 23:  # Ctrl-W
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


def pick(stdscr, prompt: str, choices: str) -> str | None:
    """Single-key picker at the bottom. Returns the chosen char, or None on Esc.
    *choices* is a case-sensitive string of valid keys.
    """
    h, w = stdscr.getmaxyx()
    y = h - 1
    safe_addstr(stdscr, y, 0, " " * w, 0)
    safe_addstr(stdscr, y, 0, prompt[:w],
                curses.color_pair(C_SEARCH) | curses.A_BOLD)
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
    safe_addstr(stdscr, y, 0, prompt[:w],
                curses.color_pair(C_SEARCH) | curses.A_BOLD)
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
    choice = pick(stdscr,
                  "New yak type: t=task b=bug f=feature i=idea  (Esc=cancel)",
                  "tbfi")
    if choice is None:
        return None
    return type_map[choice]


def fuzzy_pick_task(stdscr, root: Path, prompt: str,
                    exclude_ids: set[str] | None = None) -> str | None:
    """Interactive fuzzy search over all tasks. Returns task ID or None.

    Shows a floating results list above the prompt that updates as the
    user types. Arrow keys / Ctrl-N/P / Tab cycle, Enter selects, Esc cancels.
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
    buf = ""
    pos = 0
    sel = 0
    max_visible = min(10, h - 4)
    curses.curs_set(1)
    try:
        while True:
            matches = _match(buf)
            sel = max(0, min(sel, len(matches) - 1))

            list_y = h - 2 - max_visible
            for i in range(max_visible):
                y = list_y + i
                if y < 1:
                    continue
                safe_addstr(stdscr, y, 0, " " * w, 0)
                if i < len(matches):
                    ms, mt = matches[i]
                    line = f"  [{status_char(ms)}] {mt['id']}  {mt.get('title', '')}"
                    attr = (curses.color_pair(C_SELECTED) | curses.A_BOLD
                            if i == sel else 0)
                    safe_addstr(stdscr, y, 0, line[:w], attr)

            prompt_y = h - 1
            max_vis = max(1, w - len(prompt) - 1)
            offset = max(0, pos - max_vis + 1)
            visible = buf[offset:offset + max_vis]
            count_str = f" ({len(matches)} matches)" if buf else ""
            safe_addstr(stdscr, prompt_y, 0, " " * w, 0)
            safe_addstr(stdscr, prompt_y, 0, prompt,
                        curses.color_pair(C_SEARCH) | curses.A_BOLD)
            safe_addstr(stdscr, prompt_y, len(prompt), visible, 0)
            cs = len(prompt) + len(visible)
            if cs + len(count_str) < w:
                safe_addstr(stdscr, prompt_y, cs, count_str, curses.A_DIM)
            try:
                stdscr.move(prompt_y, len(prompt) + (pos - offset))
            except curses.error:
                pass
            stdscr.refresh()

            ch = stdscr.getch()
            if ch == -1:
                continue
            if ch == 27:
                return None
            if ch in (ord("\n"), curses.KEY_ENTER, 10, 13):
                if matches and 0 <= sel < len(matches):
                    return matches[sel][1]["id"]
                return None
            if ch in (curses.KEY_UP, 16):
                sel = max(0, sel - 1)
            elif ch in (curses.KEY_DOWN, 14):
                sel = min(len(matches) - 1, sel + 1) if matches else 0
            elif ch == 9:
                sel = min(len(matches) - 1, sel + 1) if matches else 0
            elif ch in (curses.KEY_BACKSPACE, 127, 8):
                if pos > 0:
                    buf = buf[:pos - 1] + buf[pos:]
                    pos -= 1
                    sel = 0
            elif ch == 21:
                buf = ""
                pos = 0
                sel = 0
            elif ch == 23:
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
