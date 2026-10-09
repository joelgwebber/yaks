---
id: yaks-4e39
title: 'yaks -C <path>: run any command as if started in <path> (global flag, like git -C)'
type: feature
priority: 3
created: '2026-10-09T20:23:49Z'
updated: '2026-10-09T20:31:53Z'
parent: yaks-a3d2
labels:
- cli
---

---
▸ 2026-10-09T20:24:48Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-09T20:24:48Z [delta:Delta :Yaks (cont'd)]
Claim (coordinator). Owner: cflag-1.
Goal: a global `-C <path>` on `yaks` (`yaks -C <path> <command> ...`), exactly like `git -C`: every command behaves as if started with cwd = <path>. That covers farm discovery (store::discover / Farm::open), `sheds`, `discover`, `status`, `commit`, `init`, `skills`, `tui`, the project-skills notice, and anything else that reads `env::current_dir()`. A relative <path> resolves against the real cwd. Repeated -C composes like git's (each later one relative to the previous). A missing path is an error naming the path. YAKS_DIR still wins over discovery, as it does today.
Implementation lean (choose and record a Decision: note): add `#[arg(short = 'C', global = true)]` to `struct Cli` in src/main.rs, resolve it once at the top of `main`, then `std::env::set_current_dir` before anything reads the cwd. Rejected alternative to weigh: threading a `cwd` through every call site (many call sites, easy to miss one). If you choose set_current_dir, check that no code captured the cwd before that point.
Scope: src/main.rs (struct Cli, main/run top) plus a new integration test file tests/dash_c.rs. Docs: docs/cli.md (a global-flags line near the top), README.md, the `yaks` skill (`.agents/skills/yaks/SKILL.md`, one line in "Running yaks" or the command table), and the clap help.
Out of scope: --shed (later), any change to discovery semantics.
Evidence: tests/dash_c.rs, written to FAIL before your change, covering: (1) `yaks -C <repo> list` from an unrelated cwd lists that farm; (2) `yaks -C <sub-dir-of-repo> show <id>` works; (3) `yaks -C <a-delta-shaped-or-worktree-checkout> sheds --json` equals running `sheds --json` with that cwd; (4) a missing path is an error that names it; (5) `-C` after the subcommand (`yaks list -C <repo>`) also works, since the flag is global. Judge: the coordinator, at landing.

---
▸ 2026-10-09T20:26:48Z [cflag-1]
Start: Fri Oct  9 20:26:46 UTC 2026. yaks path: /Users/joel/src/yaks/.delta/worktrees/nw0v12c3ymte/yaks/.yaks/shaving/yaks-4e39.md

---
▸ 2026-10-09T20:28:46Z [cflag-1]
Decision: `-C` is `Vec<PathBuf>` on `struct Cli` (`short = 'C', global = true`), applied once at the top of `main` with `env::set_current_dir`, each step relative to the previous (git semantics). Rejected: threading a `cwd` through every `env::current_dir()` call site (9 sites in main.rs/init.rs, easy to miss one); `Option<PathBuf>` (would not compose). Nothing captures the cwd before `Cli::parse()` returns, and the only earlier reader is none (checked with grep for current_dir).

---
▸ 2026-10-09T20:28:46Z [cflag-1]
Decision: a missing path errors via anyhow `with_context("cannot change to directory '<path>'")` (as written, not canonicalised), so the message names the path the user typed. Rejected: canonicalising first (would name a resolved path, hides the typo).

---
▸ 2026-10-09T20:28:46Z [cflag-1]
Decision: `$YAKS_DIR` is untouched, so a relative value resolves against the new cwd (after -C), the same as if the process had started there; documented in cli.md and the clap help. Rejected: resolving YAKS_DIR against the original cwd (not "as if started in <path>").

---
▸ 2026-10-09T20:28:46Z [cflag-1]
Evidence: tests/dash_c.rs, 7 tests (list from unrelated cwd; show via sub-dir; sheds --json equals direct run in a linked worktree; missing path names it; `list -C` after the subcommand; repeated -C composes; init + discover honour -C). With src/main.rs stashed (no change) all 7 FAIL ("unexpected argument '-C' found"); with the change all 7 pass.
Note: `sheds` reports the primary checkout path even when run in a linked worktree, so the test asserts equality with the direct run rather than the worktree path. `cargo fmt -- src/main.rs` follows `mod` and reformats src/actor.rs, src/tui/docshots.rs, src/tui/tests.rs; reverted those, so only the intended hunks in main.rs remain.

---
▸ 2026-10-09T20:29:17Z [cflag-1]
verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-10-09T20:29:41Z [cflag-1]
verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-10-09T20:30:26Z [cflag-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-09T20:31:49Z [cflag-1]
Fix: field renamed `dir` -> `chdir`; clap used the field name as arg id and clashed with `skills install --dir` (panic in tests/skills_install.rs). Gate `cargo test --release` (all suites) passes; `yaks verify` PASS (it runs `cargo test -p yaks`, debug). Summary: global `-C <path>` (before or after subcommand), set_current_dir once at top of main; docs/cli.md, README.md, yaks SKILL.md, clap help updated; tests/dash_c.rs (7 tests).

---
▸ 2026-10-09T20:31:53Z [cflag-1]
moved: shaving -> shorn
