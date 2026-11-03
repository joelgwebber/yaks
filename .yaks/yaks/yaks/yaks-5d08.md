---
id: yaks-5d08
title: 'TUI phase 3: rows can be documents and sheds; Wiki and Sheds become ordinary views'
status: hairy
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T03:46:57Z'
parent: yaks-947a
depends_on:
- yaks-fbb0
tags:
- tui
---

Phase 3 of [the design](/wiki/design/tui-model.md#1-rows-and-views) (yaks-f1e6): a row is a yak, a document or a shed. Wiki becomes a flat view of document rows and Sheds a tree view (actor roots, touched yaks as children, outstanding first). Their separate cursors and key handlers go.
