---
id: yaks-7a25
title: 'Exclusive claims on one machine: shave --exclusive, next --claim, take over a stale claim'
status: hairy
type: idea
priority: 2
created: '2026-11-25T04:24:41Z'
updated: '2026-11-25T04:31:31Z'
tags:
- cli
- agent
---

Make a claim mean something on one machine: refuse to shave a yak another actor is shaving, and let a worker take the next ready yak in one atomic step.

Today `yaks shave` on a yak someone else is shaving prints "already being shaved" and exits 0 ([`transition`](https://github.com/joelgwebber/yaks/blob/b013ff062a72f96f3118c1894fd6424e9d25d887/src/main.rs#L3889-L3891)), and a stale claim only reads `stale` in `sheds` after two hours. Vercel Board's claims are exclusive leases (`task_claimed`, `claim_lost`); see [the research](/doks/research/vercel-board.md#where-they-really-differ).

## Proposal

- `yaks shave --exclusive <id>`: under `store::lock`, refuse (non-zero, a stable reason) when the yak is shaving and its latest claim is another actor's.
- `yaks next --claim [filters]`: pick the top ready yak matching the filters and shave it as this actor, under one lock; print it (`--json`). Like Board's `tasks claim --ready --label`.
- `--take-over`: shave a yak another actor holds only when that claim is stale (the sheds' liveness), with a note naming whom it was taken from. The yaks-shaped version of a lease running out.

## Limits to state in the docs

Every checkout on one machine shares the one farm worktree and its lock, so this is exclusive on one machine (a coordinator and its subagents). Across machines it stays advisory: two machines can each claim before they sync, and the merge must keep both visible rather than silently pick one.

## Open

- [ ] Should `--exclusive` become the default for `shave` when an actor is named?
- [ ] What decides "whose claim": the actor on the latest `moved: hairy -> shaving` note?
- [ ] How does the merge driver surface two claims on one yak after a sync (a `doctor` finding)?

Related: Haks built 5-minute expiring claims out of notes ([haks](/doks/research/haks-orchestration-layer.md#patterns-worth-keeping)); claim hygiene in yaks-0071.
