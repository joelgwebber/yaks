---
id: yaks-c354
title: 'TUI phase 3b: Sheds becomes a tree view (actor roots, touched yaks as children)'
status: shaving
type: task
priority: 2
created: '2026-11-03T04:10:57Z'
updated: '2026-11-03T04:24:10Z'
parent: yaks-5d08
depends_on:
- yaks-be65
tags:
- tui
---

Phase 3, second half, of [the design](/wiki/design/tui-model.md#1-rows-and-views); implements yaks-f1e6: the Sheds tab becomes an ordinary tree view. Each actor is a collapsible root row; its children are the yaks it touched (working, asking, then recent, as `Shed::rows` orders them). The tab's own cursors, key handlers and list renderer go.
