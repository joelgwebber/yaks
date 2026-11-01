---
id: yaks-35a4
title: 'Refuse opening a yak under a finished parent: create/reparent --parent, regrow/revive/shave of a child of a shorn or dead yak'
status: hairy
type: task
priority: 2
created: '2026-11-01T16:31:51Z'
updated: '2026-11-01T16:43:01Z'
parent: yaks-9b41
labels:
- cli
---

Found shearing yaks-62f2; widened under yaks-9b41. Every move that makes a yak hairy or shaving while its parent is shorn or dead breaks the family invariant:

- `yaks create --parent <finished>`
- `yaks reparent <open> --parent <finished>` (also `bulk --reparent`, the TUI reparent)
- `yaks regrow` / `revive` / `shave` of a child whose parent is finished

## Plan

- Refuse each, naming the parent and its state and the two fixes: reopen the parent first (`yaks regrow <parent>`), or file it under an open ancestor or top-level.
- A batch that opens takes parents before children (the mirror of `children_first`), so `yaks regrow <parent> <child>` works in either order.
- One check in `Farm` used by create, reparent and transition, so the CLI and TUI share it.
- Tests that fail first; `--help`, docs/cli.md.

Decided: refuse rather than warn or auto-reopen the parent. A warning is how the state got here; auto-reopening changes someone else's yak silently.
