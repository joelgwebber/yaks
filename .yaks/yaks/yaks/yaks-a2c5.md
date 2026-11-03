---
id: yaks-a2c5
title: 'TUI phase 5: one history of locations'
status: shaving
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T05:46:42Z'
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
