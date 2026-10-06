---
id: yaks-22f1
title: 'yaks sheds: use the nearer of the fork point and the merge-base with the viewer as a shed''s baseline'
type: bug
priority: 2
created: '2026-10-06T22:19:03Z'
updated: '2026-10-06T22:23:22Z'
parent: yaks-38dc
labels:
- cli
- delta
---

Found by Joel (2026-10-06) reading `yaks sheds` from ~/src/yaks after yaks-37fa: the yakherd's own shed (xp49gyb9fg59) shows `who:` with eighteen actors and a farm delta of 21 new and 16 moved yaks, though the shed is only ONE commit ahead of his checkout (`+1 -0`). He asked whether that is a misfire.

## Cause
yaks-37fa measures a shed from the OLDEST entry of its HEAD reflog (its fork point) whenever that is an ancestor of the shed's HEAD. That is right for a short-lived worker shed (everything since the spawn is the worker's own work) and wrong for a long-lived shed that has been synced with the viewer since it forked. Measured on the real data: the fork point is 23 commits back, the merge-base with the viewer is the viewer's own HEAD, 1 commit back. Everything in between is already in the viewer's history, so it is not "what this shed has that the viewer lacks".

## Fix
The baseline is the NEARER of the two candidates: the validated fork point F and the merge-base M with the viewer. If F is an ancestor of M (M descends from F, i.e. the shed was synced after it forked) use M; otherwise use F (a worker forked from a coordinator whose commits are not ancestors of the viewer's, for instance after a squash landing, has an older or unrelated M, so F wins). If neither can be read, fall back as today.
- M chosen because it is nearer is NOT a fallback: it is exact. It must not carry the `(vs merge-base)` warning that marks the "no usable reflog" case, which means something different ("this also counts what the shed inherited"). Decide the JSON `farm.vs` value and the wording, and record it as a Decision (for example `sync`, or keep `merge-base` and add a field that says why).
- The label (who, shaving) and the farm delta both use the chosen baseline.

## Evidence
Tests with temp repos: a shed forked, then merged/synced with the viewer: it shows only what came after the sync (one commit, one note, no old actors); a worker forked from a coordinator while the viewer is unrelated by squash: F still wins and shows only the worker; a shed with no sync behaves exactly as yaks-37fa tests expect; missing/unusable reflog unchanged. A real run from ~/src/yaks: the yakherd's shed shows one new note and the actors who wrote since the sync, not eighteen; every worker shed is unchanged from the 37fa output. Docs parity (docs/cli.md sheds row, README, `.agents/skills/yaks/SKILL.md`, the `Sheds` help).

---
▸ 2026-10-06T22:19:09Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-06T22:19:09Z [delta-lead]
Claim (coordinator delta-lead). Owner: worker nbr-1. Scope: `src/sheds.rs` (the baseline choice and its tests), the `sheds` rows in docs/cli.md, README.md and `.agents/skills/yaks/SKILL.md`, the `Sheds` help in src/main.rs. Judge: the coordinator re-runs the gate and `yaks sheds` from ~/src/yaks. Forbidden: ahead/behind semantics, the TUI, other commands, the coordination skills' prose. Terminology: a former lane is a SHED everywhere.

---
▸ 2026-10-06T22:19:29Z [nbr-1]
Start: Tue Oct  6 22:19:26 UTC 2026
Path: /Users/joel/src/yaks/.delta/worktrees/qw0xx3s64g6e/yaks/.yaks/shaving/yaks-22f1.md

![sheds-before](artifacts/yaks-22f1/sheds-before.txt)

![sheds-after](artifacts/yaks-22f1/sheds-after.txt)

---
▸ 2026-10-06T22:22:56Z [nbr-1]
Decision: a baseline chosen because it is nearer is its own kind, `BaseKind::Sync`: JSON `farm.vs: "sync"`, `farm.base` = the merge-base's short sha, NO `(vs ...)` marker on the farm or label line (it is exact, like the fork point). Rejected: keeping `merge-base` plus a `why` field (a consumer reading `vs` would still conflate it with the inherited-work fallback, which is the thing the marker warns about); marking it `(vs sync)` (nothing to warn about, and the table would grow noise).

---
▸ 2026-10-06T22:22:56Z [nbr-1]
Decision: "nearer" is tested as `M != F` and `git merge-base --is-ancestor F M` (in the viewer's repo). When M == F, or M is older than / unrelated to F, the shed stays `Fork` (so unsynced sheds and workers forked from a coordinator whose commits the viewer lacks, e.g. after a squash landing, are unchanged). Rejected: picking by commit date or rev-list count (clock/graph-size heuristics; ancestry is the exact statement of "already in the viewer's history").

---
▸ 2026-10-06T22:22:56Z [nbr-1]
Decision: the fallback chain is [chosen exact baseline (Fork or Sync), merge-base (marked)], with the existing "farm unreadable at this commit, try the next" behaviour. If M's farm cannot be read the chain tries M again as `merge-base`, which fails identically; it does not fall back to F. Rejected: adding F as a third candidate (a shed whose farm exists at F but not at its own later merge-base is not a real case worth extra states).

---
▸ 2026-10-06T22:22:56Z [nbr-1]
Decision: a test-setup point worth recording: "sync" means only commits that are in the viewer's history are dropped. A shed's own pre-sync commits that the viewer does not have are still the shed's own work and still show (first draft of the shed-merged-viewer test wrongly expected otherwise). The tests: shed landed on the viewer by a merge commit; shed that merged the viewer; unsynced shed with a viewer that moved on (Fork); squash-landed worker (Fork, asserted sha); missing/unusable reflog (MergeBase) unchanged.

---
▸ 2026-10-06T22:23:03Z [nbr-1]
Evidence: real `yaks sheds` from /Users/joel/src/yaks, before (copy of the coordinator's binary) and after (this change's release binary), attached in full as sheds-before.txt / sheds-after.txt. `diff before after` shows only two changes:
- the yakherd's shed xp49gyb9fg59 (HEAD 93b3fb9, +3 -0): before `who:` 18 actors, `farm: 22 new, 16 moved, 94 new notes`; after `who: delta-lead · shaving: yaks-22f1, yaks-b5a0`, `farm: 1 new, 1 new notes` (yaks-b5a0 +1 notes). Baseline is the merge-base with the viewer.
- my own shed's DIRTY count (1 -> 6, because I was editing).
Every other shed's lines are byte-identical (worker sheds unchanged).
Docs parity: updated the `sheds` row in docs/cli.md, README.md, .agents/skills/yaks/SKILL.md and the `Sheds` help in src/main.rs; grep -rnE "merge-base|forked|fork point|vs this checkout" over docs README.md .agents/skills/yaks .agents/skills/yaks-tracker src/main.rs src/tui src/skills.rs found no other description of the baseline (docs/research and tui snapshots excluded). The `?` help overlay does not mention sheds. Coordination skills' prose left alone (out of scope).
Summary: `nearer_baseline` in src/sheds.rs picks M (new `BaseKind::Sync`) when M != F and F is an ancestor of M, else F; label and farm delta both use it; 3 new temp-repo tests + a strengthened squash test.

---
▸ 2026-10-06T22:23:19Z [nbr-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-06T22:23:22Z [nbr-1]
moved: shaving -> shorn
