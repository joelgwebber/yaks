"""Structured task create/edit form for the yaks TUI.

Layout
------
  Line 0    form title bar
  Line 1    ─── separator
  Line 2    title      [...]          ─┐
  Line 3    type        task ●bug …   │ compact metadata zone (1-row pitch)
  Line 4    priority    1 2 ●3 4 5   │
  Line 5    labels     [...]          ─┘
  Line 6    ─── separator
  Line 7…   content zone (scrollable):
              ─ description ──────────────────
                First line of description…
                …
              ─ ▸ 2026-06-03T12:00:00Z ──────
                Comment text…
  h-2       ─── separator
  h-1       help bar

Navigation: Tab/Shift-Tab, arrows, Ctrl-N/P, and j/k all move between rows
(meta + description + each comment) — the same set the fuzzy picker uses.
Metadata rows use LineEditor / chip pickers. Content rows open PT on Enter.
Comments support x (delete) and n (new comment) from anywhere outside a
text-editing row.
"""

from __future__ import annotations

import curses
from dataclasses import dataclass, field

from yaklib.model import now_iso

from yaktui.colors import C_HEADER, C_HELP, C_P2, C_SELECTED, C_TYPE
from yaktui.dialogs import NAV_NEXT_KEYS, NAV_PREV_KEYS, line_editor_window, safe_addstr
from yaktui.vim_edit import CANCEL, COMMIT, LineEditor

# ---------------------------------------------------------------------------
# Layout constants
# ---------------------------------------------------------------------------

_META_Y = 2  # first metadata row (after title + separator)
_META_H = 4  # title / type / priority / labels
_META_SEP_Y = _META_Y + _META_H  # = 6
_CONTENT_Y = _META_SEP_Y + 1  # = 7
_FOOTER_H = 2  # separator + help bar

# Row indices
_ROW_TITLE = 0
_ROW_TYPE = 1
_ROW_PRIORITY = 2
_ROW_LABELS = 3
_ROW_DESC = 4  # description — content zone starts here
# Comments: _ROW_DESC + 1 + i  for comment[i]

_TYPE_CHOICES = ["task", "bug", "feature", "idea"]
_PRI_CHOICES = [1, 2, 3, 4, 5]

_META_LABELS = {
    _ROW_TITLE: "title     ",
    _ROW_TYPE: "type      ",
    _ROW_PRIORITY: "priority  ",
    _ROW_LABELS: "labels    ",
}
_LABEL_W = max(len(v) for v in _META_LABELS.values())
_FIELD_X = 2

_PREVIEW_DESC = 3  # max lines shown for description preview
_PREVIEW_COMMENT = 2  # max lines shown per comment preview


# ---------------------------------------------------------------------------
# Comment parsing helpers
# ---------------------------------------------------------------------------


def _split_desc(body: str) -> tuple[str, str]:
    """Split body at the first ``---\\n▸`` comment marker.

    Returns ``(description, tail)`` where *tail* starts with the marker.
    """
    marker = "\n---\n▸"
    idx = body.find(marker)
    return (body, "") if idx < 0 else (body[:idx], body[idx:])


def _parse_comments(tail: str) -> list[tuple[str, str]]:
    """Parse the comment tail into [(timestamp, text), …]."""
    if not tail:
        return []
    sep = "\n---\n▸"
    raw = tail[len(sep) :]  # strip the leading separator
    parts = raw.split(sep)
    result = []
    for part in parts:
        first_nl = part.find("\n")
        if first_nl < 0:
            result.append((part.strip(), ""))
        else:
            result.append((part[:first_nl].strip(), part[first_nl + 1 :].strip()))
    return result


def _build_tail(comments: list[tuple[str, str]]) -> str:
    """Rebuild the raw comment tail from a list of (timestamp, text) pairs."""
    return "".join(f"\n---\n▸ {ts}\n{text}" for ts, text in comments)


# ---------------------------------------------------------------------------
# Form state
# ---------------------------------------------------------------------------


@dataclass
class _FormState:
    title: LineEditor
    yak_type: str  # "task" | "bug" | "feature" | "idea"
    priority: int  # 1-5
    labels: LineEditor
    description: str  # description body (no comment blocks)
    comments: list  # [(timestamp, text), …]
    row: int  # current focused row (see _ROW_* constants)
    vim: bool
    content_scroll: int = 0  # visual lines scrolled in the content zone

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
        lbls_str = ", ".join(lbls) if isinstance(lbls, list) else str(lbls) if lbls else ""
        return cls(
            title=LineEditor(task.get("title", "") or "", vim=vim),
            yak_type=task.get("type", yak_type) or yak_type,
            priority=task.get("priority", 3) or 3,
            labels=LineEditor(lbls_str, vim=vim),
            description=desc.strip(),
            comments=_parse_comments(tail),
            row=0,
            vim=vim,
        )

    def close(self) -> None:
        self.title.close()
        self.labels.close()

    def is_valid(self) -> bool:
        return bool(self.title.buf.strip())

    def total_rows(self) -> int:
        """Total number of navigable rows."""
        return _ROW_DESC + 1 + len(self.comments)

    def comment_idx(self) -> int | None:
        """Index into self.comments for the current row, or None."""
        if self.row > _ROW_DESC:
            return self.row - _ROW_DESC - 1
        return None

    def to_dict(self) -> dict:
        raw = self.labels.buf.strip()
        labels = [l.strip() for l in raw.split(",") if l.strip()] if raw else []
        desc = self.description.strip()
        tail = _build_tail(self.comments)
        return {
            "title": self.title.buf.strip(),
            "type": self.yak_type,
            "priority": self.priority,
            "labels": labels,
            "description": (desc + tail) or None,
        }


# ---------------------------------------------------------------------------
# Content-zone geometry helpers
# ---------------------------------------------------------------------------


def _section_specs(state: _FormState) -> list[tuple[int, str | None, str, int]]:
    """Return (row, timestamp_or_None, text, max_preview) for every content section."""
    specs = [(_ROW_DESC, None, state.description, _PREVIEW_DESC)]
    for i, (ts, text) in enumerate(state.comments):
        specs.append((_ROW_DESC + 1 + i, ts, text, _PREVIEW_COMMENT))
    return specs


def _section_height(text: str, max_p: int) -> int:
    """Visual height of one content section (header + content + more + blank gap)."""
    lines = text.strip().split("\n") if text and text.strip() else []
    shown = min(max_p, len(lines)) if lines else 1  # at least 1 for placeholder
    more = max(0, len(lines) - shown)
    return 1 + shown + (1 if more else 0) + 1  # header + content + [more] + gap


def _compute_vis_starts(state: _FormState) -> dict[int, int]:
    """Map each content row index → visual start line within the content zone."""
    starts: dict[int, int] = {}
    y = 0
    for row, ts, text, max_p in _section_specs(state):
        starts[row] = y
        y += _section_height(text, max_p)
    return starts


def _fix_content_scroll(state: _FormState, zone_h: int) -> None:
    """Ensure the active content-zone section's header is visible."""
    if state.row < _ROW_DESC:
        return
    starts = _compute_vis_starts(state)
    vis = starts.get(state.row, 0)
    if vis < state.content_scroll:
        state.content_scroll = vis
    elif vis >= state.content_scroll + zone_h:
        state.content_scroll = max(0, vis - zone_h + 1)


# ---------------------------------------------------------------------------
# Rendering helpers
# ---------------------------------------------------------------------------


def _draw_text_field(stdscr, ed: LineEditor, y: int, x: int, width: int, active: bool) -> tuple[int, int] | None:
    """Draw a bracketed single-line field. When *active*, return the (y, x)
    where the caret belongs so the caller can place it as the final pre-refresh
    step (placing it inline here would be clobbered by later draws)."""
    inner_w = max(1, width - 2)
    visible, cur = line_editor_window(ed, inner_w)
    padded = visible.ljust(inner_w)[:inner_w]
    safe_addstr(stdscr, y, x, "[", curses.A_DIM)
    safe_addstr(stdscr, y, x + 1, padded, curses.A_BOLD if active else curses.A_DIM)
    safe_addstr(stdscr, y, x + 1 + inner_w, "]", curses.A_DIM)
    return (y, x + 1 + cur) if active else None


def _draw_meta_zone(stdscr, state: _FormState, w: int) -> tuple[int, int] | None:
    """Render the four compact metadata rows. Returns the caret (y, x) for the
    active text field (Title/Labels), or None on a non-text row."""
    value_x = _FIELD_X + _LABEL_W + 1
    cursor = None

    for ri in range(_META_H):
        y = _META_Y + ri
        active = ri == state.row
        label = _META_LABELS[ri]
        safe_addstr(stdscr, y, _FIELD_X, label, curses.A_BOLD if active else curses.A_DIM)

        if ri == _ROW_TITLE:
            field_w = max(10, w - value_x - 2)
            c = _draw_text_field(stdscr, state.title, y, value_x, field_w, active)
            if active:
                cursor = c
            if state.vim and active:
                badge = f"[{'N' if state.title.mode == 'normal' else 'I'}]"
                safe_addstr(stdscr, y, w - len(badge) - 1, badge, curses.A_DIM)

        elif ri == _ROW_TYPE:
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

        elif ri == _ROW_PRIORITY:
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

        elif ri == _ROW_LABELS:
            field_w = max(10, w - value_x - 2)
            c = _draw_text_field(stdscr, state.labels, y, value_x, field_w, active)
            if active:
                cursor = c
            if state.vim and active:
                badge = f"[{'N' if state.labels.mode == 'normal' else 'I'}]"
                safe_addstr(stdscr, y, w - len(badge) - 1, badge, curses.A_DIM)

    return cursor


def _draw_content_zone(stdscr, state: _FormState, y_start: int, zone_h: int, w: int) -> None:
    """Render the scrollable description + comments zone."""
    specs = _section_specs(state)
    starts = _compute_vis_starts(state)

    for row, ts, text, max_p in specs:
        vis_y = starts[row]
        active = row == state.row
        lines = text.strip().split("\n") if text and text.strip() else []
        shown = min(max_p, len(lines))
        more = max(0, len(lines) - shown)

        # Build the renderable lines for this section
        is_comment = ts is not None

        # ── header ──
        if is_comment:
            hint = "  Enter:edit  x:del" if active else ""
            avail = w - len(hint)
            head_label = f"\u2500 \u25b8 {ts} "
            head_dashes = "\u2500" * max(0, avail - len(head_label))
            header_text = (head_label + head_dashes)[:avail]
        else:
            hint = "  Enter:edit" if active else ""
            avail = w - len(hint)
            head_label = "\u2500 description "
            head_dashes = "\u2500" * max(0, avail - len(head_label))
            header_text = (head_label + head_dashes)[:avail]

        head_attr = curses.A_BOLD if active else curses.A_DIM

        def _try_draw(vis_local: int, text_str: str, attr: int) -> None:
            vy = vis_y + vis_local
            sy = y_start + vy - state.content_scroll
            if y_start <= sy < y_start + zone_h:
                safe_addstr(stdscr, sy, 0, text_str[:w], attr)

        _try_draw(0, header_text, head_attr)
        if hint and active:
            vy = vis_y
            sy = y_start + vy - state.content_scroll
            if y_start <= sy < y_start + zone_h:
                safe_addstr(stdscr, sy, len(header_text), hint, curses.A_BOLD)

        # ── content lines ──
        if lines:
            content_attr = curses.A_BOLD if active else 0
            for i, line in enumerate(lines[:shown]):
                _try_draw(1 + i, "  " + line, content_attr)
        else:
            placeholder = "  (empty \u2014 Enter to add)"
            _try_draw(1, placeholder, curses.A_DIM)

        # ── more indicator ──
        if more > 0:
            more_text = f"  \u2026 {more} more line{'s' if more > 1 else ''}"
            _try_draw(1 + shown, more_text, curses.A_DIM)


def draw_task_form(stdscr, state: _FormState, form_title: str) -> None:
    stdscr.erase()
    h, w = stdscr.getmaxyx()

    # Title bar
    safe_addstr(stdscr, 0, 0, f"  {form_title}", curses.color_pair(C_HEADER) | curses.A_BOLD)
    safe_addstr(stdscr, 1, 0, "\u2500" * w, curses.A_DIM)

    # Metadata zone
    cursor = _draw_meta_zone(stdscr, state, w)

    # Separator between meta and content
    safe_addstr(stdscr, _META_SEP_Y, 0, "\u2500" * w, curses.A_DIM)

    # Content zone
    zone_h = max(1, h - _CONTENT_Y - _FOOTER_H)
    _draw_content_zone(stdscr, state, _CONTENT_Y, zone_h, w)

    # Footer
    sep_y = h - 2
    safe_addstr(stdscr, sep_y, 0, "\u2500" * w, curses.A_DIM)

    save_part = "Ctrl-S:save" if state.is_valid() else "(need title)"
    on_comment = state.comment_idx() is not None
    hints_parts = [
        "Tab/j/k:move",
        "\u2190\u2192:pick",
        "Enter:edit",
        "n:comment",
    ]
    if on_comment:
        hints_parts.append("x:delete")
    hints_parts.append(save_part)
    hints_parts.append("Esc:cancel")
    hints = "  " + "  ".join(hints_parts)
    safe_addstr(stdscr, h - 1, 0, " " * w, curses.color_pair(C_HELP))
    safe_addstr(stdscr, h - 1, 0, hints[:w], curses.color_pair(C_HELP))

    return cursor


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

    Returns a field dict on save (Ctrl-S), or None on cancel (Esc).
    """
    from yaktui import editor as _editor

    if task:
        form_title = f"Edit {task.get('id', 'task')}"
    elif parent:
        form_title = f"New child of {parent}"
    else:
        form_title = "New task"

    state = _FormState.from_task(task, yak_type=yak_type, vim=vim)

    def _zone_h() -> int:
        h, _ = stdscr.getmaxyx()
        return max(1, h - _CONTENT_Y - _FOOTER_H)

    def _nav(delta: int) -> None:
        state.row = (state.row + delta) % state.total_rows()
        _fix_content_scroll(state, _zone_h())

    curses.curs_set(1)
    try:
        while True:
            cursor = draw_task_form(stdscr, state, form_title)

            # Cursor visibility: only the text rows (Title/Labels) show a caret,
            # and it must be positioned as the *last* op before refresh — drawing
            # the content zone + footer afterward would otherwise leave the
            # hardware cursor parked at the bottom of the screen.
            if cursor is not None:
                curses.curs_set(1)
                try:
                    stdscr.move(*cursor)
                except curses.error:
                    pass
            else:
                curses.curs_set(0)
            stdscr.refresh()

            key = stdscr.getch()
            if key == -1:
                continue

            # ── global ────────────────────────────────────────────────────
            if key == 19:  # Ctrl-S: save
                if state.is_valid():
                    return state.to_dict()
                continue

            # New comment from anywhere outside a text-editing row
            ed_active = state.row in (_ROW_TITLE, _ROW_LABELS)
            if key == ord("n") and not ed_active:
                curses.curs_set(0)
                text = _editor.edit_multiline(stdscr, vim=vim, label="new comment")
                curses.curs_set(1)
                if text and text.strip():
                    state.comments.append((now_iso(), text.strip()))
                    state.row = _ROW_DESC + len(state.comments)
                    _fix_content_scroll(state, _zone_h())
                continue

            # ── metadata rows ─────────────────────────────────────────────
            if state.row == _ROW_TITLE:
                ed = state.title
                nav_down = key in NAV_NEXT_KEYS or (vim and ed.mode == "normal" and key == ord("j"))
                nav_up = key in NAV_PREV_KEYS or (vim and ed.mode == "normal" and key == ord("k"))
                if nav_down:
                    _nav(+1)
                    continue
                if nav_up:
                    _nav(-1)
                    continue
                r = ed.step(key)
                if r == COMMIT:
                    _nav(+1)
                elif r == CANCEL:
                    return None

            elif state.row == _ROW_TYPE:
                if key in NAV_NEXT_KEYS or key == ord("j"):
                    _nav(+1)
                elif key in NAV_PREV_KEYS or key == ord("k"):
                    _nav(-1)
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

            elif state.row == _ROW_PRIORITY:
                if key in NAV_NEXT_KEYS or key == ord("j"):
                    _nav(+1)
                elif key in NAV_PREV_KEYS or key == ord("k"):
                    _nav(-1)
                elif key in (curses.KEY_LEFT, ord("h")):
                    state.priority = max(1, state.priority - 1)
                elif key in (curses.KEY_RIGHT, ord("l"), ord(" "), 10, 13):
                    state.priority = min(5, state.priority + 1)
                elif key == 27:
                    return None
                elif ord("1") <= key <= ord("5"):
                    state.priority = key - ord("0")

            elif state.row == _ROW_LABELS:
                ed = state.labels
                nav_down = key in NAV_NEXT_KEYS or (vim and ed.mode == "normal" and key == ord("j"))
                nav_up = key in NAV_PREV_KEYS or (vim and ed.mode == "normal" and key == ord("k"))
                if nav_down:
                    _nav(+1)
                    continue
                if nav_up:
                    _nav(-1)
                    continue
                r = ed.step(key)
                if r == COMMIT:
                    _nav(+1)
                elif r == CANCEL:
                    return None

            # ── description ───────────────────────────────────────────────
            elif state.row == _ROW_DESC:
                if key in NAV_NEXT_KEYS or key == ord("j"):
                    _nav(+1)
                elif key in NAV_PREV_KEYS or key == ord("k"):
                    _nav(-1)
                elif key in (10, 13, curses.KEY_ENTER, ord("e"), ord("i")):
                    curses.curs_set(0)
                    edited = _editor.edit_multiline(stdscr, initial=state.description, vim=vim, label="description")
                    curses.curs_set(1)
                    if edited is not None:
                        state.description = edited.strip()
                elif key == 27:
                    return None

            # ── individual comment rows ───────────────────────────────────
            else:
                cidx = state.comment_idx()
                if key in NAV_NEXT_KEYS or key == ord("j"):
                    _nav(+1)
                elif key in NAV_PREV_KEYS or key == ord("k"):
                    _nav(-1)
                elif key in (10, 13, curses.KEY_ENTER, ord("e"), ord("i")):
                    ts, text = state.comments[cidx]
                    curses.curs_set(0)
                    edited = _editor.edit_multiline(stdscr, initial=text, vim=vim, label=f"comment ▸ {ts}")
                    curses.curs_set(1)
                    if edited is not None:
                        state.comments[cidx] = (ts, edited.strip())
                elif key == ord("x"):
                    # Delete this comment (no confirm — form not saved yet)
                    state.comments.pop(cidx)
                    new_len = len(state.comments)
                    if new_len == 0:
                        state.row = _ROW_DESC
                    elif cidx >= new_len:
                        state.row = _ROW_DESC + new_len
                    _fix_content_scroll(state, _zone_h())
                elif key == 27:
                    return None

    finally:
        curses.curs_set(0)
        state.close()
