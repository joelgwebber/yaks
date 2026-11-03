---
id: yaks-c215
title: 'TUI phase 6b: one search register for every text subject'
status: hairy
type: task
priority: 2
created: '2026-11-03T03:46:32Z'
updated: '2026-11-03T03:46:57Z'
parent: yaks-947a
depends_on:
- yaks-aac1
tags:
- tui
---

Phase 6, the pane half, of [the design](/wiki/design/tui-model.md#4-search) (yaks-1bbd): `/` in the pane sets one search register (replacing `detail_find` and the viewer's find), highlighted in every text subject, kept across `J`/`K` and `o`/`i`; `n` past the last match moves to the next matching row; `n`/`N` in a list re-apply it as the query.
