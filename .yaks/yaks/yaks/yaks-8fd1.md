---
id: yaks-8fd1
title: 'yaks render: rasterize an SVG to a PNG with the embedded fonts (no browser)'
status: shorn
type: feature
priority: 3
created: '2026-11-04T01:14:21Z'
updated: '2026-11-08T01:25:07Z'
depends_on:
- yaks-4f4d
tags:
- images
- cli
---

The deferred half of yaks-faf2. That yak was shorn as a documented headless-Chrome recipe, on two grounds: yaks is one self-contained binary, and a pure-Rust rasterizer drew our color emoji as tofu. Neither holds now: yaks already embeds the rasterizer (toque's `png` feature, `resvg`) and its own fonts (yaks-4e9c), and yaks-4f4d widens the glyph coverage. faf2's note deferred "a pure-Rust `yaks render` if emoji-capable rasterization ever lands"; this is that.

## The gap

An agent whose file tools only read in-project paths cannot look at a docshot or an attached SVG without leaving the binary: run headless Chrome into a gitignored dir, then copy the PNG somewhere readable. The rasterizer is already in the binary, but only the TUI's image viewer reaches it. Seen while landing yaks-0370: the render was checked by writing a throwaway example binary that called `toque::svg_to_png`.

## Proposal

`yaks render <svg> [-o <png>]`: rasterize an SVG with the embedded fonts and write a PNG.

- Same rasterizer and fonts as the viewer, so it shows what a human sees in `yaks tui`, on any machine.
- `<svg>` is a path, or an attachment (`<id>/<file>`) or farm path, like the other link-taking commands.
- Open points for the human: where the PNG goes by default (beside the input, or a gitignored dir) and a `--scale` for legibility (Chrome's recipe uses 2x; toque's `svg_to_png` renders at the SVG's own size, so scale needs a small toque addition).

The Chrome recipe in `docs/tui.md` stays for the one thing it does better: a pixel-faithful look at what a browser shows (GitHub, system fonts).

## Depends on

yaks-4f4d: until the subset covers the emoji and box-drawing corners our frames use, a rendered frame has holes (a blank tab icon, no italic) and the helper would mislead more than the browser does.

## Done when

An agent can go from a docshot or attachment to a PNG its file tools can open with one yaks command, and `docs/tui.md`'s recipe section, the `yaks` skill's evidence guidance and the pointer in `AGENTS.md` ("Verifying changes") say so.

---
▸ 2026-11-08T00:49:17Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-08T00:52:11Z [herd-stars]

Claim (yakherd herd-stars): worker after yaks-4f4d lands in toque. Open points decided by the yakherd (reversible; Joel can overrule):

- **Default output:** `<stem>.png` in the current directory (where an agent's file tools can read it), printed on stdout; `-o <png>` overrides; `-o -` writes to stdout. Rejected: beside the input (would write into the farm for an attachment) and a gitignored dir (needs a project to know about it).
- **`--scale <n>`** (default 2, as the Chrome recipe uses, since a 1x frame is small for a vision model), via toque's new scaled rasterizer.

![peek-render](yaks-8fd1/peek-render.png "yaks render docs/assets/tui-yak-peek.svg: 2x, embedded fonts, no browser")

---
▸ 2026-11-08T01:25:06Z [herd-stars]

Done by herd-stars in `cddd0bcd`: `yaks render <svg> [-o <png>] [--scale N]` ([src/render.rs](code:src/render.rs@cddd0bcd)), on toque `4fdb374` (yaks-4f4d's fonts).

- `<svg>` is a file path, else a farm path or an attachment link; no farm needed for a plain file. Default output `<stem>.png` in the current directory, `-o -` stdout, `--scale` 2 by default (decisions in the claim note above).
- The TUI's svg cache generation is bumped (`SVG_RENDER_GEN` 2), so PNGs cached with the old fonts are not served.
- Docs: `docs/cli.md`, `docs/tui.md` (the rasterizing section and the svg fonts paragraph), the `yaks` skill's TUI evidence line, `AGENTS.md` "Verifying changes".
- Evidence: unit tests for resolution and scale (IHDR 80×40 at 2x); `yaks render yaks-e5f9/zoom-3-zoomed-panned.svg` from the repo resolves the attachment; the attached PNG of `tui-yak-peek.svg`, looked at. Full gate `cargo test --workspace` green.

---
▸ 2026-11-08T01:25:07Z [herd-stars]
kind: move
from: shaving
to: shorn
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
