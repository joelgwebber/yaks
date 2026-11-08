---
id: yaks-1a5f
title: Regenerate the docs/assets docshots in one commit
status: shaving
type: task
priority: 3
created: '2026-11-07T18:24:51Z'
updated: '2026-11-08T00:55:23Z'
tags:
- tui
- docs
---

After yaks-7c6e the docshot SVGs in `docs/assets/` show stale hints (`l:detail`, `h:list`, `Esc/h close`), and a full `cargo test -p yaks docshots -- --ignored` rewrites about 30 SVGs, much of it older renderer drift every worker since has reverted. Run it once, look at the rasterized frames (the Chrome recipe in `docs/tui.md`), and commit them all together.

---
▸ 2026-11-08T00:52:58Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

![tui-image-viewer](yaks-1a5f/tui-image-viewer.png "The image-viewer docshot after the fix: the scene opens the pane with Enter")
