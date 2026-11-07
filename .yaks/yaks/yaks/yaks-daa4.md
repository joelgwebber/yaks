---
id: yaks-daa4
title: 'Refresh every docshot: docs/assets SVGs drift from the current toque output'
status: hairy
type: chore
priority: 2
created: '2026-11-07T15:48:14Z'
updated: '2026-11-07T15:48:14Z'
tags:
- docs
- tui
---

Found while landing yaks-85a6: `cargo test -p yaks docshots -- --ignored` rewrites every SVG in `docs/assets/`, not just the frames a change touches. The checked-in frames predate two changes:

- toque's SVG output now writes `fill-opacity` for dim text and `xml:space="preserve"` on each `<text>`, with a different selection background.
- The tab row: frames still show `🛖 Sheds (2)` where the current TUI shows `📖 Doks (0)` (the Doks tab came in after they were drawn).

yaks-85a6 committed only the four `tui-doks*.svg` it changed. Regenerate the rest in one commit, look at a few rasterized (the `docs/tui.md` recipe), and check each still shows what its `docs/` caption says. Worth doing before the next release, so its docs don't show an older TUI.
