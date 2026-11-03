---
id: yaks-be65
title: 'TUI phase 3a: row kinds (yak, document, shed); Wiki becomes a flat view of document rows'
status: shaving
type: task
priority: 2
created: '2026-11-03T04:10:57Z'
updated: '2026-11-03T04:21:56Z'
parent: yaks-5d08
tags:
- tui
---

Phase 3, first half, of [the design](/wiki/design/tui-model.md#1-rows-and-views) (yaks-f1e6 is the second half's yak): a list row is a yak or a document, and the Wiki tab becomes an ordinary flat view of document rows, using the list's own cursor, rendering and key dispatch.

---
▸ 2026-11-03T04:11:26Z [herd-tui]

Claim (yakherd herd-tui): worker `rows-1` (Sonnet 5.5), from main at `642be42e` (phases 1 and 2 landed).

**Task.** Rows get a kind, and the Wiki tab stops being a special surface.

1. **Row kinds.** `tree::Row<'a>` replaces `task: &'a Task` with `item: Item<'a>`, an enum with `Yak(&'a Task)` and `Doc(&'a WikiEntry)` (phase 3b adds `Shed`). `memo::Loc` gains `Doc(usize)`, an index into `App.wiki.entries`, and the memo keys whatever the wiki rows read (bump `data_gen` via `touch` when the entries are replaced). `App::selected()` keeps meaning *the selected yak* (`None` on any other row), so its many callers stay unchanged; add `selected_row()` and `selected_doc()`.
2. **Wiki as a view.** `build_rows` returns the Wiki view's rows: pages then sources, filtered by `filter.search` exactly as `wiki_rows` does today. `render_list` draws a document row with today's formatting (title, kind, tags, dim path; reuse `entry_row`). Delete `WikiTab.cursor` and `WikiTab.list_offset`, `move_wiki`, `clamp_wiki_cursor`, `render_wiki_list` and `selected_wiki` (now `selected_doc`). `App.cursor` and `list_offset` serve every view. Keep `wiki_count`, `reload_wiki` and the entries.
3. **Keys.** `Ctx::Wiki` goes; the Wiki view is `Ctx::List`. A `Binding` may name a row kind; `lookup` tries bindings for (ctx, the selected row's kind) first, then (ctx, any kind).
   - Document-row bindings, the same keys the Wiki tab has today: `l`/`→`/`Enter` open the viewer; `E` edits the page; `N` makes a new page; `y` copies the path; `r` rereads the wiki.
   - Yak commands on a document row find no yak and do nothing, as on an empty list.
   - The bottom hint with a document row selected must equal today's Wiki hint literal (the existing keymap fixture). In `?`, the "Wiki tab" section becomes the document-row section.

**Out of scope.** Sheds (phase 3b, next), the viewer overlay (phase 4), search semantics (phase 6a), `mouse.rs` beyond what compiles. Anything else: `yaks ask yaks-be65`.

**Evidence.**

- `cargo test --workspace` passes. Run it with neither `YAKS_DIR` nor `YAKS_ACTOR` exported and a fresh `TMPDIR`: `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace`.
- Every Wiki snapshot (insta and `docs/assets` docshots) is unchanged. `git diff --stat -- src/tui/snapshots/` is empty, or each change is named in a `Decision:` note.
- The `wiki_tab` tests still pass. Adjust only those that name removed internals (`wiki.cursor`, `move_wiki`), keeping what they assert.
- New tests:
  - on the Wiki view, `j` moves `App.cursor` and `selected_doc()` follows;
  - `selected()` is `None` on a document row, and `S` opens no picker there;
  - `Enter` on a document row opens the viewer.
- Docs and help parity: `docs/tui.md` (the Wiki tab section, if any wording changes) and the generated `?` and hint.

**Judge.** herd-tui at landing.

---
▸ 2026-11-03T04:12:04Z [rows-1]

Start Tue Nov  3 04:12:04 UTC 2026. Checkout /home/user/yaks-4, HEAD 642be42 yaks-fbb0: TUI command table (keymap.rs) for the non.
Farm path: /home/user/.local/share/yaks/farms/yaks-b0f8dcf9/.yaks/yaks/yaks/yaks-be65.md

---
▸ 2026-11-03T04:21:56Z [rows-1]

Progress: implemented, gate not yet run.

- `tree::Item { Yak, Doc }`, `tree::RowKind`, `Row.item`, `Row::yak()`; `memo::Loc::Doc`; `selected_row()` / `selected_doc()` / `row_kind()`; `build_rows` returns document rows for the Wiki view; `render_list` draws them via `wiki_tab::DocCols`.
- Removed: `WikiTab.cursor`/`list_offset`, `move_wiki`, `clamp_wiki_cursor`, `render_wiki_list`, `selected_wiki`, `revert_wiki_view`, `Ctx::Wiki`.
- Keymap: `Binding.kind`, `bind_row`, `lookup(ctx, kind, key)`, `hint_for(ctx, kind, ..)`, `SECTIONS` for `?`.
- Headless: Wiki view with `j` moves `cursor` and `entry=`; `S` does nothing there.
- Docshots: regenerated on this tree and on the baseline tree; the four Wiki SVGs are byte-identical.
