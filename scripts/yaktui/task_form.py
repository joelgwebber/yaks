"""Structured task create/edit form for the yaks TUI.

Provides run_task_form() which renders a full-screen modal form with:
  - LineEditor for title and labels
  - Chip selectors for type and priority
  - A description field that opens prompt_toolkit for multi-line editing
  - Ctrl-S to save, Esc / double-Esc (vim) to cancel
"""

from __future__ import annotations

import curses
from dataclasses import dataclass

from yaktui.colors import C_HEADER, C_HELP, C_P2, C_SEARCH, C_SELECTED, C_TYPE
from yaktui.dialogs import safe_addstr
from yaktui.vim_edit import CANCEL, COMMIT, CONTINUE, LineEditor

_TYPE_CHOICES = ["task", "bug", "feature", "idea"]
_PRI_CHOICES = [1, 2, 3, 4, 5]

# Display order of form rows
_ROWS = ["title", "type", "priority", "labels", "description"]

# Left-column labels (padded to the same width)
_ROW_LABEL = {
    "title": "title     ",
    "type": "type      ",
    "priority": "priority  ",
    "labels": "labels    ",
    "description": "desc      ",
}

_TEXT_ROWS = {"title", "labels"}
_CHIP_ROWS = {"type", "priority"}
_LABEL_W = max(len(v) for v in _ROW_LABEL.values())
_FIELD_X = 2  # left margin for field labels


# ---------------------------------------------------------------------------
# Body splitting helpers
# ---------------------------------------------------------------------------


def _split_desc(body: str) -> tuple[str, str]:
    """Split a task body into (description, comments_tail).

    Comments begin at the first ``---\\n▸`` marker.  The tail (including
    that marker) is preserved verbatim so comments are never disturbed.
    """
    marker = "\n---\n▸"
    idx = body.find(marker)
    if idx < 0:
        return body, ""
    return body[:idx], body[idx:]


# ---------------------------------------------------------------------------
# Form state
# ---------------------------------------------------------------------------


@dataclass
class _FormState:
    title: LineEditor
    yak_type: str  # "task" | "bug" | "feature" | "idea"
    priority: int  # 1-5
    labels: LineEditor
    description: str  # description text only (no comment blocks)
    _comments: str  # raw "---\n▸…" suffix, preserved unchanged
    row: int  # index into _ROWS for the currently focused field
    vim: bool

    @classmethod
    def from_task(
        cls,
        task: dict | None,
        yak_type: str = "task",
        vim: bool = False,
    ) -> "_FormState":
        task = task or {}
        body = task.get("description", "") or ""
        desc, tail = _split_desc(body)
        lbls = task.get("labels", [])
        if isinstance(lbls, list):
            lbls_str = ", ".join(lbls)
        elif isinstance(lbls, str):
            lbls_str = lbls
        else:
            lbls_str = ""
        return cls(
            title=LineEditor(task.get("title", "") or "", vim=vim),
            yak_type=task.get("type", yak_type) or yak_type,
            priority=task.get("priority", 3) or 3,
            labels=LineEditor(lbls_str, vim=vim),
            description=desc.strip(),
            _comments=tail,
            row=0,
            vim=vim,
        )

    def close(self) -> None:
        self.title.close()
        self.labels.close()

    def is_valid(self) -> bool:
        return bool(self.title.buf.strip())

    def to_dict(self) -> dict:
        """Return form values as a plain dict ready for task construction."""
        raw = self.labels.buf.strip()
        labels = [l.strip() for l in raw.split(",") if l.strip()] if raw else []
        # Re-join description with any preserved comment tail
        desc = self.description.strip()
        if self._comments:
            desc = desc + self._comments
        return {
            "title": self.title.buf.strip(),
            "type": self.yak_type,
            "priority": self.priority,
            "labels": labels,
            "description": desc or None,
        }


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------


def _draw_text_field(stdscr, ed: LineEditor, y: int, x: int, width: int, active: bool) -> None:
    """Render a LineEditor inside bracket delimiters; position cursor if active."""
    inner_w = max(1, width - 2)
    offset = max(0, ed.pos - inner_w + 1)
    visible = ed.buf[offset : offset + inner_w]
    padded = visible.ljust(inner_w)[:inner_w]
    dim = curses.A_DIM
    bold = curses.A_BOLD if active else curses.A_DIM
    safe_addstr(stdscr, y, x, "[", dim)
    safe_addstr(stdscr, y, x + 1, padded, bold)
    safe_addstr(stdscr, y, x + 1 + inner_w, "]", dim)
    if active:
        try:
            stdscr.move(y, x + 1 + (ed.pos - offset))
        except curses.error:
            pass


def draw_task_form(stdscr, state: _FormState, form_title: str) -> None:
    """Render the full form onto stdscr (erases first)."""
    stdscr.erase()
    h, w = stdscr.getmaxyx()

    # Title row
    header = f"  {form_title}"
    safe_addstr(stdscr, 0, 0, header, curses.color_pair(C_HEADER) | curses.A_BOLD)
    safe_addstr(stdscr, 1, 0, "\u2500" * w, curses.A_DIM)

    value_x = _FIELD_X + _LABEL_W + 1

    for ri, row_name in enumerate(_ROWS):
        y = 3 + ri * 2  # 2-row pitch keeps things airy
        if y >= h - 3:
            break

        active = ri == state.row
        label = _ROW_LABEL[row_name]
        safe_addstr(stdscr, y, _FIELD_X, label, curses.A_BOLD if active else curses.A_DIM)

        if row_name in _TEXT_ROWS:
            ed = state.title if row_name == "title" else state.labels
            field_w = max(10, w - value_x - 2)
            _draw_text_field(stdscr, ed, y, value_x, field_w, active)
            # vim mode badge
            if state.vim and active:
                badge = f"[{'N' if ed.mode == 'normal' else 'I'}]"
                safe_addstr(stdscr, y, w - len(badge) - 1, badge, curses.A_DIM)

        elif row_name == "type":
            cx = value_x
            for choice in _TYPE_CHOICES:
                sel = choice == state.yak_type
                if active and sel:
                    attr = curses.color_pair(C_SELECTED) | curses.A_BOLD
                elif sel:
                    attr = curses.color_pair(C_TYPE) | curses.A_BOLD
                else:
                    attr = curses.A_DIM
                chip = f" {choice} "
                safe_addstr(stdscr, y, cx, chip, attr)
                cx += len(chip) + 1

        elif row_name == "priority":
            cx = value_x
            for p in _PRI_CHOICES:
                sel = p == state.priority
                if active and sel:
                    attr = curses.color_pair(C_SELECTED) | curses.A_BOLD
                elif sel:
                    attr = curses.color_pair(C_P2) | curses.A_BOLD
                else:
                    attr = curses.A_DIM
                chip = f" {p} "
                safe_addstr(stdscr, y, cx, chip, attr)
                cx += len(chip) + 1

        elif row_name == "description":
            if state.description:
                first_line = state.description.split("\n")[0]
                max_w = w - value_x - 4
                preview = first_line[:max_w]
                if len(first_line) > max_w or "\n" in state.description:
                    preview += "\u2026"
            else:
                preview = "(empty \u2014 Enter to add)"
            attr = curses.A_BOLD if active else curses.A_DIM
            safe_addstr(stdscr, y, value_x, preview, attr)

    # Separator + help bar
    sep_y = h - 2
    if sep_y > 0:
        safe_addstr(stdscr, sep_y, 0, "\u2500" * w, curses.A_DIM)
    save_part = "Ctrl-S:save" if state.is_valid() else "(need title)"
    hints = f"  Tab/\u2191\u2193:move  \u2190\u2192:pick  Enter:edit  {save_part}  Esc:cancel"
    safe_addstr(stdscr, h - 1, 0, " " * w, curses.color_pair(C_HELP))
    safe_addstr(stdscr, h - 1, 0, hints[:w], curses.color_pair(C_HELP))


# ---------------------------------------------------------------------------
# Main form loop
# ---------------------------------------------------------------------------


def run_task_form(
    stdscr,
    root,
    vim: bool = False,
    task: dict | None = None,
    yak_type: str = "task",
    parent: str | None = None,
) -> dict | None:
    """Run the task create/edit form modal.

    *task* — pre-populate for edit mode; None for creation.
    *yak_type* — default type for new tasks.
    *parent* — parent task ID when creating a child.

    Returns a dict of field values on save, or None on cancel.
    """
    from yaktui import editor as _editor

    if task:
        form_title = f"Edit {task.get('id', 'task')}"
    elif parent:
        form_title = f"New child of {parent}"
    else:
        form_title = "New task"

    state = _FormState.from_task(task, yak_type=yak_type, vim=vim)

    curses.curs_set(1)
    try:
        while True:
            draw_task_form(stdscr, state, form_title)
            cur_row = _ROWS[state.row]
            if cur_row not in _TEXT_ROWS:
                curses.curs_set(0)
            else:
                curses.curs_set(1)
            stdscr.refresh()

            key = stdscr.getch()
            if key == -1:
                continue

            # Ctrl-S: save from anywhere
            if key == 19:
                if state.is_valid():
                    return state.to_dict()
                continue

            # --- Text rows ---
            if cur_row in _TEXT_ROWS:
                ed = state.title if cur_row == "title" else state.labels

                # Row navigation: Tab / arrows, and j/k in vim normal mode
                nav_down = key in (9, curses.KEY_DOWN) or (vim and ed.mode == "normal" and key == ord("j"))
                nav_up = key in (curses.KEY_BTAB, curses.KEY_UP) or (vim and ed.mode == "normal" and key == ord("k"))
                if nav_down:
                    state.row = (state.row + 1) % len(_ROWS)
                    continue
                if nav_up:
                    state.row = (state.row - 1) % len(_ROWS)
                    continue

                r = ed.step(key)
                if r == COMMIT:
                    # Enter moves to the next field
                    state.row = (state.row + 1) % len(_ROWS)
                elif r == CANCEL:
                    # double-Esc (vim) or single Esc (non-vim) → cancel form
                    return None

            # --- Chip rows ---
            elif cur_row == "type":
                if key in (9, curses.KEY_DOWN, ord("j")):
                    state.row = (state.row + 1) % len(_ROWS)
                elif key in (curses.KEY_BTAB, curses.KEY_UP, ord("k")):
                    state.row = (state.row - 1) % len(_ROWS)
                elif key in (curses.KEY_LEFT, ord("h")):
                    idx = _TYPE_CHOICES.index(state.yak_type)
                    state.yak_type = _TYPE_CHOICES[(idx - 1) % len(_TYPE_CHOICES)]
                elif key in (curses.KEY_RIGHT, ord("l"), ord(" "), 10, 13):
                    idx = _TYPE_CHOICES.index(state.yak_type)
                    state.yak_type = _TYPE_CHOICES[(idx + 1) % len(_TYPE_CHOICES)]
                elif key == 27:
                    return None
                else:
                    for t in _TYPE_CHOICES:
                        if key == ord(t[0]):
                            state.yak_type = t
                            break

            elif cur_row == "priority":
                if key in (9, curses.KEY_DOWN, ord("j")):
                    state.row = (state.row + 1) % len(_ROWS)
                elif key in (curses.KEY_BTAB, curses.KEY_UP, ord("k")):
                    state.row = (state.row - 1) % len(_ROWS)
                elif key in (curses.KEY_LEFT, ord("h")):
                    state.priority = max(1, state.priority - 1)
                elif key in (curses.KEY_RIGHT, ord("l"), ord(" "), 10, 13):
                    state.priority = min(5, state.priority + 1)
                elif key == 27:
                    return None
                elif ord("1") <= key <= ord("5"):
                    state.priority = key - ord("0")

            # --- Description row ---
            elif cur_row == "description":
                if key in (9, curses.KEY_DOWN, ord("j")):
                    state.row = (state.row + 1) % len(_ROWS)
                elif key in (curses.KEY_BTAB, curses.KEY_UP, ord("k")):
                    state.row = (state.row - 1) % len(_ROWS)
                elif key in (10, 13, curses.KEY_ENTER, ord("e"), ord("i")):
                    curses.curs_set(0)
                    edited = _editor.edit_multiline(
                        stdscr,
                        initial=state.description,
                        vim=vim,
                        label="description",
                    )
                    curses.curs_set(1)
                    if edited is not None:
                        state.description = edited.strip()
                elif key == 27:
                    return None

    finally:
        curses.curs_set(0)
        state.close()
