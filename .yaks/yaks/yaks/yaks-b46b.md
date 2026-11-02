---
id: yaks-b46b
title: 'Farm bootstrap on a Delta cloud runner: create the worktree in the bare host instead of refusing'
status: shorn
type: bug
priority: 2
created: '2026-11-01T01:10:22Z'
updated: '2026-11-01T16:17:57Z'
tags:
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

---
▸ 2026-11-01T16:17:55Z [yakherd]

kind: move
from: hairy
to: shaving
---

moved: hairy -> shaving

---
▸ 2026-11-01T16:17:56Z [yakherd]

Fixed in `5b00c04`. When discovery runs in a clone that borrows a **bare** host's objects, bootstrap now adds the farm worktree in the host (`git -C <host> worktree add`). Every clone of that host finds it through the host's `git worktree list`.

- If the host lacks the farm branch, it is created from the host's `origin/<branch>`, else fetched from the host's `origin` (the one fetch bootstrap makes). The branch is set to track `origin` so `yaks sync` works.
- Discovery also notices a farm ref in the bare host when the clone has none of its own.
- A clone whose host is a checkout still refuses and names that checkout, as before.

## Evidence

`tests/bare_host.rs` builds the runner's layout: an upstream with `main` plus a farm, a bare managed host with `origin` and only `main`, and two `--shared` clones with `origin` set to the upstream.

- The first `yaks show` in clone a creates the worktree in the host.
- Clone b gets the same `yaks path`, and sees a note written in a.
- `yaks sync` from a pushes the note to upstream.
- Before the fix, the same test fails with the runner's exact error ("this checkout borrows another repository's objects (a Delta clone) … no checkout there: set YAKS_DIR to the farm").
- `cargo test --workspace` is green, both normally and with no Delta env and no git identity (CI-like).

Once released, memcraft's `scripts/runner-setup.sh` no longer needs its farm-worktree block. That's their repo to trim.

---
▸ 2026-11-01T16:17:56Z [yakherd]

kind: landed
commit: 5b00c04453f42139a6aec1d2b87ca153c37d719a
branch: main
---

landed: 5b00c04

---
▸ 2026-11-01T16:17:57Z [yakherd]

kind: move
from: shaving
to: shorn
---

moved: shaving -> shorn
