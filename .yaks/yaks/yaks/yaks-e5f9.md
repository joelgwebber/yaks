---
id: yaks-e5f9
title: Image zoom affordances
status: shorn
type: feature
priority: 3
created: '2026-11-03T23:29:24Z'
updated: '2026-11-07T17:37:45Z'
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

---
▸ 2026-11-07T17:21:23Z [stars-zoom]
kind: verify
result: fail
---

verify: `cargo test --workspace` -> FAIL (exit 101)

---
▸ 2026-11-07T17:27:05Z [stars-zoom]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

![zoom-1-fit](yaks-e5f9/zoom-1-fit.svg "Half-block frame at fit (default), status line: zoom fit")

![zoom-3-zoomed-panned](yaks-e5f9/zoom-3-zoomed-panned.svg "Half-block frame zoomed in (150%) and panned right/down; status line gains the pan keys")

![zoom-4-fit-width](yaks-e5f9/zoom-4-fit-width.svg "Half-block frame at fit width (w): picture spans the pane, taller than it, pans down")

[zoom-3-zoomed-panned](yaks-e5f9/zoom-3-zoomed-panned.txt "Text serialization of the zoomed+panned frame")

---
▸ 2026-11-07T17:28:03Z [stars-zoom]

Decision: zoom and pan are done by cropping and scaling the decoded pixels myself on each change, then handing `ratatui-image` a finished `Protocol` (`Image` widget). Rejected: `StatefulImage` with `Resize::Crop` (it can only clip the bottom/right or top/left, so no arbitrary pan offset), and per-protocol code. One path serves kitty, sixel, iTerm2 and half-blocks; the encoded protocol is cached by (scale, offset, pane cells).

---
▸ 2026-11-07T17:28:05Z [stars-zoom]

Decision: for half-block cells the zoom grid is one column by half a row (the library draws two pixels per cell whatever the font), so `100%` is one image pixel per half-block. Rejected: the picker's 10x20 font grid, where `100%` would be one pixel per 10x20 cell and the library would resample twice. Consequence: half-block `fit` is no longer a tiny natural-size icon (64x64 used to take 7x4 cells); it fills up to the pane, never past 100%.

---
▸ 2026-11-07T17:28:07Z [stars-zoom]

Decision: zoom steps are a fixed ladder (5, 10, 12.5, 25, 33, 50, 67, 75, 100, 150, 200, 300, 400, 600, 800, 1200, 1600, 2400, 3200 %) stepped from the factor on screen, so `+` from `fit` goes to the first rung above the fit size. Rejected: a x1.25 multiplier (ugly percentages in the status line).

---
▸ 2026-11-07T17:28:10Z [stars-zoom]

Decision: changing zoom keeps the middle of the window on the same part of the picture; so `0` from `fit` lands centred, not top-left. `f` returns to offset 0 (the picture fits). Rejected: always resetting the offset to the top-left.

---
▸ 2026-11-07T17:28:12Z [stars-zoom]

Decision: `w` (fit width) may enlarge a narrow image; `f`/default `fit` still never enlarges. Rejected: capping fit width at 100%.

---
▸ 2026-11-07T17:28:14Z [stars-zoom]

Decision: pan keys move a quarter of the pane (at least one cell) per press, clamped to the picture's edge; with the picture no larger than the pane they do nothing. Pan is four new Cmds (`image.pan_left/right/up/down`), not the viewer's `move.*`/`pane.scroll_right`, so a `keys:` override can rebind them without touching the text viewer. The bottom line lists `h/j/k/l pan` only when the picture overflows.

---
▸ 2026-11-07T17:28:15Z [stars-zoom]

Decision: the state (zoom, offset, measured pane) lives in `Cell`s on `ImageView`, because `render` takes `&App` and measures the pane while drawing; `follow.rs` (another worker's file) is untouched (no `image_mut`). `docs/assets/tui-image-viewer.svg` regenerated (`cargo test docshots_image_viewer -- --ignored`) since half-block fit now draws bigger; no new docshots scene (that file is out of my scope), the frames are attached instead.

---
▸ 2026-11-07T17:28:18Z [stars-zoom]

Summary. In `src/tui/imageview.rs`: `Zoom` (Fit / FitWidth / Scale), pure helpers `scale_for` / `shown_size` / `clamp_offset` with unit tests, `ImageView::{set_zoom, zoom_step, pan, window, encode}`, `render_image` draws the window. `keymap.rs`: nine new `Cmd`s (`image.zoom_in/out/actual`, `image.fit_width`, `image.fit`, `image.pan_*`) and their `Image` bindings (`+`/`=`, `-`, `0`, `w`, `f`, `h`/`←`, `l`/`→`, `k`/`↑`, `j`/`↓`); `Esc` alone closes now; `h` no longer closes. `render.rs`: bottom line `zoom <fit|fit width|N%> · +/- · 0 actual · w width · f fit[ · h/j/k/l pan] · O open externally · Esc close · q quit`. `docs/tui.md`: the `A`/`O` row and a new "Zooming and panning an image" paragraph. Tests (src/tui/tests.rs): start-at-fit + status line, +/-/= steps and the 5% floor, 0 / w / f, hjkl and arrows pan + clamp at the edge + Esc closes + h never closes, centre-preserving zoom, half-block red/blue frames at fit / actual top-left / panned to the bottom. The tests were written after the code (they cannot compile without the new API, but I did not run them red first).
Evidence attached: half-block SVG frames at fit, at 150% panned, at fit width, and a text serialisation. I rasterised the SVGs with headless Chrome and looked at them (the disc looks slightly tall: the SVG's cells are 8.6x18.7 px, so a half-block pixel is not square; real terminals are nearer 1:2).
Not verified: a real kitty/sixel/iTerm2 terminal (only the half-block path runs headlessly); the protocol path shares the crop code and builds via `Picker::new_protocol`, but I have not seen it drawn.
Friction: the first `yaks verify` failed in `tests/needs_loop.rs::pickup_errors_name_the_state` ("git status failed" inside `init`); passed alone and on the second full run, so flaky under parallel load.

---
▸ 2026-11-07T17:28:20Z [stars-zoom]
kind: move
from: shaving
to: shorn
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEp3x5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-07T17:37:45Z [herd-stars]
kind: landed
commit: 0490703606fd1fcbb2a34b844d639de96fc38d01
---

Landed by herd-stars as `04907036` (worker `c46e089a`), by cherry-pick: `docs/tui.md`'s `A`/`O` row conflicted with yaks-a036; kept both. Gate `cargo test --workspace` green. Not seen on a graphics-protocol terminal yet.
