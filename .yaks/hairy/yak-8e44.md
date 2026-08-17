---
id: yak-8e44
title: Always show state emoji in list rows (drop the differ-from-view cleverness)
type: feature
priority: 3
created: '2026-08-17T14:17:49Z'
updated: '2026-08-17T14:17:49Z'
parent: yak-1bef
labels:
- ui
---

Today the per-row status emoji is only shown when a row's status differs from the current view/tab (a cleverness that's more confusing than helpful in practice — user feedback). Just always render the status emoji (format.status_emoji(status)) on every list row.

Where: yaktui/render.py list-row drawing; find the conditional that suppresses the emoji when it matches the active view's status and remove it. Note this pairs well with the flat views (Recent/Starred) that mix statuses, where a per-row state emoji is genuinely useful. Verify alignment/indentation math still lines up once the emoji is always present.
