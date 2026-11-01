---
id: yaks-04b8
title: Release v0.0.28
status: shorn
type: task
priority: 1
created: '2026-11-01T17:32:03Z'
updated: '2026-11-01T17:41:13Z'
labels:
- release
---

Joel (2026-11-01): land the families arc (yaks-9b41) and cut 0.0.28. Carries yaks-5d1c (whoami), yaks-de7a (glibc 2.35), yaks-b46b (bare host), yaks-6344, yaks-aad3, and the yaks-9b41 arc.

---
▸ 2026-11-01T17:32:03Z [yakherd]
kind: move
from: hairy
to: shaving
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T17:41:13Z [yakherd]
kind: landed
sha: 6dae2b8
---

Released v0.0.28: main 8d9a1b1..6dae2b8 pushed to origin (and to the local checkout's main), tag v0.0.28, release run 41332133697 passed (test, 5 builds, publish). npm: @j15r/yaks@0.0.28 and the platform packages published per the run log; `npx -y @j15r/yaks@0.0.28 --version` prints 0.0.28 on macOS. `cargo install --path .` put 0.0.28 in ~/.cargo/bin; it reads this farm and memcraft's cleanly.

---
▸ 2026-11-01T17:41:13Z [yakherd]
kind: move
from: shaving
to: shorn
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
