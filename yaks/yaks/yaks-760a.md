---
id: yaks-760a
title: 'TUI: a source with a large extracted text is slow to move in (~150 ms per key on YASS)'
status: shaving
type: bug
priority: 2
created: '2026-11-17T21:16:48Z'
updated: '2026-11-17T22:06:12Z'
parent: yaks-4cfe
tags:
- tui
- perf
- doks
---

Part of yaks-4cfe. In the Doks tab, opening `sources/2026-11-05-04642f1f-yass-...` (YASS) and holding `j` costs about **150 ms per key** (headless, 120x40, 30 × `j`: 5.7 s against 1.1 s for 0).

## Measured cause

I built two copies of it in a scratch farm:

- `source.md` with its 250-line `files:` list and a one-line `extracted.md`: 30 keys took 0.04 s more than none. **The file list is not the problem.**
- `source.md` without the list and the real 842 KiB `extracted.md`: 30 keys took 4.4 s more than none. **The appended text is the cause.**

`read_farm_doc` (`src/tui/follow.rs`) caps `source.md` at `VIEW_MAX_BYTES` (1 MiB), but reads `extracted.md` uncapped and renders all of it under the source. Every keypress seems to re-render or re-wrap the whole document.

## Fix

- Re-render only on change. Cache the rendered rows per width, as yaks-bc30 did for the detail pane, so a cursor move doesn't rebuild 11k lines.
- Then cap or page what is shown. Even after yaks-4cfe's layout decision a big PDF text will be this size.

Evidence: the same headless timing before and after, and a test that pins how many renders a keypress causes.

#### 2026-11-17T21:42:27Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-17T21:43:00Z

*sources-triage*

Claim (yakherd `sources-triage`). Owner: worker `srcperf-1`. Scope: the text viewer's per-keypress cost on a big document (`AttachView` in `src/tui/handlers.rs`, its rendering in `src/tui/render.rs`, `read_farm_doc` in `src/tui/follow.rs`). Render and wrap once per (text, width) and reuse it across cursor moves, as yaks-bc30 did for the detail pane.

- Out of scope: the source layout, and the new directory view (yaks-3329, worker `browse-1`, which adds a `DocView` variant). Don't restructure `DocView`.
- Evidence: the headless timing in the description, before and after, on a scratch farm holding the YASS text (copy `sources/2026-11-05-04642f1f-*/extracted.md` from the real farm read-only), plus a test that pins the renders per keypress.

#### 2026-11-17T21:44:51Z

*srcperf-1*

Start: worker srcperf-1. `date -u`: Tue Nov 17 21:44:47 UTC 2026. `yaks path yaks-760a`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-760a.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/y0m0g7nb9wf6/yaks

#### 2026-11-17T22:04:54Z

*srcperf-1 · kind: verify · result: pass*

verify: `cargo test --workspace` -> PASS (exit 0)

#### 2026-11-17T22:06:11Z

*srcperf-1*

Decision: cache the viewer's wrapped rows on the `AttachView` itself (`rows_cache`, filled by `App::viewer_rows` in `src/tui/memo.rs`), keyed on (`data_gen`, width, inline-image layout); the text is not in the key because every text change goes through the new `AttachView::set_text`, which drops the cache. Rejected: a cache on `App` keyed on a hash or copy of the text (hashing 842 KiB per call is a cost of its own, and one cache would serve only one viewer); a key holding the text length (a same-length edit would go stale).

Decision: `attachment_rows()` now returns `Rc<Vec<DLine>>` (derefs like the Vec, as `detail_dlines` did in yaks-bc30), so its 20-odd callers did not change. The one test helper that named the type changed.

Decision: under `cfg(test)` every cache hit is asserted equal to a fresh build ("stale viewer rows cache"), as yaks-bc30 did, so all 1200+ existing viewer tests double as an invalidation check.

Decision: no cap or paging of the extracted text. After the cache, the remaining cost per key is the ~6 ms `render_nav_rows` pass over all rows. Capping would change what a user sees (docs/tui.md); the description says "then cap or page", a layout call that belongs to yaks-4cfe. Left as a follow-up: paint only the visible window in `render_nav_rows` (find_matches scans every row too).

#### 2026-11-17T22:06:12Z

*srcperf-1*

Summary. Cause: temporary timers (removed) on the YASS source showed `AttachView::rows` (md_rows over the 842 KiB extracted.md, 11k rows) at about 50 ms per call, called 3 times per `j` (once by `render_attachment`, twice by `attachment_rows()` from the key handler and the status/find line), plus 6 ms in `render_nav_rows`. Nothing was cached, so each key re-parsed and re-wrapped the whole document.

Fix: `App::viewer_rows` (src/tui/memo.rs) memoizes the rows on the `AttachView` per (data_gen, width, image layout); `set_text` invalidates. `render_attachment` and `attachment_rows` use it. Output unchanged: `--diff` headless output for 30 j is byte-identical (`cmp`) between the before binary and after. No docs: no user-visible change.

Test: `a_viewer_keypress_builds_its_rows_once_per_width_and_text` (doks_tab.rs, counter `VIEWER_ROWS_CALLS`): 1 build on first frame, 0 across j/j/k/j/G/g with a draw and two `attachment_rows()` calls each, 1 on a resize, 0 on a redraw, 1 after `set_text`.

Timing (scratch farm holding the real YASS source, release build, `tui --headless --size 120x40 --diff`, wall clock `time -p`; the machine was noisy, runs vary +-20%):
- before: 0 keys 0.72-0.85 s; 30 keys 5.86, 5.96 s; 100 keys 18.5, 21.1 s => about 170-190 ms per key (matches the 5.03 s of the first, quieter, run).
- after: 0 keys 0.41-0.43 s; 30 keys 0.57, 0.72 s; 100 keys 1.09, 1.17 s => about 6-7 ms per key.
That is roughly 25-30x per key; the rest is `render_nav_rows` over 11k rows.

Gate: `cargo fmt --check` clean; `cargo test -p yaks`: no failures (1239 passed, 18 ignored in the unit tests); `yaks verify yaks-760a` PASS from the checkout root.
