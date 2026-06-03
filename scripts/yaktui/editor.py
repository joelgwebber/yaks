"""Suspend-curses multi-line editor via prompt_toolkit.

Called by the task form and comment editor whenever multi-line text
editing is needed.  Suspends the curses display, runs a prompt_toolkit
inline (non-full-screen) editor, then resumes curses.
"""

from __future__ import annotations

import curses
import sys
import termios


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

    # curses.endwin() restores the shell terminal state, which typically
    # has IXON (XON/XOFF flow control) enabled.  That means Ctrl-S is
    # intercepted by the tty driver and never reaches the application.
    # Disable it for the duration of the PT session; reset_prog_mode()
    # restores the curses state (which had it off) on the way back.
    _old_tc = None
    try:
        fd = sys.stdin.fileno()
        _old_tc = termios.tcgetattr(fd)
        _new_tc = termios.tcgetattr(fd)
        _new_tc[0] &= ~termios.IXON  # iflag: disable start/stop output ctrl
        termios.tcsetattr(fd, termios.TCSADRAIN, _new_tc)
    except (termios.error, AttributeError, OSError):
        _old_tc = None

    # Clear the terminal so the frozen TUI frame (and any previous editor
    # sessions) don't appear as detritus above the new editor.
    sys.stdout.write("\033[2J\033[H")
    if label:
        sys.stdout.write(f"\033[1m  {label}\033[0m\n\n")
    sys.stdout.flush()

    result: str | None = None
    try:
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
            bottom_toolbar="Ctrl-S: save  │  Esc Esc: cancel",
        )
    except (KeyboardInterrupt, EOFError):
        result = None
    finally:
        if _old_tc is not None:
            try:
                termios.tcsetattr(fd, termios.TCSADRAIN, _old_tc)
            except (termios.error, OSError):
                pass
        curses.reset_prog_mode()
        stdscr.refresh()

    return result
