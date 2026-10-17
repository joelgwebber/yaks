---
id: yaks-4fa6
title: E opens the editor at the exact source line
status: hairy
type: feature
priority: 3
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T17:02:15Z'
parent: yaks-ea13
depends_on:
- yaks-649d
- yaks-597d
labels:
- markdown
- tui
---

Uses the core's row source map: `E` on a body row opens the edit form on that content block with the editor cursor on the row's first source line (column best-effort). Also: after saving, the detail line cursor returns to the row for the edited line. Depends on the core; overlaps the view/edit unification design (yaks under yaks-faf5), so check that design first.

---
▸ 2026-10-17T17:02:15Z [delta-lead]

Delivered by yaks-597d (the in-place editor opens at the source line behind the cursor row); shear together.
