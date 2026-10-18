---
id: yaks-6a34
title: 'Trial: one live farm per machine (sparse farm worktree); sheds become git facts'
status: shaving
type: task
priority: 1
created: '2026-10-18T05:15:02Z'
updated: '2026-10-18T06:15:40Z'
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

---
▸ 2026-10-18T05:24:49Z [delta-farm]

Setup done on this machine (delta-farm, 2026-10-18): `~/src/yaks-farm` = sparse worktree of branch `farm` (yaks-da69's `live_branch: farm` is committed on main, 92a4ac5). Verified with `~/go/bin/yaks`: `yaks path` resolves to `/home/joel/src/yaks-farm/.yaks` from the primary, from a subdirectory of it, from a Delta clone and from the farm worktree; a shear written from the Delta clone landed in the farm with both code checkouts clean; `yaks commit` committed on `farm`; the coordinator-side landing (`git fetch local farm && git merge local/farm`, push `main`) brought it to main as a fast-forward; `yaks sheds` from the primary lists the clone and the farm worktree as `shares this farm`; `preflight: ok` in the farm.

Hazard found: the `yaks` on this machine's PATH is the pnpm 0.0.20 install, which ignores `live_branch:` and writes the checkout's stale `.yaks/` (`yaks path yaks-6a34` there printed `~/src/yaks/.yaks/...`), and `~/go/bin` is not ahead of it on PATH here. Until a release carries this, every human shell and every worker must use `~/go/bin/yaks` (the brief names the binary the coordinator ran). A write by an old binary shows up as a dirty `.yaks/` in a code checkout: `yaks status` there, then move the edit into the live farm by hand.

---
▸ 2026-10-18T05:54:58Z [delta-farm]

Round 1 spawned (delta-farm, 2026-10-18 ~05:55Z), base 1ffd6d8, worker profile default model (not overridden):
- `shear-1` -> yaks-27c6 (shear-with-comment), dir wzmvex2kk3z0
- `attach-view-1` -> yaks-699b (in-TUI text attachment viewer/editor), dir f78d99v69xkt
- `backrefs-1` -> yaks-7c76 (incoming refs: lib, `refs --incoming`, TUI picker), dir ykkpe8g4qv9a

Not spawned from Joel's starred set: yaks-1bbd (search context) is folded into yaks-953f, which has an open design ask (4 questions, needs: human) gating phases 2-3; yaks-8d6a (label completion) depends on yaks-b2ed and on predefined labels (yaks-f04d), neither built.

First observation: 20 s after spawn `yaks sheds` lists all three clones by thread-title slug, each `shares this farm`; every brief printed the live-farm finish with no `YAKS_DIR`.

---
▸ 2026-10-18T06:07:45Z [delta-farm]

Round 1, first landing (delta-farm): `shear-1` (yaks-27c6) landed by SHA with land.sh, clean (base an ancestor, computed tree equal); my review and real run on a scratch farm agreed with the worker's report. Farm commit + `merge local/farm` + push to main worked as designed: the farm commit swept up 15 of Joel's TUI edits from tonight and the worker's attachment with no extra step, main fast-forwarded, the primary stayed clean.

Trial finding #1 (fixed, yaks-5924): `yaks preflight --push-main` asked the FARM's checkout for the `local` remote; under a live farm that is the farm worktree, which has none, so the landing check failed though the push was fine. Now asked of the invoking checkout. Exactly the kind of "the farm root is not the code checkout" assumption to look for elsewhere (status/commit/preflight's git-state are correct to use the farm root).

---
▸ 2026-10-18T06:15:40Z [delta-farm]

Round 1 result (delta-farm): 3 workers, 3 clean landings (all land.sh merges by SHA, base an ancestor, computed tree equal, no conflicts despite all three touching handlers.rs / help_content / tests.rs / docs/tui.md), 854 tests green on main, one trial bug found and fixed (yaks-5924).

What the live farm changed, observed:
- Zero `.yaks/` content in any worker commit (all three `git show --stat` clean); no stale-copy writes (`git status` of ~/src/yaks clean after every landing; every clone clean).
- Claims, progress notes, Decision notes and attachments were visible in the live farm the moment they were written; the coordinator never had to read another checkout (`yaks changes`, `inbox --sheds`, `show --sheds`, `attachment` were not used once).
- Landing the farm is one step per landing (`yaks commit`, merge `local/farm`) and absorbed Joel's 15 concurrent TUI edits with no coordination.
- No stale shaving copy / reverted yak files in any landing (the O22/O43/O48 failure shape needs yak files in the worker's tree; there were none).
- No asks this round, so the live ask/answer path is still untested under the live farm.

Friction: `preflight --push-main` (fixed). Nothing else.
