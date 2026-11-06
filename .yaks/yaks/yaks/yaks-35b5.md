---
id: yaks-35b5
title: One farm worktree per repository on a machine, shared by every checkout of it
status: hairy
type: feature
priority: 2
created: '2026-11-06T22:00:49Z'
updated: '2026-11-06T22:00:49Z'
tags:
- cli
- delta
---

From yaks-8a92 (Joel, 2026-11-06: "fix this, and if possible merge the checkouts, so I don't have to do a full sync cycle on a single machine").

A farm worktree belongs to one host git dir (`store::farm_home` hashes it; discovery asks that host's `git worktree list`). A repository Joel works on both in his own checkout and through Delta (whose managed repository is a separate bare repo) therefore gets two farm worktrees on one machine (`schism-38838c3f`, `repository-c0e58d55`), whose writes meet only through `yaks sync` through `origin`.

## Fix

- Key the farm home by the repository's normalized `origin` URL when it has one (`github.com/rocketsurgery-games/schism`, whatever the scheme or `.git` suffix), named after its last segment (also fixes yaks-9a13's `repository-…`). No origin: the git dir, as now.
- Discovery: a host with no registered farm worktree adopts the one at its farm home when that worktree is on the farm branch and its repository has the same origin, instead of bootstrapping a second.
- Existing duplicates: a recipe to merge them (move one to the shared home, remove the other), and do it for schism.
