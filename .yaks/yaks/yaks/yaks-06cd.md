---
id: yaks-06cd
title: 'Load-sensitive tests: init_modes worktree test and sync''s private-farm test fail under parallel builds'
status: hairy
type: bug
priority: 2
created: '2026-11-07T00:04:53Z'
updated: '2026-11-10T22:01:13Z'
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
