---
id: yaks-2fc5
title: 'Due dates and recurring yaks: due:/every: fields, list --due, overdue, yaks regrow-due, TUI tick'
status: shorn
type: feature
priority: 2
created: '2026-11-02T00:36:10Z'
updated: '2026-11-04T20:03:34Z'
parent: yaks-9cc1
tags:
- cli
- doks
verify: 'cd /Users/joel/src/yaks/.delta/worktrees/g5vagmccz0gg/yaks && cargo test --workspace > /tmp/due1-verify.txt 2>&1; rc=$?; grep "test result" /tmp/due1-verify.txt | awk "{p+=\$4; f+=\$6} END {print \"passed\", p, \"failed\", f}"; echo "error/FAILED lines: $(grep -c "FAILED\|^error" /tmp/due1-verify.txt)"; exit $rc'
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

---
▸ 2026-11-02T00:36:58Z [due-1]

Started Mon Nov  2 00:36:57 UTC 2026 by due-1. Yak file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-2fc5.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/g5vagmccz0gg/yaks (branch wiki, f4c86b8).

---
▸ 2026-11-02T01:13:23Z [due-1]

Progress ($(date -u) approx). Done so far: src/recur.rs (parser moved from check.rs; When/Every/plan/due_rows/due_mark + unit tests), Farm::regrow_due, --due/--every on create/update, list --due [--before], regrow-due command, due in list/next/show/json, doctor known extras. Next: scratch-farm run, tests/recur.rs, TUI (list row, detail header, tick), docs.

Decision: the schedule anchors on `due:` once a yak has been regrown (due = last occurrence + period, so it IS the next occurrence, k from 0), not on `regrown:`. The spec says "start from regrown (else due ...)"; counting from regrown (the moment yaks ran) lets every late regrow push the schedule later (drift). `regrown:` is the anchor only when `due:` is absent. Never-regrown yaks count from due (k>=1), else the shear time.
Decision: a date-only `due:` is overdue once its day has passed (end of day UTC); a datetime the moment it passes. Date-only due stays date-only after a regrow.
Decision: a recurring yak under a shorn parent reopens the parent first (child: note like shave lifts); under a dead parent it is held and reported, not revived.

---
▸ 2026-11-02T01:25:38Z [due-1]

kind: verify
result: pass
---

verify: `cd /Users/joel/src/yaks/.delta/worktrees/g5vagmccz0gg/yaks && cargo test --workspace > /tmp/due1-verify.txt 2>&1; rc=$?; grep "test result" /tmp/due1-verify.txt | awk "{p+=\$4; f+=\$6} END {print \"passed\", p, \"failed\", f}"; echo "error/FAILED lines: $(grep -c "FAILED\|^error" /tmp/due1-verify.txt)"; exit $rc` -> PASS (exit 0)

---
▸ 2026-11-02T01:25:52Z [due-1]

Summary. Due dates and recurring yaks are in. Gate: `cargo test --workspace` 1015 passed, 0 failed, 0 error/FAILED lines (recorded by `yaks verify`).

What changed:
- New `src/recur.rs`: `parse_duration` (moved from check.rs, which now calls it), `When` (date or RFC 3339), `Every`, the pure `plan(task, now)`, `due_rows`, `due_mark`, `field_problem`, and 20 unit tests (catch-up skips, idempotence, Jan 31 + P1M, no-drift after a late regrow, long gap by gallop+bisect).
- `Farm::regrow_due(now, dry_run, actor)`: one farm lock and one commit per yak; reopens shorn ancestors first (`child:` on their move note); holds a yak under a dead ancestor or with an unreadable field.
- CLI: `create`/`update --due/--every` (validated, empty clears), `list --due [--before]`, `regrow-due [--dry-run] [--json] [--as]`, `due ... overdue` in list/next, `due/every/regrown` rows in show, `overdue: true` in --json. `NewTask.fields` carries them on create.
- doctor: `due/every/regrown` are known extras, never flagged against `schemas/yak.md`.
- TUI: red/dim due date at the right of an open row, `Due:` (warn accent when overdue)/`Every:`/`Regrown:` detail rows, `regrow_due` on load and on each tick (also with no remote), with a notice and a reload.
- Docs: docs/cli.md (new section), docs/tui.md, README.md, the yaks skill (Task format + new section), clap help. No key changed, so `help_content` is untouched.
- Tests: tests/recur.rs (3 integration), a TUI snapshot (`an_overdue_date_shows_red_...`), a TUI test for regrow on load and on the tick.

Scratch-farm run (target/debug/yaks, temp farm):
$ yaks create "Check the water softener salt" --every P1W --due <14 days ago>; yaks shorn <id>
$ yaks regrow-due --dry-run
Would regrow: yak-2972  Check the water softener salt (for 2026-11-02, 1 skipped; due 2026-11-09)
$ yaks regrow-due
Regrown: yak-2972  Check the water softener salt (for 2026-11-02, 1 skipped; due 2026-11-09)
$ yaks regrow-due
Nothing to regrow.
show: status Hairy, due 2026-11-09, every P1W, regrown 2026-11-02T01:22:54Z, note "regrew for the 2026-11-02 occurrence (1 skipped)". Farm log: created, shorn, regrown (one commit each).
`yaks create x --every monday` exits 1: "monday" is not an ISO 8601 duration.

Decision: `--set due=...` / `--set every=...` stay unchecked (the generic escape hatch; doctor flags a bad one against a `date` schema); only `--due`/`--every` validate. Validating inside --set broke an existing test that sets `due=soon` on purpose to see doctor warn.
Decision: the `kind: regrow` note carries `occurrence:` in its metadata and is stamped when written, so it follows the move note.
Decision: the TUI also regrows on a tick when there is no remote (the tick is the clock); `YAKS_NO_AUTOSYNC` stops the ticks, not the load-time pass.
Decision: `list --due` composes with the shared filters; `--before` is exclusive and requires `--due`. `next` shows due/overdue too.

Not done / noted: not run in a live TUI session (covered by in-process tests and the snapshot, no docshots scene added). The first design-note wording "regrown + every" is replaced by the due-anchored rule above (see the earlier Decision).

---
▸ 2026-11-02T01:25:52Z [due-1]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOAD38RZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-02T01:26:21Z [delta:farm-wiki]

kind: landed
commit: 7560285d96f72fc9bf16f20ae079228798dd79f7
source: eeb48f4e33d07cccd8d043cbced596218cce97b4
branch: wiki
---

landed: 7560285 yaks-2fc5: due dates and recurring yaks (due:/every:, list --due, overdue, regrow-due, TUI tick)

---
▸ 2026-11-02T01:29:27Z [delta:farm-wiki]

Landed as 7560285 (yakherd). Gate: 1015 passed. I reproduced it on a scratch farm: a weekly yak due 15 days ago listed as overdue; once shorn, `regrow-due --dry-run` then `regrow-due` regrew it for 2026-11-01 (1 skipped, next due 2026-11-08); a second run did nothing; there was one commit per move.
