"""Curses rendering for the yaks TUI — list, detail, tabs, help popup.

Every function takes the App instance; all are pure readers of App state.
"""

from __future__ import annotations

import curses

from yaklib.format import status_emoji
from yaklib.model import HAIRY, SHAVING, SHORN, DEAD
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
    """Filtered counts per tab status. When a filter is active, each tab
    shows how many tasks would appear under it given the spec (ignoring
    the spec's own `statuses` override)."""
    from dataclasses import replace
    from yaktui.tree import build_tree
    spec = app.filter_spec
    if spec.is_empty():
        counts = {}
        for status, _ in TABS:
            d = app.root / status
            counts[status] = len(list(d.glob("*.md"))) if d.exists() else 0
        return counts
    # Clear statuses in spec so each tab can scope independently.
    per_tab_spec = replace(spec, statuses=frozenset())
    counts = {}
    for status, _ in TABS:
        # If the spec explicitly narrows statuses and this tab isn't in scope,
        # the tab will show nothing — count=0 is the honest number.
        if spec.statuses and status not in spec.statuses:
            counts[status] = 0
            continue
        counts[status] = sum(
            1 for _, _, _, ghost in build_tree(
                app.root, status, per_tab_spec,
                tasks_cache=app._task_cache,
                resolved_cache=app._resolved_cache)
            if not ghost
        )
    return counts


def draw(app):
    app.stdscr.erase()
    h, w = app.stdscr.getmaxyx()
    if h < 5 or w < 40:
        app.stdscr.addstr(0, 0, "Terminal too small")
        app.stdscr.refresh()
        return

    draw_tabs(app, 0, w)

    # Drawer occupies lines between tab row and the list.
    drawer_h = 0
    if app._drawer is not None:
        drawer_h = app._drawer.height
        draw_filter_drawer(app, 1, w)

    list_y = 1 + drawer_h + 1  # tab + drawer + separator/gap

    if app.focus == "detail":
        detail_w = max(w * 2 // 3, 40)
        list_w = w - detail_w - 1
        detail_x = list_w + 1
        list_h = max(1, h - list_y - 1)

        if app._detail_build_width != detail_w:
            app._rebuild_detail(detail_w)

        draw_list(app, list_y, 0, list_h, list_w)
        draw_separator(app, list_y - 1, list_w, list_h + 1)
        draw_detail(app, list_y - 1, detail_x, list_h + 1, detail_w)
    else:
        list_h = max(1, h - list_y - 1)
        draw_list(app, list_y, 0, list_h, w)

    draw_help_bar(app, h - 1, w)

    # Cursor positioning for inline search / drawer text fields.
    if app._inline_search is not None:
        _position_inline_search_cursor(app, 0, w)
    elif app._drawer is not None:
        _position_drawer_cursor(app, 1, w)
    else:
        curses.curs_set(0)

    if app.show_help:
        draw_help_popup(app, h, w)

    app.stdscr.refresh()


def draw_tabs(app, y, w):
    x = 0
    counts = tab_counts(app)
    spec_statuses = app.filter_spec.statuses
    for i, (status, label) in enumerate(TABS):
        # When the filter explicitly overrides statuses, mark which tabs
        # are still in scope so the tab row honestly reflects what'll show.
        marker = "*" if (spec_statuses and status in spec_statuses) else ""
        text = f" {label}{marker} ({counts[status]}) "
        if i == app.tab:
            attr = curses.color_pair(C_TAB_ACTIVE) | curses.A_BOLD
        else:
            attr = curses.A_DIM
        try:
            app.stdscr.addstr(y, x, text, attr)
        except curses.error:
            pass
        x += len(text) + 1

    if app._inline_search is not None:
        buf, pos = app._inline_search
        prompt = "  search: "
        safe_addstr(app.stdscr, y, x, prompt,
                    curses.color_pair(C_SEARCH) | curses.A_BOLD)
        safe_addstr(app.stdscr, y, x + len(prompt), buf,
                    curses.color_pair(C_SEARCH))
    else:
        chip = app.filter_spec.summary()
        if chip:
            # Total across in-scope statuses (or the current tab if
            # statuses aren't overridden). Reuses the cached counts.
            spec_statuses = app.filter_spec.statuses
            if spec_statuses:
                total = sum(counts[s] for s, _ in TABS if s in spec_statuses)
            else:
                total = counts[TABS[app.tab][0]]
            text = f"  filter: {chip}  ({total})"
            safe_addstr(app.stdscr, y, x, text,
                        curses.color_pair(C_SEARCH))

    if app.notification:
        nx = max(x + 2, w - len(app.notification) - 2)
        safe_addstr(app.stdscr, y, nx, app.notification,
                    curses.color_pair(C_SEARCH) | curses.A_BOLD)


# ---------------------------------------------------------------------------
# Filter drawer
# ---------------------------------------------------------------------------

_DRAWER_LABEL_COL = 12

_DRAWER_CHIP_MAP = {
    "status_chips": [HAIRY, SHAVING, SHORN, DEAD],
    "type_chips": ["task", "bug", "feature", "idea"],
    "pri_chips": ["p1", "p2", "p3"],
    "deps_chips": ["ready only", "tangled only"],
}


def _drawer_selected_set(d, kind):
    if kind == "status_chips":
        return d.statuses
    if kind == "type_chips":
        return d.types
    if kind == "pri_chips":
        return {f"p{p}" for p in d.priorities}
    if kind == "deps_chips":
        sel = set()
        if d.ready:
            sel.add("ready only")
        if d.tangled:
            sel.add("tangled only")
        return sel
    return set()


def draw_filter_drawer(app, y0, w):
    """Render the filter drawer starting at row y0."""
    from tui import _DRAWER_ROWS
    d = app._drawer
    if d is None:
        return
    stdscr = app.stdscr
    cx_start = 2 + _DRAWER_LABEL_COL
    content_w = w - cx_start - 2

    for ri, (kind, label) in enumerate(_DRAWER_ROWS):
        y = y0 + ri
        is_active = (ri == d.row)
        label_attr = curses.A_BOLD if is_active else curses.A_DIM
        safe_addstr(stdscr, y, 2, label, label_attr)
        cx = cx_start

        if kind.endswith("_chips"):
            choices = _DRAWER_CHIP_MAP[kind]
            selected = _drawer_selected_set(d, kind)
            for ci, c in enumerate(choices):
                mark = "x" if c in selected else " "
                chip = f"[{mark}] {c}  "
                attr = 0
                if is_active and ci == d.chip_idx:
                    attr = curses.color_pair(C_SELECTED) | curses.A_BOLD
                elif c in selected:
                    attr = curses.A_BOLD
                safe_addstr(stdscr, y, cx, chip, attr)
                cx += len(chip)
        elif kind.endswith("_text"):
            if kind == "labels_text":
                buf, pos = d.labels_buf, d.labels_pos
            elif kind == "search_text":
                buf, pos = d.search_buf, d.search_pos
            else:
                buf, pos = d.parent_buf, d.parent_pos
            field_w = min(content_w, max(20, content_w))
            _draw_text_field(stdscr, y, cx, buf, pos, is_active, field_w)
            # Labels hint
            if is_active and kind == "labels_text":
                avail = getattr(app, "_available_labels", [])
                if avail:
                    hint = "available: " + ", ".join(avail[:12])
                    if len(avail) > 12:
                        hint += f" +{len(avail) - 12}"
                    safe_addstr(stdscr, y, cx + field_w + 2,
                                hint[:w - cx - field_w - 4], curses.A_DIM)

    # Separator line under the drawer.
    sep_y = y0 + len(_DRAWER_ROWS)
    safe_addstr(stdscr, sep_y, 0, "\u2500" * w, curses.A_DIM)
    # Key hints on the separator.
    hints = " Enter:commit  Esc:revert  C:clear "
    hx = w - len(hints) - 1
    if hx > 0:
        safe_addstr(stdscr, sep_y, hx, hints, curses.A_DIM)


def _draw_text_field(stdscr, y, x, buf, pos, active, field_w):
    """Render a text input with bracket boundaries."""
    inner_w = field_w - 2  # inside the [ ]
    safe_addstr(stdscr, y, x, "[", curses.A_DIM)
    safe_addstr(stdscr, y, x + field_w - 1, "]", curses.A_DIM)
    # Scroll offset
    offset = max(0, pos - inner_w + 1)
    vis = buf[offset:offset + inner_w]
    pad = " " * max(0, inner_w - len(vis))
    attr = curses.A_BOLD if active else curses.A_DIM
    safe_addstr(stdscr, y, x + 1, vis + pad, attr)


def _position_drawer_cursor(app, y0, w):
    """Place the terminal cursor on the active text field in the drawer."""
    from tui import _DRAWER_ROWS
    d = app._drawer
    kind = _DRAWER_ROWS[d.row][0]
    if kind.endswith("_text"):
        cx = 2 + _DRAWER_LABEL_COL
        field_w = min(w - cx - 2, max(20, w - cx - 2))
        inner_w = field_w - 2
        if kind == "labels_text":
            pos = d.labels_pos
        elif kind == "search_text":
            pos = d.search_pos
        else:
            pos = d.parent_pos
        offset = max(0, pos - inner_w + 1)
        screen_x = cx + 1 + (pos - offset)
        screen_y = y0 + d.row
        curses.curs_set(1)
        try:
            app.stdscr.move(screen_y, screen_x)
        except curses.error:
            pass
    else:
        curses.curs_set(0)


def _position_inline_search_cursor(app, y, w):
    """Place the terminal cursor on the inline search in the tab row."""
    if app._inline_search is None:
        curses.curs_set(0)
        return
    buf, pos = app._inline_search
    # Find the x of the search area — after the tab texts.
    counts = tab_counts(app)
    x = 0
    for i, (status, label) in enumerate(TABS):
        text = f" {label} ({counts[status]}) "
        x += len(text) + 1
    prompt = "  search: "
    curses.curs_set(1)
    try:
        app.stdscr.move(y, x + len(prompt) + pos)
    except curses.error:
        pass


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
            if app.filter_spec.search:
                display_title = f"{status_emoji(status)} {display_title}"[:remaining]
            safe_addstr(stdscr, y, x, display_title, title_attr)

        if ghost and not app.filter_spec.search:
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
        has_inline_links = bool(dl.links)

        # Whole-line link highlight only applies to sectioned links
        # (task_id / open_path) — not to description lines with inline spans.
        whole_line_link = (dl.task_id is not None or dl.open_path is not None) \
            and not has_inline_links

        if is_cursor:
            fill_attr = (curses.color_pair(C_LINK_SEL) if whole_line_link
                         else curses.color_pair(C_SELECTED))
            safe_addstr(stdscr, y, x_start, " " * width, fill_attr)
            safe_addstr(stdscr, y, x_start, text, fill_attr | curses.A_BOLD)
        elif whole_line_link:
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

        # Overlay inline link spans. Active span (when this line is the
        # detail cursor) gets C_LINK_SEL + BOLD; others get C_LINK underline.
        if has_inline_links:
            active = app.detail_span_cursor if is_cursor else -1
            active = max(0, min(active, len(dl.links) - 1)) if is_cursor else -1
            for si, (start, end, _) in enumerate(dl.links):
                if start >= width:
                    break
                end_clip = min(end, width)
                span_text = text[start:end_clip]
                if si == active:
                    attr = curses.color_pair(C_LINK_SEL) | curses.A_BOLD
                else:
                    attr = curses.color_pair(C_LINK) | curses.A_UNDERLINE
                safe_addstr(stdscr, y, x_start + start, span_text, attr)

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
    filter_active = not app.filter_spec.is_empty()
    filter_hint = "f:filter  Esc:clear" if filter_active else "f:filter  /:search"
    if app.focus == "detail":
        keys = ("h:list  j/k:move  Tab:link  Enter:follow  i/o:fwd/back  "
                f"E:edit  D:dep  S:state  {filter_hint}  q:quit  ?:help")
    else:
        keys = ("Tab:tab  j/k:move  l:detail  c/C:new  E:edit  X:del  "
                f"S:state  D:dep  P/T/L:adjust  {filter_hint}  ?:help")
    safe_addstr(app.stdscr, y, 0, " " * w, curses.color_pair(C_HELP))
    safe_addstr(app.stdscr, y, 0, keys[:w], curses.color_pair(C_HELP))


_HELP_SECTIONS = [
    ("Movement", [
        "j / k / ↓ / ↑         Move cursor",
        "d / u                 Half-page down / up",
        "PgDn / PgUp           Full-page down / up",
        "g / G                 Top / bottom",
        "Esc                   Clear search / back",
    ]),
    ("Search & filter", [
        "f                     Open filter editor",
        "/                     Filter editor focused on search",
        "Esc                   Clear all filters (when active)",
        "n / N                 Next / prev detail match",
        "y                     Copy yak ID to clipboard",
    ]),
    ("Editing", [
        "E                     Edit task in $EDITOR",
        "M                     Add comment / note",
        "A                     Attach artifact",
        "X                     Delete task (confirm)",
        "D                     Add dep (or remove if cursor on one)",
        "P / T / L / S         Change priority / type / labels / state",
        "R                     Reparent (move in the tree)",
    ]),
    ("List pane", [
        "Tab / Shift-Tab       Next / previous status tab",
        "[ / ]                 Previous / next tab",
        "l / → / Enter         Show detail pane",
        "c / C                 New root / child task",
    ]),
    ("Detail pane", [
        "h / ←                 Hide detail pane",
        "Tab / Shift-Tab       Next / previous link",
        "[ / ]                 Previous / next link",
        "Enter                 Follow link / open artifact",
        "O                     Open artifact externally",
        "J / K                 Next / prev task in list",
        "i / o                 Nav forward / back in jumplist",
        "Backspace             Nav back (alt)",
    ]),
    ("General", [
        "F / Ctrl-L            Refresh",
        "?                     Toggle this help",
        "q                     Quit",
    ]),
]


_PANE_SECTIONS = {"List pane", "Detail pane"}


def draw_help_popup(app, h, w):
    # Show every section except the pane the user isn't currently in. The
    # always-shown sections (Movement, Search, Editing, General) apply in
    # both contexts, so they stay regardless.
    keep_pane = "List pane" if app.focus == "list" else "Detail pane"
    sections = [(name, lines) for name, lines in _HELP_SECTIONS
                if name not in _PANE_SECTIONS or name == keep_pane]
    title = f"Yaks TUI — {keep_pane} shortcuts"
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
