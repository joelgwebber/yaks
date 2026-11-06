---
id: yaks-525b
title: Attribute an agent's hand edits committed with yaks commit
status: shaving
type: feature
priority: 3
created: '2026-11-06T04:57:49Z'
updated: '2026-11-06T23:12:42Z'
parent: yaks-1c89
tags:
- cli
---

`yaks commit` writes no `Yaks-Actor` trailer for hand edits (they are the person's by design), so a dok an agent edited in its editor and committed is missing from `yaks session touched` and `undo --by`. The yaks-upkeep skill tells agents to list such edits in `## Saved` themselves. Option: `yaks commit --as <actor>` (or the checkout's whoami name when it is an agent's) stamps them. Found while building yaks-17f5.

---
▸ 2026-11-06T23:12:42Z [herd-upkeep-2]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
