---
id: yaks-38dc
title: 'Lanes view: show in-flight yak state from sibling checkouts'
type: feature
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T20:14:31Z'
parent: yaks-b5a0
labels:
- ui
- cli
---

Let yaks tui (and list/show) overlay yak state from other checkouts of the same repo: git lanes via git worktree list; Delta lanes via <repo>/.delta/worktrees/<id>/<repo>/.yaks as a stopgap (opaque ids, internal layout, local runs only). Read-only; the main checkout stays authoritative. Make discovery pluggable so the planned Delta CLI ('connect any agent to Delta threads and worktrees from the command line') can replace the directory scan. Relates to yaks-5f12 (live cross-worktree sharing).
