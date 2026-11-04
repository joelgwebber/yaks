---
id: yaks-f1e6
title: 'Sheds view: More like list/tree views'
status: shorn
type: idea
priority: 3
created: '2026-10-26T16:15:27Z'
updated: '2026-11-03T04:46:15Z'
parent: yaks-039b
tags:
- tui
---

The current sheds view implementation wastes a lot of space -- there are never more than a few sheds at a time, so we can use that space more effectively. My idea:
- Make it more like the existing tree views, with collapsible sheds at the top.
- Open sheds show their PRs, and perhaps the metadata in the shed detail (state, path, git, et al).
- Shed children become the yaks it's touching. Selecting them gives you the yak details.
- Navigation within yak-details uses the same affordances as regular list/tree views, so you don't lose the shed context.
- Have sheds assume the same place in the global navigation stack that other lists/details do (modulo the generalization to be done in yaks-158d).

---
▸ 2026-10-31T15:56:25Z [delta:Cloud Runners]

Adopted into `yaks-039b` under `yaks-da23` (2026-10-31): the note-based sheds rebuild is the natural place for this view work.

---
▸ 2026-10-31T19:28:04Z [yakherd]

Graduated from `yaks-da23` (shorn) to `yaks-039b`: a shorn parent must not hold unfinished children (Joel, 2026-10-31).

---
▸ 2026-11-03T02:19:22Z [tui-design]

Folded into the unified TUI design, [wiki/design/tui-model.md](/doks/design/tui-model.md), tracked by yaks-9c3c: §1 "Rows and views" and phase 3.

Sheds becomes an ordinary tree view:

- Each actor is a collapsible root; its children are the yaks it touched, outstanding first and landed folded. There are no PR rows (Joel's decision 4).
- Its own key handler and cursors go, so `U`, `/`, `J`/`K`, `o`/`i` and collapse work there like in any view.
- A selected yak shows in the pane with the shed tree still beside it.

---
▸ 2026-11-03T04:46:15Z [herd-tui]

Done by herd-tui through the unified model's phase 3 (yaks-5d08): Sheds is now an ordinary tree view, landed as `200d48fa`.

- Each actor is a collapsible root; its children are the yaks it touched (working, asking, recent).
- Selecting a yak shows the normal yak detail with the shed tree beside it; `J`/`K`, `U`, `W` and every yak key work there.
- Its place in the global history comes with phase 5 (yaks-a2c5).

---
▸ 2026-11-03T04:46:15Z [herd-tui]

kind: move
from: hairy
to: shorn
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shorn
