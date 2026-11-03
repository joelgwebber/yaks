---
id: yaks-aec7
title: Navigation back skips current list view
status: hairy
type: bug
priority: 3
created: '2026-11-03T21:11:08Z'
updated: '2026-11-03T21:11:08Z'
tags:
- tui
---

The navigation stack *does* support forward/back affordances across list views; but when backing out of a *detail* it skips the list view in which the detail was found. This is slightly jarring as it often navigates to a *different* list view.
