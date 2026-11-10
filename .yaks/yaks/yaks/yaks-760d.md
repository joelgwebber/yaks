---
id: yaks-760d
title: 'yaks land: record the landing after a conflicting cherry-pick is resolved'
status: shaving
type: bug
priority: 3
created: '2026-11-10T17:56:51Z'
updated: '2026-11-10T21:12:25Z'
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
