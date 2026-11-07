---
id: yaks-a036
title: Consistent copy behavior across yaks/docs
status: shaving
type: bug
priority: 3
created: '2026-11-04T00:06:56Z'
updated: '2026-11-07T17:04:43Z'
tags:
- tui
---

Copy in a yak does the right thing (copying the yak id) by default, if no line is selected with [v]isual row selection. But docs always copy the current line, never the actual doc path, which would be both more consistent, and sometimes what we actually want.

---
▸ 2026-11-07T17:04:43Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
