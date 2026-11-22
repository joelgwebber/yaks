---
id: yaks-daa4
title: 'Refresh every docshot: docs/assets SVGs drift from the current toque output'
status: shaving
type: chore
priority: 2
created: '2026-11-07T15:48:14Z'
updated: '2026-11-22T17:05:03Z'
tags:
- docs
- tui
---

Found while landing yaks-85a6: `cargo test -p yaks docshots -- --ignored` rewrites every SVG in `docs/assets/`, not just the frames a change touches. The checked-in frames predate two changes:

- toque's SVG output now writes `fill-opacity` for dim text and `xml:space="preserve"` on each `<text>`, with a different selection background.
- The tab row: frames still show `🛖 Sheds (2)` where the current TUI shows `📖 Doks (0)` (the Doks tab came in after they were drawn).

yaks-85a6 committed only the four `tui-doks*.svg` it changed. Regenerate the rest in one commit, look at a few rasterized (the `docs/tui.md` recipe), and check each still shows what its `docs/` caption says. Worth doing before the next release, so its docs don't show an older TUI.

#### 2026-11-22T16:40:41Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-22T16:41:01Z

*herd-files*

Claim (herd-files): worker shots-1 (Sonnet), base 6ae40a60, batch before v0.0.41. Scope: src/tui/docshots.rs, docs/assets/*.svg, the docs that embed them. Judge: herd-files at landing.

#### 2026-11-22T16:43:19Z

*shots-1*

Start 2026-11-22T16:43:18Z. Checkout /Users/joel/src/yaks/.delta/worktrees/d3wvgsm9qny3/yaks, base 6ae40a60. Farm file /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-daa4.md. Plan: regenerate all docshots, eyeball a sample, add archive-member + line-range scenes, check Sheds tab shots.

![tui-archive-tree](yaks-daa4/tui-archive-tree.png "New docshot tui-archive-tree: a zip attachment as a directory in the directory view (rasterized)")
