---
id: yaks-760d
title: 'yaks land: record the landing after a conflicting cherry-pick is resolved'
status: shorn
type: bug
priority: 3
created: '2026-11-10T17:56:51Z'
updated: '2026-11-10T21:43:20Z'
tags:
- cli
- git
---

From the first reflection round (yaks-2b40): when `yaks land` falls back to a cherry-pick that stops on conflicts, the coordinator resolves and runs `git cherry-pick --continue`, and nothing records the `kind: landed` notes; herd-upkeep-2 added them by hand ([session](/sessions/2026/11/herd-upkeep-2-2026-11-06-1904-wrapping-up-yaks-1c89-tools-refs.md)).

Options: `yaks land --continue` (runs the continue and records), or `yaks land <sha>` on an already-applied pick recognises it and records only. Lean: `--continue`, mirroring git.

---
▸ 2026-11-10T21:11:51Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T21:12:25Z [herd-unlock]

Claim (yakherd herd-unlock-2): worker `landcont-1`, Sonnet 5.5.

## Scope

`yaks land --continue`: after `yaks land <sha>` stopped on a conflicting cherry-pick and the user resolved the files, it runs `git cherry-pick --continue` (non-interactive: `GIT_EDITOR=true`) and then records the same `kind: landed` notes the clean path records, for the same yaks (from the commit message, plus any `--yak` given to the first run). The first run must leave behind what `--continue` needs (the worker SHA, the `--yak` list): store it under the code repo's `.git/` (e.g. `.git/yaks-land.json`), never in the farm; remove it on success or `--abort`. Add `--abort` (runs `git cherry-pick --abort`, removes the state). The stop message names `yaks land --continue` instead of the bare git command. `src/land.rs`, the clap args in `src/main.rs`, `docs/cli.md`, the `yaks-coordinating` skill's Landing section (one line), `.agents/skills/land/` if it mentions the continue.

## Evidence

An integration test (like `tests/` ones for land, if any; else new) that builds a conflict, resolves it, runs `--continue`, and checks the landed note. Gate `cargo test --workspace`.

---
▸ 2026-11-10T21:17:56Z [landcont-1]

Started 2026-11-10T21:14Z (`date -u`), checkout `/Users/joel/src/yaks/.delta/worktrees/a1y6t2gq6x45/yaks`.

Built so far: `land::Action` (Land/Continue/Abort), state file `<git-dir>/yaks-land.json`, `--continue`/`--abort` clap args.

Decision: state lives in the checkout's own git dir (`rev-parse --absolute-git-dir`, per worktree like `CHERRY_PICK_HEAD`), as `yaks-land.json` with `sha`, `yaks`, `head`. Rejected: the farm (forbidden; a commit-less state is not yak knowledge) and `.git/yaks-land.json` via common dir (a linked worktree's pick is per worktree).

Decision: `yaks land <sha>` while a stopped landing's CHERRY_PICK_HEAD matches the saved sha refuses with exit 8 (dry run just says so). Rejected: the old behaviour (abort and reset), which would throw the user's resolution away.

Decision: `--continue` also accepts a pick the user already finished with `git cherry-pick --continue` (HEAD moved on from the saved head) and only records. Rejected: refusing, which would force a hand-written note, the exact bug.

---
▸ 2026-11-10T21:43:11Z [landcont-1]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-11-10T21:43:19Z [landcont-1]

Summary: `yaks land --continue` / `--abort` shipped.

- First run that stops on a conflicting pick saves `yaks-land.json` (worker sha, `--yak` ids, HEAD) in the checkout's git dir; the stop message names `yaks land --continue`.
- `--continue` runs `git cherry-pick --continue` with `GIT_EDITOR=true`, then records via the same `record` path as a clean landing; `--abort` aborts the pick and clears the state.
- A second `yaks land <sha>` while the stop waits refuses with exit 8.
- Tests: `tests/land_continue.rs` (6, binary against temp repos with a branch farm: conflict, resolve, `--continue`, `yaks commits` shows the landed note; also `--yak` carry-over, unresolved retry, pick finished by git, `--abort`, flag exclusivity). Also exercised by hand in a scratch repo with `target/debug/yaks`.
- Gate: `cargo test --workspace` green on the final run (one `init_modes` test failed once in a run alongside other workers, passed alone and in the rerun: not in this change); `cargo fmt --check` ok; clippy warning count unchanged at 104.
- Docs: clap help, `docs/cli.md`, `yaks-coordinating` section 6. `.agents/skills/land/SKILL.md` says only "Conflicts: pause", unchanged.

---
▸ 2026-11-10T21:43:20Z [landcont-1]
kind: move
from: shaving
to: shorn
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFVgG5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
