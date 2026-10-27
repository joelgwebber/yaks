---
id: yaks-bc30
title: 'Perf: cache rows() and the detail lines between input changes'
status: shorn
type: task
priority: 2
created: '2026-10-17T16:54:33Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-faf5
labels:
- tui
- perf
needs: agent
---

Follow-up to yaks-48d6 (read its diagnosis note first: harness, numbers, call graph).

`App::rows()` rebuilds the whole visible list on every call (about 1.5 ms on a 450-yak farm), and `selected()`, `detail_dlines()`, `detail_line_count()`, `detail_jumps()` and `selected_shed_only()` all go through it. After the yaks-48d6 quick fix a `j` in the detail pane still runs it about a dozen times plus 4–5 `detail_dlines()` builds: about 34 ms per key on yaks-a893 in Joel's checkout.

**Goal:** one `rows()` rebuild and at most one detail-lines build per keypress or frame.

**Shape:** memoize `rows()` and the wrapped detail lines on `App` (`RefCell<Option<…>>`, like `detail_width`), keyed on, or invalidated by, everything they read: the farm tasks (`all`), the active view and live filter, collapsed set, family scope, cursor, the sheds snapshot, `detail_width`, the clock's minute (relative dates), and the selected yak's body. The risk is a stale cache, so prefer one `invalidate()` called from the few mutation points (reload, view/filter change, collapse, cursor move, snapshot apply, resize) and a debug assertion in tests that the cached value equals a fresh build.

**Evidence:** extend the `ROWS_CALLS` test counter (src/tui/viewmodel.rs) with a test pinning `rows()` to at most 1 per keypress and per frame; the headless timing harness from yaks-48d6 before and after.

---
▸ 2026-10-17T18:14:58Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T18:14:58Z [delta-lead]

Claim (yakherd delta-lead, 2026-10-17): owner **cache-1**. Joel: "peel this off to a worker so it doesn't bite us in the future".

Read yaks-48d6's diagnosis note first (harness, numbers, call graph, the `viewmodel::ROWS_CALLS` counter).

Scope:
- src/tui/viewmodel.rs (`rows()` and its callers' hot paths) and src/tui/detail_nav.rs (`detail_dlines()`): memoize each behind an explicit cache invalidated by a cheap generation key. Bump it on everything that can change the output: reload/watch events, view/filter/tab change, collapse/expand, selection change, sheds snapshot, detail width, clock tick for relative dates (a coarse once-per-minute bucket is fine), star/mark changes. A `Cell`/`RefCell` on `App` is fine since render takes `&App`.
- src/tui.rs: the cache fields on `App`.
- Prefer one invalidation point (a `touch()` / generation counter) over scattered `cache = None`. Every mutation that affects output must go through it; add a debug assertion or test that a stale cache can't be observed (e.g. compare cached vs fresh in `cfg(test)`).

Out of scope: rendering changes, mdrender.rs internals.

Evidence:
- Failing-first tests via `ROWS_CALLS` and a similar counter for `detail_dlines`: one rebuild per input change, zero for a plain re-render or a cursor move that doesn't change the selection.
- The yaks-48d6 harness numbers before/after on this repo's real farm (450+ yaks).
- The full suite stays green; in particular every snapshot is unchanged (a cache must not change output).

Judge: coordinator at landing.

---
▸ 2026-10-17T18:38:52Z [cache-1]

Started cache-1, 2026-10-17 18:15 UTC. `yaks path yaks-bc30` = /home/joel/src/yaks/.delta/worktrees/2ctx82g6p0j3/yaks/.yaks/yaks-bc30.md (my checkout).

Baseline harness (target/bench/run.sh; this repo's `.yaks/` copied to target/bench/farm; `yaks tui --headless --diff --size 120x50`: Tab Tab, `/`, "unify view and edit", Enter, 6×j to land on yaks-a893, `l`, then N×j): 0 keys ~1.07 s; 60 keys 4.8–7.1 s on a noisy machine, i.e. roughly 60–100 ms per key. The unmodified release binary is target/bench/yaks-before.

---
▸ 2026-10-17T18:46:45Z [cache-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-17T18:47:02Z [cache-1]

Decision: key the caches on the inputs themselves, not on a scattered `cache = None`. `rows()` is keyed on (view key/sort/limit, live filter, collapsed set, family-scope map, starred set, `data_gen`); detail lines on (`data_gen`, selected yak's location, `copies: home`, detail width, the in-place edit slot, the clock's minute). Only the two big inputs, `App::all` and `sheds.snapshot`, use a generation counter: `App::touch()` (src/tui/memo.rs), called from `reload`, `reload_preserving_selection` and `apply_sheds_snapshot`. So the single invalidation point is `touch()`, and the cheap inputs cannot be forgotten. Rejected: `cache = None` at every mutation site (tui.rs, handlers.rs, actions.rs and viewmodel.rs mutate filter/view/collapsed in ~15 places); a hash of `all` per call (costs as much as the rebuild).

Decision: the cursor is deliberately NOT in the rows key (rows don't depend on it); the detail-lines key holds the selected yak's `Loc`, so a cursor move to the same yak is free and a move to another yak rebuilds the lines only. The yak's body is covered by `data_gen`: every body change goes through the farm and `reload`.

Decision: a cached row holds a `Loc` (index into `all`, or into a shed's `only_tasks`) instead of `&Task`, so the cache owns no borrow and `rows()` keeps its `Vec<Row<'_>>` signature (no caller changed). `locate()` maps a freshly built row's `&Task` back to its index by pointer arithmetic over those two slices; it is only used when building the cache. Rejected: changing `tree::build_over` to emit indices (touches the tree builder and its tests, out of scope); looking tasks up by id (a hash lookup per row, and ids are not unique between a home yak and a shed copy).

Decision: `detail_dlines()` now returns `Rc<Vec<DLine>>` (derefs like the Vec, no caller changed), and `selected_shed_only()` reads the `Loc` instead of rebuilding rows plus an id search.

Decision: new `App::row_count()` for the callers that only needed `rows().len()`. `selected()` / `selected_shed_only()` no longer build `Row`s at all.

Decision: staleness is checked, not trusted: under `cfg(test)` every cache hit asserts equality with a fresh build (`stale rows cache` / `stale detail-lines cache`), so all 700+ existing tests double as an invalidation check; a test pins that a missed `touch()` panics. `FilterSpec`, `DLine` and `EditSlot` gained `PartialEq` for this (and for the key).

Tests (src/tui/tests.rs `mod memo`), failed first: with both caches defeated (`is_current` and the detail hit forced to miss) four of the six fail with `'j' rebuilt rows/lines (13, 4) times`, `(8, 2)` vs `(0, 1)`, `(27, 8)` vs `(0, 0)`, and the missed-touch test no longer panics. Restored: all six pass. The existing `rendering_a_code_block_does_not_rebuild_rows_per_row` still passes.

Not changed: output. No insta snapshot moved (`git status` shows none); the headless harness frames, 20×j before vs after, are byte-identical (`diff` clean). No docs/help change: no flag, key or format changed (internal perf only).

Numbers (target/bench/run.sh, this repo's `.yaks/` copy, 120x50, yaks-a893 detail pane, 60×j, release; the machine was quieter than in the first note, which is why 'before' is lower than 4.8–7.1 s):
- before: 0 keys 963 ms; 60 keys 2287, 2291, 2294, 2277 ms, so about 22 ms per key.
- after: 0 keys 916, 910 ms; 60 keys 953, 957, 965, 964 ms, so about 0.7 ms per key.
That is about 30× per key; startup dominates what is left.

Gates: `cargo test --workspace` 825 passed 0 failed; `cargo test --release` 825 passed 0 failed.

---
▸ 2026-10-17T18:47:02Z [cache-1]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-bc30` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-bc30 --drop-request`, which removes this request note, so no new note is left behind.
