"""yaks-flavored rendering for the demo: a two-pane composition of an agent
chat transcript (left) and the yaks TUI board (right).

The *view-model* is shared with the real tool: this module calls the actual
``yaktui.tree.build_tree`` and ``yaktui.detail.build_detail_lines`` (fed by an
in-memory BoardRepo) so the list tree, ghosts, detail sections, deps and links
stay in lockstep with the TUI automatically. What stays forked is only the
*painter* — turning that view-model into ANSI for the cast instead of curses
attributes. Both are curses-free.
"""

from __future__ import annotations

import pathlib
import sys
from dataclasses import dataclass

# Make the shipped package importable so we can pull real constants/helpers.
_SCRIPTS = pathlib.Path(__file__).resolve().parent.parent / "scripts"
if str(_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS))

from yaklib.filter import FilterSpec  # noqa: E402
from yaklib.format import status_char  # noqa: E402
from yaklib.model import HAIRY, SHAVING, SHORN, parent_id  # noqa: E402
from yaktui.detail import DetailLine, build_detail_lines  # noqa: E402
from yaktui.tree import build_tree  # noqa: E402

from castkit import (  # noqa: E402
    BOLD,
    DIM,
    FG_BLUE,
    FG_CYAN,
    FG_GREEN,
    FG_MAGENTA,
    FG_RED,
    FG_WHITE,
    FG_YELLOW,
    ITALIC,
    UNDERLINE,
    Screen,
    bg256,
    fg256,
    sgr,
    text_width,
)

# Tab order + labels, mirroring yaktui/render.py TABS. The emoji come straight
# from the source so a glyph change there flows through here.
TAB_STATUSES = (HAIRY, SHAVING, SHORN)


def _tab_label(status: str) -> str:
    # ASCII-only for now: color-emoji glyphs don't advance exactly 2 monospace
    # cells in the player's DOM renderer, so they nudge alignment. The tab name
    # alone is unambiguous. See yak-c393.5 for the full diagnosis.
    return status.capitalize()


# Priority -> SGR foreground codes, matching curses pairs C_P1..C_P5.
_PRIORITY_FG = {1: FG_RED, 2: FG_MAGENTA, 3: FG_YELLOW, 4: FG_GREEN, 5: FG_BLUE}

# Structural styles (ANSI equivalents of the curses color pairs in colors.py).
S_TAB_ACTIVE = sgr(30, 47, BOLD)  # black on white, bold (C_TAB_ACTIVE)
S_TAB_INACTIVE = sgr(DIM)
S_SEL_BG = bg256(237)  # selected-row background (C_SELECTED)
S_DIVIDER = sgr(DIM)
S_PANE_HEAD = sgr(DIM, BOLD)
S_MUTED = fg256(245)

# Floating annotation card (a tinted panel that occludes what's beneath it).
S_CARD_BG = bg256(238)
S_CARD_TEXT = bg256(238) + fg256(253)
S_CARD_TITLE = bg256(238) + sgr(FG_CYAN, BOLD)
S_CARD_ACCENT = bg256(38)  # cyan spine down the left edge

# Occlusion focus model: the agent is laid out at a FIXED width and the board
# slides OVER it. board_x is the board's left edge; as it decreases the board
# covers more of the agent. The agent never reflows — it is simply clipped at
# the moving divider, so narrowing can't produce the vertical letter-stacking
# that reflowing to a tiny width did.
AGENT_LAYOUT_W = 52          # fixed agent layout width (never reflows)
BOARD_SPLIT_X = AGENT_LAYOUT_W + 2   # board left edge when the agent is fully shown
BOARD_FULL_X = 0             # board covers the whole terminal (agent hidden)

# Bottom help-bar key strips, mirroring draw_help_bar in render.py.

S_AGENT = {
    "user": sgr(FG_GREEN, BOLD),
    "assistant": sgr(FG_CYAN, BOLD),
    "tool": sgr(FG_YELLOW),
}


def _style(codes: tuple[int, ...], *, dim: bool = False, sel: bool = False) -> str:
    parts = list(codes)
    if dim:
        parts.append(DIM)
    s = sgr(*parts) if parts else ""
    if sel:
        s = S_SEL_BG + s
    return s


def _ghost_badge_style(status: str) -> str:
    # Mirrors colors.ghost_badge_attr: hairy=yellow+bold, shorn=green, else dim.
    if status == HAIRY:
        return sgr(FG_YELLOW, BOLD)
    if status == SHORN:
        return sgr(FG_GREEN)
    return sgr(DIM)


# --- board model -----------------------------------------------------------


@dataclass
class Yak:
    id: str
    title: str
    type: str = "task"
    priority: int = 3
    status: str = HAIRY
    labels: tuple[str, ...] = ()
    blocked: bool = False
    depends_on: tuple[str, ...] = ()
    source: str | None = None
    created: str | None = None
    updated: str | None = None
    commit: str | None = None
    description: str = ""


class Board:
    """Ordered collection of yaks with status transitions, mirroring how the
    real board groups by status directory + nests by dotted id."""

    def __init__(self) -> None:
        self._yaks: dict[str, Yak] = {}

    def add(self, id: str, title: str, **kw) -> Yak:
        y = Yak(id=id, title=title, **kw)
        self._yaks[id] = y
        return y

    def get(self, id: str) -> Yak:
        return self._yaks[id]

    def move(self, id: str, status: str) -> None:
        self._yaks[id].status = status

    def set(self, id: str, **kw) -> None:
        y = self._yaks[id]
        for k, v in kw.items():
            setattr(y, k, v)

    def counts(self) -> dict[str, int]:
        c = {s: 0 for s in TAB_STATUSES}
        for y in self._yaks.values():
            if y.status in c:
                c[y.status] += 1
        return c

    def blocked_ids(self) -> set[str]:
        """Hairy yaks flagged blocked — the TUI's overlay set (app.blocked_ids)."""
        return {y.id for y in self._yaks.values() if y.blocked and y.status == HAIRY}


def _child_status_rank(status: str) -> int:
    if status == SHAVING:
        return 0
    if status == SHORN:
        return 2
    return 1


def _yak_to_task(y: Yak) -> dict:
    """Project a demo Yak into the frontmatter-style dict the shared builders
    (build_tree / build_detail_lines) expect."""
    return {
        "id": y.id,
        "title": y.title,
        "type": y.type,
        "priority": y.priority,
        "labels": list(y.labels),
        "depends_on": list(y.depends_on),
        "created": y.created or "",
        "updated": y.updated or "",
        "commit": y.commit,
        "source": y.source,
        "description": y.description,
    }


class BoardRepo:
    """TaskRepo (yaklib.repo) over an in-memory Board, so the demo can drive the
    real view-model builders with no filesystem."""

    def __init__(self, board: Board):
        self.board = board

    def all_tasks(self) -> list[tuple[str, dict]]:
        return [(y.status, _yak_to_task(y)) for y in self.board._yaks.values()]

    def resolved_ids(self) -> set[str]:
        return {y.id for y in self.board._yaks.values() if y.status == SHORN}

    def find(self, task_id: str):
        y = self.board._yaks.get(task_id)
        return (y.status, _yak_to_task(y)) if y else None

    def children(self, task_id: str) -> list[tuple[str, dict]]:
        kids = [y for y in self.board._yaks.values() if parent_id(y.id) == task_id]
        kids.sort(key=lambda y: (_child_status_rank(y.status), y.priority, y.id))
        return [(y.status, _yak_to_task(y)) for y in kids]

    def resolve_link_spans(self, text: str, self_id: str):
        from yaklib.links import find_link_spans
        return [(s, e, tid) for (s, e, tid) in find_link_spans(text)
                if tid != self_id and tid in self.board._yaks]

    def artifacts(self, task_id: str, body: str):
        return []


def board_reverse_deps(board: Board) -> dict[str, list[tuple[str, dict]]]:
    """{dep_id: [(status, task), ...]} for the detail pane's Blocks section."""
    rev: dict[str, list[tuple[str, dict]]] = {}
    for y in board._yaks.values():
        for dep in y.depends_on:
            rev.setdefault(dep, []).append((y.status, _yak_to_task(y)))
    return rev


# --- rendering: tabs / list ------------------------------------------------


def render_tabs(screen: Screen, x0: int, board: Board, active_status: str, y0: int) -> None:
    counts = board.counts()
    x = x0
    for status in TAB_STATUSES:
        text = f" {_tab_label(status)} ({counts[status]}) "
        style = S_TAB_ACTIVE if status == active_status else S_TAB_INACTIVE
        screen.put(y0, x, text, style)
        x += text_width(text) + 1


def render_list(
    screen: Screen,
    x0: int,
    width: int,
    rows: list[tuple[str, dict, int, bool]],
    blocked_ids: set[str],
    cursor_id: str | None,
    y0: int,
) -> None:
    """Paint the shared build_tree output: (status, task, depth, ghost) rows.

    Mirrors the geometry + colors of yaktui/render.py draw_list, but the *data*
    (which rows, depth, ghost) comes from the shared builder.
    """
    if not rows:
        screen.put(y0, x0 + 2, "No yaks.", sgr(DIM))
        return
    id_col = max([4] + [len(t["id"]) + d * 2 for _, t, d, _ in rows]) + 1
    for i, (status, task, depth, ghost) in enumerate(rows):
        tid = task["id"]
        pri = task.get("priority", "-")
        ttype = task.get("type", "-")
        title = task.get("title", "")
        labels = task.get("labels") or []
        blocked = tid in blocked_ids and status == HAIRY

        row = y0 + i
        selected = tid == cursor_id
        if selected:
            screen.fill(row, x0, width, " ", S_SEL_BG)

        indent = "  " * depth
        x = x0
        lead = "*" if blocked else " "
        id_text = f"{lead}{indent}{tid}".ljust(id_col + 1)
        if blocked and not selected:
            screen.put(row, x, lead, sgr(FG_MAGENTA, BOLD))
            screen.put(row, x + 1, id_text[1:], _style((FG_BLUE,), dim=ghost))
        else:
            screen.put(row, x, id_text, _style((FG_BLUE,), dim=ghost, sel=selected))
        x += len(id_text)

        pri_text = f"p{pri} "
        screen.put(row, x, pri_text, _style((_PRIORITY_FG.get(pri, FG_YELLOW),), dim=ghost, sel=selected))
        x += len(pri_text)

        type_text = f"{ttype:8s} "
        screen.put(row, x, type_text, _style((FG_CYAN,), dim=ghost, sel=selected))
        x += len(type_text)

        # Right side: ghost status badge (single-width ASCII) + labels to its left.
        badge = f" {status_char(status)}" if ghost else ""
        label_str = "[" + ", ".join(labels) + "]" if labels else ""

        right_w = text_width(badge) + (text_width(label_str) + 1 if label_str else 0)
        avail = width - (x - x0) - 1 - right_w
        if avail > 0:
            screen.put_clipped(row, x, title, avail, _style((), dim=ghost, sel=selected))
        if label_str:
            lx = x0 + width - 1 - text_width(badge) - text_width(label_str)
            if lx > x:
                screen.put(row, lx, label_str, _style((FG_MAGENTA, DIM), sel=selected))
        if badge:
            bx = x0 + width - text_width(badge) - 1
            if bx > x:
                screen.put(row, bx, badge, _ghost_badge_style(status))


# --- rendering: detail pane ------------------------------------------------


# Style per DetailLine.kind, mirroring draw_detail in yaktui/render.py.
_DETAIL_STYLE = {
    "header": sgr(FG_WHITE, BOLD),
    "subheader": sgr(FG_WHITE, BOLD),
    "field": "",
    "link": sgr(FG_BLUE),
    "dep_link": sgr(FG_BLUE),
    "desc": sgr(DIM),
    "code": sgr(FG_CYAN),
    "md_heading": sgr(FG_WHITE, BOLD),
    "quote": sgr(DIM, ITALIC),
    "": "",
}


def _wrap(text: str, width: int) -> list[str]:
    if width <= 10 or len(text) <= width:
        return [text]
    stripped = text.lstrip(" ")
    lead = text[: len(text) - len(stripped)]
    words, out, cur = stripped.split(), [], ""
    wrap_w = max(1, width - len(lead))
    for w in words:
        if not cur:
            cur = w
        elif len(cur) + 1 + len(w) <= wrap_w:
            cur += " " + w
        else:
            out.append(lead + cur)
            cur = w
    if cur:
        out.append(lead + cur)
    return out or [text]


def render_detail(
    screen: Screen,
    x0: int,
    width: int,
    lines: list[DetailLine],
    y0: int,
    height: int,
    cursor_line: int | None = None,
) -> None:
    """Paint a list of the real yaktui DetailLine objects."""
    for i in range(min(height, len(lines))):
        dl = lines[i]
        row = y0 + i
        has_inline = bool(dl.links)
        whole_line_link = (dl.task_id is not None or dl.open_path is not None) and not has_inline

        if cursor_line is not None and i == cursor_line:
            screen.fill(row, x0, width, " ", S_SEL_BG)
            style = (S_SEL_BG + sgr(FG_BLUE, BOLD)) if whole_line_link else (S_SEL_BG + sgr(BOLD))
            screen.put_clipped(row, x0, dl.text, width, style)
            continue

        base = sgr(FG_BLUE) if whole_line_link else _DETAIL_STYLE.get(dl.kind, "")
        screen.put_clipped(row, x0, dl.text, width, base)
        # Inline yak-id mentions in description text render underlined.
        for start, end, _tid in dl.links:
            if start < width:
                screen.put(row, x0 + start, dl.text[start:end], sgr(FG_BLUE, UNDERLINE))


# --- rendering: agent pane -------------------------------------------------


def render_agent(
    screen: Screen,
    layout_w: int,
    visible_w: int,
    messages: list[tuple[str, str]],
    y0: int,
    height: int,
) -> None:
    """Render a chat transcript, bottom-anchored, at column 0.

    Text is wrapped to the FIXED *layout_w* and then painted clipped to
    *visible_w*. That separation is the occlusion trick: as the board slides
    over the agent, only *visible_w* shrinks — the wrap (and therefore the line
    breaks) never change, so nothing reflows. Keep the content ASCII: East
    Asian Ambiguous glyphs (em dash, curly quotes, box angles) can be sized
    width-2 by the player's terminal and drift into the divider.
    """
    lines: list[tuple[str, str]] = []
    for role, text in messages:
        style = S_AGENT.get(role, "")
        head = "  " if role == "tool" else f"{role}  "
        wrap_w = max(4, layout_w - len(head))
        for j, seg in enumerate(_wrap_words(text, wrap_w)):
            if j == 0:
                lines.append((style, head + seg))
            else:
                pad = " " * len(head)
                lines.append((S_MUTED if role == "tool" else "", pad + seg))
        lines.append(("", ""))

    visible = lines[-height:] if len(lines) > height else lines
    for i, (style, text) in enumerate(visible):
        screen.put_clipped(y0 + i, 0, text, visible_w, style)


def _wrap_words(text: str, width: int) -> list[str]:
    words, out, cur = text.split(), [], ""
    for w in words:
        if not cur:
            cur = w
        elif len(cur) + 1 + len(w) <= width:
            cur += " " + w
        else:
            out.append(cur)
            cur = w
    if cur:
        out.append(cur)
    return out or [""]


# --- layout ----------------------------------------------------------------


OVERLAY_W = 36  # narrow-but-tall floating card


def render_overlay(
    screen: Screen,
    cols: int,
    rows: int,
    text: str,
    anchor: str,
    board_x: int,
) -> None:
    """Draw a floating caption card that narrates a beat.

    It's a tinted panel (with a cyan left spine) that occludes what's beneath,
    positioned near the area it's talking about: 'agent' floats over the left
    pane, 'board' over the board pane, 'center' spans the middle.
    """
    inner = OVERLAY_W - 5
    wrapped = _wrap(text, inner)
    height = len(wrapped) + 2
    y = rows - height - 2

    if anchor == "agent":
        region_l, region_r = 0, max(2, board_x - 1)
    elif anchor == "board":
        region_l, region_r = (board_x if board_x >= 8 else 0), cols
    else:  # center
        region_l, region_r = 0, cols
    x = region_l + max(2, (region_r - region_l - OVERLAY_W) // 2)
    x = max(1, min(x, cols - OVERLAY_W - 1))

    for r in range(y, y + height):
        screen.fill(r, x, OVERLAY_W, " ", S_CARD_BG)
        screen.put(r, x, " ", S_CARD_ACCENT)
    for i, ln in enumerate(wrapped):
        screen.put(y + 1 + i, x + 3, ln, S_CARD_TITLE if i == 0 else S_CARD_TEXT)


def render_board_pane(
    screen: Screen,
    x0: int,
    width: int,
    top: int,
    height: int,
    board: Board,
    active_status: str,
    cursor_id: str | None,
    detail_id: str | None,
    detail_cursor: int | None,
    mode: str,
) -> None:
    """Tabs on top, then list, full-pane detail, or the real list|detail split.

    The list tree and detail lines come from the SHARED builders
    (yaktui.tree.build_tree / yaktui.detail.build_detail_lines) via a BoardRepo,
    so content stays in lockstep with the TUI; only the painting is local.
    """
    repo = BoardRepo(board)
    tree_rows = build_tree(
        None, active_status, FilterSpec(),
        tasks_cache=repo.all_tasks(), resolved_cache=repo.resolved_ids(),
    )
    blocked = board.blocked_ids()

    def detail_lines_for(tid: str, w: int) -> list[DetailLine]:
        status, task = repo.find(tid)
        return build_detail_lines(
            repo, task, status, w,
            reverse_deps=board_reverse_deps(board), status_glyph=status_char,
        )

    render_tabs(screen, x0, board, active_status, top)
    content_y = top + 2
    content_h = height - 2
    bottom = top + height

    split = detail_id is not None and (mode == "split" or (mode == "auto" and width >= 84))
    if split:
        list_w = max(24, width // 3)
        sep_x = x0 + list_w
        det_x = sep_x + 2
        det_w = width - (det_x - x0)
        render_list(screen, x0, list_w - 1, tree_rows, blocked, cursor_id, content_y)
        # Separator runs the content height only — the tabs span full width.
        screen.vline(sep_x, top + 1, bottom, "\u2502", S_DIVIDER)
        render_detail(screen, det_x, det_w - 1, detail_lines_for(detail_id, det_w - 1), content_y, content_h, detail_cursor)
    elif detail_id is not None:
        render_detail(screen, x0, width - 1, detail_lines_for(detail_id, width - 1), content_y, content_h, detail_cursor)
    else:
        render_list(screen, x0, width - 1, tree_rows, blocked, cursor_id, content_y)


@dataclass
class Layout:
    """Two-pane composition with an occluding focus divider.

    The agent transcript is laid out at a fixed width; the board slides over it
    from the right. board_x (the board's left edge) is supplied per-frame: at
    BOARD_SPLIT_X the agent is fully shown, at BOARD_FULL_X (0) the board covers
    the whole terminal (the real app view). Intermediate values just clip the
    agent — it never reflows.
    """

    cols: int
    rows: int
    board_split_x: int = BOARD_SPLIT_X

    def compose(
        self,
        messages: list[tuple[str, str]],
        board: Board,
        active_status: str,
        cursor_id: str | None = None,
        *,
        board_x: int | None = None,
        detail_id: str | None = None,
        detail_cursor: int | None = None,
        board_mode: str = "auto",
        annotation: str | None = None,
        annotation_anchor: str = "board",
    ) -> Screen:
        cols, rows = self.cols, self.rows
        bx = self.board_split_x if board_x is None else board_x
        bx = max(0, min(bx, cols))
        agent_visible = bx >= 8

        screen = Screen(cols, rows)
        body_h = rows  # no help bar — content uses the full height

        if agent_visible:
            div_x = bx - 1
            render_agent(screen, AGENT_LAYOUT_W, div_x - 1, messages, 0, body_h)
            screen.vline(div_x, 0, rows, "\u2502", S_DIVIDER)
            board_x0 = bx
        else:
            board_x0 = 0

        render_board_pane(
            screen, board_x0, cols - board_x0, 0, body_h, board,
            active_status, cursor_id, detail_id, detail_cursor, board_mode,
        )

        if annotation:
            render_overlay(screen, cols, rows, annotation, annotation_anchor, board_x0)
        return screen
