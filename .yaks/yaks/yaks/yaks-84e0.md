---
id: yaks-84e0
title: Use the same internal navigation affordances in text/md viewing as in yak details
status: shaving
type: task
priority: 3
created: '2026-10-22T19:53:58Z'
updated: '2026-11-01T22:25:17Z'
labels:
- tui
---

Ie, the ability to move the line-cursor, search, tab among links, etc. Basically everything but going into edit mode, and any yak-detail specific affordances I'm not thinking of.

Hopefully this can share most of the code with the yak detail rendering, so that fixes and improvements apply to both.

---
▸ 2026-11-01T22:25:16Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T22:25:17Z [delta:farm-wiki]

Claim (yakherd delta:farm-wiki): worker `nav-1`, on branch `wiki`, base a150387.

This is the prerequisite for the farm-wiki TUI layer (yaks-200f): the wiki tab (yaks-ebb2) and link following (yaks-7395) open pages in this viewer.

**Task.** The TUI's text/markdown viewer (the one that opens a text attachment) gets the yak detail pane's navigation, by sharing its code rather than copying it:

- line cursor movement;
- search;
- Tab/Shift-Tab among links;
- following a link with the same keys as the detail pane.

Leave out edit mode and anything specific to yak details (notes, fields).

**Scope.**

- The text viewer and the detail-navigation code under `src/tui/` (`detail_nav.rs`, `detail.rs`, the viewer's module, `handlers.rs` where keys dispatch, `render.rs` for its frame).
- Factor the shared part out of the detail pane into one place both use.
- Update the `?` help overlay (`help_content`) and `docs/tui.md`.

**Out of scope.** Anything outside `src/tui/` and `docs/tui.md`. Ask first.

**Evidence.**

- Insta snapshot or headless toque tests: in the viewer, the cursor moves, search finds a match, and Tab moves to the next link.
- Existing detail-pane tests unchanged.
- Attach a rendered frame (a docshots scene or a toque SVG) of the viewer with the cursor on a link.
- The full gate passes.

**Judge.** The coordinator, at landing.
