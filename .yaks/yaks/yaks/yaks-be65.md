---
id: yaks-be65
title: 'TUI phase 3a: row kinds (yak, document, shed); Wiki becomes a flat view of document rows'
status: shaving
type: task
priority: 2
created: '2026-11-03T04:10:57Z'
updated: '2026-11-03T04:22:45Z'
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

---
▸ 2026-11-03T04:21:56Z [rows-1]

Decision: yak commands are bound with `bind_row(Yak, ..)`, not left kind-less; a document row finds no binding for them and does nothing. Rejected: leaving them kind-less and making each handler check `selected()` (the bottom hint then shows `c/C:new`, `X:del`, `f:filter` on a document row and breaks the Wiki hint literal).

Decision: with no row selected, the kind is the view's (`row_kind()`: Doc on the Wiki view, else Yak), so an empty or fully-filtered Wiki still takes `N`, `r`, `v`. Rejected: kind `None` on an empty list (then `c` on an empty yak list would stop working).

Decision: `lookup` order is (modifier-named, kind) -> (modifier-named, any) -> (plain, kind) -> (plain, any), not the claim's literal (kind, any) with modifiers first inside each. Reason: a kind-bound plain `c` must not shadow any-kind Ctrl-C (quit). Test `lookup_prefers_a_binding_for_the_rows_kind`.

Decision: the document-row hint is the Wiki literal, got by letting a kind binding shadow any-kind bindings that share a key in `hint_for`; `v`, `?` and `q` are repeated as Doc bindings (empty help, so not listed in `?`) only so the hint matches. Rejected: a per-binding "hide for kind" flag (more concepts for three keys).

Decision: the Wiki's `d`/`PgDn`/`Home`/`End` are now the list's: `PgDn`/`PgUp` are full-page (were half-page on Wiki) and `g`/`G` gained `Home`/`End` in the yak list (so the Wiki keeps them). Visible change: `snapshots/yaks__tui__tests__help_overlay.snap` now shows `g / Home` and `G / End` (the only snapshot changed). Rejected: unlisted Home/End bindings (hidden keys), or Doc-only copies of d/u/PgDn/PgUp/g/G.

Decision: `revert_wiki_view` is gone: `Esc` is `revert_list_view` for every row, since `set_view` already clamps the cursor (the only thing the Wiki copy added).

Decision: the list highlight on a Doc row stays bg-only (spans keep their colours) and the list is drawn unfocused whenever any overlay is up, exactly as the old Wiki list; yak rows keep fg-reset and `focus == List`. Reason: otherwise four Wiki SVGs change. Slightly kind-specific; phase 4 (viewer overlay) should end it.

Decision: `mouse.rs`: the second click on a document row opens it (like `Enter`) instead of focusing a detail pane that has nothing to show. The Wiki list now records hit rects, so clicks reach it for the first time.

Decision: `set_wiki_entries` calls `touch()` (memo key), and re-seats `App.cursor` on the same path only when the Wiki view is active.

[wiki-view-cursor-moved](yaks-be65/wiki-view-cursor-moved.txt)
