---
id: yaks-35b5
title: One farm worktree per repository on a machine, shared by every checkout of it
status: shaving
type: feature
priority: 2
created: '2026-11-06T22:00:49Z'
updated: '2026-11-06T22:23:34Z'
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

---
▸ 2026-11-06T22:00:55Z [schism-feedback]
kind: move
from: hairy
to: shaving
thread: ksQQNtlWqzuUT5COVXyFQVm8cZLOAEgKKZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T22:09:25Z [schism-feedback]

Built on branch `shared-farm-home` (fc891687), not landed.

- `store::farm_home` names the home by the normalized `origin` URL (`normalize_remote_url`: scheme, `user@`, port, `.git` and trailing `/` dropped, host lowercased), after its last segment; no origin keeps the git-dir key. [fn farm_home_with](code:src/store.rs@fc891687#L225)
- `store::farm_worktree`: the host's registered worktree, else `shared_farm_worktree` adopts the worktree at the farm home when it is on the farm branch and its repository has the same origin identity. Discovery and `yaks init` both use it.
- Docs: `docs/cli.md` "One worktree per repository" with the merge recipe for existing pairs; README, the `yaks` skill, `init --help`.

## Evidence

- Tests: `remote_urls_normalize_to_host_and_path`, `git_dirs_with_one_origin_share_a_farm_home`, `a_farm_worktree_at_the_shared_home_is_adopted_by_the_same_repository_only`; full `cargo test` green (991 + integration), clippy adds nothing in the touched code.
- End to end with the release build and a scratch `XDG_DATA_HOME`: a checkout of schism (origin `https://github.com/rocketsurgery-games/schism.git`), a bare `repository.git` (origin without `.git`) and a `--shared` clone of it (Delta's shape). The clone bootstrapped `schism-6afd61e7`, the checkout resolved to the same `.yaks`, and a yak created through the clone was listed from the checkout with no sync.

Still to do once installed: merge Joel's two schism worktrees (`schism-38838c3f`, `repository-c0e58d55`) per the recipe. Doing it before the new binary is installed would make an old binary re-bootstrap `repository-…`.

---
▸ 2026-11-06T22:23:34Z [schism-feedback]

Landed on `main` as e52342db (rebased onto c5eba6a6), with yaks-8a92's 34ded4b5 on top; pushed, and installed with `cargo install --path .`.

Merged Joel's two schism farm worktrees, which were clean and both at b6f8cc3 = `origin/yaks`:

- `git worktree remove` on `repository-c0e58d55` (Delta's managed repo) and `schism-38838c3f` (`~/src/rs/schism`)
- `yaks -C ~/src/rs/schism path` bootstrapped `schism-6afd61e7`
- `yaks path` from the schism Delta clone and from `~/src/rs/schism` both print `/Users/joel/.local/share/yaks/farms/schism-6afd61e7/.yaks`; `yaks status` is clean and up to date with `origin/yaks`.

The `yaks tui` running in `~/src/rs/schism` (pid 84648) still has the removed directory open and needs a restart.
