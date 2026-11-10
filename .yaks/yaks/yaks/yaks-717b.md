---
id: yaks-717b
title: 'tests/init_modes: farm_works_from_a_git_worktree... fails on a rerun (leftover share2-web-wt)'
status: shaving
type: bug
priority: 4
created: '2026-11-04T23:13:02Z'
updated: '2026-11-10T21:13:53Z'
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

---
▸ 2026-11-10T21:12:25Z [herd-unlock]

Claim (yakherd herd-unlock-2): worker `flaky-1`, Sonnet 5.5. Covers yaks-06cd's three cases too (record what you find there; the yakherd closes it).

## Scope

- `tests/init_modes.rs` `farm_works_from_a_git_worktree_and_a_shared_clone_of_the_sharing_repo`: siblings `share2-web-wt`/`-clone` with fixed names; put them under the test's own temp dir (or pid them) and clean up. Reproduce first: run the test twice.
- `tests/sync.rs` `a_private_farm_syncs_locally_but_never_pushes` and `tui::farmsync::tests` (10 s wait): find what makes them load-sensitive (a timeout, a shared path, a fixed port, a global env) and fix the cause, not the timeout, where you can.

## Evidence

`cargo test --test init_modes` twice in a row passes; the whole gate run three times in a row passes while another `cargo build --release` runs in parallel (say so in the note). Gate `cargo test --workspace`. Test files only, plus test helpers.

---
▸ 2026-11-10T21:13:53Z [flaky-1]

Start Tue Nov 10 21:13:53 UTC 2026 ; checkout /Users/joel/src/yaks/.delta/worktrees/axjw1h424can/yaks
