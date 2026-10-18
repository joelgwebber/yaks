---
id: yaks-bd7e
title: 'yaks migrate --to-branch: move an in-tree committed farm onto an orphan yaks branch (the yaks-87a3 cutover as one command)'
status: hairy
type: feature
priority: 1
created: '2026-10-18T21:39:13Z'
updated: '2026-10-18T21:39:13Z'
parent: yaks-73f4
labels:
- cli
- git
---

What yaks-87a3 did by hand for this repo, as a command for every other team repo: `yaks migrate --to-branch [--branch yaks] [--worktree <path>] [--dry-run]`, run from a checkout whose committed `.yaks/` is the farm. Steps: refuse unless the tree is clean and no merge is pending; create the branch from HEAD in a new worktree (default `../<repo>-yaks`, later the XDG home, yaks-0ef9); in it, one commit that keeps only `.yaks/` (history of every past farm change kept, so `yaks commits` and `git log --follow` work); in the code checkout, one commit removing `.yaks/` and the leftover dir; print what to push (`git push -u origin yaks`, then main) and the one-line setup for other machines/teammates (`git worktree add <path> yaks`). Never pushes. Idempotent: a repo already on a branch says so. Evidence: a temp repo with a committed farm and history -> after: main has no .yaks/, the branch tree is only .yaks/, `yaks path` from main/subdir/a --shared clone resolves the worktree, `yaks commits <old id>` finds its history; dry run changes nothing.
