---
id: yaks-7bfd
title: 'A committed in-tree .yaks/ in a repo that has a farm branch is a stale copy: redirect to the farm and warn'
status: shorn
type: bug
priority: 1
created: '2026-10-18T23:24:43Z'
updated: '2026-10-21T22:49:46Z'
parent: yaks-73f4
labels:
- cli
- git
---

Found answering Joel (2026-10-18): after `yaks migrate --to-branch` on main, an OLD checkout (an open Delta clone or git worktree forked before the migration) still has the committed `.yaks/`. It has no `live_branch:` marker (that was only this repo's trial), so discovery (`store::discover_with` -> `redirect_live`) uses it as the farm: that checkout's writes go to a dead copy and its reads miss everything since the migration. Rule: when the walk finds a `.yaks/` directory and the repository has a farm-branch worktree that is not this directory, the in-tree copy is stale: use the farm branch's worktree and print a one-line warning naming the stale copy (once per command). Also: `yaks preflight` / `land.sh` should flag a worker commit that touches `.yaks/` once main has no `.yaks/` (it would conflict modify/delete). Evidence: migrate a temp repo after creating a git worktree from the pre-migration commit; from that worktree `yaks path` resolves to the farm branch and warns; before the fix it printed the stale copy.

---
▸ 2026-10-21T22:10:10Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T22:49:46Z [delta:herd-dirs]

Done by the yakherd: d138472. A top-level committed in-tree `.yaks/` in a repository whose farm branch has a worktree elsewhere resolves to that worktree, with one stderr warning naming the stale copy; `YAKS_DIR`, the farm worktree itself and nested farms (fixtures) are untouched. The preflight/land half (a worker commit touching `.yaks/` under a branch farm) is in `yaks land` (exit 7, yaks-e5fb).

Evidence: unit test `a_committed_in_tree_copy_beside_a_farm_branch_worktree_is_stale` (old worktree + its .yaks/ subdir redirect; farm worktree, YAKS_DIR and a repo with no farm-branch worktree are not). Real run: a worktree of this repo at 98d3f48~1 (pre-migration, `live_branch:` line removed so only the new rule applies) -> `yaks path` printed the warning and resolved to ~/.local/share/yaks/farms/yaks-539d3edb/.yaks. Full gate green (20 suites).

---
▸ 2026-10-21T22:49:46Z [delta:herd-dirs]

moved: shaving -> shorn
