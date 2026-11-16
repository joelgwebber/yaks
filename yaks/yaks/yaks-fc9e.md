---
id: yaks-fc9e
title: Improved detail cursor/link handling
status: shorn
type: feature
priority: 2
created: '2026-11-03T03:23:32Z'
updated: '2026-11-03T21:55:06Z'
parent: yaks-f677
tags:
- tui
---

The cursor model for yaks details and documents is currently only line at a time. Which is good for simplicity, but it also creates some issues when there are multiple links on a single line, which is quite common.

While I don't think we need a full row:col:range selection model for viewing mode -- that would just be unnecessarily confusing -- I do think we need at least a concept of selected row *and* targeted link within that row. We could continue to use up/dn/j/k to change the row selection, we would allow [/] to move among the available links, including those on the same row.

---
▸ 2026-11-03T12:17:37Z [Joel Webber]

Also note: The cursor doesn't update when you search, then move to the [n]ext result. The view jumps, but the cursor stays where it was. We should try to make all scenarios like this consistent.

---
▸ 2026-11-03T21:48:07Z [delta:starred-batch]

kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

[fc9e-tabs](yaks-fc9e/fc9e-tabs.txt)

---
▸ 2026-11-03T21:55:06Z [delta:starred-batch]

Done in the shape you described: rows stay the cursor model, plus a *targeted link* within the row.

- `LineNav` gains `link: Option<(row, col)>`. Tab/`]` and Shift-Tab/`[` step through **every** link in document order, several on one row included; the cursor goes to the link's row ([fn jump_link](code:src/tui/linenav.rs)).
- [fn link_at](code:src/tui/linenav.rs) returns the targeted link (else the row's first). It is the one rule now: Enter (detail and viewer), `O`, `r`, the peek pane, the status-bar destination and the highlight all go through it (`App::detail_link`). Four copies of "first link on the cursor row" are gone.
- Any other cursor move (`j`/`k`, `g`/`G`, a jump, a find) drops the target, so returning to a row starts at its first link.
- **Search:** `n`/`N` and Enter already moved the cursor after upstream's yaks-c215; typing in `/` now does too (the view jumped, the cursor stayed). Esc still restores both.
- `?` help and `docs/tui.md` updated.

Evidence: tests `tab_visits_every_link_on_a_row_and_link_at_follows_it` and `the_detail_cursor_follows_find_matches`; on this farm (attached), in yaks-f677's detail, Tab x14-17 follow `yaks-0e15`, `yaks-d0b0`, `yaks-c5d7`, `yaks-91d9`, the four links of one description row, in order.

Not done: mouse click to target a specific link on a row (that is yaks-000b's territory).

---
▸ 2026-11-03T21:55:06Z [delta:starred-batch]

kind: move
from: shaving
to: shorn
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
