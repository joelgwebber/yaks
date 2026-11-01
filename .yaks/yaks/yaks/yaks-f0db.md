---
id: yaks-f0db
title: 'TUI: auto-sync the farm on an interval'
status: shaving
type: feature
priority: 2
created: '2026-11-01T20:09:16Z'
updated: '2026-11-01T22:26:26Z'
parent: yaks-200f
labels:
- wiki
- tui
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
