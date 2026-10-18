---
id: yaks-6a34
title: 'Trial: one live farm per machine (sparse farm worktree); sheds become git facts'
status: shaving
type: task
priority: 1
created: '2026-10-18T05:15:02Z'
updated: '2026-10-18T05:15:36Z'
labels:
- sheds
- git
- design
---

Decision (Joel + delta-farm, 2026-10-18): try the simple model for a few rounds before building more shed machinery. One **live farm per machine**, shared by every shed (the primary, the coordinator's clone, every worker clone), instead of one committed `.yaks/` copy per checkout reconciled by diffing. The farm stays git-tracked (team mode), but lives in its **own sparse `git worktree`** on a branch `farm` of this repo, so yak drift never dirties the primary and the farm has a carrier (`origin/farm`) on a machine with no primary checkout.

Why: the per-shed-copy model generated a two-week tail of reconciliation bugs (yaks-e475 lost answer, yaks-29e9 double answer, yaks-e92e phantom edits, yaks-51b3, yaks-5cea, yaks-31e2, yaks-78ff), `src/sheds.rs` is the largest file in the repo (6.4k lines, reflog-parsing fork points), and the team coordinating skill is a list of traps while the private skill ("one live farm, nothing to merge") describes this design. The revisit trigger set on yaks-5f12 (Oct 5) fired several times over. yaks-5f12 was part of the now-defunct git-store investigation; this yak supersedes it as the decision record. The full assessment is in the Delta thread that opened this yak.

## The model
- `.yaks/` stays committed. A committed `live_branch: farm` in `config.yaml` says: the live farm is the worktree where branch `farm` is checked out. Discovery from ANY checkout of the repo (primary, git worktree, Delta clone via its host) resolves there; a checkout's own `.yaks/` is a stale, never-written mirror. No worktree for that branch = an error naming the fix (`YAKS_DIR` overrides, for a machine whose host is Delta's bare repo).
- Writes: every shed writes the live farm through the CLI/TUI. The farm lock (`.yaks/.lock`) serializes every process on the machine. Workers never stage anything under `.yaks/`; yak ids in commit messages are fine.
- Commits: `yaks commit` commits on `farm` (its root is the farm worktree). `main` takes the herding commits by merging `farm` (the only branch that changes `.yaks`, so never a conflict): the coordinator does `git fetch local farm && git merge local/farm` into the branch it is about to push. PRs and feature branches carry no yak churn (yaks-79a5, option 5).
- Machines: `git push origin farm` before a switch, `git -C ~/src/yaks-farm pull` after. Same path on every machine (`~/src/yaks-farm`) so briefs do not change.
- Sheds: `yaks sheds` keeps the discovery + git half (clones, branches, ahead/behind, dirty, merged, liveness, names). The farm-diff half (fork points, `landed`, `needs_superseded`, ghost rows, `inbox --sheds`, `answer @shed`, `show --sheds`, `changes <shed>`) goes quiet by construction (every shed shares the farm) and is deleted if the trial holds; shed -> yak then comes from a shed stamp on each write (child yak).

## Setup (done once per machine, by hand for now)
```
git -C ~/src/yaks worktree add -b farm ~/src/yaks-farm main
git -C ~/src/yaks-farm sparse-checkout set .yaks
```
Verified on a scratch clone before adopting: write from a clone via the farm, primary clean, `yaks commit` on `farm`, push to the primary's `main` under `updateInstead` not blocked, `git merge farm` brings only `.yaks` changes, `yaks sheds` lists the farm worktree as `shares this farm`.

## What to watch during the trial (record each as a note here)
- A write that landed in a stale copy (a checkout's `.yaks/` dirty): who, and why discovery did not redirect.
- A worker commit touching `.yaks/`.
- The TUI description-edit clobber window (child yak) biting.
- Anything the quiet farm-diff half of the Sheds tab is missed for.
- Friction at a machine switch.

Undo: merge `farm` once more, `git worktree remove ~/src/yaks-farm`, `git branch -d farm`, remove `live_branch:` from config.

---
▸ 2026-10-18T05:15:36Z [delta-farm]

moved: hairy -> shaving
