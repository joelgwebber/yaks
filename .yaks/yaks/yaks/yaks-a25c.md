---
id: yaks-a25c
title: 'TUI phase 6a: global query with its own chip, and the All view'
status: shorn
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T04:58:44Z'
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

---
▸ 2026-11-03T04:57:42Z [query-1]

Start: 2026-11-03T04:43:52Z (date -u). Farm file `/home/user/.local/share/yaks/farms/yaks-b0f8dcf9/.yaks/yaks/yaks/yaks-a25c.md`; checkout `/home/user/yaks-7`; base `200d48f`.

Progress: the code, docs and tests are in; full gate green (`cargo test --workspace`, 890 passed in the bin, 30 `test result: ok` lines, none failed). The disk filled once mid-run (other workers' temp dirs); I only deleted stale test binaries and `incremental/` in my own `target/`.

- `App.query`; `/`, its live edit, Enter and Esc-cancel use it; `filter.search` stays the view's own search and ANDs.
- `tree::query_matches(item, q)`: yak id/title/body; doc title/path/kind/tags/`text`; shed name. Shed child rule in `shed_rows`/`busy_sheds_count`.
- `WikiEntry.text`: page file or `source.md` + `extracted.md`, lowercased once at load.
- `tree::build` takes `query: Option<&str>` (focus needs spec AND query); flat, working set, Wiki, Sheds and `view_count` apply it; memo key has `query`.
- `view::all_view()` (key `all`, unpinned, flat, updated desc, no limit), last in `default_views`.
- Query chip in `render_tabs`; Esc: query first, then facets.

---
▸ 2026-11-03T04:57:43Z [query-1]

Decision: the All view is appended LAST in `default_views` (after Sheds), not beside Inbox. Rejected: next to Inbox (shifts the Starred/Wiki/Sheds indices that ~dozens of tests and the saved-order reconcile rely on, and it is unpinned so tab order does not show). The `v` picker lists it at the bottom.
Decision: All's spec lists all four statuses (so the flat path keeps dead yaks). Rejected: a special case on the view key in `flat_rows`, which would hide a second source of truth for "dead is wanted".
Decision: in All, documents are listed only while the live facets equal the view's (search aside): a type/priority/status/ready facet describes yaks and documents have none. Rejected: always listing documents (a `p1` filter would still show pages), and hiding them whenever any search is set (the query must find pages there).
Decision: `V` saves the facets (`filter.search` from the drawer), not the query, as the claim says ("V keep writing it"). So `/foo` then `V` saves a view with no search. Rejected: copying the query into the saved spec (it would undo "a query does not belong to a view" and double-apply).
Decision: the query chip is Cyan, text `/"q" · N in All` with straight quotes as in the design, query truncated to 30 columns. Rejected: Yellow (the facets' `filter:` chip), Magenta (sources/blocked marks), DarkGray (reads as disabled). Without an All view in `views` the chip drops ` in All`.
Decision: yak matching in `query_matches` is per field (id, title, body), not one joined blob as `FilterSpec::matches` does, so a query cannot match across the title/body seam. A saved view's own search still uses the blob, and both must pass.
Decision: `WikiEntry.text` is lowercased once at load and holds a page's whole file (frontmatter + body, as `yaks find` reads it) and a source's `source.md` + `extracted.md`. Rejected: raw text lowercased per match (re-allocates per keystroke, per tab, per frame since every tab count applies the query).
Decision: tab counts: Starred counts the starred yaks the query keeps; Wiki counts the pages (sources not) it keeps; Sheds counts the busy sheds it keeps (name or a yak of theirs); All = matching yaks + matching documents. The counts still ignore the live facets, as before.
Decision: snapshots changed: `inline_search_field_and_recolor` (tab counts now apply the query: Shaving (0), Recent (2); the `Hairy*` marker is gone), `search_commit_then_tab_no_toast` (the query survives Tab, so the next view is filtered: `No tasks.`, all counts 0), and `view_picker_overlay` (the new All row, unavoidable once All exists). No others changed. Tests changed in step: `/` plus `V` tests now set the view's search through the drawer (`set_drawer_search`), view counts 8->9, `esc_reverts_modified_filter_to_view` uses a facet, the Wiki `/search` test now also finds the source whose extracted text says "search" and counts 2 pages.
Decision: All uses the globe-free basket emoji (U+1F9FA). Rejected: U+1F5C3 (a narrow-width emoji: the picker column misaligned in the frame).

[query-chip-on-shaving](yaks-a25c/query-chip-on-shaving.txt)

[query-on-all-view](yaks-a25c/query-on-all-view.txt)

---
▸ 2026-11-03T04:57:55Z [query-1]
kind: verify
result: fail
---

verify: `cargo test --workspace` -> FAIL (exit 127)

---
▸ 2026-11-03T04:58:37Z [query-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-03T04:58:44Z [query-1]

Done. `yaks verify yaks-a25c`: PASS. Gate `cargo test --workspace`: all green. Attached headless frames: `query-chip-on-shaving.txt` (Shaving tab, chip `/"widget" · 6 in All` beside the tabs, no `filter:` chip) and `query-on-all-view.txt` (All view: matching yaks, then the matching page). New tests (`tui::tests::query`, 7, plus a Sheds one) cover each bullet of the claim. Docs: `docs/tui.md` (Layout, query chip, Search & filter, All, Wiki, Sheds), `README.md`, `?` help lines in `keymap.rs`. docshots regenerated and discarded: none of the SVGs shows a query or the picker, and this machine's toque restyles every SVG.

---
▸ 2026-11-03T04:58:44Z [query-1]
kind: move
from: shaving
to: shorn
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAEAkGJIBxBCtY3f3KUhDW5IL-PWT3Nbm
---

moved: shaving -> shorn
