---
id: yaks-a2c5
title: 'TUI phase 5: one history of locations'
status: shaving
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T05:27:15Z'
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
