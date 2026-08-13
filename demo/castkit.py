"""asciicast v2 builder + a tiny virtual screen for composing terminal frames.

No third-party dependencies. The output is a ``.cast`` file (asciicast v2,
line-delimited JSON) playable by asciinema-player — the same player we vendor
under ``docs/vendor/``.

Design notes
------------
* We *generate* the cast deterministically rather than recording a real
  terminal. Timestamps are explicit, so "the agent thinks for 20s" can be
  compressed to a 0.4s beat while the interesting state changes hold on screen.
* Frames are full-screen repaints. That keeps the emitter dead simple and
  robust; casts stay tiny because the content is small (a repaint is a few KB,
  gzipped away by the browser).
* The virtual :class:`Screen` is a display-column grid that understands
  double-width glyphs (emoji), so the yaks board and the agent pane stay
  aligned in the terminal emulator that asciinema-player runs.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field

# --- ANSI / SGR helpers ----------------------------------------------------

RESET = "\x1b[0m"


def sgr(*codes: int) -> str:
    """Build a Select-Graphic-Rendition escape from raw SGR codes."""
    if not codes:
        return ""
    return "\x1b[" + ";".join(str(c) for c in codes) + "m"


# Foreground base colors (match the curses palette in yaktui/colors.py).
FG_BLACK, FG_RED, FG_GREEN, FG_YELLOW = 30, 31, 32, 33
FG_BLUE, FG_MAGENTA, FG_CYAN, FG_WHITE = 34, 35, 36, 37
BOLD, DIM, ITALIC, UNDERLINE = 1, 2, 3, 4


def fg256(n: int) -> str:
    return f"\x1b[38;5;{n}m"


def bg256(n: int) -> str:
    return f"\x1b[48;5;{n}m"


# --- display width ---------------------------------------------------------

# A code point that continues a wide glyph occupies this sentinel in the grid;
# the renderer emits nothing for it (the wide glyph already advanced the
# emulator's cursor by two columns).
_CONT = "\x00"


def char_width(ch: str) -> int:
    """Approximate terminal display width of a single code point.

    Good enough for our content: emoji + dingbats render double-width, the
    variation selector is zero-width, everything else is single-width.
    """
    o = ord(ch)
    if o == 0xFE0F:  # VARIATION SELECTOR-16 (emoji presentation)
        return 0
    if 0x1F000 <= o <= 0x1FAFF:  # emoji blocks
        return 2
    if 0x2600 <= o <= 0x27BF:  # misc symbols + dingbats (e.g. scissors ✂)
        return 2
    if 0x1100 <= o <= 0x115F or 0x2E80 <= o <= 0xA4CF or 0xAC00 <= o <= 0xD7A3:
        return 2  # CJK / Hangul ranges
    return 1


def text_width(s: str) -> int:
    return sum(char_width(c) for c in s)


# --- virtual screen --------------------------------------------------------


class Screen:
    """A grid of (glyph, style) cells addressed by display row/column."""

    def __init__(self, cols: int, rows: int):
        self.cols = cols
        self.rows = rows
        self.cells: list[list[tuple[str, str]]] = []
        self.clear()

    def clear(self, style: str = "") -> None:
        self.cells = [[(" ", style) for _ in range(self.cols)] for _ in range(self.rows)]

    def fill(self, row: int, col: int, width: int, ch: str = " ", style: str = "") -> None:
        for c in range(col, min(col + width, self.cols)):
            if 0 <= row < self.rows and 0 <= c:
                self.cells[row][c] = (ch, style)

    def put(self, row: int, col: int, text: str, style: str = "") -> int:
        """Write *text* at (row, col); returns the next free column.

        Handles double-width glyphs by reserving a continuation cell so later
        writes and pane dividers stay column-aligned.
        """
        if not (0 <= row < self.rows):
            return col
        c = col
        last_base = -1  # column of the last real glyph written (for combining marks)
        for ch in text:
            if ch in ("\x1b",):  # never let raw escapes into the grid
                continue
            w = char_width(ch)
            if w == 0:
                # Zero-width mark (e.g. variation selector): fold onto the base
                # glyph, NOT c-1 — for a preceding wide glyph, c-1 is its _CONT
                # continuation cell, which must stay exactly _CONT.
                if 0 <= last_base < self.cols:
                    pch, pst = self.cells[row][last_base]
                    self.cells[row][last_base] = (pch + ch, pst)
                continue
            if 0 <= c < self.cols:
                self.cells[row][c] = (ch, style)
                last_base = c
            if w == 2 and 0 <= c + 1 < self.cols:
                self.cells[row][c + 1] = (_CONT, style)
            c += w
        return c

    def put_clipped(self, row: int, col: int, text: str, maxw: int, style: str = "") -> int:
        """Like :meth:`put` but truncates to *maxw* display columns."""
        out = []
        used = 0
        for ch in text:
            w = char_width(ch)
            if used + w > maxw:
                break
            out.append(ch)
            used += w
        return self.put(row, col, "".join(out), style)

    def vline(self, col: int, row0: int, row1: int, ch: str = "│", style: str = "") -> None:
        for r in range(row0, min(row1, self.rows)):
            if 0 <= col < self.cols:
                self.cells[r][col] = (ch, style)

    def render_frame(self) -> str:
        """Emit the full frame as ANSI.

        Every run of same-styled cells is positioned with *absolute* cursor
        addressing (CUP for the row, CHA for the column). This is the key to
        stable structure: a glyph whose display width the emulator disagrees
        with — or a write that lands on the right margin and auto-wraps — can
        no longer cascade and shove the divider (or any later column) out of
        place, because the next run re-anchors to a known column regardless.

        Cells are 1:1 with display columns (a double-width glyph reserves a
        continuation cell), so a cell's index *is* its column.
        """
        out: list[str] = []
        for r in range(self.rows):
            row = self.cells[r]
            # Anchor to the row, clear stray style, wipe the whole line.
            out.append(f"\x1b[{r + 1};1H{RESET}\x1b[2K")
            c = 0
            while c < self.cols:
                if row[c][0] == _CONT:
                    c += 1
                    continue
                start = c
                run_style = row[c][1]
                buf: list[str] = []
                while c < self.cols and row[c][1] == run_style:
                    if row[c][0] != _CONT:
                        buf.append(row[c][0])
                    c += 1
                text = "".join(buf)
                if not run_style and not text.strip(" "):
                    continue  # nothing to paint over an already-cleared line
                out.append(f"\x1b[{start + 1}G")
                if run_style:
                    out.append(run_style)
                out.append(text)
                out.append(RESET)
        return "".join(out)


# --- cast document ---------------------------------------------------------


@dataclass
class Cast:
    cols: int
    rows: int
    title: str | None = None
    idle_limit: float = 2.0
    t: float = 0.0
    events: list = field(default_factory=list)

    def __post_init__(self) -> None:
        # Hide the cursor and clear once; every frame just repaints from home.
        self.events.append([0.0, "o", "\x1b[?25l\x1b[2J\x1b[H"])

    def wait(self, seconds: float) -> "Cast":
        self.t += seconds
        return self

    def frame(self, screen: Screen) -> "Cast":
        self.events.append([round(self.t, 3), "o", screen.render_frame()])
        return self

    def marker(self, label: str) -> "Cast":
        self.events.append([round(self.t, 3), "m", label])
        return self

    def to_json(self) -> str:
        header = {
            "version": 2,
            "width": self.cols,
            "height": self.rows,
            "idle_time_limit": self.idle_limit,
            "env": {"TERM": "xterm-256color"},
        }
        if self.title:
            header["title"] = self.title
        lines = [json.dumps(header)]
        lines.extend(json.dumps(e, ensure_ascii=False) for e in self.events)
        return "\n".join(lines) + "\n"

    def write(self, path: str) -> None:
        with open(path, "w", encoding="utf-8") as f:
            f.write(self.to_json())
