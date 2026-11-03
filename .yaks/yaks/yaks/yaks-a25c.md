---
id: yaks-a25c
title: 'TUI phase 6a: global query with its own chip, and the All view'
status: hairy
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T03:46:57Z'
parent: yaks-947a
depends_on:
- yaks-5d08
tags:
- tui
---

Phase 6, the list half, of [the design](/wiki/design/tui-model.md#4-search) (yaks-953f, yaks-c7ba): the `/` query leaves the view's filter, survives view switches, matches what `yaks find` searches, and has its own chip with the All count; tab counts apply it; Esc clears it first. A built-in unpinned All view holds every yak (dead included), page and source.
