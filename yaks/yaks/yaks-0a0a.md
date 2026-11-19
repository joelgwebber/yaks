---
id: yaks-0a0a
title: 'MERGE_HEAD/CHERRY_PICK_HEAD checked as files: wrong on reftable repos (init.defaultrefformat=reftable)'
status: shorn
type: bug
priority: 2
created: '2026-11-19T23:42:03Z'
updated: '2026-11-19T23:52:19Z'
tags:
- git
- cli
---

Found while gating yaks-b4d7 (herd-files). Since `~/.gitconfig` gained `init.defaultrefformat=reftable` (2026-11-19 16:38), new repos keep pseudo-refs in the reftable, so `.git/CHERRY_PICK_HEAD` and `.git/MERGE_HEAD` are not files.

- **Product:** `changes::pending_operation` checks `f.git_dir.join(head).is_file()` for a farm branch, so on a reftable farm repo a pending merge or cherry-pick is not seen, and `yaks commit` would not refuse during one. Use `git rev-parse -q --verify <head>` in the farm worktree for both cases, as `land.rs` already does.
- **Tests:** `land::tests::a_conflicting_merge_falls_back_to_cherry_pick_and_names_the_files` fails today (it asserts `.git/CHERRY_PICK_HEAD` exists); `land.rs` (MERGE_HEAD setup asserts) and `commit.rs` assert the same files and pass only by luck of order or file format. Make them ask git, and make test repos explicit about the ref format or test both.

Repro: `git init` (with reftable default), make a conflicting cherry-pick: `git rev-parse --verify CHERRY_PICK_HEAD` succeeds, `ls .git/CHERRY_PICK_HEAD` fails.

#### 2026-11-19T23:42:09Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-19T23:52:18Z

*herd-files*

Done (herd-files).

- `changes::pending_operation`: on a farm branch it still checks the file under the git dir (no git process per write; the spawn budget in `tests/autocommit.rs` holds) unless the repository uses reftable (`uses_reftable`: `reftable/` in the common dir, following a linked worktree's `commondir`), where it asks `git rev-parse --verify`, as for any other farm.
- Tests no longer assume the files format: `land.rs` (`has_ref`), `tests/land_continue.rs` (`picking`), `commit.rs` (asks `pending_operation`). `tests/common::git_init` pins `init.defaultRefFormat=files`, since the spawn budgets count a files-format repository.

Evidence:

- New tests: `changes::tests::a_pending_cherry_pick_is_seen_in_either_ref_format`; `tests/autocommit.rs::commit_refuses_during_a_cherry_pick_in_either_ref_format`, a farm-branch run of `yaks commit` during a conflicting cherry-pick. With the old `changes.rs` it fails for reftable ("git commit failed ... the farm changes are staged, not committed").
- Real run in a reftable scratch repo: the installed `973a47e1` binary left the farm changes staged under git's error; the new build refuses with yaks' own message.
- Gate: `cargo test -p yaks` 1525 passed, 0 failed (with Joel's reftable default in `~/.gitconfig`); `cargo fmt --check` clean.

#### 2026-11-19T23:52:19Z

*herd-files · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn
