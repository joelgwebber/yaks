---
id: yaks-6ffd
title: 'Layout: drop the .yaks root (discovery, farm_git prefix, path classifiers)'
status: shorn
type: feature
priority: 2
created: '2026-11-14T11:21:02Z'
updated: '2026-11-14T12:33:17Z'
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

---
▸ 2026-11-14T11:32:23Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T11:32:23Z [linkplan-lead]

Claim: worker `layout-1` (Sonnet tier, standing delegation). Owns, by symbol: `store::farm_git` / `FarmGit` and the `root.parent()` users in `status.rs`, `sync.rs`, `weight.rs`, `sheds.rs`, `farm.rs`, `tui/farmsync.rs`; `preflight::yak_id_of`; `weight::kind_of`; the land guard in `land.rs`; `store::init` and `init.rs`; `store::migration_refusal`; the TUI per-user state paths (`views_store.rs`, `cache.rs`, `drafts.rs`); and in `links.rs` only `read_at` and `code_base`. Out of scope: `.gitattributes` and LFS rules, moving files, link rewriting, any stored form. Another worker (`locate-1`, yaks-f505) owns `locate.rs`, `hosting.rs`, `gitrepo.rs`, `links::exists` and `tui/follow.rs`: do not edit those. Judge: the coordinator re-runs the evidence on its own build.

---
▸ 2026-11-14T11:32:52Z [layout-1]

Start: layout-1. date -u: Sat Nov 14 11:32:45 UTC 2026.

- `yaks path yaks-6ffd`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-6ffd.md
- checkout top-level: /Users/joel/src/yaks/.delta/worktrees/rknhx6hbsrkc/yaks (main e4d0ff5a)

Read the brief, the claim note and the audit (W1-W3, W9). Starting with store::farm_git / FarmGit.

---
▸ 2026-11-14T12:03:01Z [layout-1]

Milestone: core compiles clean (`cargo build`, no warnings). Not yet tested.

Done so far:

- `store::farm_root_in(wt)`: the one place that tells the layouts apart.
- `FarmGit.prefix` + `FarmGit::repo_path`; `farm_git` runs git in the root and accepts `top == root` (prefix `""`) or `top == root.parent()` (prefix `.yaks/`).
- `store::init` (`config.yaml`/`schema` markers instead of `root.exists()`), `migration_refusal` (`{upstream}:schema` then `:.yaks/schema`), `init.rs` (`branch_holds_farm`, root via `farm_root_in`, still creates `.yaks/` when fresh), `shared_farm_worktree`, `farm_branch_worktree`.
- `store::farm_relative` shared by `preflight::yak_id_of`, `changes::farm_rel`, `commit::generate_message`, `weight::kind_of`, and the land guard (`land::is_yak_path`).
- `root.parent()` users now go through `farm_git().top`: `status::sync_state`, `sync::farm_worktree` (+ dirty pathspec `-- .`), `weight::farm_top`, `sheds::Freshness::read`, `tui/farmsync.rs` x2, `farm::code_repo`, `links::code_base`.
- `links::read_at`: `{rev}:.yaks/{path}` fallback (and `.yaks/wiki/...`).

Next: TUI per-user state copy, then tests for both layouts, then the scratch-repo run.

[layout1-evidence](yaks-6ffd/layout1-evidence.txt "Scratch repo, farm at the branch root: create/shave/note/shorn autocommits, status, commit, sync --no-push, weight, land guard exit 7")

---
▸ 2026-11-14T12:17:54Z [layout-1]

Decision: layouts are told apart in one place, `store::farm_root_in(worktree)`: a `config.yaml` at the worktree top means the root layout and wins over a `.yaks/` beside it (the migration's stub `.yaks/schema`); else a `.yaks/` directory means the old layout; else a bare top-level `schema`. Rejected: probing `schema` first (the stub would then make a migrated farm read as old) and looking at `git ls-tree` (slower, needs a commit).

---
▸ 2026-11-14T12:17:55Z [layout-1]

Decision: path-only classifiers (`preflight::yak_id_of`, `changes::farm_rel`, `commit::generate_message`, `weight::kind_of`, the land guard) share `store::farm_relative(repo_path)`: strip through the last `.yaks` component, else the path as is. Rejected: threading `FarmGit.prefix` into every signature (they see only a path, from git output that may be older than the farm's current layout, for example a pre-move commit). `FarmGit.prefix`/`repo_path` is used where a farm is at hand (`weight::enable_lfs`).

---
▸ 2026-11-14T12:17:56Z [layout-1]

Decision: land guard rule. A worker commit is refused (exit 7) when it touches a path that `path_owner` says is a yak file or attachment (at the top or under `.yaks/`) AND HEAD tracks no farm. HEAD tracks a farm when any tracked path is under `.yaks/` or is a yak file, or it has both `config.yaml` and `schema` at the root. Rejected: refusing on the path alone (a repo that legitimately keeps a farm or yak fixtures would never land), and only checking `.yaks/` (what could never fire after the move). Known residual: a code repo with a tracked `*/yaks/*.md` of its own is not guarded.

---
▸ 2026-11-14T12:17:56Z [layout-1]

Decision: `sync` dirty check is `git status --porcelain -- . ':(exclude).agents'` from the top. The old `-- .yaks .gitattributes` matched nothing in the root layout (the farm then read clean and a rebase was attempted on a dirty tree). `.` alone broke two sync tests: `yaks init` run from inside the farm worktree installs untracked `.agents/skills`, which are no part of the farm; excluded. Rejected: listing farm top-level names (a new herd dir would be missed).

---
▸ 2026-11-14T12:17:57Z [layout-1]

Decision: TUI per-user state. At `App::with_farm`, before anything loads, `views_store::adopt_legacy_state` copies the old key's files (`~/.config/yaks/<slug(<root>/.yaks)>/**` = views, working set, drafts, and the cache file `~/.cache/yaks/<slug>.json`) to the new key. Conflict rule: a file that already exists under the new key is kept, never overwritten; old files are never deleted (an older yaks keeps reading them). Run on every start but a no-op once the new files exist. Not run for a root that is itself `.yaks/` (key unchanged). Rejected: moving (breaks an older binary alongside) and keying by worktree (still changes the key once).

---
▸ 2026-11-14T12:17:57Z [layout-1]

Decision: `yaks init` is unchanged for a fresh farm (`.yaks/`), but on an existing branch `init.rs` now takes the root from `farm_root_in` and `branch_holds_farm` accepts `config.yaml`/`schema`/`.yaks` at the branch tip, so init no longer calls a root-layout branch "not a farm". `store::init` decides `AlreadyExists` by `config.yaml`/`schema`, or an existing `.yaks/` directory, not `root.exists()`.

---
▸ 2026-11-14T12:33:07Z [layout-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-14T12:33:16Z [layout-1]

Summary: yaks now understands a farm branch whose root IS the farm root and still reads `.yaks/`; migrates nothing, `yaks init` still writes `.yaks/`.

- `store::farm_root_in` decides the layout in one place; `FarmGit` gains `prefix` + `repo_path`; `farm_git` accepts `top == root` or `top == root.parent()`.
- Silent failures fixed: `yak_id_of`, `kind_of`, `farm_rel`, `generate_message` (shared `store::farm_relative`); the land guard (`is_yak_path` + `tracks_farm`); sync's dirty pathspec; `store::init`; `init.rs`; `migration_refusal`; `shared_farm_worktree`; `farm_branch_worktree`.
- `root.parent()` users now use `farm_git().top`: `status`, `sync`, `weight`, `sheds`, `tui/farmsync` (x2), `farm::code_repo`, `links::code_base`.
- `links::read_at` falls back to `{rev}:.yaks/{path}` (and `.yaks/wiki/...`).
- TUI per-user state is copied from the old key on first run (views, working set, drafts, cache).
- Docs: `docs/cli.md` and the `main.rs` help line for land exit 7.

Evidence:

- Gate `cargo test -p yaks` all green (1143 unit + every integration binary), `cargo fmt --check` clean, no warnings. `yaks verify yaks-6ffd` PASS.
- New tests: `store::discover_tests` (5, both layouts through `discover_with`/`farm_git`/`init`/`migration_refusal`), `tests/root_layout.rs` (2, real binary), land guard (2), plus classifier, `read_at`, `code_base`, and state-copy tests. Existing `.yaks` tests pass untouched.
- Scratch repo output attached (`layout1-evidence.txt`): per-change autocommits in `git log`, `status`, `commit`, `sync --no-push`, `weight`, and `yaks land` exit 7 in the root layout.
- Real farm (`.yaks`) still works with this binary: `status`, `weight`.

Not done / for others: `.gitattributes` and LFS rules still say `.yaks/**` (migration yak); `yaks session touched` and `check` staleness across the move; the `.yaks` text in brief and other docs; known residual in the land guard (a code repo that legitimately tracks `*/yaks/*.md`).

---
▸ 2026-11-14T12:33:17Z [layout-1]
kind: move
from: shaving
to: shorn
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAGEcKpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
