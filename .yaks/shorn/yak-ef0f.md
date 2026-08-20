---
id: yak-ef0f
title: TUI / terminal rendering libraries
type: task
priority: 3
created: '2026-08-16T23:19:19Z'
updated: '2026-08-16T23:23:49Z'
parent: yak-2219
labels:
- rust
---

Survey Rust TUI stack (ratatui + backends) to replace the curses TUI in tui.py / yaktui.*. Immediate-mode vs retained, styling, input, resize, force-repaint (cf yak-be65).

---
▸ 2026-08-16T23:23:49Z
FINDINGS (versions verified 2026-08 via lib.rs).

ratatui 0.30.2 (Jun 2026) is the de-facto standard TUI crate: 5.4M downloads/mo, ~5.9k dependents, crossterm backend by default. Immediate-mode: you redraw the whole UI each frame and ratatui diffs it against a back buffer. That maps almost 1:1 onto our existing full-repaint model (render.py / castkit render_frame) and, crucially, MAKES yak-be65 (force-repaint) moot — there are no stale cells to force, every frame is reconciled from scratch.

Backend = crossterm (pure Rust, cross-platform). This REMOVES the ncurses C dependency entirely (curses in tui.py / editor.py / vim_edit.py). crossterm gives raw mode, full key events with modifiers, resize events, bracketed paste, and DECSCUSR cursor shapes (we already hand-emit those in vim_edit.set_cursor_shape).

Coverage of our TUI surface: Layout (constraint solver) for the board/detail split; Style/Color incl 256 + truecolor maps onto yaktui/colors.py; widgets List (board), Paragraph with wrap+scroll (detail), Block/borders, Tabs (status tabs), Table, Gauge. tui-tree-widget covers yaktui/tree.py (the hierarchy view); tui-scrollview, tui-input exist too. The modal keymaps (keys_list.py / keys_detail.py) become a match over crossterm KeyEvent — mechanical translation.

TestBackend renders to an in-memory Buffer (cells+style) with no real terminal — this is the hook for snapshot tests and for the demo pipeline (see yak-199b): one painter can drive both the live TUI and the cast frames, which is exactly yak-3291.

SCOPE this replaces: scripts/tui.py (1183) + yaktui/render.py (766) + colors/tree/detail/dialogs/mutate = the bulk of the ~3.6k-loc TUI half. This is the largest and riskiest part of a port, but also where the real UX wins land (see yak-6819 editor, yak-be65 repaint).

RECOMMENDATION: ratatui + crossterm, immediate-mode, reusing the current full-repaint mental model. No serious alternative for our needs (cursive is ncurses-based; iocraft is declarative/younger).
