---
id: yaks-87a3
title: 'Migrate this repo: the farm moves to an orphan yaks branch; .yaks/ leaves main; live_branch, sync_live and --land retire'
status: shaving
type: task
priority: 1
created: '2026-10-18T21:15:00Z'
updated: '2026-10-18T21:15:04Z'
parent: yaks-73f4
labels:
- git
- cli
- skills
---

The build-and-cutover half of yaks-73f4's design note. One arc, done by the yakherd while nothing is in flight:
1. Code: delete `Config::live_branch` handling (`redirect_live`, the config read), `commit::sync_live`, `commit::land` and `yaks commit --land`; `store::farm_branch` keys only on the worktree's HEAD; `yaks commit` in a farm-branch worktree is the whole farm story. `land.sh` stays (code only). Tests rewritten on orphan farms.
2. Docs/skills/help: the live-farm wording becomes the farm-branch model everywhere (docs/cli.md discovery, README, AGENTS.md, the yaks skill's two-workflows section, yaks-working, yaks-coordinating + team skill "Live farm", Delta landing, `yaks commit --help`).
3. Cutover in ~/src/yaks: commit the farm; create branch `yaks` from `farm` whose next commit removes everything but `.yaks/` (history of every past .yaks change kept); `git branch -m`/worktree moved so `~/src/yaks-farm` checks out `yaks` with no sparse checkout; drop `live_branch:` from its config; one commit on main removing `.yaks/`; delete branch `farm`. Pushing `yaks` to origin waits for Joel.
Evidence: full test suite; on this machine `yaks path` resolves the farm from ~/src/yaks, a subdir, this Delta clone and the farm worktree; a worker-less round trip (note, commit, push main) leaves main with no `.yaks/` and farm history intact (`git log yaks -- .yaks/yaks-6a34.md` reaches its creation).

---
▸ 2026-10-18T21:15:04Z [delta-farm]

moved: hairy -> shaving
