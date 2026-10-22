---
id: yaks-4e9c
title: Display svg attachments in the image viewer
status: hairy
type: feature
priority: 3
created: '2026-10-22T20:35:28Z'
updated: '2026-10-22T20:35:28Z'
labels:
- tui
- images
---

Follow-up to yaks-4305. The image viewer decodes with the `image` crate (png, jpeg, gif, webp), which cannot read svg, so an `.svg` attachment still goes to the OS opener. That is the format most yak evidence uses: toque frames and `docshots` write SVG.

Options to weigh:

- Rasterize with `resvg`/`usvg` (pure Rust) into a `DynamicImage`, then reuse `ImageView` as is. Caveat from `docs/tui.md`: a pure-Rust rasterizer renders our color emoji as tofu, so toque frames would look worse than in a browser. Fonts also need loading (system fonts via `fontdb`) or the text vanishes.
- Keep svg external and make the evidence habit attach a PNG next to the SVG (what the recipe in `docs/tui.md` already does).
- Teach toque to emit a PNG itself, so `yaks attach` gets something the viewer can show. That belongs in toque, not here.

Lean: the third or second, not `resvg` in yaks: this binary stays self-contained and out of rasterization on purpose (see `docs/tui.md`, "Rasterizing an SVG to view it"). Decide before building.

Acceptance: Enter on an `.svg` attachment either opens in the pane or has a documented reason it does not.
