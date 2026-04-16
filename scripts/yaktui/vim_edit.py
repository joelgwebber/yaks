"""Vim-aware single-line editor. Provides a LineEditor that handles
insert/normal modes with double-Esc cancel. When vim_mode is False it
degrades to plain readline-ish editing (matching the pre-vim behavior).

This is intentionally a subset — enough for muscle memory, not a full
vim emulation. See yak-b574 notes for scope decisions.
"""

from __future__ import annotations

import curses
import sys

# Return codes from LineEditor.step()
CONTINUE = "continue"
COMMIT = "commit"
CANCEL = "cancel"
ESCALATE = "escalate"  # for callers that want Tab to escalate (e.g. inline search)


# DECSCUSR cursor shapes. Unsupported terminals silently ignore.
_CURSOR_INSERT = "\x1b[5 q"   # blinking bar
_CURSOR_NORMAL = "\x1b[2 q"   # steady block
_CURSOR_DEFAULT = "\x1b[0 q"  # reset to terminal default


def set_cursor_shape(mode: str | None) -> None:
    """Emit a DECSCUSR sequence. *mode* = 'insert', 'normal', or None for default."""
    if mode == "insert":
        seq = _CURSOR_INSERT
    elif mode == "normal":
        seq = _CURSOR_NORMAL
    else:
        seq = _CURSOR_DEFAULT
    try:
        sys.stdout.write(seq)
        sys.stdout.flush()
    except (OSError, ValueError):
        pass


def _is_word_char(c: str) -> bool:
    return c.isalnum() or c == "_"


def _next_word(buf: str, pos: int) -> int:
    n = len(buf)
    if pos >= n:
        return n
    # Skip current run (word or non-word), then skip whitespace.
    in_word = _is_word_char(buf[pos])
    while pos < n and (_is_word_char(buf[pos]) == in_word) and not buf[pos].isspace():
        pos += 1
    while pos < n and buf[pos].isspace():
        pos += 1
    return pos


def _prev_word(buf: str, pos: int) -> int:
    if pos == 0:
        return 0
    pos -= 1
    while pos > 0 and buf[pos].isspace():
        pos -= 1
    if pos == 0:
        return 0
    in_word = _is_word_char(buf[pos])
    while pos > 0 and (_is_word_char(buf[pos - 1]) == in_word) and not buf[pos - 1].isspace():
        pos -= 1
    return pos


class LineEditor:
    """Single-line editor with optional vim modes.

    Usage:
        ed = LineEditor(initial="hello", vim=True)
        while True:
            render(ed.buf, ed.pos, ed.mode)
            key = stdscr.getch()
            r = ed.step(key)
            if r == COMMIT: return ed.buf
            if r == CANCEL: return None
    """

    def __init__(self, initial: str = "", vim: bool = False,
                 allow_escalate: bool = False,
                 emit_cursor_shape: bool = True):
        self.buf = initial
        self.pos = len(initial)
        self.vim = vim
        self.mode = "insert" if vim else "none"
        self.allow_escalate = allow_escalate
        self._emit_cursor = emit_cursor_shape
        self._pending_d = False  # first 'd' of dd
        self._last_esc = False   # first Esc of double-Esc
        if vim and emit_cursor_shape:
            set_cursor_shape("insert")

    # -- public -----------------------------------------------------------

    def step(self, key: int) -> str:
        """Process one keypress. Returns CONTINUE / COMMIT / CANCEL / ESCALATE."""
        if self.allow_escalate and key == ord("\t"):
            return ESCALATE

        if key in (ord("\n"), curses.KEY_ENTER, 10, 13):
            return COMMIT

        if not self.vim:
            if key == 27:
                return CANCEL
            self._insert_edit(key)
            return CONTINUE

        if self.mode == "insert":
            return self._step_insert(key)
        return self._step_normal(key)

    def close(self) -> None:
        """Restore the terminal cursor shape. Call once when done."""
        if self.vim and self._emit_cursor:
            set_cursor_shape(None)

    # -- mode handlers ----------------------------------------------------

    def _step_insert(self, key: int) -> str:
        if key == 27:
            self._set_mode("normal")
            self._last_esc = True
            return CONTINUE
        self._last_esc = False
        self._insert_edit(key)
        return CONTINUE

    def _step_normal(self, key: int) -> str:
        if key == 27:
            if self._last_esc:
                return CANCEL
            self._last_esc = True
            return CONTINUE
        # Any other key clears the last-Esc latch.
        self._last_esc = False

        # Multi-key: dd
        if self._pending_d:
            self._pending_d = False
            if key == ord("d"):
                self.buf = ""
                self.pos = 0
                return CONTINUE
            # Other keys fall through to normal dispatch.

        # Motions
        if key in (ord("h"), curses.KEY_LEFT):
            self.pos = max(0, self.pos - 1)
        elif key in (ord("l"), curses.KEY_RIGHT):
            self.pos = min(len(self.buf), self.pos + 1)
        elif key == ord("0") or key == ord("^"):
            self.pos = 0
        elif key == ord("$"):
            self.pos = max(0, len(self.buf) - 1)
        elif key == ord("w"):
            self.pos = _next_word(self.buf, self.pos)
        elif key == ord("b"):
            self.pos = _prev_word(self.buf, self.pos)

        # Enter insert
        elif key == ord("i"):
            self._set_mode("insert")
        elif key == ord("a"):
            self.pos = min(len(self.buf), self.pos + 1)
            self._set_mode("insert")
        elif key == ord("I"):
            self.pos = 0
            self._set_mode("insert")
        elif key == ord("A"):
            self.pos = len(self.buf)
            self._set_mode("insert")

        # Edits
        elif key == ord("x"):
            if self.pos < len(self.buf):
                self.buf = self.buf[:self.pos] + self.buf[self.pos + 1:]
                if self.pos >= len(self.buf) and self.pos > 0:
                    self.pos -= 1
        elif key == ord("D"):
            self.buf = self.buf[:self.pos]
        elif key == ord("d"):
            self._pending_d = True
        elif key == ord("C"):
            # Change to end of line: clear tail, enter insert.
            self.buf = self.buf[:self.pos]
            self._set_mode("insert")
        elif key == ord("s"):
            # Substitute char: delete, enter insert.
            if self.pos < len(self.buf):
                self.buf = self.buf[:self.pos] + self.buf[self.pos + 1:]
            self._set_mode("insert")
        # Silently ignore anything else in normal mode.
        return CONTINUE

    # -- insert-mode editing (shared with non-vim mode) -------------------

    def _insert_edit(self, key: int) -> None:
        buf, pos = self.buf, self.pos
        if key in (curses.KEY_BACKSPACE, 127, 8):
            if pos > 0:
                buf = buf[:pos - 1] + buf[pos:]
                pos -= 1
        elif key == curses.KEY_DC:
            if pos < len(buf):
                buf = buf[:pos] + buf[pos + 1:]
        elif key == curses.KEY_LEFT:
            pos = max(0, pos - 1)
        elif key == curses.KEY_RIGHT:
            pos = min(len(buf), pos + 1)
        elif key in (curses.KEY_HOME, 1):
            pos = 0
        elif key in (curses.KEY_END, 5):
            pos = len(buf)
        elif key == 11:  # Ctrl-K
            buf = buf[:pos]
        elif key == 21:  # Ctrl-U
            buf = buf[pos:]
            pos = 0
        elif key == 23:  # Ctrl-W
            i = pos
            while i > 0 and buf[i - 1] == " ":
                i -= 1
            while i > 0 and buf[i - 1] != " ":
                i -= 1
            buf = buf[:i] + buf[pos:]
            pos = i
        elif 32 <= key < 127:
            buf = buf[:pos] + chr(key) + buf[pos:]
            pos += 1
        self.buf, self.pos = buf, pos

    def _set_mode(self, mode: str) -> None:
        if self.mode != mode:
            self.mode = mode
            if self._emit_cursor:
                set_cursor_shape(mode)

    # -- UI helpers -------------------------------------------------------

    def mode_badge(self) -> str:
        """Returns '[I]' or '[N]' when vim is on, '' otherwise. Reserves
        the same width regardless of mode so layouts don't shift."""
        if not self.vim:
            return ""
        return "[N]" if self.mode == "normal" else "[I]"
