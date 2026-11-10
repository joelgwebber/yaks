---
id: yaks-717b
title: 'tests/init_modes: farm_works_from_a_git_worktree... fails on a rerun (leftover share2-web-wt)'
status: shaving
type: bug
priority: 4
created: '2026-11-04T23:13:02Z'
updated: '2026-11-10T22:00:46Z'
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

---
▸ 2026-11-10T22:00:22Z [flaky-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-10T22:00:44Z [flaky-1]

Decision: `init_modes` worktree test uses the file's own `temp()` for the `-wt` and `-clone` dirs (pid-suffixed, cleared first) plus an explicit cleanup at the end.

- Rejected: only pid-suffixing the siblings (still leaves them behind).
- Rejected: nesting them inside `web`'s dir (a worktree inside its own repo's tree dirties `git status`).
- git accepts an existing empty directory for `worktree add` and `clone`, so `temp()` works as the target.

---
▸ 2026-11-10T22:00:44Z [flaky-1]

Decision: `tui::farmsync::tests` siblings (origin, other clone) are derived from `temp_farm`'s own pid+counter dir name, via a `sibling()` helper, and a `cleanup()` removes them. `published_farm` lost its `tmp` argument.

- Rejected: keeping per-test fixed strings (`yaks-f0db-pull-origin.git`): those are shared by every `cargo test` on the machine.

---
▸ 2026-11-10T22:00:45Z [flaky-1]

Decision: `settle_auto_sync` waits on a 120 s wall-clock deadline instead of 400 polls of 25 ms.

- The old loop failed after ~10 s of sleeping, plus tick time. The sync is a thread running several git processes, so its duration scales with load; there is no fixed path, port or env var to fix there.
- Rejected: only raising the iteration count (still a guess); a mutex over all tests (hides the cause and slows the suite).
- No non-test code needed, so no `yaks ask`.

---
▸ 2026-11-10T22:00:45Z [flaky-1]

Decision: `tests/sync.rs` `origin_and_a` now sets `XDG_DATA_HOME` (and removes `YAKS_DIR`) on the `yaks init` it runs, like `Machine::yaks_out` does.

- Before, that one call wrote a farm home under the real `~/.local/share/yaks/farms` (not the temp root), which is a leak outside the test's temp root.
- It is not a proven cause of the load failure. Stated honestly under the sync case below.

---
▸ 2026-11-10T22:00:46Z [flaky-1]

## Findings (also yaks-06cd's three cases)

Machine: 14 cores, load average 18–35 throughout (6 other workers building and testing, plus up to 40 busy-loop processes and 8–16 parallel copies of the test binaries I started). `cargo build --release` ran in parallel with each of the three full-gate runs.

**1. init_modes worktree test (yaks-717b, yaks-06cd case 1).**

- Reproduced: ran `cargo test --test init_modes farm_works_from_a_git_worktree` twice; the second run failed with `fatal: '.../share2-web-wt' already exists`.
- Cause: `share2-web-wt` and `share2-web-clone` were `web.with_file_name(..)` siblings with fixed names, never removed. `temp()` clears only its own pid-suffixed dirs.
- Fix: siblings come from `temp()` (pid-suffixed, cleared) and the test removes them and its xdg dirs at the end.
- Shown: `cargo test --test init_modes` twice in a row: 13 passed both times; no `yaks-init-share2*` dir left in the temp dir.

**2. `tui::farmsync::tests` under load (yaks-06cd case 3).**

- Reproduced: 8 parallel copies of `cargo test --bin yaks farmsync` with a shared `TMPDIR`. All 8 failed. Two distinct failures:
  1. `published_farm` / the `other` clone used fixed names (`yaks-f0db-pull-origin.git`, ...), so concurrent runs collided: `git push` rejected ("fetch first", "reference already exists") and one `remove_dir_all(...).unwrap()` hit ENOENT.
  2. After fixing 1: `the interval sync did not finish` (the 400 x 25 ms poll budget) when the machine was loaded.
- Fix: names derived from `temp_farm`'s pid+counter dir (`sibling()`), `cleanup()` at the end of each test, and a 120 s wall-clock deadline in `settle_auto_sync`.
- Shown: same 8-way stress after the fix: 8 of 8 runs, 8 passed each, nothing left in the shared `TMPDIR` except `yaksrs-tui-<pid>-<n>` (each test's own `temp_farm` dir, which `cleanup()` does not remove if a test fails first; on success it is removed).
- Not fixed, outside scope: `temp_farm` itself never deletes its project dir, so other TUI tests leave `yaksrs-tui-*` dirs behind. Left alone, belongs to a different yak.

**3. `tests/sync.rs` `a_private_farm_syncs_locally_but_never_pushes` (yaks-06cd case 2).**

- NOT reproduced: 6 sequential runs of `cargo test --test sync` under 24 busy-loop processes; 16 parallel copies of the whole binary; 40 parallel copies of that one test with a shared `TMPDIR`; all passed (the slowest run took 4+ min, but none failed).
- Reading the test: its paths are all unique (`tmp()` is pid-suffixed, and it removes `base`). It runs git hooks-free local pushes only. No fixed port, no shared path inside the temp root, no timeout in `sync::run`.
- One real flaw found and fixed: `origin_and_a`'s `yaks init` did not set `XDG_DATA_HOME`, so it created a farm home dir under the real `~/.local/share/yaks/farms`. A shared global dir is the only thing the test touched that two runs could share; I could not make it fail, so I do not claim it was the cause.
- Honest status: could not reproduce the one-off failure; closing this case as "not reproducible, one global-path leak fixed". The original report did not capture the failure message, which would have settled it.

**Unrelated flake seen once.** `preflight::tests::verify_check_passes_and_fails_on_the_last_verify_note` failed once in my first full-gate run (it runs `true` / `false` verify commands in a temp farm). It passed in 5 later full runs, 1 stress run with 30 busy loops, and 8 parallel copies of the preflight tests (about 90 s each). I did not capture the assertion message (my filter hid it), so I cannot say why. Worth its own yak if it recurs; I have not filed one (not my yak).

## Gate

- `cargo test --workspace` three times in a row, each while `cargo build --release` ran in parallel: exit 0, all 32 result lines `ok`. A 4th run through `yaks verify yaks-717b`: PASS.
- `cargo fmt --check`: clean.
- `cargo clippy --all-targets`: 104 warning lines before and after my change (same count; nothing added).
