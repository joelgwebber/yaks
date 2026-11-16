---
id: yaks-cc2a
title: Shed details renders read-only, unscrollable yaks list
status: shorn
type: bug
priority: 3
created: '2026-11-03T20:53:54Z'
updated: '2026-11-03T21:40:29Z'
parent: yaks-f677
tags:
- tui
- sheds
---

Not quite sure what we *should* show for these -- we don't have a lot of information available for them besides the yaks list. But it feels kind of broken to see it duplicate the yaks list that's *also* available in the list/tree view, but have it be wholly non-interactive. I think I'd err on the side of keeping it simple, and leaving the yak list to the list view.

---
▸ 2026-11-03T21:24:34Z [delta:starred-batch]

kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T21:40:28Z [delta:starred-batch]

Done in d3aa34a, keeping it simple as suggested: the shed's pane keeps its header and checkpoint line and replaces the yak list with a count by part (`Yaks: 1 shaving · 1 recent`) and a pointer to the rows under the shed. The yaks live in the list only, where every key works.

Evidence: `sheds_tab_detail` snapshot, docshot `tui-sheds-detail.svg`, and on this farm: `Yaks: 4 shaving · 5 recent` for this shed. Help text (`keymap.rs`) and `docs/tui.md` updated.

---
▸ 2026-11-03T21:40:29Z [delta:starred-batch]

kind: move
from: shaving
to: shorn
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
