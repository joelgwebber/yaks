---
id: yaks-fc9e
title: Improved detail cursor/link handling
status: hairy
type: feature
priority: 2
created: '2026-11-03T03:23:32Z'
updated: '2026-11-03T03:23:32Z'
tags:
- tui
---

The cursor model for yaks details and documents is currently only line at a time. Which is good for simplicity, but it also creates some issues when there are multiple links on a single line, which is quite common.

While I don't think we need a full row:col:range selection model for viewing mode -- that would just be unnecessarily confusing -- I do think we need at least a concept of selected row *and* targeted link within that row. We could continue to use up/dn/j/k to change the row selection, we would allow [/] to move among the available links, including those on the same row.
