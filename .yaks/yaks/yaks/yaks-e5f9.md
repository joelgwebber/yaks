---
id: yaks-e5f9
title: Image zoom affordances
status: shaving
type: feature
priority: 3
created: '2026-11-03T23:29:24Z'
updated: '2026-11-07T17:07:12Z'
tags:
- tui
---

Would be nice to be able to zoom in/out of an image when it's open in the detail window. Zoom in/out, native size, and fit-width affordances.

---
▸ 2026-11-07T17:04:42Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-07T17:05:34Z [herd-stars]

Claim (yakherd herd-stars): worker `stars-zoom`.

**Scope.** Zoom in the pane's image viewer (`src/tui/imageview.rs`, the `Image` rows of `src/tui/keymap.rs`, its render, its bottom line):

- `+` (and `=`) zoom in, `-` zoom out, by steps; `0` actual size (one image pixel per pixel of the cell grid's resolution, as near as the backend allows); `w` fit width; `f` fit the pane (today's default, shrink to fit).
- When the image is larger than the pane, `h`/`j`/`k`/`l` and the arrows pan it.
- `h`/`←` no longer close the image viewer: `Esc` does. This follows Joel's answer on yaks-7c6e (Enter goes in, Esc comes out; `h`/`l` are movement only).
- Both drawing paths: the terminal graphics protocol and half-block cells.
- The status line shows the zoom (`fit`, `100%`, `200%`).

**Out of scope.** The text viewer and yak pane keys (yaks-7c6e does those), toque.

**Evidence.** Tests on the zoom state and the viewport offset (fit, zoom in, pan, `0`, `w`). Half-block headless frames at fit and zoomed, attached. Gate `cargo test -p yaks`.

**Judge.** herd-stars at landing.

---
▸ 2026-11-07T17:07:12Z [stars-zoom]

Start: Sat Nov  7 17:07:08 UTC 2026

- yaks path yaks-e5f9: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-e5f9.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/kt9f4mff96hd/yaks
