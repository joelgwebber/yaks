---
id: yaks-22f1
title: 'yaks sheds: use the nearer of the fork point and the merge-base with the viewer as a shed''s baseline'
type: bug
priority: 2
created: '2026-10-06T22:19:03Z'
updated: '2026-10-06T22:19:09Z'
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
