---
id: yaks-f1e6
title: 'Sheds view: More like list/tree views'
status: hairy
type: idea
priority: 3
created: '2026-10-26T16:15:27Z'
updated: '2026-10-31T19:28:04Z'
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
