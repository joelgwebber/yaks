---
id: yak-c393.4
title: 'Agent pane: fix width so it occludes instead of wrapping'
type: bug
priority: 2
created: '2026-08-13T01:24:51Z'
updated: '2026-08-15T03:27:27Z'
---

When the focus divider narrows the agent pane (esp. mid-animation toward board-full), the agent text reflows to a tiny column -> vertical letter-stacking. Fix: lay the agent out at a FIXED width and slide the board OVER it (occlusion), clipping the agent at the moving divider instead of re-wrapping.

---
▸ 2026-08-13T01:56:51Z
Occlusion model: agent laid out at fixed AGENT_LAYOUT_W (52); board slides over it via board_x. render_agent wraps at layout_w, paints clipped to visible_w -> line breaks never change, so narrowing clips instead of reflowing. Verified mid-slide (board_x=28/16): identical breaks, no letter-stacking.
