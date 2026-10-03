---
id: yaks-38dc
title: 'Lanes view: show in-flight yak state from sibling checkouts'
type: feature
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T21:13:29Z'
parent: yaks-b5a0
labels:
- ui
- cli
---

Let yaks tui (and list/show) overlay yak state from other checkouts of the same repo: git lanes via git worktree list; Delta lanes via <repo>/.delta/worktrees/<id>/<repo>/.yaks as a stopgap (opaque ids, internal layout, local runs only). Read-only; the main checkout stays authoritative. Make discovery pluggable so the planned Delta CLI ('connect any agent to Delta threads and worktrees from the command line') can replace the directory scan. Relates to yaks-5f12 (live cross-worktree sharing).

---
▸ 2026-10-03T21:13:29Z [delta-lead]
Feasibility (Delta, 2026-10-03): from one thread, <repo>/.delta/worktrees/*/<repo> is readable and each has its own .git and .yaks (status counts, branch, HEAD, dirty count all obtainable); git worktree list inside a Delta clone shows ONLY that clone, so a Delta lanes view needs the directory scan (or the planned Delta CLI), not git. Sibling repos keep their own .delta (edtui: /Users/joel/src/edtui/.delta/worktrees/<id>). Finished threads' checkouts persist (log O14), so lanes need a liveness hint (dirty/uncommitted yak state, mtime) or they will list stale ones. Related primitive: yaks-70e5 (diff of two refs) gives the 'what changed in that lane' summary.
