---
id: yaks-a260
title: 'Bootstrap the farm worktree on first use: a repo with a yaks branch (local or origin/) but no worktree gets one at farm_home'
status: hairy
type: feature
priority: 1
created: '2026-10-18T23:24:43Z'
updated: '2026-10-18T23:24:43Z'
parent: yaks-73f4
labels:
- cli
- git
---

Joel (2026-10-18): after `yaks migrate --to-branch`, another machine or a teammate has to run `git fetch origin yaks && git worktree add <path> yaks` by hand. Instead, when discovery (`store::farm_branch_worktree`) finds the farm branch with no worktree, create it at `store::farm_home(repo)` (a local branch tracking `origin/<b>` when only the remote exists), print one stderr line saying where, and continue. Only from a repo's own checkout (a Delta/--shared clone points at its main checkout, as migrate does); never fetches unless the branch is missing locally AND remotely (then the old error). Non-destructive, so no prompt; `YAKS_NO_BOOTSTRAP=1` (or a config/flag) for CI that should fail instead. Evidence: a fresh clone of a migrated repo: first `yaks list` creates the worktree at farm_home and lists; a second run is silent.
