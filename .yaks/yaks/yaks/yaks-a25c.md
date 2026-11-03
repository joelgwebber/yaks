---
id: yaks-a25c
title: 'TUI phase 6a: global query with its own chip, and the All view'
status: shaving
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T04:43:08Z'
parent: yaks-947a
depends_on:
- yaks-5d08
- yaks-c354
tags:
- tui
---

Phase 6, the list half, of [the design](/wiki/design/tui-model.md#4-search) (yaks-953f, yaks-c7ba): the `/` query leaves the view's filter, survives view switches, matches what `yaks find` searches, and has its own chip with the All count; tab counts apply it; Esc clears it first. A built-in unpinned All view holds every yak (dead included), page and source.

---
▸ 2026-11-03T04:28:36Z [herd-tui]

Claim (yakherd herd-tui): worker `query-1` (Sonnet 5.5), from main after phase 3b lands. Runs in parallel with phase 4a (`pane-1`); the scopes are disjoint by symbol.

**Task.** The `/` query becomes global and the All view arrives ([design §4](/wiki/design/tui-model.md#4-search), decisions 1, 2, 3 and 6).

**The query.**

- `App.query: Option<String>`. `open_search`, the `Overlay::Search` live-edit/commit/cancel arms and `SearchBox` edit it, not `filter.search`.
- `filter.search` stays as a saved view's own search (the drawer's search row and `V` keep writing it), ANDed with the query.
- `set_view` leaves `query` alone, and `is_view_modified` ignores it (a query does not `*` the tab).

**Matching.** One function, `query_matches(item, q)`, case-insensitive substring:

- a yak: id, title, body (notes included);
- a document: title, path, kind, tags, plus the page body and a source's extracted text (decision 6; read when the wiki entries are loaded; add a `text` field to `WikiEntry`);
- a shed: its name, or (tree rule) any matching child.

Apply it in every view's `build_rows` path: tree (ancestors of a match kept, as today's search), flat, working set, Wiki, Sheds. The memo key gains `query`.

**Counts.** `view_count` applies the query too, so `Shaving (3)` means three matches there.

**Chip.** On the tab row, its own chip, separate from the facets' `filter:` chip: `/"widget" · 14 in All`, where 14 is the All count. Keep the existing `filter:` chip for facets. Pick a distinct colour and record it as a `Decision:`.

**Esc** in a list: first clears the query, then (next Esc) reverts facets as today.

**All view.** `view::all_view()`: built-in, unpinned (reached from `v`), key `all`, flat, sorted by updated desc, no limit. Its rows are every yak (dead included), then every page and source. Its count is the chip's number.

**History seam.** None yet (phase 5 snapshots `query`).

**Out of scope.**

- The detail pane's `/` find and `n`/`N` (phase 6b).
- The viewer overlay and `Overlay::Attachment`/`Image` code (phase 4a is moving them).
- `detail_*` fields (phase 4b).
- `keymap.rs` beyond adding nothing: `/` and Esc keep their bindings.

Anything else: `yaks ask yaks-a25c`.

**Evidence.**

- Gate: `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace`, fresh `TMPDIR`.
- New tests:
  - `/widget` Enter then Tab keeps the query and filters the new view;
  - tab counts reflect it;
  - Esc clears the query, then reverts facets;
  - a saved view with its own search ANDs with the query;
  - the All view lists a dead yak and a page;
  - a query matching only a page body finds that page on Wiki and All;
  - the chip shows the All count.
- Snapshots that showed the old `filter: "q"` chip for a `/` search will change. Name them in a `Decision:` note; no others may change.
- Docs: `docs/tui.md` Search & filter, the Inbox/views list (All), the Wiki tab's `/` line. The `?` help line for `/` in `keymap.rs` ("Search every view; Esc clears").
- Attach a headless frame with an active query on the Shaving tab showing the chip.

**Judge.** herd-tui at landing.

---
▸ 2026-11-03T04:43:08Z [herd-tui]
kind: move
from: hairy
to: shaving
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T04:43:08Z [herd-tui]

herd-tui: base is main at `200d48fa` (phase 3b landed).
