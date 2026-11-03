---
id: yaks-a2c5
title: 'TUI phase 5: one history of locations'
status: shorn
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T05:48:16Z'
parent: yaks-947a
depends_on:
- yaks-aac1
- yaks-a25c
tags:
- tui
---

Phase 5 of [the design](/wiki/design/tui-model.md#3-one-history) (yaks-158d): one stack of locations replaces the three back stacks; jumps push, movement doesn't; `o`/`i` work everywhere; restore clamps.

---
▸ 2026-11-03T05:09:17Z [herd-tui]

Claim draft (yakherd herd-tui), to start after phase 6b (yaks-c215) lands: worker `hist-1`.

**Task.** One history of locations replaces the three back stacks ([design §3](/wiki/design/tui-model.md#3-one-history); decision 1 puts the query in a location).

- `struct Location { view_key, filter, query, cursor: Option<RowKey>, cursor_idx, list_offset, focus, pane: Option<PaneAt> }`.
  - `RowKey` names a row by kind: a yak id, a doc path, `shed:<name>`.
  - `PaneAt` is the open subject and its position: a yak detail's `LineNav` line and scroll, or an open doc (path, rev, its `LineNav` line and scroll).
- `App.history: History { back: Vec<Location>, fwd: Vec<Location> }`, capped at 100.
- `nav_back`/`nav_fwd`, `viewer_back`/`viewer_fwd`, `LineNav.back`/`fwd` and `NavEntry` go.
- **Jumps push** (snapshot the location being left, clear forward):
  - following any link: yak, doc, footnote, `#section`;
  - a view switch (`Tab`/`[`/`]`, view picker, tab click);
  - `#`.
  Skip a push identical to the current location.
- **Not jumps** (update in place, nothing pushed): cursor and line movement, `J`/`K`, opening or closing the pane, committing a search or the drawer.
- **`o`/`i` everywhere**: bind them in `Ctx::List` (free there today) as well as the pane contexts. Back snapshots the current location onto forward, then restores.
- **Restore**, in order: view by key (gone: the subject's status view, else the first view, with a notification), filter, query, cursor by key (gone or moved: clamp to `cursor_idx` with a notification, `<id> moved` / `<id> gone`), list offset, focus, pane (a doc re-read from disk; line and scroll clamped).
- **Following a link to a yak** no longer switches the list to its status view (`select_task`'s view switch goes; decision 5). If the current list has the yak, the cursor moves to it; otherwise the pane shows it and the list stays. This needs the pane to show a yak that is not the selected row: `PaneSubject::Yak(id)` beside the doc. Check how 4a modelled the pane before choosing; ask if it needs a structural call.
- Each jump's notification names the view and subject (`← Recent · yaks-1234`).

---
▸ 2026-11-03T05:27:15Z [herd-tui]
kind: move
from: hairy
to: shaving
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T05:27:16Z [herd-tui]

Claim (yakherd herd-tui): worker `hist-1` (Sonnet 5.5), base main `acabcb7f` (phases 1–4 and 6 landed). The draft above is the spec, with these resolutions:

**The pane's yak.** Today the yak detail reads `rows()[cursor]` (`detail_dlines`, `selected()`), so it cannot show a yak outside the list.

- Add `App.pane_yak: Option<String>`, the id the pane shows when it differs from the selected row: `None` means "the selected row's yak", as today.
- `selected()` (the yak commands' target) returns the pane's yak in pane focus when `pane_yak` is set, else the row's.
- `detail_dlines` and the memo key read the same subject: `memo::DlinesKey.loc` already names a yak by `Loc::Home(index)`, so resolve `pane_yak` to its index.
- `pane_yak` clears when the cursor moves (`j`/`k`, `J`/`K`, a click), on a view switch, and on `Esc`/`h` to the list.
- The pane header shows a dim `not in this list` when `pane_yak` is set.

**Link-follow** (`goto_task`, used by detail links, viewer yak links, `#`, `B`, a shed child):

- if the current list has the yak, move the cursor to it (expanding ancestors), `pane_yak = None`;
- otherwise set `pane_yak` and leave the list alone.

`select_task`'s status-view switch goes. `select_id` (after create) keeps jumping to Hairy: that is a create, not a link.

**History.** The draft's `Location` and `History`. `PaneAt` is one of:

- `Yak { id, line, scroll }`;
- `Doc { path, rev, from, line, scroll }`;
- `Shed(name)`.

It replaces `nav_back`/`nav_fwd`, `NavEntry`, `viewer_back`/`viewer_fwd` (`follow.rs`) and `LineNav.back`/`fwd` (in-page footnote and `#section` jumps push a Location whose pane differs only in line).

**Keys.** Bind `o`/`i` in `Ctx::List` too (keymap: `NavBack`/`NavForward` already exist). `Esc` closing a doc no longer clears history.

**Out of scope.**

- Key changes beyond `o`/`i` in the list (phase 7).
- Persisting history.
- The search register (6b, landed).

Anything else: `yaks ask yaks-a2c5`.

**Evidence.**

- Gate: `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace`, fresh `TMPDIR`.
- New tests:
  - in Recent with a query, follow a yak link to a yak in another status, then `o`: back in Recent with the query, cursor and detail line restored;
  - following a link to a dead yak shows it in the pane with `not in this list`;
  - `o` from a page opened from a Wiki row returns to the list row and then the previous view;
  - footnote jump then `o` returns to the reference;
  - a view switch then `o` returns to the previous view;
  - a restored yak that was slaughtered meanwhile says `gone`.
- Existing nav tests: keep their assertions where behaviour is the same, rewrite where the old status-view switch was asserted (one `Decision:` note listing them).
- Docs: `docs/tui.md` (`o`/`i` everywhere, link-follow keeps the list), `?` lines.
- Headless frame: the Recent-with-query round trip.

**Judge.** herd-tui at landing.

---
▸ 2026-11-03T05:38:23Z [hist-1]

Start 2026-11-03T05:38:23Z. Setup: farm file /home/user/.local/share/yaks/farms/yaks-b0f8dcf9/.yaks/yaks/yaks/yaks-a2c5.md; checkout /home/user/yaks-10 (top-level); base `acabcb7f` (git log --oneline -1: "yaks-c215: one search register..."). Worker hist-1.

Progress: core is implemented and `cargo build` passes: new `src/tui/history.rs` (Location/RowKey/PaneAt/History, `App::jump`, `history_back/forward`, `restore`), `App.pane_yak` with `selected()`/`detail_dlines` following it, `goto_task` rewritten, the viewer/nav stacks and `LineNav.back/fwd` removed, `o`/`i` bound in the List, Image and Shed pane. Next: fix the existing tests, write the new ones, docs, headless frame.

[history-roundtrip-frames](yaks-a2c5/history-roundtrip-frames.txt)

---
▸ 2026-11-03T05:46:42Z [hist-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-03T05:47:06Z [hist-1]

Decision: a jump is wrapped, not pushed by hand. `App::jump(|app| ..)` snapshots the location before, runs the move, and pushes only if the location differs afterwards. That gives "skip a push identical to the current location" for free (a link to the yak already shown, a failed link, a missing page push nothing) and means no call site can forget the snapshot or push twice. Rejected: an explicit `push_location()` at each call site (the old `nav_back.push` pattern: three stacks, each with its own forgotten cases).

---
▸ 2026-11-03T05:47:06Z [hist-1]

Decision: opening a page from a list row (Wiki `Enter`/`l`, a new page) is a jump, though opening a yak's pane is not. The claim's test requires `o` from such a page to return to the list row, which only works if the open pushed the list location. `o` also skips stops identical to where you are, so `Enter`, `Esc`, `o` does not waste a step on the page just closed. Rejected: treating a page like a yak pane (no push), which makes `o` from a page skip the row and go to the previous view.

---
▸ 2026-11-03T05:47:06Z [hist-1]

Decision: `moved` vs `gone` on restore. The cursor row is found by key (tried at its old index first, so a yak listed under two sheds keeps its place); found anywhere = silent, even if its index changed (Recent reorders constantly, a note per restore would be noise). Not found: the cursor clamps to the old index and the notification says `<id> moved` if the yak/page still exists and is listed somewhere (dead yaks count as gone), else `<id> gone`. Rejected: saying `moved` whenever the index differs; clamping to the index even when the row is found.

---
▸ 2026-11-03T05:47:06Z [hist-1]

Decision: `pane_yak` only means something with the pane focused (`pane_yak_id()` filters on `focus == Detail`), and every place that returns focus to the list or moves the cursor clears it (`back_to_list`, `move_cursor`, `show_row`, `open_detail_pane`, `set_view`). Row-scoped lookups (`row_yak`, used by `reload_preserving_selection` and the sheds snapshot) ignore it so a reload does not drag the cursor to the pane's yak; `row_kind`/`selected_shed` answer for a yak while it shows. Rejected: clearing it centrally after each key (fails for a restore that sets both a new cursor and a pane yak).

---
▸ 2026-11-03T05:47:06Z [hist-1]

Decision: `PaneAt::Doc` gains `under: Option<String>` beyond the claim's `{path, rev, from, line, scroll}`: the pane's own yak while the page covers it. Without it, restoring a page opened over a pane yak would make `Esc` land on the cursor row's yak instead of the yak the page was opened from. A page is re-opened by `read_doc(path, rev)`: a farm file when `rev` is set or the path exists under the farm root, else an attachment (`<id>/<file>` never exists under the root). Rejected: a `kind` field on `PaneAt::Doc`.

---
▸ 2026-11-03T05:47:06Z [hist-1]

Decision: the image viewer's `Esc`/`q`/`h` now closes the pane like the text viewer does; the old "back to the text viewer it was followed from" lived in `viewer_back`, which is gone, and `o` now does that. `o`/`i` are bound in the image viewer and the shed pane too ("everywhere"); the Detail `i/o` binding moved from `bind_pane(Yak)` to `bind(Detail)` so the shed pane takes it, which adds `i/o:fwd/back` to the Shed detail hint line (snapshot `sheds_tab_detail` and `docs/assets/tui-sheds-detail.svg` updated by hand; the other docshot SVGs regenerate with unrelated renderer drift, so none were committed). In the list `o`/`i` carry no hint (the list hint bars are unchanged).

---
▸ 2026-11-03T05:47:06Z [hist-1]

Decision: messages. Back/forward say `← Recent · yaks-1234` / `→ ...` with the tab name minus its icon; empty ends say `no earlier location` / `no later location` (replacing `no earlier yak`/`no earlier spot` etc.); a view that is gone adds `(view <key> is gone)`; when the view is gone the filter is not applied (the saved one belonged to the deleted view) but the query is.

---
▸ 2026-11-03T05:47:07Z [hist-1]

Decision: existing tests changed (the rest keep their assertions, including the footnote `o`, viewer `o`/`i` and nav-position-restore tests):
- `follow_link_to_yak_outside_any_view_says_so` -> `follow_link_to_a_dead_yak_shows_it_in_the_pane_and_leaves_the_list` (old assertion: "d0 isn't shown in any view" and no history; now the dead yak shows with `not in this list`).
- `nav_history_back_and_forward`, `nav_back_on_empty_history_is_noop`: message text only.
- `switching_view_or_moving_to_another_row_closes_the_page`: asserted `viewer_back/fwd` empty; now asserts J/K push nothing.
- follow.rs `enter_opens_a_page_at_its_heading_and_o_comes_back`: `o` past the first page now returns to the list it was opened from (was "no earlier spot"), and `Esc` no longer forgets history.
- keymap `each_contexts_hint_is_what_it_was` Shed detail hint, and three snapshots: `sheds_tab_detail` (hint), `goto_jumped_to_detail` (the `#` jump to a shaving yak from Hairy now keeps the list and says `not in this list`), `help_overlay_shows_the_farm_keys` (i/o help text, scroll).

---
▸ 2026-11-03T05:47:07Z [hist-1]

Evidence ($(date -u +%FT%TZ)): gate `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace` (fresh TMPDIR): exit 0, bin tests `920 passed; 0 failed; 12 ignored`, every integration binary ok. `env -u YAKS_DIR -u YAKS_ACTOR yaks verify yaks-a2c5`: PASS (exit 0). New tests in `src/tui/history.rs` (14 total in that module with the existing ones counted by filter `history`): Recent+query round trip (query, cursor, detail line and scroll restored; `i` forward; `no later location`), dead yak in the pane `not in this list` (tests.rs), `o`/`i` from the list, view-switch jumps and `no earlier location`, page from a Wiki row `o` -> row -> previous view, page re-read from disk with clamped cursor and a deleted page (`<path> gone`), a yak link from a page on the Wiki tab (pane yak, header, `o` back to the page), `a0 gone` (slaughtered) and `a0 moved` (state changed), deleted view fallback, cap of 100, no push for a link to the yak already shown. Footnote jump + `o` is covered by the existing `following_a_footnote_reference_jumps_to_its_definition_and_o_returns` and the viewer footnote test (both pass on the one history). Headless frame attached: `history-roundtrip-frames.txt` (Recent with query `cache`, follow a link to a yak the query hides: `not in this list`, `pane_yak=` in the state header; `o` returns with query, cursor and line: `← Recent · demo-98ac`).

---
▸ 2026-11-03T05:47:16Z [hist-1]

Summary 2026-11-03T05:47:16Z: one history of locations landed in the working tree (src/tui/history.rs + pane_yak + keys + docs/tui.md History section + AGENTS.md layout line). Gate green (920 passed), verify PASS. The date in the Evidence note above is literal text, the gate ran just before this note.

---
▸ 2026-11-03T05:47:16Z [hist-1]
kind: move
from: shaving
to: shorn
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAECzPpIBxBCtY3f3KUhDW5IL-PWT3Nbm
---

moved: shaving -> shorn

---
▸ 2026-11-03T05:48:02Z [herd-tui]
kind: landed
commit: 270ec67797123a5bfcc7503cb12677fc7a3037e4
source: 5bff181953b418466aebf7ad172d993d36fd202e
branch: main
---

landed: 270ec677 yaks-a2c5: one history of locations (o/i everywhere; a link keeps the list, the pane shows yaks outside it)

---
▸ 2026-11-03T05:48:16Z [herd-tui]

Landed by herd-tui as `270ec677` (worker `5bff1819`).
