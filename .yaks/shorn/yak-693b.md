---
id: yak-693b
title: Show a star marker on starred yaks in list rows
type: feature
priority: 3
created: '2026-08-17T14:17:49Z'
updated: '2026-08-17T14:34:05Z'
parent: yak-1bef
labels:
- ui
---

Visual feedback for the working set (yak-597c): a yak that's in app.working_set should render a star (⭐ or a subtler glyph) in its list row, the way labels render, so you can see membership at a glance in ANY view — not just via the notification or the ⭐ tab.

Where: the list row renderer in yaktui/render.py (the draw-list function that renders each (status, task, depth, ghost) row; near where labels/show_labels are drawn). Membership test = task id in app.working_set (a set() copy for O(1)). Consider dimming for ghost rows. Keep it cheap (per-row set lookup).

---
▸ 2026-08-17T14:34:05Z
Done. draw_list shows a star (⭐, \u2b50) on rows whose id is in the working set, as a right-side tag just left of any labels (starred_ids = set(app.working_set) computed once per draw for O(1) row lookup). Consistent with how labels render; gives at-a-glance working-set membership in any view. Note: emoji cell-width vs len() is a pre-existing quirk shared with the other badge emojis.
