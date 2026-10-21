---
id: yaks-b1e9
title: 'yaks init --mode branch + yaks farm status: start a new farm on an orphan branch at farm_home; report where it is and whether it is pushed'
status: hairy
type: feature
priority: 3
created: '2026-10-18T05:15:36Z'
updated: '2026-10-21T19:54:36Z'
parent: yaks-6a34
labels:
- cli
---

Tooling for the setup yaks-6a34 does by hand: `yaks init --mode live [--branch farm] [--path ~/src/<repo>-farm]` creates the branch and the sparse worktree (idempotent, like the other modes), writes `live_branch:` into config and prints the result; `yaks farm status` says where the live farm is, whether `farm` is pushed/behind `origin/farm`, and the uncommitted drift count (the two facts needed before a machine switch). Consider a per-user registry (`~/.config/yaks/farms`, keyed by `remote.origin.url`) only if `~/`-relative `YAKS_DIR` proves insufficient on a machine whose host is Delta's bare repo.

---
▸ 2026-10-19T04:28:53Z [delta:herd-pass]

Rescoped in the 2026-10-19 herd pass: 'live' and the sparse worktree are gone (yaks-73f4). Today a new farm needs `git worktree add --orphan -b yaks <path>` by hand; `yaks init --mode branch` should do it at store::farm_home (the same place migrate and the first-use bootstrap, yaks-a260, use), idempotently. `yaks farm status`: the farm worktree, its branch, ahead/behind origin, uncommitted yak count. Probably the default `yaks init` mode once the skills recommend it (yaks-b048).

---
▸ 2026-10-21T19:54:36Z [delta:herd-dirs]

2026-10-21: the 'pushed / behind origin' half of `yaks farm status` is part of the sync design, yaks-472e (question 8).
