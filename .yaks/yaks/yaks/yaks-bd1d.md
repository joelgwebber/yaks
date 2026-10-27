---
id: yaks-bd1d
title: 'yaks status: farm branch ahead/behind its upstream (no fetch), reusable sync-state for the TUI'
status: shorn
type: feature
priority: 2
created: '2026-10-21T20:32:27Z'
updated: '2026-10-27T01:19:39Z'
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

---
▸ 2026-10-21T20:38:48Z [status-1]

Real run of my built ./target/release/yaks in this machine's real farm (no fetch done by me):

```
$ ./target/release/yaks status
farm changes git does not have yet (* = staged):
  yaks-85bb  notes +2
run `yaks commit` to commit these as: yaks: updated yaks-85bb
farm branch yaks: 8 ahead, 0 behind origin/yaks (as of the last fetch); `yaks sync` publishes
$ ./target/release/yaks status --json | .sync
{'branch': 'yaks', 'upstream': 'origin/yaks', 'ahead': 8, 'behind': 0}
$ ./target/release/yaks preflight
preflight: warning: farm has 1 uncommitted change(s): run `yaks commit`
preflight: ok
```

Before the change, preflight on a branch farm with a dirty yak failed with `.yaks/yak/yaks/yak-0001.md has unstaged changes (git add it)` etc. (seen by disabling the branch rule in test `a_branch_farm_with_uncommitted_yaks_warns_once_and_does_not_fail`; it fails there, passes with the rule).

---
▸ 2026-10-21T20:38:49Z [status-1]

Decision: preflight's branch-farm line is a new `Report.warnings` (text `preflight: warning: ...`, JSON `warnings`), not `skipped` and not a failure.
- Rejected: reusing `skipped` (it means "check could not apply", and the check did run); failing (the spec says it must not fail, and `ok` stays true).
- Cost: a 4-line addition in the `Command::Preflight` arm of main.rs.

Decision: on a branch farm every non-conflict git-status entry (staged, unstaged or untracked) counts toward N; N counts git-status entries (an untracked attachment dir is one entry), not yaks. Unresolved merge conflicts still FAIL (with their existing message, no "git add it") since `yaks commit` would refuse them.

Decision: the sync state rides on `status::Report.sync` (filled in `collect`), so `render`/`to_json` keep their signatures and main.rs's Status arm is untouched; the TUI calls `status::sync_state(root)` directly.

Decision: `sync_state` returns `Some{upstream: None, ahead 0, behind 0}` when the branch has no upstream OR the configured upstream ref is gone (both read "not published"); `None` when git cannot count (unparseable) so the TUI shows nothing rather than a wrong state. Rejected: a distinct "upstream gone" state (not in the spec).

Decision: text wording when behind>0 says `yaks sync` reconciles (ahead only: publishes), since "publishes" is false when you are behind; the spec's example only showed the ahead case. `yaks sync` does not exist in this change; it is named per the spec.

Decision: the sync line is the LAST line of `yaks status` (after the commit hint), and is absent for in-tree and private farms. `--check` unchanged.

---
▸ 2026-10-21T20:46:45Z [status-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-21T20:46:53Z [status-1]

Done (status-1).
- `status::SyncState` + `status::sync_state(root)` (None unless `store::farm_branch(root)`; no fetch; no upstream -> `upstream: None`); `Report.sync`; one extra last line in `yaks status` and JSON `sync`.
- preflight on a farm-branch worktree: uncommitted yak changes -> ONE `warnings` line `farm has N uncommitted change(s): run yaks commit`, no failure (also with `--push-main`); conflicts still fail; in-tree farms unchanged.
- Tests (all fail/not compile before): status: not published, up to date, ahead, behind+ahead (second clone push + fetch), behind only, in-tree None, writes nothing; preflight: dirty branch farm warns once w/ `--push-main`, no "git add"; branch-farm conflict still fails.
- Docs: docs/cli.md rows, README rows, yaks skill rows, clap help for status/preflight.
- Gate `cargo test --release`: all suites ok (754 passed lib + integration suites); `yaks verify`: PASS.

---
▸ 2026-10-21T20:46:54Z [status-1]

moved: shaving -> shorn

---
▸ 2026-10-21T20:52:02Z [delta:herd-dirs]

Landed (delta:herd-dirs): 8f77dde. Review: re-ran full cargo test --release with driver-1's work merged (767 unit, every suite ok); real `yaks status` on this machine's farm shows 'farm branch yaks: N ahead, 0 behind origin/yaks'. Accepted the warnings field and the publishes/reconciles wording.
