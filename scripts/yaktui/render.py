"""Curses rendering for the yaks TUI — list, detail, tabs, help popup.

Every function takes the App instance; all are pure readers of App state.
"""

from __future__ import annotations

import curses

from yaklib.format import status_emoji
from yaklib.model import HAIRY, SHAVING, SHORN
from yaktui.colors import (
    C_CODE,
    C_HEADER,
    C_HELP,
    C_ID,
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
    ghost_badge_attr,
)
from yaktui.dialogs import safe_addstr

TABS = [
    (HAIRY, "\U0001f9ac Hairy"),
    (SHAVING, "\u2702\ufe0f  Shaving"),
    (SHORN, "\U0001f411 Shorn"),
]


def tab_counts(app):
    counts = {}
    for status, _ in TABS:
        d = app.root / status
        counts[status] = len(list(d.glob("*.md"))) if d.exists() else 0
    return counts


def draw(app):
    app.stdscr.erase()
    h, w = app.stdscr.getmaxyx()
    if h < 5 or w < 40:
        app.stdscr.addstr(0, 0, "Terminal too small")
        app.stdscr.refresh()
        return

    draw_tabs(app, 0, w)

    if app.focus == "detail":
        detail_w = max(w * 2 // 3, 40)
        list_w = w - detail_w - 1
        detail_x = list_w + 1

        if app._detail_build_width != detail_w:
            app._rebuild_detail(detail_w)

        draw_list(app, 2, 0, h - 3, list_w)
        draw_separator(app, 1, list_w, h - 2)
        draw_detail(app, 1, detail_x, h - 2, detail_w)
    else:
        draw_list(app, 2, 0, h - 3, w)

    draw_help_bar(app, h - 1, w)

    if app.show_help:
        draw_help_popup(app, h, w)

    app.stdscr.refresh()


def draw_tabs(app, y, w):
    x = 0
    counts = tab_counts(app)
    for i, (status, label) in enumerate(TABS):
        text = f" {label} ({counts[status]}) "
        attr = (curses.color_pair(C_TAB_ACTIVE) | curses.A_BOLD
                if i == app.tab else curses.A_DIM)
        try:
            app.stdscr.addstr(y, x, text, attr)
        except curses.error:
            pass
        x += len(text) + 1

    if app.search_query:
        safe_addstr(app.stdscr, y, x, f'  search: "{app.search_query}"',
                    curses.color_pair(C_SEARCH))
    elif app.filter_mode != "all":
        safe_addstr(app.stdscr, y, x, f"  {app.filter_mode}",
                    curses.color_pair(C_SEARCH))

    if app.notification:
        nx = max(x + 2, w - len(app.notification) - 2)
        safe_addstr(app.stdscr, y, nx, app.notification,
                    curses.color_pair(C_SEARCH) | curses.A_BOLD)


def draw_list(app, y_start, x_start, height, width):
    stdscr = app.stdscr
    if not app.tasks:
        safe_addstr(stdscr, y_start + 1, x_start + 2, "No tasks.", curses.A_DIM)
        return

    max_id_len = 4
    for _, t, depth, _ in app.tasks:
        id_len = len(t["id"]) + depth * 2
        max_id_len = max(max_id_len, id_len)
    id_col = max_id_len + 1

    for i in range(height):
        idx = app.scroll + i
        if idx >= len(app.tasks):
            break
        status, task, depth, ghost = app.tasks[idx]
        y = y_start + i
        is_selected = idx == app.cursor

        if is_selected:
            safe_addstr(stdscr, y, x_start, " " * width,
                        curses.color_pair(C_SELECTED))

        indent = "  " * depth
        tid = task["id"]
        pri = task.get("priority", "-")
        ttype = task.get("type", "-")
        title = task.get("title", "")

        x = x_start
        ghost_attr = curses.A_DIM if ghost else 0
        base_attr = curses.color_pair(C_SELECTED) if is_selected else 0

        blocked = tid in app.blocked_ids and status == HAIRY
        lead = "*" if blocked else " "
        id_text = f"{lead}{indent}{tid}".ljust(id_col + 1)
        id_attr = base_attr if is_selected else (curses.color_pair(C_ID) | ghost_attr)
        if blocked and not is_selected:
            block_attr = curses.color_pair(C_P2) | curses.A_BOLD
            safe_addstr(stdscr, y, x, lead, block_attr)
            safe_addstr(stdscr, y, x + 1, id_text[1:], id_attr)
        else:
            safe_addstr(stdscr, y, x, id_text, id_attr)
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
        safe_addstr(stdscr, y, x, pri_text, pri_attr)
        x += len(pri_text)

        type_text = f"{ttype:8s} "
        type_attr = base_attr if is_selected else (curses.color_pair(C_TYPE) | ghost_attr)
        safe_addstr(stdscr, y, x, type_text, type_attr)
        x += len(type_text)

        remaining = width - (x - x_start) - 1
        if remaining > 0:
            display_title = title[:remaining]
            title_attr = base_attr | ghost_attr
            if app.search_query:
                display_title = f"{status_emoji(status)} {display_title}"[:remaining]
            safe_addstr(stdscr, y, x, display_title, title_attr)

        if ghost and not app.search_query:
            badge = f" {status_emoji(status)}"
            bx = x_start + width - len(badge) - 1
            if bx > x:
                badge_attr = ghost_badge_attr(status) | base_attr
                safe_addstr(stdscr, y, bx, badge, badge_attr)


def draw_separator(app, y_start, x, height):
    for y in range(y_start, y_start + height):
        try:
            app.stdscr.addch(y, x, curses.ACS_VLINE, curses.A_DIM)
        except curses.error:
            pass


def draw_detail(app, y_start, x_start, height, width):
    stdscr = app.stdscr
    if not app.detail_lines:
        return

    detail_focused = app.focus == "detail"

    if app.detail_search:
        match_info = f"  /{app.detail_search}  ({len(app.detail_matches)} matches)"
        safe_addstr(stdscr, y_start, x_start, match_info[:width],
                    curses.color_pair(C_SEARCH))
        y_start += 1
        height -= 1

    for i in range(height):
        line_idx = app.detail_scroll + i
        if line_idx >= len(app.detail_lines):
            break
        dl = app.detail_lines[line_idx]
        y = y_start + i
        text = dl.text[:width]

        is_cursor = detail_focused and line_idx == app.detail_line_cursor
        is_match = line_idx in app.detail_matches

        if is_cursor:
            fill_attr = (curses.color_pair(C_LINK_SEL) if dl.is_link
                         else curses.color_pair(C_SELECTED))
            safe_addstr(stdscr, y, x_start, " " * width, fill_attr)
            safe_addstr(stdscr, y, x_start, text, fill_attr | curses.A_BOLD)
        elif dl.is_link:
            safe_addstr(stdscr, y, x_start, text, curses.color_pair(C_LINK))
        elif dl.kind == "header":
            safe_addstr(stdscr, y, x_start, text,
                        curses.color_pair(C_HEADER) | curses.A_BOLD)
        elif dl.kind == "subheader":
            safe_addstr(stdscr, y, x_start, text,
                        curses.color_pair(C_HEADER) | curses.A_BOLD)
        elif dl.kind == "desc":
            safe_addstr(stdscr, y, x_start, text, curses.A_DIM)
        elif dl.kind == "code":
            safe_addstr(stdscr, y, x_start, text, curses.color_pair(C_CODE))
        elif dl.kind == "md_heading":
            safe_addstr(stdscr, y, x_start, text,
                        curses.color_pair(C_MD_HEADING) | curses.A_BOLD)
        elif dl.kind == "quote":
            safe_addstr(stdscr, y, x_start, text, curses.A_DIM | curses.A_ITALIC)
        else:
            safe_addstr(stdscr, y, x_start, text, 0)

        if is_match and not is_cursor:
            highlight_matches(app, y, x_start, text, width)


def highlight_matches(app, y, x_start, text, width):
    if not app.detail_search:
        return
    q = app.detail_search.lower()
    tl = text.lower()
    pos = 0
    while True:
        idx = tl.find(q, pos)
        if idx < 0 or idx >= width:
            break
        end = min(idx + len(q), width)
        safe_addstr(app.stdscr, y, x_start + idx, text[idx:end],
                    curses.color_pair(C_MATCH) | curses.A_BOLD)
        pos = idx + 1


def draw_help_bar(app, y, w):
    if app.message:
        safe_addstr(app.stdscr, y, 0, app.message[:w], curses.color_pair(C_SEARCH))
        app.message = ""
        return
    if app.focus == "detail":
        keys = ("h:list  j/k:move  Tab:next link  Enter:follow  i/o:fwd/back  "
                "/:search  Esc:clear  q:quit")
    else:
        keys = ("Tab:tab  j/k:move  l:detail  c/C:new  e:edit  D:del  "
                "s/x/r:shave/shorn/regrow  n/t/a:filter  /:search  ?:help")
    safe_addstr(app.stdscr, y, 0, " " * w, curses.color_pair(C_HELP))
    safe_addstr(app.stdscr, y, 0, keys[:w], curses.color_pair(C_HELP))


_HELP_SECTIONS = [
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


def draw_help_popup(app, h, w):
    sections = _HELP_SECTIONS
    title = "Yaks TUI - Keyboard shortcuts"
    footer = "Press any key to close"

    vertical_h = 4 + sum(len(lines) + 2 for _, lines in sections)
    max_line = max(max(len(l) for l in lines) for _, lines in sections)

    if h >= vertical_h + 2:
        _draw_help_vertical(app, h, w, title, footer, sections, max_line)
    else:
        _draw_help_horizontal(app, h, w, title, footer, sections, max_line)


def _draw_help_vertical(app, h, w, title, footer, sections, max_line):
    lines = [title, ""]
    for name, entries in sections:
        lines.append(name)
        for e in entries:
            lines.append("  " + e)
        lines.append("")
    lines.append(footer)
    box_w = max(len(l) for l in lines) + 4
    box_h = len(lines) + 2
    _render_popup(app, h, w, box_w, box_h, lines,
                  bold_lines={0} | {
                      i for i, l in enumerate(lines)
                      if l in {name for name, _ in sections}
                  })


def _draw_help_horizontal(app, h, w, title, footer, sections, max_line):
    stdscr = app.stdscr
    col_w = max_line + 4
    col_lines_count = max(len(entries) + 1 for _, entries in sections)
    total_w = col_w * len(sections) + 2

    box_h = col_lines_count + 4
    box_w = min(total_w, w - 2)

    start_y = max(0, (h - box_h) // 2)
    start_x = max(0, (w - box_w) // 2)

    for i in range(box_h):
        y = start_y + i
        if y >= h:
            break
        safe_addstr(stdscr, y, start_x, " " * box_w,
                    curses.color_pair(C_TAB_ACTIVE))
        if i == 0 or i == box_h - 1:
            border = "\u2500" * (box_w - 2)
            corner = "\u250c" if i == 0 else "\u2514"
            end = "\u2510" if i == 0 else "\u2518"
            safe_addstr(stdscr, y, start_x, corner + border + end,
                        curses.color_pair(C_TAB_ACTIVE))
        else:
            safe_addstr(stdscr, y, start_x, "\u2502",
                        curses.color_pair(C_TAB_ACTIVE))
            safe_addstr(stdscr, y, start_x + box_w - 1, "\u2502",
                        curses.color_pair(C_TAB_ACTIVE))

    safe_addstr(stdscr, start_y + 1, start_x + 2, title[:box_w - 4],
                curses.color_pair(C_TAB_ACTIVE) | curses.A_BOLD)

    for si, (name, entries) in enumerate(sections):
        col_x = start_x + 2 + si * col_w
        if col_x + col_w > start_x + box_w - 1:
            break
        safe_addstr(stdscr, start_y + 3, col_x, name,
                    curses.color_pair(C_TAB_ACTIVE) | curses.A_BOLD)
        for ei, entry in enumerate(entries):
            safe_addstr(stdscr, start_y + 4 + ei, col_x, entry[:col_w - 2],
                        curses.color_pair(C_TAB_ACTIVE))

    safe_addstr(stdscr, start_y + box_h - 2, start_x + 2, footer[:box_w - 4],
                curses.color_pair(C_TAB_ACTIVE) | curses.A_DIM)


def _render_popup(app, h, w, box_w, box_h, lines, bold_lines=None):
    stdscr = app.stdscr
    bold_lines = bold_lines or set()
    start_y = max(0, (h - box_h) // 2)
    start_x = max(0, (w - box_w) // 2)

    for i in range(box_h):
        y = start_y + i
        if y >= h:
            break
        safe_addstr(stdscr, y, start_x, " " * box_w,
                    curses.color_pair(C_TAB_ACTIVE))
        if i == 0 or i == box_h - 1:
            border = "\u2500" * (box_w - 2)
            corner = "\u250c" if i == 0 else "\u2514"
            end = "\u2510" if i == 0 else "\u2518"
            safe_addstr(stdscr, y, start_x, corner + border + end,
                        curses.color_pair(C_TAB_ACTIVE))
        elif i - 1 < len(lines):
            line = lines[i - 1]
            attr = curses.color_pair(C_TAB_ACTIVE)
            if (i - 1) in bold_lines:
                attr |= curses.A_BOLD
            safe_addstr(stdscr, y, start_x, "\u2502",
                        curses.color_pair(C_TAB_ACTIVE))
            safe_addstr(stdscr, y, start_x + 2, line[:box_w - 4], attr)
            safe_addstr(stdscr, y, start_x + box_w - 1, "\u2502",
                        curses.color_pair(C_TAB_ACTIVE))
