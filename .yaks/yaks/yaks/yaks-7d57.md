---
id: yaks-7d57
title: 'Trial: cheaper-model workers follow the sync, heartbeat, actor and branch-checkpoint conventions'
status: hairy
type: task
priority: 2
created: '2026-10-31T15:57:06Z'
updated: '2026-10-31T15:57:06Z'
parent: yaks-039b
depends_on:
- yaks-0071
labels:
- skills
- agent
---

Joel (2026-10-31): before building tooling such as `yaks checkpoint`, run experiments with workers on cheaper models (e.g. Sonnet) and see whether they follow the conventions by hand.

## What to observe

- Do they `yaks sync` after every batch of yak writes, especially on a cloud runner or another machine?
- Do they write heartbeat notes often enough for `stale` to mean dead?
- Do they set `YAKS_ACTOR`, so each thread writes under one stable name?
- Do they push their worker branch to `origin` and note the branch and sha, in the order push, record, sync?

## Shape

- At least one run on a Delta cloud runner and one local, a few workers each, on real yaks.
- Score each convention per worker from the farm and the remote afterwards: `yaks log --by <worker>`, sync times against write times, branches on origin against the shas the notes name.
- Record a `Decision:` per convention: rule is enough, or needs a tool.
