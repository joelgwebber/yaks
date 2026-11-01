---
id: yaks-b46b
title: 'Farm bootstrap on a Delta cloud runner: create the worktree in the bare host instead of refusing'
status: hairy
type: bug
priority: 2
created: '2026-11-01T01:10:22Z'
updated: '2026-11-01T01:10:22Z'
labels:
- delta
- cli
---

On a Delta cloud runner the repo is a Delta-managed BARE host plus clones that borrow its objects (alternates). There, `branchfarm::bootstrap` refuses to create the farm worktree ("this checkout borrows another repository's objects (a Delta clone)… Run any yaks command in its main checkout"). But there is no main checkout: the host is bare, and the error's only fix is `YAKS_DIR`.

Found 2026-11-01 watching the memcraft runner: its `scripts/runner-setup.sh` works around this by hand, after every runner wipe:

```sh
git -C "$bare" fetch -q origin '+refs/heads/yaks:refs/remotes/origin/yaks'
git -C "$bare" branch -q yaks origin/yaks
git -C "$bare" worktree add -q "$farm" yaks
```

That is exactly what yaks could do itself. A worktree added in the HOST repo is listed by the host's `git worktree list`, which `gitrepo::worktree_of_branch` already reads, so every clone of that host finds it. The refusal's reason (a worktree made from a clone is invisible to the others) doesn't apply when it is made in the host.

## Fix shape

- **A bare host reached through alternates:** create the farm worktree in the host (fetch `origin/yaks` into it if needed, branch, `worktree add` under `farm_home`), not refuse.
- **A non-bare host (a human's checkout):** keep today's behavior, which points at that checkout.
- **Test:** a bare host plus a clone with alternates; the first yaks command in the clone creates the worktree, and a second clone finds it.
- **Docs and skills:** the coordinating skill's Delta section, and the error text.
