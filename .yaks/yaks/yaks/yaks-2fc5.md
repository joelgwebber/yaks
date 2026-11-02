---
id: yaks-2fc5
title: 'Due dates and recurring yaks: due:/every: fields, list --due, overdue, yaks regrow-due, TUI tick'
status: shaving
type: feature
priority: 2
created: '2026-11-02T00:36:10Z'
updated: '2026-11-02T00:36:11Z'
parent: yaks-9cc1
labels:
- wiki
- cli
---

Implements yaks-9cc1's proposal as Joel decided it; read that yak's proposal and decision notes.

---
▸ 2026-11-02T00:36:10Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-02T00:36:11Z [delta:farm-wiki]

Claim (yakherd delta:farm-wiki): worker `due-1`, on branch `wiki` at f4c86b8.

**Spec.** The proposal and Joel's answers are on yaks-9cc1.

- **Fields** (extra yak fields, through `Fields`):
  - `due:` is a date (`2026-11-15`) or an RFC 3339 datetime.
  - `every:` is an ISO 8601 duration only: `P1D`, `P2W`, `P1M`, `P3M`, `P1Y`, and combinations in Y M W D order. Reuse the parser `src/check.rs` has (`stale_after`) by moving it to one shared place.
  - `regrown:` is a datetime written by yaks.
  - `yaks create`/`update` take `--due <date>` and `--every <dur>` (sugar for `--set`). Validate them: refuse an unparseable value with a message.
- **Display:**
  - `list`/`next`/`show` show `due` (`--json` too).
  - A yak still hairy or shaving whose `due` is in the past is overdue. Mark it in the CLI list (`overdue` after the date) and in the TUI list row (the date in red, using the existing warn style) and the detail header.
  - `yaks list --due [--before <date>]` filters to yaks with a `due`, sorted by it.
- **`yaks regrow-due [--dry-run] [--json]`:** for each shorn yak with `every:`, compute the next occurrence:
  - Start from `regrown` (else `due`, else the shear time); the occurrence is the latest `start + k*every` that is not in the future, with k ≥ 1. If one exists, regrow the yak: move it to hairy, set `due:` to that occurrence plus `every` (the next deadline), set `regrown:` to now, and add a `kind: regrow` note saying which occurrence it is and how many were skipped.
  - Hold the farm lock: one command, one commit per yak through per-change commits.
  - It is idempotent: a second run does nothing.
  - Dead yaks are never regrown.
- **TUI:** run the same regrow on load and after each auto-sync tick (`src/tui/farmsync.rs`, `sync_interval`), and reload after.
- **Schema:** document `due`/`every`/`regrown` as built-in-known extras, so doctor's `schemas/yak.md` check never flags them.

**Scope.**

- A new `src/recur.rs`.
- `src/main.rs`: the `RegrowDue` variant plus arm, the `--due`/`--every` flags on create/update, and the list/next/show rendering.
- The TUI list/detail rendering for due dates, and the tick in `farmsync.rs`.
- `src/check.rs` for the shared duration parser.
- Docs: `docs/cli.md`, `docs/tui.md`, the `yaks` skill (the Task format section, plus a short "Due dates and recurring yaks" note), `README.md`, and `help_content` if any key changes.

**Out of scope.** Word schedules (`monday`).

**Evidence.**

- Unit tests of the occurrence math: catch-up skips, idempotence, month-end (`P1M` from Jan 31).
- An integration test in `tests/recur.rs`: create with `--every P1W --due <two weeks ago>`, shear, `regrow-due` regrows once with the right `due` and note, and a second run does nothing.
- A TUI snapshot with an overdue date.
- The full gate passes.

**Judge.** The coordinator, at landing.
