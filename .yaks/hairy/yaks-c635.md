---
id: yaks-c635
title: 'yaks sheds: use the Delta root scan and the origin match from yaks discover; from a Delta clone also list the primary''s git worktrees'
type: task
priority: 3
created: '2026-10-09T03:48:59Z'
updated: '2026-10-09T03:48:59Z'
parent: yaks-7eea
labels:
- cli
- delta
---

Follows yaks-2177 (yaks discover). Its findings 2 and 3: from a linked Delta clone the primary's plain git worktrees are missed (run git worktree list in the repo alternates/local names, when it is a checkout); from a managed clone only the root scan finds siblings. Wire the root scan into sheds::discover; then the -C/--shed reference-shed selector (yaks-7eea).
