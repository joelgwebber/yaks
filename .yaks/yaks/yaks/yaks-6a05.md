---
id: yaks-6a05
title: TUI visibly slower when typing / navigating with many yaks visible
status: shorn
type: bug
priority: 3
created: '2026-11-10T19:11:25Z'
updated: '2026-11-10T21:45:08Z'
tags:
- perf
- tui
verify: cargo test --workspace
---

Pretty clear on a TUI in a large-ish ~200x100 cell window, on the yaks-repo's "All" tab (~750 yaks). Every keystroke has a visible latency.

---
▸ 2026-11-10T21:11:50Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T21:12:24Z [herd-unlock]

Claim (yakherd herd-unlock-2): worker `perf-1`, Sonnet 5.5.

## Scope

Measure first, then fix. Build a farm of ~800 yaks (a test fixture or a scratch farm), drive the TUI at 200x100 on the All view, and time one keystroke's work (`handle_key` + `render`) per key (`j`, typing into the search). Find where the time goes (likely candidates: `filter::resolved_ids` and the full filter/sort recomputed per frame in `flat_rows`/`view_count` for every tab's count, the tab bar counting every view, markdown/detail rendering of the selected yak). Fix the biggest costs within `src/tui/viewmodel.rs`, `src/tui/render.rs` and the existing memo/cache (`src/tui/memo.rs`, `src/tui/cache.rs`); the derived values must stay derived (no second source of truth).

## Out of scope

The on-disk format; the farm loading path (unless it runs per key).

## Evidence

Before/after timings per keystroke on the same fixture, in a note, with the command that measures them (a `#[ignore]` bench test is fine). Target: under 16 ms per keystroke at 800 yaks in a debug-free (`--release`) build, or say what stands in the way. Gate `cargo test --workspace`; no snapshot changes.

---
▸ 2026-11-10T21:13:20Z [perf-1]

Start 2026-11-10T21:13:18Z; checkout /Users/joel/src/yaks/.delta/worktrees/73m8tackk228/yaks

---
▸ 2026-11-10T21:38:08Z [perf-1]

Measured (release build, 800 synthetic yaks, 200x100, `handle_key` + `render` to a TestBackend):

- Command: `cargo test --release --bin yaks keystroke_bench -- --ignored --nocapture` (new `#[ignore]` bench in `src/tui/memo.rs`).

Before, mean per keystroke:

- Hairy tab `j`: 7.2 ms (handle_key 0.35, render 6.9)
- All tab `j`: 15.1 ms (handle_key 0.7, render 14.4)
- All tab search typing: 13.0 ms
- Detail open: 12.8 ms

Cause: `render_list` called `app.is_multi_herd()` once per visible row. That walks every yak and builds a `BTreeSet` of herds (`herd_choices`), so a frame was visible rows x yaks. The All tab shows ~100 rows at 200x100, so it cost the most.

Fix: hoist `is_multi_herd()` out of the per-row closure in `render_list` (`src/tui/render.rs`), one call per frame. Nothing cached, so no new derived state.

After, mean per keystroke: Hairy `j` 2.5 ms, All `j` 3.4 ms, All typing 3.2 ms, detail open 3.0 ms (max 5.4 ms). Under the 16 ms target.

Cross-check with the real binary on a copy of this farm (872 files): 200 `j` keys at 200x100 took 0.78 s after against 0.84 s before. That wall time includes ~0.35 s of startup and is noisy, so the bench is the better measure.

Decision: hoist, not memoize. Rejected: caching `herd_choices` or `view_count` in `App`. The hoist fixes the only quadratic; what remains is about 2.5 ms per frame, linear in yaks.

Decision: left `view_count`, `blocked_ids` and `resolved_ids` uncached. Each is O(n) once per frame (tab bar x ~7 tabs), and the shorn-filter worker is editing `flat_rows`/`view_count` in parallel.

Decision: bench lives in `memo.rs` `#[cfg(test)]` next to the cache it exercises, `#[ignore]`d. Rejected: a headless shell script (the test harness gives the handle_key/render split).

---
▸ 2026-11-10T21:45:03Z [perf-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-10T21:45:08Z [perf-1]
kind: move
from: shaving
to: shorn
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFVQdJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
