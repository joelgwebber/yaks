---
id: yaks-79a5
title: 'Fewer yak-herding commits: ideas to bring them down to a dull roar'
type: idea
priority: 4
created: '2026-10-06T21:32:29Z'
updated: '2026-10-06T21:32:29Z'
parent: yaks-b5a0
labels:
- git
---

Joel (2026-10-06), for later, notes and ideas only: is there any clever git trickery to reduce the frequency and visibility of all the yak-herding commits? "Not a huge deal, but would be nice to get them down to a dull roar."

First observation: what is noisy is mostly LOCAL. Everything I publish is already squashed into a few per-yak commits, so the shared history carries far fewer herding commits than the working history does. The remaining noise is in `git log` of a working clone and in a PR diff.

Ideas, roughly cheapest first:
1. Fold, don't stack: a coordinator's notes and moves could ride in the NEXT code commit, or `yaks commit` could amend an unpushed herding commit (a rolling "yak herding" commit that is amended until pushed) instead of adding a new one each time. Risk: amending is a history rewrite, safe only while nothing depends on it (workers pin history, yaks-df61 O42).
2. Fixup and autosquash: `yaks commit` makes `fixup!` commits against the last herding commit; one `git rebase --autosquash` before publishing collapses them. Same rewrite caveat.
3. Hide them from reading: a documented `git log -- . ':!.yaks'` (or a `yaks log --code`) and a pathspec-excluding alias, plus a trailer or a fixed subject so tools can filter. Costs nothing, hides rather than removes.
4. Batch at natural points: commit the farm at session start/end and before a spawn or a landing (the moments the skills already require), not after every edit; `yaks status` (yaks-ee0a) tells you when it is dirty.
5. The structural option: the farm on its own ref or worktree (yaks-5f12, option 3), so yak-only commits never appear in code history at all; it costs the provenance that comes free when a yak move rides in the code commit, and PR-visible yak moves.
6. Publishing already helps: squash per yak at publish (what we do) turns N herding commits into one.

Lean: 3 and 4 first (no history rewriting), 6 stays the practice, 5 is the answer only if the noise stays a problem after those; 1 and 2 are the clever ones and the ones that can bite.
