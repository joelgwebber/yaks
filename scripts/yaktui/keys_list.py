"""Key dispatch for the list (left) pane of the TUI."""

from __future__ import annotations

import curses

from yaktui import dialogs as _dialogs
from yaktui import mutate as _mutate


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
    # Viewport scroll (Ctrl-E / Ctrl-Y): move the viewport without moving
    # the cursor; cursor is pulled to the edge if it goes off-screen.
    elif key == 5:  # Ctrl-E
        app._scroll_viewport(+1)
    elif key == 25:  # Ctrl-Y
        app._scroll_viewport(-1)

    # Filter drawer / inline search
    elif key == ord("f"):
        app._open_filter_drawer()
    elif key == ord("/"):
        app._open_inline_search()
    elif key == 27:  # Escape clears filter
        if not app.filter_spec.is_empty():
            from yaklib.filter import FilterSpec as _FS

            app.filter_spec = _FS()
            app._reset_list()

    # Status change via single-key picker.
    elif key == ord("S"):
        tid = app._current_task_id()
        if tid:
            app._quick_adjust_state(tid)

    # Create — type is selected inside the form itself
    elif key == ord("c"):
        app._create_task(parent=None)
    elif key == ord("C"):
        parent = app._current_task_id()
        if parent:
            app._create_task(parent=parent)

    # Edit / delete
    elif key == ord("E"):
        tid = app._current_task_id()
        if tid:
            app._edit_task(tid)
    elif key == ord("X"):
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
    elif key == ord("L"):
        tid = app._current_task_id()
        if tid:
            app._quick_adjust_labels(tid)

    # Dependencies: B is context-aware (list view always adds, since there's
    # no way to disambiguate which dep to remove without seeing them).
    elif key == ord("D"):
        _mutate.handle_dep_key(app)

    # Clipboard / comment / attach
    elif key == ord("y"):
        tid = app._current_task_id()
        if tid:
            app._copy_to_clipboard(tid)
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

    # Space toggles collapse/expand of the current row's subtree.
    elif key == ord(" "):
        tid = app._current_task_id()
        if tid:
            app._toggle_collapse(tid)

    return True
