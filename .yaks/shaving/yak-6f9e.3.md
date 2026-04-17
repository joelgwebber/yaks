---
id: yak-6f9e.3
title: 'Rethink layout: kanban is wrong for read-only'
type: task
priority: 1
created: '2026-04-17T23:13:29Z'
updated: '2026-04-17T23:47:43Z'
---

The kanban view generates a ton of scrolling (72 shorn tasks in one column). Since this is read-only, the interactive nature of kanban is pointless. Consider a TUI-like stateful approach — tabbed status views, or a list with a detail pane. Something more familiar to yaks users and more compact.

### 2026-04-17T23:47:43Z
Implemented in docs/index.html v2 rewrite.
