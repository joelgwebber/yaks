"""Key dispatch for the detail (right) pane of the TUI."""

from __future__ import annotations

import curses

from yaktui import dialogs as _dialogs


def handle(app, key) -> bool:
    # Escape: clear search first, else back to list
    if key == 27:
        if app.detail_search:
            app.detail_search = ""
            app.detail_matches = []
        else:
            app.focus = "list"
        return True

    # Back to list
    if key in (ord("h"), curses.KEY_LEFT):
        app.focus = "list"
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

    # Find next / prev match
    elif key == ord("n"):
        if app.detail_matches:
            app._jump_match(+1)
    elif key == ord("N"):
        if app.detail_matches:
            app._jump_match(-1)

    # Follow link on current line
    elif key in (ord("\n"), curses.KEY_ENTER):
        app._follow_link()

    # Open artifact externally (also available via Enter)
    elif key == ord("O"):
        if 0 <= app.detail_line_cursor < len(app.detail_lines):
            dl = app.detail_lines[app.detail_line_cursor]
            if dl.open_path:
                app._open_externally(dl.open_path)

    # Edit / delete the displayed task
    elif key == ord("e"):
        tid = app._current_task_id()
        if tid:
            app._edit_task(tid)
    elif key == ord("D"):
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
        tid = app._current_task_id()
        if tid:
            app._copy_to_clipboard(tid)
    elif key == ord("m"):
        tid = app._current_task_id()
        if tid:
            app._add_comment(tid)
    elif key == ord("A"):
        tid = app._current_task_id()
        if tid:
            app._attach_file(tid)

    # Reparent (move in the tree)
    elif key == ord("M"):
        tid = app._current_task_id()
        if tid:
            app._reparent_task(tid)

    # Detail search
    elif key == ord("/"):
        query = _dialogs.input_prompt(app.stdscr, "Detail search: ")
        if query:
            app.detail_search = query
            app._apply_detail_search()
            if app.detail_matches:
                app.detail_line_cursor = app.detail_matches[0]
                app._fix_detail_scroll()

    return True
