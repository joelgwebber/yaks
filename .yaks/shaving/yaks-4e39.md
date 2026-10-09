---
id: yaks-4e39
title: 'yaks -C <path>: run any command as if started in <path> (global flag, like git -C)'
type: feature
priority: 3
created: '2026-10-09T20:23:49Z'
updated: '2026-10-09T20:24:48Z'
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
