---
id: yaks-66a3
title: 'Pinned header: a rule under it, and the status icon in the full header'
status: hairy
type: task
priority: 2
created: '2026-10-17T04:25:39Z'
updated: '2026-10-17T04:25:39Z'
parent: yaks-faf5
labels:
- tui
---

Two small fixes in `render_detail` / `pinned_header_line` (src/tui/render.rs) and `detail::build`:
- When the header collapses to the pinned one-line summary, give it a bottom border (a dim `─` rule row) so it is clearly separated from the content.
- The full (unscrolled) header shows no status emoji; the `Status:` field (or the `Task:` line) should carry the same icon the pinned header shows.
Before/after frames (pinned and unpinned) attached; update the `tui-detail*.svg` docshots.
