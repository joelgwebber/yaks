---
id: yaks-6ffd
title: 'Layout: drop the .yaks root (discovery, farm_git prefix, path classifiers)'
status: hairy
type: feature
priority: 2
created: '2026-11-14T11:21:02Z'
updated: '2026-11-14T11:21:19Z'
parent: yaks-1151
tags:
- git
- links
---

The code half of dropping the `.yaks/` root (yaks-1151 decision 1). The audit with every site is [audit.md](yaks-9714/audit.md); this is its W1 to W3 and W9. It migrates no farm: it makes yaks understand a branch whose root is the farm root, and keep reading the `.yaks/` layout until the migration yak runs.

## Key point

`store::farm_git` treats `root.parent()` as the worktree top, so a farm at the branch root silently stops counting as a farm branch: no per-change commits, no `changes::read`, no sync, no brief. Add a `prefix` to `FarmGit` (`""` after, `".yaks/"` before) and make `top == root` valid, so every farm-relative to repo-relative conversion goes through it. Replace the other `root.parent()` users with `farm_git().top`: `status.rs`, `sync.rs`, `weight.rs`, `sheds.rs`, `tui/farmsync.rs`, `farm.rs`, `links.rs`.

## Silent failures to fix and test

- `preflight::yak_id_of` and `weight::kind_of` require a `.yaks` path component.
- The land guard in `land.rs` could never fire.
- `sync.rs` checks for a dirty farm with `-- .yaks`.
- `store::init`'s `root.exists()` check, `init.rs`, and `migration_refusal` (reads `{upstream}:.yaks/schema`).
- The TUI keys saved views, the starred set and drafts by the farm root path, which changes: copy the old state across on first run (`views_store.rs`, `cache.rs`, `drafts.rs`).
- `links::read_at` needs a `.yaks/` fallback for pinned links into pre-move commits, next to the pre-doks one.

## Out of scope

`.gitattributes` and LFS rule rewriting, moving files, link rewriting: the migration yak. `yaks init` keeps writing `.yaks/` until the migration yak flips the default.

## Evidence

Tests for both layouts through `Farm::at` / `discover_with`, with a fixture of each; the existing `.yaks` tests keep passing. A scratch repo whose farm branch has the farm at its root: `yaks create`, `shave`, `sync --no-push`, `status` and the land guard, with the autocommit visible in `git log`.

---
▸ 2026-11-14T11:21:19Z [linkplan-lead]

Part of the plan with [yaks-f505](yaks-f505) (resolver, independent) and [yaks-acc0](yaks-acc0) (the migration that uses this). Shave the resolver and this in parallel only if the scopes stay disjoint: this owns `store.rs`, `init.rs`, `status.rs`, `sync.rs`, `weight.rs`, `preflight.rs`, `land.rs`, the TUI state files; the resolver owns the new `locate.rs`, `hosting.rs`, `links.rs` and `follow.rs`. `farm_git`'s `top == root` change lands first and the resolver rebases onto it.
