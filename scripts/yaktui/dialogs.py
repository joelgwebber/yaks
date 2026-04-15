"""Interactive input dialogs for the curses TUI.

Every function here is stateless with respect to the App — it takes
stdscr (and whatever extra context it needs) and blocks until the user
commits or cancels. Returns the entered value (or None / "" on Escape,
depending on the dialog's convention).
"""

from __future__ import annotations

from pathlib import Path

import curses

from yaklib.filter import FilterSpec
from yaklib.format import status_emoji
from yaklib.model import DEAD, HAIRY, SHAVING, SHORN, STATUSES, all_tasks
from yaktui.colors import C_SEARCH, C_SELECTED, C_TAB_ACTIVE


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
                    line = f"  {status_emoji(ms)} {mt['id']}  {mt.get('title', '')}"
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


# ---------------------------------------------------------------------------
# Filter editor (yak-3990)
# ---------------------------------------------------------------------------

_FILTER_STATUS_CHOICES = [HAIRY, SHAVING, SHORN, DEAD]
_FILTER_TYPE_CHOICES = ["task", "bug", "feature", "idea"]
_FILTER_PRI_CHOICES = [1, 2, 3]
_FILTER_DEPS_CHOICES = ["ready only", "tangled only"]


def filter_editor(stdscr, spec: FilterSpec,
                  focus_search: bool = False) -> FilterSpec | None:
    """Modal overlay to edit a FilterSpec. Returns the new spec, or None on
    Cancel/Esc. If *focus_search* is true, cursor opens on the search row.
    """
    # Working copies of spec fields.
    statuses = set(spec.statuses)
    types = set(spec.types)
    priorities = set(spec.priorities)
    labels_buf = ", ".join(spec.labels)
    search_buf = spec.search
    ready = spec.ready_only
    tangled = spec.tangled_only
    parent_buf = spec.parent

    # Rows: (kind, label). Chip rows also have their own chip cursor.
    rows = [
        ("status_chips", "status"),
        ("type_chips", "type"),
        ("pri_chips", "priority"),
        ("labels_text", "labels"),
        ("search_text", "search"),
        ("deps_chips", "deps"),
        ("parent_text", "parent"),
        ("buttons", ""),
    ]
    row_idx = 4 if focus_search else 0
    chip_idx = 0
    # Text-field cursor positions (one per text row).
    labels_pos = len(labels_buf)
    search_pos = len(search_buf)
    parent_pos = len(parent_buf)
    button_idx = 0  # 0=Apply, 1=Clear, 2=Cancel

    def build_spec() -> FilterSpec:
        lbls = tuple(s.strip() for s in labels_buf.split(",") if s.strip())
        return FilterSpec(
            statuses=frozenset(statuses),
            types=frozenset(types),
            priorities=frozenset(priorities),
            labels=lbls,
            search=search_buf.strip(),
            ready_only=ready,
            tangled_only=tangled,
            parent=parent_buf.strip(),
        )

    curses.curs_set(0)
    while True:
        h, w = stdscr.getmaxyx()
        box_w = min(max(60, w - 4), w - 2)
        box_h = min(len(rows) + 6, h - 2)
        y0 = max(0, (h - box_h) // 2)
        x0 = max(0, (w - box_w) // 2)

        # Background / border
        for i in range(box_h):
            y = y0 + i
            if y >= h:
                break
            safe_addstr(stdscr, y, x0, " " * box_w,
                        curses.color_pair(C_TAB_ACTIVE))
            if i == 0 or i == box_h - 1:
                border = "\u2500" * (box_w - 2)
                corner = "\u250c" if i == 0 else "\u2514"
                end = "\u2510" if i == 0 else "\u2518"
                safe_addstr(stdscr, y, x0, corner + border + end,
                            curses.color_pair(C_TAB_ACTIVE))
            else:
                safe_addstr(stdscr, y, x0, "\u2502",
                            curses.color_pair(C_TAB_ACTIVE))
                safe_addstr(stdscr, y, x0 + box_w - 1, "\u2502",
                            curses.color_pair(C_TAB_ACTIVE))

        safe_addstr(stdscr, y0 + 1, x0 + 2, "Filter",
                    curses.color_pair(C_TAB_ACTIVE) | curses.A_BOLD)

        # Render each row at y0 + 2 + i
        label_col = 12
        for i, (kind, label) in enumerate(rows):
            y = y0 + 2 + i
            if y >= y0 + box_h - 1:
                break
            is_active_row = (i == row_idx)
            attr_base = curses.color_pair(C_TAB_ACTIVE)
            safe_addstr(stdscr, y, x0 + 2, label, attr_base | curses.A_DIM)
            cx = x0 + 2 + label_col

            if kind == "status_chips":
                _render_chips(stdscr, y, cx, _FILTER_STATUS_CHOICES,
                              statuses, is_active_row, chip_idx)
            elif kind == "type_chips":
                _render_chips(stdscr, y, cx, _FILTER_TYPE_CHOICES,
                              types, is_active_row, chip_idx)
            elif kind == "pri_chips":
                _render_chips(stdscr, y, cx,
                              [f"p{p}" for p in _FILTER_PRI_CHOICES],
                              {f"p{p}" for p in priorities},
                              is_active_row, chip_idx)
            elif kind == "deps_chips":
                selected = set()
                if ready:
                    selected.add("ready only")
                if tangled:
                    selected.add("tangled only")
                _render_chips(stdscr, y, cx, _FILTER_DEPS_CHOICES,
                              selected, is_active_row, chip_idx)
            elif kind == "labels_text":
                _render_text(stdscr, y, cx, labels_buf, labels_pos,
                             is_active_row, box_w - (cx - x0) - 2,
                             "  (any-of, comma-separated)")
            elif kind == "search_text":
                _render_text(stdscr, y, cx, search_buf, search_pos,
                             is_active_row, box_w - (cx - x0) - 2,
                             "  (substring)")
            elif kind == "parent_text":
                _render_text(stdscr, y, cx, parent_buf, parent_pos,
                             is_active_row, box_w - (cx - x0) - 2,
                             "  (descendants of)")
            elif kind == "buttons":
                _render_buttons(stdscr, y, cx,
                                ["Apply", "Clear all", "Cancel"],
                                button_idx if is_active_row else -1)

        footer = "↑/↓ row  ←/→ chip  Space toggle  Enter apply  Esc cancel"
        safe_addstr(stdscr, y0 + box_h - 2, x0 + 2, footer[:box_w - 4],
                    curses.color_pair(C_TAB_ACTIVE) | curses.A_DIM)
        stdscr.refresh()

        ch = stdscr.getch()
        if ch == -1:
            continue
        if ch == 27:
            return None

        kind = rows[row_idx][0]

        # Global Enter = Apply, unless on Cancel/Clear button row.
        if ch in (ord("\n"), curses.KEY_ENTER, 10, 13):
            if kind == "buttons":
                if button_idx == 0:
                    return build_spec()
                if button_idx == 1:
                    statuses.clear()
                    types.clear()
                    priorities.clear()
                    labels_buf = ""
                    labels_pos = 0
                    search_buf = ""
                    search_pos = 0
                    ready = False
                    tangled = False
                    parent_buf = ""
                    parent_pos = 0
                    continue
                return None
            return build_spec()

        # Row navigation.
        if ch in (curses.KEY_DOWN, ord("\t")):
            row_idx = (row_idx + 1) % len(rows)
            chip_idx = 0
            continue
        if ch in (curses.KEY_UP, curses.KEY_BTAB):
            row_idx = (row_idx - 1) % len(rows)
            chip_idx = 0
            continue

        if kind.endswith("_chips"):
            choices = {
                "status_chips": _FILTER_STATUS_CHOICES,
                "type_chips": _FILTER_TYPE_CHOICES,
                "pri_chips": [f"p{p}" for p in _FILTER_PRI_CHOICES],
                "deps_chips": _FILTER_DEPS_CHOICES,
            }[kind]
            if ch == curses.KEY_LEFT:
                chip_idx = (chip_idx - 1) % len(choices)
            elif ch == curses.KEY_RIGHT:
                chip_idx = (chip_idx + 1) % len(choices)
            elif ch == ord(" "):
                val = choices[chip_idx]
                if kind == "status_chips":
                    statuses.symmetric_difference_update({val})
                elif kind == "type_chips":
                    types.symmetric_difference_update({val})
                elif kind == "pri_chips":
                    p = int(val[1:])
                    priorities.symmetric_difference_update({p})
                elif kind == "deps_chips":
                    if val == "ready only":
                        ready = not ready
                    else:
                        tangled = not tangled
        elif kind == "labels_text":
            labels_buf, labels_pos = _text_edit(labels_buf, labels_pos, ch)
        elif kind == "search_text":
            search_buf, search_pos = _text_edit(search_buf, search_pos, ch)
        elif kind == "parent_text":
            parent_buf, parent_pos = _text_edit(parent_buf, parent_pos, ch)
        elif kind == "buttons":
            if ch == curses.KEY_LEFT:
                button_idx = (button_idx - 1) % 3
            elif ch == curses.KEY_RIGHT:
                button_idx = (button_idx + 1) % 3


def _render_chips(stdscr, y, x, choices, selected, active_row, active_idx):
    cx = x
    for i, c in enumerate(choices):
        mark = "x" if c in selected else " "
        chip = f"[{mark}] {c}  "
        attr = curses.color_pair(C_TAB_ACTIVE)
        if active_row and i == active_idx:
            attr = curses.color_pair(C_SELECTED) | curses.A_BOLD
        elif c in selected:
            attr |= curses.A_BOLD
        safe_addstr(stdscr, y, cx, chip, attr)
        cx += len(chip)


def _render_text(stdscr, y, x, buf, pos, active, maxw, hint=""):
    vis = buf
    if len(vis) > maxw - 2:
        vis = vis[-(maxw - 2):]
    attr = curses.color_pair(C_TAB_ACTIVE)
    if active:
        attr = curses.color_pair(C_SELECTED) | curses.A_BOLD
    # Pad to show the field boundary even when empty.
    field = vis if vis else (" " * 20)
    safe_addstr(stdscr, y, x, field, attr)
    if not active and not buf and hint:
        safe_addstr(stdscr, y, x + len(field) + 1, hint,
                    curses.color_pair(C_TAB_ACTIVE) | curses.A_DIM)


def _render_buttons(stdscr, y, x, labels, active_idx):
    cx = x
    for i, lab in enumerate(labels):
        text = f"[ {lab} ]  "
        attr = curses.color_pair(C_TAB_ACTIVE)
        if i == active_idx:
            attr = curses.color_pair(C_SELECTED) | curses.A_BOLD
        safe_addstr(stdscr, y, cx, text, attr)
        cx += len(text)


def _text_edit(buf: str, pos: int, ch: int) -> tuple[str, int]:
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
    elif ch == 21:  # Ctrl-U
        buf = buf[pos:]
        pos = 0
    elif 32 <= ch < 127:
        buf = buf[:pos] + chr(ch) + buf[pos:]
        pos += 1
    return buf, pos
