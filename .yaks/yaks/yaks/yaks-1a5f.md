---
id: yaks-1a5f
title: Regenerate the docs/assets docshots in one commit
status: hairy
type: task
priority: 3
created: '2026-11-07T18:24:51Z'
updated: '2026-11-07T18:24:51Z'
tags:
- tui
- docs
---

After yaks-7c6e the docshot SVGs in `docs/assets/` show stale hints (`l:detail`, `h:list`, `Esc/h close`), and a full `cargo test -p yaks docshots -- --ignored` rewrites about 30 SVGs, much of it older renderer drift every worker since has reverted. Run it once, look at the rasterized frames (the Chrome recipe in `docs/tui.md`), and commit them all together.
