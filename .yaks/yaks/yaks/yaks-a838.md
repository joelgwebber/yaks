---
id: yaks-a838
title: 'Per-change commits: scoped commit per write, trailers, yaks undo, config commit: change|batch'
status: shaving
type: feature
priority: 1
created: '2026-11-01T18:53:11Z'
updated: '2026-11-01T20:51:43Z'
parent: yaks-f301
labels:
- wiki
- git
---

Implements yaks-63c3 (commit half). Every mutating command commits exactly its own paths under the farm lock through a temporary index; `Yaks-Actor` trailer; hand edits committed first and separately; `yaks undo`.

---
▸ 2026-11-01T20:51:43Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
