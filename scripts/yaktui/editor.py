"""Suspend-curses multi-line editor via prompt_toolkit.

Called by the task form and comment editor whenever multi-line text
editing is needed.  Suspends the curses display, runs a prompt_toolkit
inline (non-full-screen) editor, then resumes curses.
"""

from __future__ import annotations

import curses
import sys


def edit_multiline(
    stdscr,
    initial: str = "",
    vim: bool = False,
    label: str = "",
) -> str | None:
    """Suspend curses, open a prompt_toolkit multi-line editor, then resume.

    Returns the edited text on save, or None on cancel.

    Key bindings work in both vi and non-vi modes:
      Ctrl-S      — save and return the buffer text
      Esc Esc     — cancel (return None)
    In vi mode all standard vi motions and edits work as expected;
    a single Esc enters normal mode as usual, and a second Esc cancels.
    """
    try:
        from prompt_toolkit import prompt as pt_prompt
        from prompt_toolkit.cursor_shapes import ModalCursorShapeConfig
        from prompt_toolkit.key_binding import KeyBindings
    except ImportError:
        # prompt_toolkit not installed — caller should handle None
        return None

    curses.def_prog_mode()
    curses.endwin()

    result: str | None = None
    try:
        sys.stdout.write("\n")
        if label:
            sys.stdout.write(f"  \033[1m{label}\033[0m  ")
        sys.stdout.write("(Ctrl-S: save  |  Esc Esc: cancel)\n\n")
        sys.stdout.flush()

        kb = KeyBindings()

        @kb.add("c-s")
        def _save(event):
            event.app.exit(result=event.app.current_buffer.text)

        @kb.add("escape", "escape", eager=True)
        def _cancel(event):
            event.app.exit(result=None)

        result = pt_prompt(
            "",
            default=initial,
            multiline=True,
            vi_mode=vim,
            key_bindings=kb,
            cursor=ModalCursorShapeConfig() if vim else None,
        )
    except (KeyboardInterrupt, EOFError):
        result = None
    finally:
        curses.reset_prog_mode()
        stdscr.refresh()

    return result
