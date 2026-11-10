---
id: yaks-6a05
title: TUI visibly slower when typing / navigating with many yaks visible
status: shaving
type: bug
priority: 3
created: '2026-11-10T19:11:25Z'
updated: '2026-11-10T21:13:20Z'
tags:
- perf
- tui
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
