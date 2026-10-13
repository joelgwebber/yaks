---
id: yaks-3bcd
title: 'land.sh: when the worker SHA is already an ancestor, check HEAD''s tree really contains its change'
status: hairy
type: bug
priority: 2
created: '2026-10-13T02:02:44Z'
updated: '2026-10-13T02:02:44Z'
parent: yaks-38dc
labels:
- skills
- delta
---

Seen 2026-10-13 (see the incident note on yaks-38dc): a commit made while Delta held a pending merge became a merge commit with the worker's SHA as a parent but without its changes. `land.sh <sha>` then reported `landed: <HEAD>` because the SHA was an ancestor, so a silently dropped worker would have been pushed as landed.
Fix in .agents/skills/yaks-coordinating-delta/land.sh: when the SHA is already an ancestor of HEAD, verify `git merge-tree --write-tree HEAD <sha>` equals HEAD's tree (merging it again changes nothing); otherwise refuse, saying the worker's change is missing from HEAD although its commit is an ancestor, and name the commit that introduced the drop if cheap. Also: before ANY commit, refuse/warn when MERGE_HEAD is set (a pre-commit check in the skill text; maybe a `yaks preflight` check that MERGE_HEAD is unset). Update the skill's landing steps. Evidence: a scripted repo reproducing the evil merge makes land.sh refuse; a normal already-landed SHA still says landed.
