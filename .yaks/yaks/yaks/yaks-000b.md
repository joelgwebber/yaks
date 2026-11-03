---
id: yaks-000b
title: Scroll-wheel tweaks
status: hairy
type: task
priority: 3
created: '2026-11-03T21:04:56Z'
updated: '2026-11-03T21:04:56Z'
tags:
- tui
---

A few scrolling odds and ends:
- If we can get horizontal scroll events from the mouse via terminals, it would be helpful to apply that to horizontally-scrollable areas in details (eg, overflowing tables).
- Vertical scrolling in list views moves the cursor, rather than the scrollable area. I think this would be better if it worked like the detail view.
