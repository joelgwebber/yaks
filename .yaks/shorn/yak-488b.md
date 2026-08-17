---
id: yak-488b
title: 'Star marker: place at far right of labels, reserve 2 cells so it doesn''t
  jump'
type: task
priority: 3
created: '2026-08-17T16:12:16Z'
updated: '2026-08-17T16:13:39Z'
labels:
- ui
---

---
▸ 2026-08-17T16:13:39Z
Done. Reworked the list row's right side to [labels] [2-cell star slot] [badge]. The star slot is always reserved (star_slot_w=2) so labels never shift when a yak is (un)starred; the star sits at the far right of the labels (was: to their left) and is blank when absent. Positions are computed from the right edge (star_x, label_x) rather than from a variable combined string, so nothing jumps. draw_list only; curses so wants an eyeball but logic is simple.
