---
id: yaks-84e0
title: Use the same internal navigation affordances in text/md viewing as in yak details
status: shaving
type: task
priority: 3
created: '2026-10-22T19:53:58Z'
updated: '2026-11-01T22:45:57Z'
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

---
▸ 2026-11-01T22:26:42Z [nav-1]

Start Sun Nov  1 22:26:41 UTC 2026. Path: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-84e0.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/j5qrbk8fnvyz/yaks (branch wiki, base a150387).

---
▸ 2026-11-01T22:45:57Z [nav-1]

Decision: the shared navigation is a new module `src/tui/linenav.rs`, a cursor/search/link model over rendered rows (`[detail::DLine]`):
- pure functions over the rows: cursor stops/step/top-bottom, next link, link at cursor, find scan, yank text (source markdown for rendered rows), sideways scroll of no-wrap blocks, link destination for the status bar;
- `Viewport { page, pinned }`: the scroll geometry (rows shown, max scroll, scroll-into-view); the detail pane has a 2-row pinned header, the viewer none;
- `LineNavMut<'a>`: borrowed cursor state (line, scroll, anchor, find, match) with the stateful ops (step, Tab links, visual select, n/N, the find box's key routing). The detail pane borrows App's existing `detail_*` fields; the viewer borrows its own;
- `LineNav`: owned state (plus sideways scroll and an in-page back/forward stack) for hosts that own one page: the text viewer now, the wiki tab next (yaks-200f).
Why borrowed rather than moving App's `detail_*` fields into a `LineNav`: ~100 test lines read/write `app.detail_line` etc. and the claim requires existing detail tests unchanged. Folding App's fields into a `LineNav` is a mechanical follow-up.
