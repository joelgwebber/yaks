---
id: yaks-87a3
title: 'Migrate this repo: the farm moves to an orphan yaks branch; .yaks/ leaves main; live_branch, sync_live and --land retire'
status: shorn
type: task
priority: 1
created: '2026-10-18T21:15:00Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-73f4
labels:
- git
- cli
- skills
needs: agent
---

The build-and-cutover half of yaks-73f4's design note. One arc, done by the yakherd while nothing is in flight:
1. Code: delete `Config::live_branch` handling (`redirect_live`, the config read), `commit::sync_live`, `commit::land` and `yaks commit --land`; `store::farm_branch` keys only on the worktree's HEAD; `yaks commit` in a farm-branch worktree is the whole farm story. `land.sh` stays (code only). Tests rewritten on orphan farms.
2. Docs/skills/help: the live-farm wording becomes the farm-branch model everywhere (docs/cli.md discovery, README, AGENTS.md, the yaks skill's two-workflows section, yaks-working, yaks-coordinating + team skill "Live farm", Delta landing, `yaks commit --help`).
3. Cutover in ~/src/yaks: commit the farm; create branch `yaks` from `farm` whose next commit removes everything but `.yaks/` (history of every past .yaks change kept); `git branch -m`/worktree moved so `~/src/yaks-farm` checks out `yaks` with no sparse checkout; drop `live_branch:` from its config; one commit on main removing `.yaks/`; delete branch `farm`. Pushing `yaks` to origin waits for Joel.
Evidence: full test suite; on this machine `yaks path` resolves the farm from ~/src/yaks, a subdir, this Delta clone and the farm worktree; a worker-less round trip (note, commit, push main) leaves main with no `.yaks/` and farm history intact (`git log yaks -- .yaks/yaks-6a34.md` reaches its creation).

---
▸ 2026-10-18T21:15:04Z [delta-farm]

moved: hairy -> shaving

---
▸ 2026-10-18T21:25:20Z [delta-farm]

Cutover done (delta-farm, 2026-10-18).

- Code (04b0dd5): `live_branch:` is now only a stale-copy marker (`redirect_live` sends such a `.yaks/` to the farm branch's worktree), `store::farm_branch` keys on the worktree's HEAD, `commit::sync_live` / `commit::land` / `yaks commit --land` deleted, docs + skills + help say "farm on its own branch".
- Farm: branch `yaks` (from `farm`, whose history it keeps) at ~/src/yaks-farm; its tip commit drops everything but `.yaks/`, so the tree is the farm alone; sparse checkout disabled; `live_branch:` dropped from its config. Branch `farm` still exists, unused (delete at leisure).
- main (98d3f48): one commit removing `.yaks/` (628 files); pushed to Joel's checkout. A pull that removes a tracked farm leaves the ignored `.yaks/.lock` behind: discovery now walks past a `.yaks/` holding only `.lock`/`.gitignore` (fix + test `a_leftover_non_farm_dot_yaks_does_not_hide_the_farm_branch`, found on the real checkout).

Evidence: `yaks path` from ~/src/yaks (with the leftover), ~/src/yaks/src/tui, this Delta clone and ~/src/yaks-farm: all `/home/joel/src/yaks-farm/.yaks`. `git log yaks -- .yaks/yaks-6a34.md`: 7 commits back to its creation; `yaks commits yaks-ee0a` (an old yak) still finds its history. Full `cargo test --release` 867 passed; land.sh selftest all passed.

---
▸ 2026-10-18T21:25:20Z [delta-farm]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-87a3` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-87a3 --drop-request`, which removes this request note, so no new note is left behind.
