#!/usr/bin/env python3
"""Build the embedded terminal demo cast (asciicast v2) for the docs site.

    python3 demo/build_demo.py            # writes docs/demo.cast
    python3 demo/build_demo.py -o out.cast

A *build tool*, not part of the shipped package. It deterministically emits a
.cast by driving the virtual screen in yakscreen.py — no tmux, no asciinema
recorder, no real terminal. Re-run whenever the story or board styling changes.

NOTE: the screenplay below is an intentionally-simplified PLACEHOLDER that
exercises the rendering tools (variable focus divider, tree list with ghosts,
board-focused list|detail split, help bar). The full "herd evolving over time"
story lands in a follow-up.
"""

from __future__ import annotations

import argparse
import pathlib

from castkit import Cast
from yakscreen import (
    FOCUS_AGENT,
    FOCUS_BALANCED,
    FOCUS_BOARD,
    HAIRY,
    SHAVING,
    SHORN,
    Board,
    Layout,
)

COLS, ROWS = 120, 38


class Director:
    """Thin choreography layer over Cast + Layout + Board."""

    def __init__(self, cast: Cast, layout: Layout):
        self.cast = cast
        self.layout = layout
        self.board = Board()
        self.messages: list[tuple[str, str]] = []
        self.active = HAIRY
        self.cursor: str | None = None
        self.detail_id: str | None = None
        self.detail_cursor: int | None = None
        self.frac = FOCUS_BALANCED
        self.board_mode = "auto"

    def _frame(self, extra_last: str | None = None) -> None:
        msgs = self.messages
        if extra_last is not None and msgs:
            msgs = msgs[:-1] + [(msgs[-1][0], extra_last)]
        screen = self.layout.compose(
            msgs,
            self.board,
            self.active,
            self.cursor,
            agent_frac=self.frac,
            detail_id=self.detail_id,
            detail_cursor=self.detail_cursor,
            board_mode=self.board_mode,
        )
        self.cast.frame(screen)

    def beat(self, seconds: float) -> None:
        self.cast.wait(seconds)

    def mark(self, label: str) -> None:
        self.cast.marker(label)

    # --- focus (variable divider) ---
    def focus_to(self, target: float, *, steps: int = 8, dt: float = 0.035) -> None:
        """Slide the divider from the current focus to *target* over N frames."""
        start = self.frac
        if abs(target - start) < 1e-6:
            self._frame()
            return
        for i in range(1, steps + 1):
            self.frac = start + (target - start) * i / steps
            self._frame()
            self.cast.wait(dt)
        self.frac = target

    # --- chat ---
    def say(self, role: str, text: str, *, typing: bool = False, hold: float = 0.7) -> None:
        self.messages.append((role, text if not typing else ""))
        if typing:
            partial = ""
            for w in text.split():
                partial = (partial + " " + w).strip()
                self._frame(extra_last=partial)
                self.cast.wait(0.08)
            self.messages[-1] = (role, text)
        self._frame()
        self.cast.wait(hold)

    def tool(self, cmd: str, hold: float = 0.6) -> None:
        self.say("tool", cmd, hold=hold)

    # --- board ---
    def tab(self, status: str) -> None:
        self.active = status

    def add(self, id: str, title: str, **kw) -> None:
        self.board.add(id, title, **kw)

    def move(self, id: str, status: str) -> None:
        self.board.move(id, status)

    def focus_yak(self, id: str | None) -> None:
        self.cursor = id

    def detail(self, id: str | None, cursor: int | None = None) -> None:
        self.detail_id = id
        self.detail_cursor = cursor


def screenplay() -> Cast:
    cast = Cast(COLS, ROWS, title="yaks — agent + board in lockstep")
    layout = Layout(COLS, ROWS, agent_frac=FOCUS_BALANCED)
    d = Director(cast, layout)

    # Seed an existing herd so the tree + ghosts have something to show.
    d.add("yak-7c31", "rate-limit the webhook intake", type="bug", priority=2)
    d.add("yak-4e08", "dark-mode polish pass", type="task", priority=4, labels=("ui",))
    d.tab(HAIRY)
    d._frame()
    d.beat(1.0)

    # The ask — lean toward the agent while the human + agent talk it through.
    d.mark("The ask")
    d.focus_to(FOCUS_AGENT)
    d.say("user", "Add OAuth login to the settings page.", typing=True)
    d.say("assistant", "Bigger than one commit — let me break it into a small herd first.")

    d.add("yak-1a2b", "add OAuth login to settings", type="feature", priority=2,
          created="2026-07-25T20:00:00Z", updated="2026-07-25T20:00:00Z",
          description="Support Google + GitHub OAuth from the settings page.")
    d.add("yak-1a2b.1", "provider config + secrets", type="task", priority=2, status=SHAVING,
          created="2026-07-25T20:01:00Z", updated="2026-07-25T20:20:00Z")
    d.add("yak-1a2b.2", "settings toggle + callback route", type="task", priority=2,
          depends_on=("yak-1a2b.1",), blocked=True,
          created="2026-07-25T20:01:00Z", updated="2026-07-25T20:01:00Z")
    d.focus_yak("yak-1a2b")

    # Shave — back to balanced so the board move is visible.
    d.mark("Shave")
    d.tool("$ yaks shave yak-1a2b")
    d.move("yak-1a2b", SHAVING)
    d.tab(SHAVING)
    d.focus_to(FOCUS_BALANCED)
    d.beat(0.7)

    # Detail — hand the whole screen to the board: real tabs + list|detail split.
    d.mark("Detail")
    d.say("assistant", "Here's the breakdown — .2 is blocked on the provider config.")
    d.detail("yak-1a2b")
    d.focus_to(FOCUS_BOARD)
    d.beat(2.2)

    # Unblock — resolve the dependency in place.
    d.mark("Unblock")
    d.move("yak-1a2b.1", SHORN)
    d.board.set("yak-1a2b.2", blocked=False)
    d._frame()
    d.beat(1.6)

    # Shorn — come back to balanced for the closing line.
    d.mark("Shorn")
    d.detail(None)
    d.move("yak-1a2b.2", SHORN)
    d.move("yak-1a2b", SHORN)
    d.tab(SHORN)
    d.focus_yak("yak-1a2b")
    d.focus_to(FOCUS_BALANCED)
    d.say("assistant", "Herd shorn — login works and the toggle persists.", hold=2.2)

    return cast


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    default_out = pathlib.Path(__file__).resolve().parent.parent / "docs" / "demo.cast"
    ap.add_argument("-o", "--output", default=str(default_out))
    args = ap.parse_args()

    cast = screenplay()
    cast.write(args.output)
    print(f"wrote {args.output}  ({len(cast.events)} events, {cast.t:.1f}s)")


if __name__ == "__main__":
    main()
