---
id: yaks-a99f
title: 'Branch farms only: drop in-tree .yaks/ and pointer files; yaks.farm git config points a repo at another repo''s farm; docs to examples'
status: hairy
type: task
priority: 2
created: '2026-10-27T22:29:18Z'
updated: '2026-10-27T22:29:18Z'
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
