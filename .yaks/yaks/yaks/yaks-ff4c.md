---
id: yaks-ff4c
title: Remove import-hearth
status: shorn
type: task
priority: 3
created: '2026-11-04T22:13:03Z'
updated: '2026-11-07T00:04:36Z'
parent: yaks-44b3
tags:
- doks
---

It was really only needed once. This should obviate yaks-9f5e.

---
▸ 2026-11-06T04:28:55Z [herd-upkeep]

Joel, 2026-11-05: go ahead. Hearth never built a real data repo beyond tests, so nothing else needs the import.

---
▸ 2026-11-06T23:09:24Z [herd-upkeep-2]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T23:12:41Z [herd-upkeep-2]

Done in 671320f9: `src/hearth_import.rs`, its test, the clap command and every doc and skill mention removed. Full `cargo test`: 1172 passed, 0 failed. Lands with the rest of this batch.

---
▸ 2026-11-07T00:04:36Z [herd-upkeep-2]
kind: move
from: shaving
to: shorn
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
