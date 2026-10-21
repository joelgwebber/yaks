---
id: yaks-a260
title: 'Bootstrap the farm worktree on first use: a repo with a yaks branch (local or origin/) but no worktree gets one at farm_home'
status: shorn
type: feature
priority: 1
created: '2026-10-18T23:24:43Z'
updated: '2026-10-19T03:20:23Z'
parent: yaks-73f4
labels:
- cli
- git
---

Joel (2026-10-18): after `yaks migrate --to-branch`, another machine or a teammate has to run `git fetch origin yaks && git worktree add <path> yaks` by hand. Instead, when discovery (`store::farm_branch_worktree`) finds the farm branch with no worktree, create it at `store::farm_home(repo)` (a local branch tracking `origin/<b>` when only the remote exists), print one stderr line saying where, and continue. Only from a repo's own checkout (a Delta/--shared clone points at its main checkout, as migrate does); never fetches unless the branch is missing locally AND remotely (then the old error). Non-destructive, so no prompt; `YAKS_NO_BOOTSTRAP=1` (or a config/flag) for CI that should fail instead. Evidence: a fresh clone of a migrated repo: first `yaks list` creates the worktree at farm_home and lists; a second run is silent.

---
▸ 2026-10-19T01:53:39Z [delta:farm-bootstrap]

moved: hairy -> shaving

---
▸ 2026-10-19T01:53:39Z [delta:farm-bootstrap]

Plan (claim note): bootstrap lives at the command entry, not in `store::discover_with` (which read-only callers such as `yaks sheds` call for OTHER checkouts and must never create worktrees). `store::discover` (cwd + $YAKS_DIR, what `Farm::open` calls) catches a typed `NoFarmWorktree` error from `farm_branch_worktree`, creates the worktree at `store::farm_home` via a new `branchfarm::bootstrap`, prints one stderr line, retries. Also treats a registered-but-deleted worktree dir as the same case (prune, re-add). A Delta/--shared clone is refused with an error naming the main checkout. `YAKS_NO_BOOTSTRAP` set = old error. Evidence: scratch fresh-clone tests + a real fresh clone of this repo with HOME/XDG_DATA_HOME in a temp dir.

---
▸ 2026-10-19T01:59:46Z [delta:farm-bootstrap]

Built: `store::discover` (cwd + $YAKS_DIR, what `Farm::open` calls) now goes through `discover_bootstrapping`; `farm_branch_worktree` returns a typed `MissingFarmWorktree` instead of a bail, and `branchfarm::bootstrap` runs `git worktree prune` then `git worktree add` at `farm_home` (`--track -b <b> origin/<b>` when only the remote branch exists). `discover_with` stays pure/read-only (sheds walk other checkouts through it). Delta/--shared clone: refused, names the main checkout. A registered-but-deleted worktree dir is pruned and recreated. `YAKS_NO_BOOTSTRAP` (non-empty) restores the old error.

Evidence: 3 new store tests (fresh clone: silent read-only walk, one stderr line, tracks origin/yaks, second run silent; deleted worktree; clone/occupied home/no-branch refusals). Real run: fresh `git clone` of this repo with XDG_DATA_HOME in a temp dir, first `yaks path` printed the one-line notice and the path `.../farms/yaks-51df7f1d/.yaks`, second silent, `git worktree list` shows the yaks worktree, `status -sb` = `yaks...origin/yaks`. Gate: 746 unit tests pass; tests/attachment_sheds.rs fails identically on the stashed baseline (macOS /private/var path), unrelated.

---
▸ 2026-10-19T03:20:23Z [delta:farm-bootstrap]

moved: shaving -> shorn
