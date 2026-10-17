---
id: yaks-93cd
title: Comment headers in the detail pane, shared with the edit form
status: hairy
type: feature
priority: 2
created: '2026-10-17T04:25:39Z'
updated: '2026-10-17T04:36:55Z'
parent: yaks-faf5
depends_on:
- yaks-ef82
- yaks-2362
- yaks-a870
labels:
- markdown
- tui
---

Split the body with `content::parse` and render each comment under a header row instead of the raw `---` / `▸ <ts> [actor]` lines: a colored marker, the actor, the relative date (from the dates yak), and a rule to the right edge. One function draws this header for both the detail pane and the edit form (today's `render_block_separator` in src/tui/create.rs), so there is one design.

Ctrl-N/P block jumps and `E` must keep finding blocks: derive block indices from `content::parse` rather than `block_index_per_line`'s display-text match. Can land before the renderer core (body lines inside each block keep today's highlighter until the core replaces it).
