---
id: yaks-5e8d
title: 'yaks tick: regrow what is due, then report what needs you'
status: shaving
type: feature
priority: 2
created: '2026-11-04T22:31:55Z'
updated: '2026-11-04T23:00:31Z'
parent: yaks-7fa6
depends_on:
- yaks-0079,yaks-bdae,yaks-a041
tags:
- cli
---

Part of the scheduling arc ([design](/doks/design/scheduling.md)): the one command a person (or later a persistent agent) kicks off by hand.

Reopens what is due via the existing `Farm::regrow_due` (one code path, shared with the TUI), then prints: what regrew and what was held; overdue yaks and those due within `--within` (default 3d); the inbox (awaiting you, awaiting an agent); stale claims; a one-line `yaks check` count. `--dry-run` writes nothing; `--json` is the stable shape for a harness; no sync unless `--sync` (needs a two-machine duplicate-regrow test first). Exit 0 unless a write failed.

Out: running commands, notifications, installing a clock.

Done when: tests incl. dry-run vs real and idempotence (a second tick regrows nothing); `--help`, `docs/cli.md`, README updated; the coordinator runs it by hand against a scratch farm and looks at the report.

---
▸ 2026-11-04T23:00:31Z [sched-1]
kind: move
from: hairy
to: shaving
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
