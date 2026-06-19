"""Key dispatch for the detail (right) pane of the TUI."""

from __future__ import annotations

import curses

from yaktui import dialogs as _dialogs
from yaktui import mutate as _mutate


def handle(app, key) -> bool:
    # Escape: clear visual selection first, then search, then back to list.
    if key == 27:
        if app.detail_select_anchor is not None:
            app.detail_select_anchor = None
        elif app.detail_search:
            app.detail_search = ""
            app.detail_matches = []
        else:
            app.focus = "list"
        return True

    # Back to list
    if key in (ord("h"), curses.KEY_LEFT):
        app.detail_select_anchor = None
        app.focus = "list"
        return True

    # Shift-arrows: enter visual mode (if not already) and extend.
    if key in (curses.KEY_SR, curses.KEY_SF):
        if app.detail_select_anchor is None:
            app.detail_select_anchor = app.detail_line_cursor
        if key == curses.KEY_SF and app.detail_line_cursor < len(app.detail_lines) - 1:
            app.detail_line_cursor += 1
        elif key == curses.KEY_SR and app.detail_line_cursor > 0:
            app.detail_line_cursor -= 1
        app._fix_detail_scroll()
        return True

    # v toggles visual mode (vim-style).
    if key == ord("v"):
        if app.detail_select_anchor is None:
            app.detail_select_anchor = app.detail_line_cursor
        else:
            app.detail_select_anchor = None
        return True

    # Line cursor movement
    if key in (ord("j"), curses.KEY_DOWN):
        if app.detail_line_cursor < len(app.detail_lines) - 1:
            app.detail_line_cursor += 1
            app._fix_detail_scroll()
    elif key in (ord("k"), curses.KEY_UP):
        if app.detail_line_cursor > 0:
            app.detail_line_cursor -= 1
            app._fix_detail_scroll()
    elif key == ord("g"):
        app.detail_line_cursor = 0
        app._fix_detail_scroll()
    elif key == ord("G"):
        app.detail_line_cursor = max(0, len(app.detail_lines) - 1)
        app._fix_detail_scroll()

    # Cycle between link lines
    elif key in (ord("\t"), ord("]")):
        app._jump_link(+1)
    elif key in (curses.KEY_BTAB, ord("[")):
        app._jump_link(-1)

    # Jumplist forward/back ('i'/'o' vim, Backspace is browser-style back)
    elif key == ord("i"):
        app._nav_forward()
    elif key in (ord("o"), curses.KEY_BACKSPACE, 127, 8):
        app._nav_back()

    # Page scrolling. 'd'/'u' documented; Ctrl-D/Ctrl-U vim aliases.
    elif key in (curses.KEY_NPAGE, ord("d"), 4):
        app._detail_page(+1, half=(key in (ord("d"), 4)))
    elif key in (curses.KEY_PPAGE, ord("u"), 21):
        app._detail_page(-1, half=(key in (ord("u"), 21)))
    # Viewport scroll (Ctrl-E / Ctrl-Y)
    elif key == 5:  # Ctrl-E
        app._scroll_detail_viewport(+1)
    elif key == 25:  # Ctrl-Y
        app._scroll_detail_viewport(-1)

    # Find next / prev match
    elif key == ord("n"):
        if app.detail_matches:
            app._jump_match(+1)
    elif key == ord("N"):
        if app.detail_matches:
            app._jump_match(-1)

    # Follow link on current line (or yank selection if visual mode active).
    elif key in (ord("\n"), curses.KEY_ENTER):
        if app.detail_select_anchor is not None:
            _yank_selection(app)
        else:
            app._follow_link()

    # Open artifact externally (also available via Enter)
    elif key == ord("O"):
        if 0 <= app.detail_line_cursor < len(app.detail_lines):
            dl = app.detail_lines[app.detail_line_cursor]
            if dl.open_path:
                app._open_externally(dl.open_path)

    # Edit / delete the displayed task
    elif key == ord("E"):
        tid = app._current_task_id()
        if tid:
            app._edit_task(tid)
    elif key == ord("X"):
        tid = app._current_task_id()
        if tid:
            app._delete_task(tid)

    # Next / prev task in list
    elif key == ord("J"):
        app._detail_next_task(+1)
    elif key == ord("K"):
        app._detail_next_task(-1)

    # Clipboard / comment / attach
    elif key == ord("y"):
        if app.detail_select_anchor is not None:
            _yank_selection(app)
        else:
            # If cursor is on a link line, copy the link target; else task ID.
            target = None
            if 0 <= app.detail_line_cursor < len(app.detail_lines):
                dl = app.detail_lines[app.detail_line_cursor]
                if dl.links:
                    idx = max(0, min(app.detail_span_cursor, len(dl.links) - 1))
                    target = dl.links[idx][2]
                elif dl.task_id:
                    target = dl.task_id
                elif dl.open_path:
                    target = str(dl.open_path)
            if target is None:
                target = app._current_task_id()
            if target:
                app._copy_to_clipboard(target)
    elif key == ord("M"):
        tid = app._current_task_id()
        if tid:
            app._add_comment(tid)
    elif key == ord("A"):
        tid = app._current_task_id()
        if tid:
            app._attach_file(tid)

    # Reparent (move in the tree)
    elif key == ord("R"):
        tid = app._current_task_id()
        if tid:
            app._reparent_task(tid)

    # Dependencies: D adds or (if cursor is on a Depends-on row) removes.
    elif key == ord("D"):
        _mutate.handle_dep_key(app)

    # Create — type is selected inside the form itself
    elif key == ord("c"):
        app._create_task(parent=None)
    elif key == ord("C"):
        parent = app._current_task_id()
        if parent:
            app._create_task(parent=parent)

    # Quick adjusts mirrored from the list pane.
    elif key == ord("P"):
        tid = app._current_task_id()
        if tid:
            app._quick_adjust_priority(tid)
    elif key == ord("T"):
        tid = app._current_task_id()
        if tid:
            app._quick_adjust_type(tid)
    elif key == ord("L"):
        tid = app._current_task_id()
        if tid:
            app._quick_adjust_labels(tid)
    elif key == ord("S"):
        tid = app._current_task_id()
        if tid:
            app._quick_adjust_state(tid)

    # Filter drawer (list scope) — same as list pane.
    elif key == ord("f"):
        app._open_filter_drawer()

    # Detail search
    elif key == ord("/"):
        query = _dialogs.input_prompt(app.stdscr, "Detail search: ", vim=app.vim_mode)
        if query:
            app.detail_search = query
            app._apply_detail_search()
            if app.detail_matches:
                app.detail_line_cursor = app.detail_matches[0]
                app._fix_detail_scroll()

    return True


def dedent_block(lines: list[str]) -> list[str]:
    """Strip the common leading-space prefix across non-empty lines."""
    non_empty = [s for s in lines if s.strip()]
    if not non_empty:
        return lines
    strip = min(len(s) - len(s.lstrip(" ")) for s in non_empty)
    if not strip:
        return lines
    return [s[strip:] if len(s) >= strip else s for s in lines]


def _yank_selection(app) -> None:
    """Copy the selected range of detail lines to the clipboard."""
    anchor = app.detail_select_anchor
    cursor = app.detail_line_cursor
    if anchor is None or not app.detail_lines:
        return
    lo = max(0, min(anchor, cursor))
    hi = min(len(app.detail_lines) - 1, max(anchor, cursor))
    raw = [app.detail_lines[i].text for i in range(lo, hi + 1)]
    app._copy_to_clipboard("\n".join(dedent_block(raw)))
    app.detail_select_anchor = None
