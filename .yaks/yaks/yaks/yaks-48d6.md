---
id: yaks-48d6
title: 'Perf: cursor movement in some complex detail panes is slow'
status: shorn
type: bug
priority: 1
created: '2026-10-17T14:52:45Z'
updated: '2026-10-17T16:55:06Z'
parent: yaks-faf5
labels:
- tui
- perf
---

Eg, in yaks-a893, it's clearly slamming the CPU when I move the cursor around through the description in view-mode.

---
▸ 2026-10-17T16:52:15Z [delta-perf]

moved: hairy -> shaving

---
▸ 2026-10-17T16:54:33Z [delta-perf]

Diagnosis (delta-perf, 2026-10-17), so nobody has to redo it.

**How it was measured.** No profiler is installed (`perf` and `samply` are absent; `perf_event_paranoid` is 2). Instead:

- A script drives `yaks tui --headless --diff --size 120x50` from `/home/joel/src/yaks` (live sheds) with `key Tab`, `key /`, `type unify view and edit`, `key Enter`, `key l`, then N × `key j`, and times the run. Per-key cost = (time at N − time at 0) / N.
- Temporary `eprintln!` timers and a `std::backtrace::Backtrace::force_capture()` in `App::rows` (release build with `CARGO_PROFILE_RELEASE_STRIP=false CARGO_PROFILE_RELEASE_DEBUG=1`) showed who calls what per keypress. All removed afterwards.

**What one `j` cost before the fix.**

- About 31 calls to `App::rows()` (the visible list: tree, filters, shed overlay) at about 1.5 ms each on this farm (461 yaks with the shed-only ones): about 45 ms.
- 4–5 calls to `detail_dlines()` (detail pane lines) at about 3.5 ms each.
- yaks-a893: about 61 ms per key in the harness, against about 11 ms for a plain yak. Holding `j` pegs a core.

**Root cause.** `render_detail` called `App::block_hscroll` for every row of every code block or table (added with sideways scrolling, yaks-c733), and `block_hscroll` called `self.selected()`, which rebuilds `rows()` to find the cursor's yak. yaks-a893 has an 18-row code block, so that alone was 18 view rebuilds per frame.

**Aggravating factors, still present (follow-up yak).**

- `selected()` is always a full `rows()` rebuild, and so is everything built on it: `detail_line_count()`, `detail_dlines()`, `detail_jumps()`, `selected_shed_only()`, used several times per keypress by `move_detail_line`, `scroll_line_into_view`, `detail_max_scroll`, the status bar's `link_hint` and the renderer.
- For a yak that exists only in another shed (yaks-a893 in Joel's checkout), `detail_dlines()` calls `rows()` a second time via `selected_shed_only()` and clones all 461 tasks.

---
▸ 2026-10-17T16:54:41Z [delta-perf]

Quick fix (delta-perf): `App::block_hscroll` takes the yak id from its caller (`render_detail` already holds `task`; `scroll_block` already had `id`) instead of calling `selected()`. A doc note on `App::rows` now says what it costs and that per-row and per-frame loops must not reach it.

Evidence:

- New test `rendering_a_code_block_does_not_rebuild_rows_per_row` (src/tui/tests.rs), using a new test-only counter `viewmodel::ROWS_CALLS`, failed first: 45 `rows()` rebuilds in one frame of a 40-line code block. With the fix it passes (≤ 10).
- Same harness on Joel's checkout, yaks-a893 with 60 × `j`: 3.68 s before, 2.06 s after, about 61 → 34 ms per key (the 0.6 s startup excluded).
- `cargo test --release`: all 18 suites ok.

The rest (about a dozen `rows()` per key remain) is the follow-up yaks-bc30.

---
▸ 2026-10-17T16:55:06Z [delta-perf]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-17T16:55:06Z [delta-perf]

moved: shaving -> shorn
