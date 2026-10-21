---
id: yaks-d4a7
title: preflight git-state fails on a farm branch's normal uncommitted state; check the farm is committed (or skip) instead of asking to git add
status: shorn
type: bug
priority: 2
created: '2026-10-19T04:30:51Z'
updated: '2026-10-21T20:46:54Z'
parent: yaks-472e
labels:
- cli
- git
---

Seen at the 0.0.21 landing (2026-10-19): `yaks preflight --push-main` in a code checkout whose farm is on its own branch failed with `.yaks/yaks-a260.md has unstaged changes (git add it)` and `.yaks/yaks-dc63.md is untracked`. Those are the farm worktree's uncommitted yak edits; the code push does not carry them and `git add` is the wrong fix (`yaks commit` is). Rule: when the farm is a farm-branch worktree (`store::farm_branch(root)`), git-state reports "farm has N uncommitted change(s): run `yaks commit`" as a warning (or not at all for `--push-main`, which is about the code branch), never "git add it". Evidence: a scratch repo with a farm branch and a dirty yak; preflight before/after.

---
▸ 2026-10-21T20:33:01Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T20:33:01Z [delta:herd-dirs]

Claimed with yaks-bd1d by `status-1` (same files); spec is on yaks-bd1d.

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
