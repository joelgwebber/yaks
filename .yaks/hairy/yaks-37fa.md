---
id: yaks-37fa
title: 'yaks lanes: measure a lane''s own changes from where the lane forked, not from the viewer''s merge-base'
type: bug
priority: 2
created: '2026-10-06T21:05:06Z'
updated: '2026-10-06T21:32:29Z'
parent: yaks-38dc
labels:
- cli
- delta
---

Found by Joel running `yaks lanes` from ~/src/yaks (2026-10-06): twelve lanes, every one labelled `who: delta-lead` plus a mixture of the others (one lane lists six actors), the same `shaving:` yaks and the same `farm:` changes repeated across lanes. He asked whether threads share worktrees. They do not: it is a defect in how "a lane's own changes" is defined (yaks-d738 / yaks-c29a).

## Cause
A lane's own changes are measured from the merge-base with the VIEWER's HEAD. A worker forks from the coordinator's thread, not from the viewer, so its history contains everything the coordinator committed before the fork; relative to the viewer all of that becomes "this lane's" work. Measured on the real lanes (commits since the lane's actual fork vs commits attributed to it):

- inp-1's lane: 1 vs 3x; agt-1's lane: 1 vs 23; ntf-1's lane: 1 vs 3x; the coordinator's own lane: 49 vs 38.

A second effect compounds it: after a squash landing the workers' original commits are no longer ancestors of the viewer's HEAD, so a finished, landed lane still looks unmerged.

## Fix
Measure a lane from where IT forked. The fork point is the oldest entry of the lane's HEAD reflog (`<git-dir>/logs/HEAD`, first line), which exists for a Delta clone and for a `git worktree add -b` lane (core.logallrefupdates). Use it as the baseline for: who (actors on the lane's own entries), in_progress, the farm delta (new/moved yaks, new notes, needs). Fall back to today's merge-base when the reflog is missing or its first commit is unknown, and say `(vs viewer)` so nothing is silently wrong. Keep ahead/behind as they are (they are about the viewer).

## Evidence
Tests with temp repos: a worker forked from a coordinator that already has its own notes shows only the worker's entries; two lanes forked from the same coordinator do not share labels; a landed (squashed) lane shows its own commit, not the history before it; no reflog falls back and says so. A real run in ~/src/yaks must show one distinct actor set per worker lane, and `inp-1` in inp-1's lane only.

---
▸ 2026-10-06T21:32:29Z [delta-lead]
Terminology decision (Joel, 2026-10-06, via the naming subthread): a "lane" is now a SHED, everywhere, no aliases; `yaks lanes` becomes `yaks sheds`. Text above this note, and all history, keeps the word "lane": read it as "shed". Not decided: "clip" (what a shed has produced) and "barn" (the primary checkout). The rename itself is yaks-dfca; it lands before the baseline fix (yaks-37fa) because both edit the same file.
