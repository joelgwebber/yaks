---
id: yaks-bc30
title: 'Perf: cache rows() and the detail lines between input changes'
status: hairy
type: task
priority: 2
created: '2026-10-17T16:54:33Z'
updated: '2026-10-17T16:54:33Z'
parent: yaks-faf5
labels:
- tui
- perf
---

Follow-up to yaks-48d6 (read its diagnosis note first: harness, numbers, call graph).

`App::rows()` rebuilds the whole visible list on every call (about 1.5 ms on a 450-yak farm), and `selected()`, `detail_dlines()`, `detail_line_count()`, `detail_jumps()` and `selected_shed_only()` all go through it. After the yaks-48d6 quick fix a `j` in the detail pane still runs it about a dozen times plus 4–5 `detail_dlines()` builds: about 34 ms per key on yaks-a893 in Joel's checkout.

**Goal:** one `rows()` rebuild and at most one detail-lines build per keypress or frame.

**Shape:** memoize `rows()` and the wrapped detail lines on `App` (`RefCell<Option<…>>`, like `detail_width`), keyed on, or invalidated by, everything they read: the farm tasks (`all`), the active view and live filter, collapsed set, family scope, cursor, the sheds snapshot, `detail_width`, the clock's minute (relative dates), and the selected yak's body. The risk is a stale cache, so prefer one `invalidate()` called from the few mutation points (reload, view/filter change, collapse, cursor move, snapshot apply, resize) and a debug assertion in tests that the cached value equals a fresh build.

**Evidence:** extend the `ROWS_CALLS` test counter (src/tui/viewmodel.rs) with a test pinning `rows()` to at most 1 per keypress and per frame; the headless timing harness from yaks-48d6 before and after.
