---
id: yaks-717b
title: 'tests/init_modes: farm_works_from_a_git_worktree... fails on a rerun (leftover share2-web-wt)'
status: shaving
type: bug
priority: 4
created: '2026-11-04T23:13:02Z'
updated: '2026-11-10T21:11:50Z'
tags:
- tests
---

Found while landing yaks-bcd4. `farm_works_from_a_git_worktree_and_a_shared_clone_of_the_sharing_repo` (`tests/init_modes.rs`, ~line 417) makes `share2-web-wt` and `share2-web-clone` as siblings of its temp repo with fixed names and never removes them, so a second `cargo test --test init_modes` in the same temp dir fails with `fatal: '.../share2-web-wt' already exists`.

Reproduced on a clean tree (no yaks-bcd4 change): run it twice; the first passes, the second fails. The full `cargo test` passes on a fresh temp dir and fails on a rerun, which looks like a flaky gate.

Fix: give the siblings the pid like `temp()` does and remove them at the end, or put them under the test's own temp dir.

---
▸ 2026-11-10T21:11:50Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
