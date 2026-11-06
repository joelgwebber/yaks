---
id: yaks-9a13
title: Farm worktrees of Delta-managed repositories are named repository-…
status: shaving
type: bug
priority: 3
created: '2026-11-06T21:37:27Z'
updated: '2026-11-06T22:23:35Z'
tags:
- cli
- delta
---

From yaks-3f35, item 9. `store::farm_home` names a farm worktree `<repo>-<hash>`, `<repo>` being the host's main checkout's directory name or, for a bare host, the git dir's name. A Delta-managed repository is a bare `.../managed-repositories/<uuid>/repository.git`, so every Delta-managed project's farm is `repository-<hash>` (schism's is `repository-c0e58d55`), and `ls ~/.local/share/yaks/farms` can't tell them apart.

## Fix

For a bare host whose directory name says nothing (`repository`), take the name from its `origin` URL (`rocketsurgery-games/schism` → `schism`). Existing worktrees must keep working: use the old directory when it exists, or move it once and leave nothing behind (a Delta thread's discovery must still find it). See yaks-8a92 for the related question of one worktree per repository rather than per host.

## Evidence

A test on `farm_home_with` with a bare `repository.git` host that has an `origin`.

---
▸ 2026-11-06T22:00:50Z [schism-feedback]
kind: move
from: hairy
to: shaving
thread: ksQQNtlWqzuUT5COVXyFQVm8cZLOAEgKKZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T22:09:25Z [schism-feedback]

Fixed together with yaks-35b5 on branch `shared-farm-home` (fc891687): a repository with an `origin` is named after the URL's last segment, so Delta's `repository.git` gives `schism-<hash>`. Existing worktrees keep working because a registered worktree wins; see the evidence on yaks-35b5.

---
▸ 2026-11-06T22:23:35Z [schism-feedback]

Landed in e52342db with yaks-35b5. schism's farm is now `schism-6afd61e7`, not `repository-c0e58d55`.
