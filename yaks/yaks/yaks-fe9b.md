---
id: yaks-fe9b
title: 'TUI: one row type, one pane shell, one field renderer and tag style; mixed All list'
status: hairy
type: feature
priority: 2
created: '2026-11-27T02:38:25Z'
updated: '2026-11-27T02:38:25Z'
parent: yaks-7e32
tags:
- tui
---

Section 10 of [the design](/doks/design/one-document-model.md#10-tui): one row type keyed by link destination (replacing the Yak/Doc splits in `Item`, `RowKind`, `RowKey`, `Star`, `PaneAt`, `DocKind`), one pane shell, one field-row renderer (`detail::build_at` vs `frontmatter_rows`), one tag style. The All list mixes every kind by modified time (yaks-7183) and Recent merges into it (yaks-7350, decided 2026-11-27); the remaining view rules settle in yaks-6905.
