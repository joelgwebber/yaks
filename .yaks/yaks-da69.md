---
id: yaks-da69
title: 'live_branch config: discovery resolves the live farm worktree; bare yaks path prints the farm root; brief Live mode'
status: shorn
type: feature
priority: 1
created: '2026-10-18T05:15:36Z'
updated: '2026-10-18T05:24:22Z'
parent: yaks-6a34
labels:
- cli
- sheds
---

The one tweak that keeps the trial (yaks-6a34) on the rails: make "which farm" a git fact instead of an env var.

- `config.yaml` gains `live_branch: <branch>` (committed). When discovery (the walk or a pointer; never `$YAKS_DIR`, which stays the explicit override) finds a farm whose config names a live branch, it resolves to `<worktree where that branch is checked out>/.yaks`: `git worktree list --porcelain` of the HOST repository (`sheds::host`: this checkout's own repo, or the one its `alternates` names, so a Delta clone finds the human's worktrees). The farm worktree itself resolves to itself. No such worktree is an error naming the fix (`git worktree add -b <branch> <path> main` + `sparse-checkout set .yaks`, or `YAKS_DIR`): one rule, no silent fallback to the stale local copy.
- `yaks path` with no ids and no filters prints the farm root (the yaks-df61 O47 gap): the "which farm am I on" reflex for the trial.
- `yaks brief`: a third `Mode::Live` for a farm whose config has `live_branch`: never stage anything under `.yaks/` (the checkout's copy is a stale mirror), ids in messages are fine, the yakherd commits the farm; `YAKS_DIR` is not put on commands (discovery redirects) unless `--yaks-dir` is given.
- Docs/help/skills in the same change: docs/cli.md, README, AGENTS.md discovery bullet, the yaks skill (two workflows + discovery), yaks-coordinating-team (a live-farm section), clap help for `path`.

Evidence: tests that fail first (a temp repo with a `farm` worktree: discovery from the primary, from a plain worktree and from a clone-with-alternates all resolve to the farm worktree; the farm worktree resolves to itself; a missing worktree errors naming the branch; `YAKS_DIR` still wins; `path` with no args). Then the real machine: `~/go/bin/yaks path` from `~/src/yaks` and from this Delta clone both print `~/src/yaks-farm/.yaks`.

---
▸ 2026-10-18T05:15:36Z [delta-farm]

moved: hairy -> shaving

---
▸ 2026-10-18T05:23:39Z [delta-farm]

Done (delta-farm). `live_branch:` in `config.yaml` (`store::Config::live_branch`); `store::discover_with` passes the walk's find through `redirect_live`, which asks `sheds::worktree_of_branch` (host repo via `sheds::host`, then `git worktree list --porcelain`, matched on `branch refs/heads/<b>`) and resolves to `<worktree>/.yaks`; the live worktree resolves to itself; no worktree or no `.yaks/` there is an error naming the fix (`git worktree add -b <b> <path> main`, `sparse-checkout set .yaks`, or `YAKS_DIR`). `yaks path` with no ids and no filters prints the farm root. `brief`: `Mode::Live { branch }` from the config (never stage `.yaks/`, ids fine, the yakherd commits; no `YAKS_DIR` unless `--yaks-dir`).

Decision: only the walk's find is redirected. `$YAKS_DIR` stays the explicit override (the escape for a clone whose host is Delta's bare managed repo) and a pointer file already names one out-of-tree farm. Rejected: redirecting `YAKS_DIR` too (then nothing could reach a stale copy on purpose) and a silent fallback to the stale copy when the worktree is missing (that would quietly reinstate the per-shed-copy model as a second mode; yaks-b4dc made the same call for the walk).

Decision: the live-worktree lookup lives in `sheds.rs` beside `worktree_entries` (same porcelain, same host logic) and `store` calls it; store -> sheds is a reversed dependency, accepted over duplicating `host()`'s alternates handling.

Evidence: `store::discover_tests::{live_branch_redirects_every_checkout_to_the_farm_worktree, live_branch_without_its_worktree_is_an_error_naming_the_fix, yaks_dir_overrides_a_live_branch_and_a_pointer_is_not_redirected, config_reads_live_branch}` (real git: primary, a plain worktree, a `--shared` clone standing in for a Delta clone, the farm worktree itself), `brief::tests::a_live_brief_never_stages_the_stale_copy_and_keeps_ids_in_messages`, `tests::path_with_nothing_prints_the_farm_root_and_refuses_ids_plus_filter`. Full `cargo test --release`: 712 + all integration suites green. Docs/help/skills: docs/cli.md (discovery rule 4, `path` row), README, AGENTS.md (discovery bullet, task-tracking rule 2), clap help for `path`, the yaks skill (hard rule 2, a "Team, live" section, `path` row), yaks-working (live finish), yaks-coordinating (mode routing), yaks-coordinating-team ("Live farm" section), yaks-coordinating-delta (brief `--yaks-dir`). Real-machine check follows once the farm worktree exists on this machine.

---
▸ 2026-10-18T05:24:22Z [delta-farm]

Real machine (delta-farm): `~/go/bin/yaks path` prints `/home/joel/src/yaks-farm/.yaks` from ~/src/yaks, from ~/src/yaks/src/tui, from this Delta clone (host via alternates) and from the farm worktree itself; this note and the shear were written from the Delta clone into the live farm and are the first live writes of the trial.

---
▸ 2026-10-18T05:24:22Z [delta-farm]

moved: shaving -> shorn
