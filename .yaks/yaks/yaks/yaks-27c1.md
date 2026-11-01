---
id: yaks-27c1
title: 'Dead parents: doctor checks them too, and slaughter stops treating shorn children as live'
status: hairy
type: task
priority: 2
created: '2026-11-01T16:43:15Z'
updated: '2026-11-01T16:43:15Z'
parent: yaks-9b41
labels:
- cli
---

Part of yaks-9b41.

## Doctor

`unfinished-children` covers shorn parents only (yaks-62f2). A dead yak over a hairy or shaving child is the same broken state (an abandoned umbrella still holding live work), and older farms or hand edits can hold it. Extend the check to dead parents, same code.

## slaughter

Today `slaughter` refuses a yak with any non-dead descendant, *shorn included*, and `--family` moves shorn descendants to dead, erasing that they were done. Under the invariant only open (hairy or shaving) descendants matter:

- `slaughter <id>` refuses only over open children (it already names them; add the `yaks reparent` fix like `shorn`).
- `--family` kills the open descendants and `id`, and leaves shorn ones shorn.
- The TUI's slaughter confirm and bulk skip use the same rule (`live_child_count`).

Design fork (behavior change): see the ask on yaks-9b41.
