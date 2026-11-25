---
id: yaks-b1bb
title: 'A coordinator''s wait: block on state changes in a family, from a cursor'
status: hairy
type: feature
priority: 2
created: '2026-11-25T04:24:42Z'
updated: '2026-11-25T04:24:42Z'
tags:
- cli
- agent
---

Give a coordinator something to block on besides the inbox: wait for any yak in a family to change state or ask, from a cursor, so nothing that happened during its turn is missed.

`yaks wait` only watches `needs:`. A yakherd waiting on workers has no way to sleep until one shears, asks or regrows. Vercel Board's swarm guide is built on `wait --label 'board.task=completed|blocked|failed' --after <cursor>`, and calls out the trap: a turn-based lead that waits only for what happens *after* it starts waiting misses work that finished during its turn ([swarm guide](/sources/2026-11-25-b0d8da34-vercel-board-docs-building-an-agent-swarm.md#have-the-lead-follow-the-work), [research](/doks/research/vercel-board.md#ideas-worth-taking)).

## Sketch

- `yaks wait --parent-of <id> --on shorn,ask,regrow,dead [--after <cursor>]`, plus the shared filters.
- A cursor every read can print: a timestamp, or the farm commit it read (`yaks log --json`, `inbox --json`), so the coordinator notes its place at the start of a turn and waits from it at the end.
- Exit like today's `wait` (0 with what changed; 2 on timeout).
- Still polls files; no daemon.

## Open

- [ ] Cursor: farm commit sha (exact, but each write is a commit) or note timestamp (readable, but clock skew across machines)?
- [ ] Does this subsume `--new`?
- [ ] Update `yaks-coordinating` with the turn order (see the skill yak filed beside this).
