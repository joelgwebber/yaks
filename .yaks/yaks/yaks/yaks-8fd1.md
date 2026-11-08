---
id: yaks-8fd1
title: 'yaks render: rasterize an SVG to a PNG with the embedded fonts (no browser)'
status: shaving
type: feature
priority: 3
created: '2026-11-04T01:14:21Z'
updated: '2026-11-08T01:25:05Z'
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
