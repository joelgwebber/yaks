---
id: yaks-e9bb
title: 'Sheds: one baseline per shed, its fork point, from every viewer (drop the viewer-relative sync rule)'
type: task
priority: 2
created: '2026-10-10T18:43:28Z'
updated: '2026-10-10T18:43:28Z'
parent: yaks-ee0a
labels:
- cli
- delta
---

Implements decisions 1-5 of the shed semantics (see the Decisions note on yaks-ee0a). Today a shed's baseline is the NEARER of its fork point and its merge-base with the VIEWER (`nearer_baseline`, yaks-22f1), so two viewers can disagree about one shed, and the sync case needs the viewer's merge-base, which a shallow clone cannot always give.

Change: the baseline is the shed's fork point (oldest reflog entry, validated as an ancestor of its HEAD) from every viewer; the shed side is its working tree. Remove `nearer_baseline`, `BaseKind::Sync` and the `sync` value of `farm.vs` (JSON; record the change as a Decision), keep the merge-base fallback for a missing/unusable reflog (`vs merge-base`, still viewer-relative: say so). The primary's own row, once listed (yaks-ad6d), is measured from `origin/main`.

Spike results (2026-10-10, real data and a scratch repo), so nobody re-derives them:
- `git worktree add -b`, `--detach` and an existing branch all write the creation as the FIRST reflog entry (`0000.. <start commit>`), so the fork point is the start commit, as for a Delta clone (its first entry is the clone's HEAD at creation).
- A later `git merge main` in the shed is an ordinary reflog entry (`merge main: ...`); the first entry never moves. So after a sync the fork-only baseline still credits the shed with what it merged in. On this thread that is three yaks of Joel's and the coordinator's (yaks-3479, a4d8, cf2e), shown with their own actors: the accepted redundancy.
- Shallow: Delta's thread sync left the primary (~/src/yaks) and this clone shallow, boundary 2c28d23, a pinned commit (also yaks-1b39, yaks-a398). ~/src/j15r and its clones are not shallow. Fork-only needs no merge-base with the viewer, so it is less sensitive to a shallow viewer; the `is-ancestor` check of the fork point within the shed's own history remains.
Evidence: tests for two viewers (primary and a sibling) giving the same answer for one shed that merged the viewer's main after forking; a worktree shed; a clone; docs parity (docs/cli.md, README, yaks skill, `Sheds` help). Supersedes the sync part of yaks-22f1.
