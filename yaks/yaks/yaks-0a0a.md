---
id: yaks-0a0a
title: 'MERGE_HEAD/CHERRY_PICK_HEAD checked as files: wrong on reftable repos (init.defaultrefformat=reftable)'
status: hairy
type: bug
priority: 2
created: '2026-11-19T23:42:03Z'
updated: '2026-11-19T23:42:03Z'
tags:
- git
- cli
---

Found while gating yaks-b4d7 (herd-files). Since `~/.gitconfig` gained `init.defaultrefformat=reftable` (2026-11-19 16:38), new repos keep pseudo-refs in the reftable, so `.git/CHERRY_PICK_HEAD` and `.git/MERGE_HEAD` are not files.

- **Product:** `changes::pending_operation` checks `f.git_dir.join(head).is_file()` for a farm branch, so on a reftable farm repo a pending merge or cherry-pick is not seen, and `yaks commit` would not refuse during one. Use `git rev-parse -q --verify <head>` in the farm worktree for both cases, as `land.rs` already does.
- **Tests:** `land::tests::a_conflicting_merge_falls_back_to_cherry_pick_and_names_the_files` fails today (it asserts `.git/CHERRY_PICK_HEAD` exists); `land.rs` (MERGE_HEAD setup asserts) and `commit.rs` assert the same files and pass only by luck of order or file format. Make them ask git, and make test repos explicit about the ref format or test both.

Repro: `git init` (with reftable default), make a conflicting cherry-pick: `git rev-parse --verify CHERRY_PICK_HEAD` succeeds, `ls .git/CHERRY_PICK_HEAD` fails.
