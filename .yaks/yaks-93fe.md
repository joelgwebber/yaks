---
id: yaks-93fe
title: 'sheds: a plain git worktree of a Delta clone is taken for the primary (named main, measured vs origin/main)'
status: hairy
type: bug
priority: 3
created: '2026-10-13T00:25:06Z'
updated: '2026-10-13T00:25:06Z'
parent: yaks-38dc
labels:
- cli
- delta
---

Seen 2026-10-13 by delta-lead while checking the Sheds tab (yaks-98f1) on the real machine: `git worktree add --detach <scratch> HEAD` inside this thread's Delta clone (`.delta/worktrees/gcmb7srbmkvv/yaks`) made a shed that `yaks sheds` lists as

```
  worktree  (detached)  503fca3  +0 -0  1  ...  <scratch>/tabcheck
  name: main
  who: delta-lead, Joel Webber, needs-1, tabcheck · shaving: ... · active (vs origin/main)
  farm: 1 new, 3 moved, 19 new notes, 1 needs (vs origin/main)
```

so two rows are named `main`, and the worktree is measured from origin/main (crediting it with everything the clone did) instead of from its own fork point. The real primary (`/Users/joel/src/yaks`) is listed correctly too. Probably `checkout_paths` / `Found::primary` marks a worktree found through the CURRENT clone's `git worktree list` as primary. Expected: primary only for the host checkout; a worktree of a clone is a plain shed (named by its actor or branch, fork-point baseline). Rare in practice (workers do not make worktrees inside Delta clones), so P3. Fix after yaks-07fe lands, since it restructures discovery.
