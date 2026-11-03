---
id: yaks-fc9e
title: Improved detail cursor/link handling
status: shaving
type: feature
priority: 2
created: '2026-11-03T03:23:32Z'
updated: '2026-11-03T21:55:05Z'
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
