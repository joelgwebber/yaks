"""Pure-function tests for yaktui.vim_edit.LineEditor."""

from __future__ import annotations

from yaktui.vim_edit import LineEditor, COMMIT, CANCEL, CONTINUE, ESCALATE


def _keys(ed, s):
    """Feed a string of characters as keypresses and return the last action."""
    r = CONTINUE
    for ch in s:
        r = ed.step(ord(ch))
    return r


def test_non_vim_basic_typing():
    ed = LineEditor(vim=False)
    _keys(ed, "hello")
    assert ed.buf == "hello"
    assert ed.pos == 5


def test_non_vim_esc_cancels_immediately():
    ed = LineEditor(vim=False)
    assert ed.step(27) == CANCEL


def test_vim_defaults_to_insert():
    ed = LineEditor(vim=True, emit_cursor_shape=False)
    assert ed.mode == "insert"
    _keys(ed, "abc")
    assert ed.buf == "abc"


def test_vim_esc_switches_to_normal():
    ed = LineEditor(vim=True, emit_cursor_shape=False)
    _keys(ed, "abc")
    ed.step(27)
    assert ed.mode == "normal"


def test_vim_double_esc_cancels():
    ed = LineEditor(vim=True, emit_cursor_shape=False)
    _keys(ed, "abc")
    ed.step(27)  # insert -> normal
    r = ed.step(27)  # normal -> cancel
    assert r == CANCEL


def test_vim_esc_motion_esc_does_not_cancel():
    """Key between two Escs breaks the double-Esc latch."""
    ed = LineEditor(vim=True, emit_cursor_shape=False)
    _keys(ed, "abc")
    ed.step(27)       # -> normal
    ed.step(ord("h"))  # motion
    r = ed.step(27)   # should NOT cancel
    assert r == CONTINUE
    assert ed.mode == "normal"


def test_vim_normal_motions_hjkl_and_line():
    ed = LineEditor(initial="hello world", vim=True, emit_cursor_shape=False)
    ed.step(27)  # normal
    assert ed.pos == 11
    ed.step(ord("0"))
    assert ed.pos == 0
    ed.step(ord("$"))
    assert ed.pos == 10  # $ parks on last char, not past end
    ed.step(ord("h"))
    assert ed.pos == 9
    ed.step(ord("l"))
    assert ed.pos == 10


def test_vim_word_motion_w_b():
    ed = LineEditor(initial="one two three", vim=True, emit_cursor_shape=False)
    ed.step(27)
    ed.step(ord("0"))
    ed.step(ord("w"))
    assert ed.pos == 4  # 't' of 'two'
    ed.step(ord("w"))
    assert ed.pos == 8  # 't' of 'three'
    ed.step(ord("b"))
    assert ed.pos == 4
    ed.step(ord("b"))
    assert ed.pos == 0


def test_vim_i_a_A_I():
    ed = LineEditor(initial="foo", vim=True, emit_cursor_shape=False)
    ed.step(27)
    ed.step(ord("0"))
    ed.step(ord("a"))
    assert ed.mode == "insert" and ed.pos == 1
    ed.step(27)
    ed.step(ord("I"))
    assert ed.mode == "insert" and ed.pos == 0
    ed.step(27)
    ed.step(ord("A"))
    assert ed.mode == "insert" and ed.pos == 3


def test_vim_x_deletes_char():
    ed = LineEditor(initial="abcd", vim=True, emit_cursor_shape=False)
    ed.step(27)
    ed.step(ord("0"))
    ed.step(ord("x"))
    assert ed.buf == "bcd" and ed.pos == 0


def test_vim_dd_clears_line():
    ed = LineEditor(initial="hello", vim=True, emit_cursor_shape=False)
    ed.step(27)
    ed.step(ord("d"))
    ed.step(ord("d"))
    assert ed.buf == "" and ed.pos == 0


def test_vim_D_deletes_to_end():
    ed = LineEditor(initial="hello world", vim=True, emit_cursor_shape=False)
    ed.step(27)
    ed.step(ord("0"))
    for _ in range(6):
        ed.step(ord("l"))
    ed.step(ord("D"))
    assert ed.buf == "hello "


def test_vim_C_changes_to_end():
    ed = LineEditor(initial="hello world", vim=True, emit_cursor_shape=False)
    ed.step(27)
    ed.step(ord("0"))
    for _ in range(6):
        ed.step(ord("l"))
    ed.step(ord("C"))
    assert ed.buf == "hello " and ed.mode == "insert"


def test_vim_s_substitutes_char():
    ed = LineEditor(initial="abc", vim=True, emit_cursor_shape=False)
    ed.step(27)
    ed.step(ord("0"))
    ed.step(ord("s"))
    assert ed.buf == "bc" and ed.mode == "insert"


def test_enter_commits_in_any_mode():
    ed = LineEditor(initial="hi", vim=True, emit_cursor_shape=False)
    assert ed.step(10) == COMMIT
    ed.step(27)
    assert ed.step(10) == COMMIT


def test_mode_badge():
    ed = LineEditor(vim=False)
    assert ed.mode_badge() == ""
    ed_vim = LineEditor(vim=True, emit_cursor_shape=False)
    assert ed_vim.mode_badge() == "[I]"
    ed_vim.step(27)
    assert ed_vim.mode_badge() == "[N]"


def test_escalate_requires_opt_in():
    ed = LineEditor(vim=False)
    _keys(ed, "foo")
    assert ed.step(ord("\t")) == CONTINUE  # tab ignored without opt-in

    ed2 = LineEditor(vim=False, allow_escalate=True)
    _keys(ed2, "foo")
    assert ed2.step(ord("\t")) == ESCALATE
