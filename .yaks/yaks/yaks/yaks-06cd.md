---
id: yaks-06cd
title: 'Load-sensitive tests: init_modes worktree test and sync''s private-farm test fail under parallel builds'
status: shaving
type: bug
priority: 2
created: '2026-11-07T00:04:53Z'
updated: '2026-11-10T22:03:36Z'
tags:
- cli
---

Two integration tests fail intermittently when the machine is busy, and since `cargo test` stops at the first failing binary they hide the rest and make `yaks verify` flaky for every worker.

- `tests/init_modes.rs` `farm_works_from_a_git_worktree_and_a_shared_clone_of_the_sharing_repo`: `git worktree add ... share2-web-wt` fails with "already exists". It failed 5 of 6 runs on `main` at 08b86fca, so it predates recent work; looks like a fixed temp path left over from an earlier run.
- `tests/sync.rs` `a_private_farm_syncs_locally_but_never_pushes`: failed once under five parallel worker builds, passed 3 of 3 alone.
- `tui::farmsync::tests` (10-second wait) failed for `tools-1` under the same load.

Found by the yakherd and workers of yaks-1c89, 2026-11-06.

---
▸ 2026-11-10T22:01:13Z [herd-unlock]
kind: landed
commit: 709ec3c5aa6d5dd48f7100ef30fc8b284e19a741
source: 669788f29fd83c6ffc350c83685bf0fa4852e31a
branch: main
---

landed: 709ec3c5 yaks-717b: unique, cleaned-up temp dirs in init_modes and farmsync tests; wall-clock wait for the interval sync

---
▸ 2026-11-10T22:03:35Z [herd-unlock]

Closed by herd-unlock-2 from flaky-1's work in yaks-717b (landed `709ec3c5`):

- `init_modes` worktree test: reproduced (fixed sibling names never removed); fixed.
- `tui::farmsync::tests`: reproduced under an 8-way stress (fixed clone names collided; a 400 × 25 ms poll gave up under load); fixed with derived names and a 120 s deadline.
- `sync`'s private-farm test: not reproduced in 60+ loaded runs; one real leak fixed (an `init` wrote under the real `~/.local/share`). Reopen with the failure message if it recurs.

Full gate after landing: 34 binaries green, run while nothing else was building.

---
▸ 2026-11-10T22:03:36Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
