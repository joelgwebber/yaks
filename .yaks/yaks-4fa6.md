---
id: yaks-4fa6
title: E opens the editor at the exact source line
status: hairy
type: feature
priority: 3
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T04:25:58Z'
parent: yaks-ea13
depends_on:
- yaks-649d
- yaks-a893
labels:
- markdown
- tui
---

Uses the core's row source map: `E` on a body row opens the edit form on that content block with the editor cursor on the row's first source line (column best-effort). Also: after saving, the detail line cursor returns to the row for the edited line. Depends on the core; overlaps the view/edit unification design (yaks under yaks-faf5), so check that design first.
