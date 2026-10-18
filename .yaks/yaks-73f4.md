---
id: yaks-73f4
title: 'Quiet mode: yaks on a perpetual branch, never merged to main'
status: hairy
type: feature
priority: 2
created: '2026-10-18T15:54:02Z'
updated: '2026-10-18T15:54:02Z'
labels:
- git
- design
---

Joel (2026-10-18), from the live-farm trial (yaks-6a34): with the farm on its own branch, option (b) is a real mode: the farm lives ONLY on a perpetual branch (`yaks`/`farm`) pushed to origin, never merged into main. Backed up, shareable with teammates who opt in (fetch the branch, add the worktree), invisible to everyone else: no `.yaks/` on main, no yak churn in PRs or `git log`. Wanted for some team repos.

Design questions:
- Discovery with no `.yaks/` on main: `live_branch:` lives in a committed config today. Quiet mode needs a non-committed signal: the presence of a worktree on the branch, a per-user registry (see the client-metadata yak), or `git config yaks.liveBranch` in the repo.
- Setup for a teammate: `yaks init --mode quiet` = fetch `origin/<branch>`, worktree it (sparse), done.
- Ids in commit messages: allowed (teammates resolve them from the branch) or kept out like private mode? Probably a per-repo choice.
- Provenance: `yaks commits <id>` joins on ids in code commits only; the farm's history is on its own branch.
- Relation to private mode (untracked) and to the nested-repo pattern, which quiet mode largely supersedes.
