---
id: yaks-440d
title: 'yaks --shed <name>: run any command in another shed (full access; -C by name)'
type: feature
priority: 3
created: '2026-10-09T22:35:47Z'
updated: '2026-10-09T22:58:29Z'
parent: yaks-a3d2
labels:
- cli
---

---
▸ 2026-10-09T22:35:48Z [delta:Delta :Yaks (cont'd)]
Claim deferred: runs after yaks-886a lands (both edit the top of main). Decided by Joel (yaks-a3d2): --shed is full access, i.e. -C by name; tighten later if it proves problematic.

---
▸ 2026-10-09T22:51:35Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-09T22:51:35Z [delta:Delta :Yaks (cont'd)]
Claim (coordinator). Owner: shed-1. Decided by Joel (yaks-a3d2, 2026-10-09): `--shed` is FULL access, i.e. `-C` by name; tighten later if it proves problematic.
Goal: a global `--shed <name>` on `yaks` (accepted before or after the subcommand, like `-C`): resolve the name with `sheds::resolve` over the sheds visible from the current checkout (after any `-C` has been applied), then behave exactly as `-C <that shed's path>`. Print nothing extra on success (no banner; full access was chosen). On no or ambiguous match: print resolve's error (`error: ...`, it lists the sheds) and exit 1. `--shed` with `-C`: apply the -C chain first, then --shed relative to that checkout. Two `--shed` flags: an error (keep it simple).
Interaction with title recording (yaks-886a): the recording call is gated on `cli.chdir.is_empty()`; `--shed` must ALSO skip recording (the caller's DELTA_THREAD_TITLE belongs to the caller, not the target shed). Record a Decision: for how you gate it.
Scope: src/main.rs (`struct Cli` beside `chdir`, and the top of `main` right after the `-C` loop), a helper in src/sheds.rs only if needed (do not change discover/resolve semantics). Docs: docs/cli.md (the global-flags paragraph beside `-C`), README.md (beside the `-C` line), `.agents/skills/yaks/SKILL.md` (beside the `-C` sentence), `.agents/skills/yaks-coordinating/SKILL.md` §3 (the `-C` sentence: add `--shed <name>`, same caution), clap help.
Evidence (tests/shed_flag.rs, real binary, FAIL first): with a git worktree shed on branch `feat`, `yaks --shed feat list` from the main checkout lists the shed's farm (a yak only the shed has); `yaks list --shed feat` too; `--shed nope` exits 1 with the candidate list; `--shed feat` from inside a Delta-shaped clone with DELTA_THREAD_TITLE set does not record a title in the target. Judge: coordinator.

---
▸ 2026-10-09T22:57:29Z [shed-1]
Start: Fri Oct  9 22:52:28 UTC 2026
yaks path: /Users/joel/src/yaks/.delta/worktrees/vks97kdt9jpt/yaks/.yaks/shaving/yaks-440d.md

---
▸ 2026-10-09T22:57:29Z [shed-1]
Decision: gating the title recording. `Cli.chdir` stays as is and the recording condition becomes `cli.chdir.is_empty() && cli.into_shed.is_none()`. Rejected: setting a `hopped` bool inside the -C loop / enter_shed (a second source of truth that must be kept in step); clearing DELTA_THREAD_TITLE from the process env (would leak into nothing useful but mutates env via unsafe set_var/remove_var in edition 2024).
Decision: the global flag is `into_shed` with `#[arg(long = "shed", id = "into_shed")]`. A plain `shed: Option<String>` shares clap arg id `shed` with the positional of `changes <SHED>`, and clap's global propagation copied that positional into the global, so `yaks changes work` hopped into `work` and broke tests/changes.rs. Rejected: renaming the positional (changes user-visible help/usage).
Decision: `--shed` resolves in a small `enter_shed` in main.rs (open the farm at the post-`-C` cwd, `farm.sheds(cwd)`, `sheds::resolve`, `set_current_dir`), before init/skills/discover early-returns. No change to sheds::discover/resolve. Needs a farm in the starting checkout (sheds::discover is farm-relative); with none it prints `error: --shed needs a farm here to find the sheds: ...` exit 1. Rejected: a farmless discovery path (would mean changing discover's signature/semantics).
Decision: two `--shed` flags are an error by clap's own `Option<String>` rejection ("cannot be used multiple times"), no custom code. Nothing is printed on success.
Decision: `--shed` with `-C`: -C chain first, then --shed relative to that checkout (spec); tested.

---
▸ 2026-10-09T22:58:22Z [shed-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-09T22:58:29Z [shed-1]
Summary: global `--shed <name>` (clap id `into_shed`) in src/main.rs: `enter_shed` resolves over the sheds visible after the -C chain and `set_current_dir`s into it; title recording skipped when -C or --shed is used. tests/shed_flag.rs (8 tests, real binary): 6 fail without the src/main.rs change (stash check; the two-flag test and positional-regression test pass either way by design). Docs: docs/cli.md, README.md, skills yaks + yaks-coordinating, clap help, and the "never for -C" title sentences now say `-C` or `--shed`. Full gate `cargo test --release`: all suites ok.

---
▸ 2026-10-09T22:58:29Z [shed-1]
moved: shaving -> shorn
