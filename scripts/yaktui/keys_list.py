"""Key dispatch for the list (left) pane of the TUI."""

from __future__ import annotations

import curses

from yaktui import dialogs as _dialogs


def handle(app, key) -> bool:
    """Dispatch a keypress while the list pane has focus. Returns True
    to keep running, False to exit (currently only 'q' does that, and
    is handled upstream in App.handle_key)."""

    # Navigation
    if key in (ord("j"), curses.KEY_DOWN):
        if app.cursor < len(app.tasks) - 1:
            app.cursor += 1
            app._fix_scroll()
            app._rebuild_detail()
    elif key in (ord("k"), curses.KEY_UP):
        if app.cursor > 0:
            app.cursor -= 1
            app._fix_scroll()
            app._rebuild_detail()
    elif key == ord("g"):
        app.cursor = 0
        app._fix_scroll()
        app._rebuild_detail()
    elif key == ord("G"):
        app.cursor = max(0, len(app.tasks) - 1)
        app._fix_scroll()
        app._rebuild_detail()

    # Focus detail
    elif key in (ord("l"), curses.KEY_RIGHT, ord("\n"), curses.KEY_ENTER):
        if app.detail_lines:
            app._enter_detail()

    # Tab switching
    elif key == ord("\t") or key == ord("]"):
        app._switch_tab(1)
    elif key == curses.KEY_BTAB or key == ord("["):
        app._switch_tab(-1)

    # Page scrolling. 'd'/'u' documented; Ctrl-D/Ctrl-U vim aliases.
    elif key in (curses.KEY_NPAGE, ord("d"), 4):
        app._list_page(+1, half=(key in (ord("d"), 4)))
    elif key in (curses.KEY_PPAGE, ord("u"), 21):
        app._list_page(-1, half=(key in (ord("u"), 21)))

    # Filters
    elif key == ord("n"):
        if app.tab == 0:
            app.filter_mode = "next" if app.filter_mode != "next" else "all"
            app._reset_list()
    elif key == ord("t"):
        if app.tab == 0:
            app.filter_mode = "tangled" if app.filter_mode != "tangled" else "all"
            app._reset_list()
    elif key == ord("a"):
        app.filter_mode = "all"
        app.search_query = ""
        app._reset_list()

    # Search
    elif key == ord("/"):
        query = _dialogs.input_prompt(app.stdscr, "Search: ")
        if query:
            app.search_query = query
            app.filter_mode = "all"
            app._reset_list()
    elif key == 27:  # Escape
        if app.search_query:
            app.search_query = ""
            app._reset_list()

    # Status changes
    elif key == ord("s"):
        app._move_current("shave")
    elif key == ord("x"):
        app._move_current("shorn")
    elif key == ord("r"):
        app._move_current("regrow")

    # Create (with type picker)
    elif key == ord("c"):
        yak_type = app._pick_type_for_create()
        if yak_type:
            app._create_task(parent=None, yak_type=yak_type)
    elif key == ord("C"):
        parent = app._current_task_id()
        if parent:
            yak_type = app._pick_type_for_create()
            if yak_type:
                app._create_task(parent=parent, yak_type=yak_type)

    # Edit / delete
    elif key == ord("e"):
        tid = app._current_task_id()
        if tid:
            app._edit_task(tid)
    elif key == ord("D"):
        tid = app._current_task_id()
        if tid:
            app._delete_task(tid)

    # Quick adjusts
    elif key == ord("P"):
        tid = app._current_task_id()
        if tid:
            app._quick_adjust_priority(tid)
    elif key == ord("T"):
        tid = app._current_task_id()
        if tid:
            app._quick_adjust_type(tid)
    elif key == ord("N"):
        tid = app._current_task_id()
        if tid:
            app._quick_adjust_title(tid)
    elif key == ord("L"):
        tid = app._current_task_id()
        if tid:
            app._quick_adjust_labels(tid)

    # Dependencies
    elif key == ord("b"):
        tid = app._current_task_id()
        if tid:
            app._add_dependency(tid)
    elif key == ord("B"):
        tid = app._current_task_id()
        if tid:
            app._remove_dependency(tid)

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

    return True
