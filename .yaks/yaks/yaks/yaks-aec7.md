---
id: yaks-aec7
title: Navigation back skips current list view
status: shorn
type: bug
priority: 3
created: '2026-11-03T21:11:08Z'
updated: '2026-11-03T21:47:58Z'
parent: yaks-f677
tags:
- tui
---

The navigation stack *does* support forward/back affordances across list views; but when backing out of a *detail* it skips the list view in which the detail was found. This is slightly jarring as it often navigates to a *different* list view.

---
▸ 2026-11-03T21:42:11Z [delta:starred-batch]

kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

[aec7-frames](yaks-aec7/aec7-frames.txt)

---
▸ 2026-11-03T21:47:57Z [delta:starred-batch]

Fixed: opening a yak or shed row in the pane (`l`/`Enter`, or a click) is now a history jump, as opening a page from a Wiki row already was. Before, it pushed nothing, so the first stop behind a detail was the view *before* the list.

- [fn open_detail_pane](code:src/tui/handlers.rs) runs through `App::jump`.
- `History::push` skips a stop equal to the last one, so `l`/`h` on one row many times leaves a single stop.
- `docs/tui.md` (history section) and the `history.rs` module docs say so.

Evidence: new test `backing_out_of_a_detail_returns_to_the_list_it_was_opened_from`; three existing tests counted the stops and were updated. Headless on this farm (attached): Recent, Enter, Tab+Enter onto a linked yak, then `o` `o` lands on Recent's list on the same row, and a third `o` says "no earlier location".

---
▸ 2026-11-03T21:47:58Z [delta:starred-batch]

kind: move
from: shaving
to: shorn
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
