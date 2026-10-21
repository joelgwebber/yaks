---
id: yaks-3bcd
title: 'land.sh: when the worker SHA is already an ancestor, check HEAD''s tree really contains its change'
status: shorn
type: bug
priority: 2
created: '2026-10-13T02:02:44Z'
updated: '2026-10-21T22:31:01Z'
parent: yaks-e5fb
labels:
- skills
- delta
---

Seen 2026-10-13 (see the incident note on yaks-38dc): a commit made while Delta held a pending merge became a merge commit with the worker's SHA as a parent but without its changes. `land.sh <sha>` then reported `landed: <HEAD>` because the SHA was an ancestor, so a silently dropped worker would have been pushed as landed.
Fix in .agents/skills/yaks-coordinating-delta/land.sh: when the SHA is already an ancestor of HEAD, verify `git merge-tree --write-tree HEAD <sha>` equals HEAD's tree (merging it again changes nothing); otherwise refuse, saying the worker's change is missing from HEAD although its commit is an ancestor, and name the commit that introduced the drop if cheap. Also: before ANY commit, refuse/warn when MERGE_HEAD is set (a pre-commit check in the skill text; maybe a `yaks preflight` check that MERGE_HEAD is unset). Update the skill's landing steps. Evidence: a scripted repo reproducing the evil merge makes land.sh refuse; a normal already-landed SHA still says landed.

---
▸ 2026-10-21T22:10:09Z [delta:herd-dirs]

Claimed with yaks-e5fb by `land-1` (the fix goes into `yaks land`; spec on yaks-e5fb).

---
▸ 2026-10-21T22:10:10Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T22:31:00Z [land-1]

Done in yaks-e5fb's change (`yaks land`, src/land.rs). Finding: the check as specced (`merge-tree --write-tree HEAD <sha>` == HEAD's tree) is vacuous for an ancestor (merge base is the SHA itself, result always HEAD's tree; observed). Implemented with `--merge-base=<worker parent>` (git 2.40+) instead: a merge that dropped the change is refused (exit 6, names the first commit lacking it), a really-landed ancestor still says `landed:`, and an unconfirmable one (later edits conflict) is refused too. Test: the_evil_merge_that_dropped_the_workers_change_is_refused fails with the vacuous check. NOT done: the "pre-commit check that MERGE_HEAD is unset / yaks preflight" part of the yak text (not in the claim note's scope).

---
▸ 2026-10-21T22:31:01Z [land-1]

moved: shaving -> shorn
