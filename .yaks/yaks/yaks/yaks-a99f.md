---
id: yaks-a99f
title: 'Branch farms only: drop in-tree .yaks/ and pointer files; yaks.farm git config points a repo at another repo''s farm; docs to examples'
status: shorn
type: task
priority: 2
created: '2026-10-27T22:29:18Z'
updated: '2026-10-31T15:05:04Z'
labels:
- cli
- docs
- git
---

Joel (2026-10-27): the farm-on-a-branch model is the only model. Retire everything that exists only for in-tree farms, and replace the `.yaks` pointer file with git config. No users outside Joel yet, so no deprecation period.

## Decisions (Joel, 2026-10-27)

- **In-tree `.yaks/` farms are removed** (not deprecated). Discovery stops looking for a `.yaks` entry in the walk. `yaks init` creates only branch farms. `--mode team`, `--mode private` (in-tree, excluded) and `--mode pointer` go.
- **Pointer files are removed.** A `.yaks` file is no longer read, by the walk or by `YAKS_DIR`.
- **New pointer: git config.** `git config yaks.farm <repo>` (plus optional `yaks.herd <prefix>`) makes a repo use another repo's farm branch. It lives in `.git/config`, so Delta clones see it through the host lookup `configured_farm_branch` already does, and workers need no `YAKS_DIR` (Joel: "Anything we can do to simplify that process is a Good Thing").
- **Docs rewrite:** README, `docs/`, the skills, `--help` and `brief` get the new model, with exposition cut hard and brief examples preferred over prose (Joel: "go long on brief examples over words and abstractions"). yaks-43dc folds in.

## Why

The farm branch (yaks-73f4 under the umbrella yaks-6a34) replaced the committed `.yaks/`. yaks-b048's second pass (`f615cfb`, "per Joel: drop the in-tree farm from the skills") cut team mode from the skills only. Code, `yaks init` defaults and most docs still created and described in-tree farms. Plain `yaks init` still made a committed team farm, so the first command a newcomer ran set up the retired model. The pointer file can't name a farm branch (only a directory). It is untracked, so no clone or worktree carries it, which is the reason private/pointer briefs needed `YAKS_DIR`. And `git clean -fdx` deletes it. yaks-73f4's design had already deferred "named farms per team via a pointer `branch:` key"; this is that item, done through git config instead.

## Scope (to confirm as built)

- `store::discover_with`: no `.yaks` probe in the walk. Order becomes `YAKS_DIR` (a farm dir only) > `yaks.farm` > own farm branch. Remove `follow_pointer`/`parse_pointer`, `redirect_live`/`live_branch`, `stale_in_tree` (yaks-7bfd), `only_leftovers` if unused.
- `init.rs`: one mode (branch); drop `Mode`, `--mode`, `--path`; keep `--branch`, `--herd`, `--skills`.
- `brief`/`preflight`/`commit`: drop the Team mode and the private-in-tree/pointer paths.
- `migrate --to-branch` (yaks-bd7e): retire with in-tree farms (Joel's repos are migrated); it stays in git history.
- Tests: in-tree fixtures move to `YAKS_DIR` or to a scratch repo with a farm branch.
- Docs per the docs-and-help rule in AGENTS.md.

## Archaeology

yaks-6a34 (one farm per machine, umbrella), yaks-73f4 (farm is a branch; the deferred pointer `branch:` key), yaks-b048 + `f615cfb` (skills dropped the in-tree farm), yaks-7bfd (stale-copy redirect), yaks-bd7e (migrate --to-branch), yaks-87a3 (this repo's migration), yaks-814a (init --mode), yaks-b1e9 (init --mode branch), yaks-43dc (pointer/private rule wording).

---
▸ 2026-10-27T22:33:58Z [delta:Check for `.yaks` file repo redirect sup]

moved: hairy -> shaving

---
▸ 2026-10-27T22:33:58Z [delta:Check for `.yaks` file repo redirect sup]

Decision (Joel, 2026-10-27), on the three open points:

- `yaks migrate --to-branch` is deleted.
- `git config yaks.farm <path>` names any checkout of the other repo. Yaks resolves that repo's farm branch from it and bootstraps the worktree as usual. A remote URL can come later if someone needs it.
- `yaks init --farm <repo> [--herd <h>]` sets `yaks.farm` and `yaks.herd` and installs the skills.

---
▸ 2026-10-27T23:14:10Z [delta:Check for `.yaks` file repo redirect sup]

Code pass 1 (discovery, init, brief, preflight, tests) builds and the full suite is green (735 unit + 15 integration files).

- `store::discover_with`: `YAKS_DIR` (a farm dir only) > `git config yaks.farm` > own farm branch. No walk, no `.yaks` probe. `follow_pointer`/`parse_pointer`, `redirect_live`/`live_branch`, `stale_in_tree` (yaks-7bfd) and `only_leftovers` are deleted. `yaks.herd` replaces the pointer's `herd:`; both keys are read from the host repo, so a Delta clone sees them.
- `init`: one mode. `yaks init [--branch] [--herd] [--skills]`, or `--farm <checkout>`. `--mode`/`--path` and `migrate --to-branch` (yaks-bd7e) are gone. Run inside a hand-made orphan farm worktree, `init` still fills it in.
- `brief`: `Mode::{Branch, Private}`. Private now means `yaks private` (or a `YAKS_DIR` farm). `YAKS_DIR` is printed only for a non-branch farm.
- `preflight`: the git-state check is conflict-only on a branch farm, and skipped for a `YAKS_DIR` farm.
- Tests: the in-tree fixtures moved to `tests/common` (a git repo, `init`, data dir beside it) or to `YAKS_DIR`.

Next: mop up the team/private wording left in `status`/`commit`/`changes`/`migrate`/`farmsync`, then docs.

[farm-sharing-e2e](yaks-a99f/farm-sharing-e2e.txt)

---
▸ 2026-10-27T23:23:34Z [delta:Check for `.yaks` file repo redirect sup]

Done in `3bb5937` (committed in this thread's worktree, not landed): 39 files, +1337 / −3135.

- **Discovery**: `YAKS_DIR` (a farm dir) > `git config yaks.farm` > the repo's own farm branch. Both keys are read through a Delta clone's host. Nothing in the tree is read; a leftover `.yaks` dir or file is ignored.
- **`init`**: `yaks init [--branch] [--herd] [--skills]`, or `--farm <checkout> [--herd]`. `--mode`, `--path` and `migrate --to-branch` are deleted.
- **`brief`** Branch/Private, **`preflight`** conflict-only, **`status --json`** `private` renamed `untracked`, plus team-mode wording cleaned up in `commit`/`status`/`changes`/`migrate`/`sync`/`farmsync`/`source`.
- **Docs**: README, `docs/README.md`, `docs/cli.md`, `docs/tui.md`, `docs/skills.md`, AGENTS.md, the four skills and `--help` are all rewritten, examples first. One commit-id rule keyed on `yaks private` (the wording half of yaks-43dc; its `--as` bug is split back out to top level).

Evidence:

- `cargo build --release && cargo test --release --no-fail-fast`: 735 unit + 15 integration files, all green.
- New tests: `yaks_farm_names_another_repositorys_farm_branch`, `an_in_tree_dot_yaks_is_never_read` (store), and `farm_*` / `init_*` in `tests/init_modes.rs`.
- Real binary in scratch repos: `api` farm, `web` sharing it via `init --farm`, then a `--shared` clone of `web` resolves `api`'s farm and lists the `web-` yak, with no `YAKS_DIR` in its brief. Transcript: [farm-sharing-e2e.txt](yaks-a99f/farm-sharing-e2e.txt).
- Parity grep for `--mode team|private|pointer`, `migrate --to-branch`, `pointer file`, `live_branch`, `team farm|mode`, `in-tree farm` across README/docs/skills/AGENTS/src/tests: only the historical note in AGENTS.md and a test comment remain.

Not done: a cold-read scout of the skills (AGENTS.md suggests one, but delegation is ask-only here), and `cargo install`, which waits until it lands.

---
▸ 2026-10-27T23:23:35Z [delta:Check for `.yaks` file repo redirect sup]

moved: shaving -> shorn

---
▸ 2026-10-31T15:05:04Z [delta:Clean up skills install and mode remnant]

Landed on `main` as `3bb5937`, together with yaks-e532's `c2b640f`. The checkout's `main` had not yet received it when it was shorn.
