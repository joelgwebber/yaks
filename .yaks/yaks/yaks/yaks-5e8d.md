---
id: yaks-5e8d
title: 'yaks tick: regrow what is due, then report what needs you'
status: shaving
type: feature
priority: 2
created: '2026-11-04T22:31:55Z'
updated: '2026-11-04T23:09:40Z'
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

---
▸ 2026-11-04T23:09:40Z [sched-1]

Done on branch `scheduling`, rebased onto `origin/main` (0.0.32, after the doks rename).

- `yaks tick [--dry-run] [--within <span>] [--json] [--as <actor>]` (`src/tick.rs`, wired in `src/main.rs`). It calls `Farm::regrow_due` (the TUI's code path), then prints a summary line and a section each for: regrew, held, overdue, due within `--within` (default 3d; a yak already in the inbox is listed there, not twice), awaiting you, awaiting an agent, stale claims (via `sheds::build`), and a `yaks check` finding count. A quiet farm prints `tick: nothing needs you.`
- The digest is a pure `tick::build` over loaded yaks with an injected clock, so it is unit-tested; the CLI is tested end to end.
- Never syncs, runs no command, posts no notification; idempotent; `--dry-run` writes nothing; a bad `--within` is refused before any write; exit 0 unless a write failed. `--json` is the `regrow-due --json` keys plus `within`, `overdue`, `due_soon`, `awaiting_human`, `awaiting_agent`, `stale`, `findings`, `quiet`.
- Evidence: 6 unit tests and 6 `tests/tick.rs` tests (a full round with a woken yak, an ask, overdue and due-soon; a second tick writes zero commits; dry run vs real; a held yak and `--within`; bad `--within`; findings), full `cargo test` green (969), clippy clean for the new files. By hand on a scratch farm with a woken recurring yak, an overdue review yak, a due-soon one, an ask and a claim: the report read well, and the second tick regrew nothing.

Not done, deliberately: `--sync`. The design said it needs a two-machine duplicate-regrow test first; a tick does not sync, and the skill will say to run `yaks sync` yourself. Filed as a follow-up below if wanted.

Decision: stale claims come from the sheds' own `Stale` liveness (two hours), not a new threshold. Decision: a woken yak due soon appears once, in the inbox section.
