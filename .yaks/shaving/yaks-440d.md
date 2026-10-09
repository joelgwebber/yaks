---
id: yaks-440d
title: 'yaks --shed <name>: run any command in another shed (full access; -C by name)'
type: feature
priority: 3
created: '2026-10-09T22:35:47Z'
updated: '2026-10-09T22:51:35Z'
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
