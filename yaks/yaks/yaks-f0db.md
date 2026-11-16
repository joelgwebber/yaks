---
id: yaks-f0db
title: 'TUI: auto-sync the farm on an interval'
status: shorn
type: feature
priority: 2
created: '2026-11-01T20:09:16Z'
updated: '2026-11-04T20:03:48Z'
parent: yaks-200f
tags:
- tui
- doks
---

Decided on yaks-63c3: the TUI syncs the farm on a regular interval (fetch, merge with the driver, push) so upstream changes appear even with no local edits. Interval in config; off for a private farm (it never pushes, but still pulls when it has an upstream).

---
▸ 2026-11-01T22:25:16Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T22:25:17Z [delta:farm-wiki]

Claim (yakherd delta:farm-wiki): worker `sync-1`, on branch `wiki`, base a150387.

**Task.** Joel decided on yaks-63c3 that the TUI syncs the farm on an interval, so upstream changes appear even with no local edits.

- A new config key `sync_interval` in `.yaks/config.yaml` takes a duration in seconds or `5m`/`30s` form. Default `5m`; `0` or `off` disables it.
- On each tick the TUI runs the same thing as `U` (`sync::run`), off the UI thread so the UI never blocks. Skip the tick when:
  - the farm has no remote;
  - a previous sync is still running;
  - an editor or overlay is open.
- A private farm (`privatefarm::is_private`) pulls but never pushes: `sync::Opts { no_push: true }`.
- Reload the farm afterwards, as `U` does. A failure shows as the usual notification once, not every tick.
- Respect `YAKS_NO_AUTOSYNC=1`, so tests and headless runs never sync.

**Scope.**

- `src/tui/farmsync.rs`, `src/tui/runtime.rs` (the tick), and the config read in `src/store.rs` (`Config`: one new field; check every place that builds `Config {`).
- Docs: `docs/tui.md`, `docs/cli.md`'s config section, the `?` help overlay (`help_content` in `src/tui/render.rs`) where it describes `U`.

**Out of scope.** Everything else, including the sync logic itself (`src/sync.rs`). Ask first.

**Evidence.**

- A unit test of the interval parsing and of the skip rules.
- A headless toque run or integration test showing that a commit pushed to the farm's remote from another clone appears in the TUI after a tick with a short interval (set it in the test).
- The full gate passes.

**Judge.** The coordinator, at landing.

---
▸ 2026-11-01T22:26:26Z [sync-1]

Started 2026-11-01T22:26Z. yak path: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-f0db.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/wwj5ev7qs72e/yaks (branch wiki).

---
▸ 2026-11-01T22:42:33Z [sync-1]

Decision: `sync_interval` is parsed by `store::parse_sync_interval` (bare seconds, or `s`/`m`/`h`; `0`/`off`/`false` = off; `false` because an unquoted YAML `off` is a boolean). An unparseable value keeps the 5m default rather than silently turning syncing off.
Decision: the background sync runs on a std thread with an mpsc result channel, polled each event-loop turn (250ms poll). `U`/`W` (`run_pending_farm_op`) first waits for any running auto-sync, so two git rebases never race on the farm.
Decision: a tick that is skipped (overlay open, queued W/U, no remote) still pushes `next` out by one interval, except overlay/pending-op skips which leave it due, so it fires as soon as the UI is idle. No-remote ticks are spent so git is not probed every frame.
Decision: failures are de-duplicated by message (`auto-sync failed: ...` shown once; cleared by the next success), so a down remote is not re-announced every tick. Success is silent.
Decision: private farms pass `no_push: true` (sync::run already skips the push for them; this just says it explicitly). They still pull when there is a remote.
Decision: no remote is detected in the TUI (`has_remote`, same rule as sync.rs: branch.<b>.remote else origin) to avoid starting a thread; sync.rs untouched.
Evidence: unit tests in src/tui/farmsync.rs (interval/skip rules, env switch, no remote, overlay/queued op, a commit pushed from a second clone appears in `app.all` after a 10ms-interval tick, private farm does not push, failing remote notifies once) and src/store.rs (parsing + config). Help snapshot updated for the new `auto` row. Gate: cargo test --workspace 934 passed, 0 failed, 12 ignored.

---
▸ 2026-11-01T22:45:05Z [sync-1]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T22:45:09Z [sync-1]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOAD0otJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-01T22:46:02Z [delta:farm-wiki]

kind: landed
commit: 6a225507d92e83e6390ee0859737771d03853f4a
source: 44728a02d51491902a6144f740358a92fdcfadf9
branch: wiki
---

landed: 6a22550 yaks-f0db: TUI syncs the farm on a sync_interval (default 5m, off with 0/off, YAKS_NO_AUTOSYNC)

---
▸ 2026-11-01T22:51:31Z [delta:farm-wiki]

Landed as 6a22550 (yakherd). Gate: 941 passed. Not yet driven in a live `yaks tui` against a real remote; the unit tests use real git clones. Joel's next `yaks tui` on this build will show it.
