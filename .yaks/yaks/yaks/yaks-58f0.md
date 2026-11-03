---
id: yaks-58f0
title: Collapse idle sheds by default
status: shorn
type: task
priority: 3
created: '2026-11-03T14:23:16Z'
updated: '2026-11-03T21:40:28Z'
parent: yaks-f677
tags:
- tui
- sheds
---

Old ones can get pretty noisy after you've been working for a while.

---
▸ 2026-11-03T21:24:33Z [delta:starred-batch]
kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

[sheds-after](yaks-58f0/sheds-after.txt)

---
▸ 2026-11-03T21:40:26Z [delta:starred-batch]

Done in d3aa34a: a quiet shed (idle, no claim, no open ask: `Shed::quiet`, the same rule that dims it) starts collapsed to its `▶ N`. `Space` opens it and that is remembered.

- The `collapsed` set stays the one store: a quiet shed's key there is `shed-open:<name>` and means *expanded* ([`collapsed_by_default`](code:src/tui/tree.rs)); a busy one keeps `shed:<name>` meaning collapsed, so a saved choice keeps its meaning when the shed changes state.
- `h` flips the key the same way, so it closes an opened idle shed.

Evidence: test `idle_sheds_start_collapsed_and_space_opens_them`, updated list snapshot, and a headless frame of this farm's Sheds tab (attached): the eleven idle sheds are one line each.

---
▸ 2026-11-03T21:40:28Z [delta:starred-batch]
kind: move
from: shaving
to: shorn
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
