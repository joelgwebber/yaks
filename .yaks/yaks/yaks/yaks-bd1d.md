---
id: yaks-bd1d
title: 'yaks status: farm branch ahead/behind its upstream (no fetch), reusable sync-state for the TUI'
status: shaving
type: feature
priority: 2
created: '2026-10-21T20:32:27Z'
updated: '2026-10-21T20:34:01Z'
parent: yaks-472e
labels:
- git
- cli
---

Shows whether the farm branch has unpublished commits or is behind its upstream as of the last fetch. A library function the TUI indicator (yaks-ce5d) reuses.

---
▸ 2026-10-21T20:33:01Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T20:33:01Z [delta:herd-dirs]

Claim (delta:herd-dirs, yakherd). Owner: `status-1`. Also owns yaks-d4a7 (preflight on a farm branch): same files, same idea.

**Goal:** `yaks status` says whether the farm branch is published, and preflight stops calling a branch farm's normal uncommitted yaks a "git add it" failure.

**Scope:**
- `src/status.rs`: add `pub struct SyncState { branch: String, upstream: Option<String>, ahead: usize, behind: usize }` and `pub fn sync_state(root: &Path) -> Option<SyncState>`: `None` unless `store::farm_branch(root)` is Some (an in-tree farm has no separate branch); uses `git rev-parse --abbrev-ref <branch>@{upstream}` and `git rev-list --left-right --count <branch>...<upstream>`; NO fetch (as of the last fetch). No upstream -> `upstream: None`.
- `status::render` and `to_json`: one extra line when `sync_state` is Some, e.g. `farm branch yaks: 2 ahead, 0 behind origin/yaks (as of the last fetch); yaks sync publishes` / `... not published (no upstream); yaks sync publishes it` / `up to date with origin/yaks`. JSON: `"sync": {branch, upstream, ahead, behind}` or null. `--check` behaviour unchanged (dirty = uncommitted only).
- `src/preflight.rs` (yaks-d4a7): when the farm root is a farm-branch worktree (`store::farm_branch(root).is_some()`), the git-state check reports uncommitted farm changes as ONE line `farm has N uncommitted change(s): run yaks commit` and does NOT fail for `--push-main` (that check is about the code branch); never says "git add it" for a branch farm. In-tree team farms: unchanged.
- Docs: `docs/cli.md` rows for `status` and `preflight`, the `status` clap help, the yaks skill's command table row for `yaks status`/`yaks preflight`.

**Out of scope:** fetching, `yaks sync`, TUI (yaks-ce5d will call `sync_state`).

**Evidence:** unit/integration tests with a scratch repo + farm-branch worktree + a bare "origin": not published, ahead 1, behind 1 (after a commit pushed from a second clone and a fetch), up to date; preflight on a branch farm with a dirty yak passes `--push-main`'s git-state and prints the one line. Real run: `yaks status` in this repo's farm.

**Judge:** the yakherd.

---
▸ 2026-10-21T20:34:01Z [status-1]

Started status-1.
- date -u: Wed Oct 21 20:33:58 UTC 2026
- yaks path yaks-bd1d: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-bd1d.md
