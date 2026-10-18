---
id: yaks-b1e9
title: 'yaks init --mode live + yaks farm status: create/check the sparse farm worktree'
status: hairy
type: feature
priority: 3
created: '2026-10-18T05:15:36Z'
updated: '2026-10-18T05:15:36Z'
parent: yaks-6a34
labels:
- cli
---

Tooling for the setup yaks-6a34 does by hand: `yaks init --mode live [--branch farm] [--path ~/src/<repo>-farm]` creates the branch and the sparse worktree (idempotent, like the other modes), writes `live_branch:` into config and prints the result; `yaks farm status` says where the live farm is, whether `farm` is pushed/behind `origin/farm`, and the uncommitted drift count (the two facts needed before a machine switch). Consider a per-user registry (`~/.config/yaks/farms`, keyed by `remote.origin.url`) only if `~/`-relative `YAKS_DIR` proves insufficient on a machine whose host is Delta's bare repo.
