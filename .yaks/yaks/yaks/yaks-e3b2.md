---
id: yaks-e3b2
title: 'yaks sync: commit, pull --rebase (with the merge driver), push the farm branch'
status: shorn
type: feature
priority: 2
created: '2026-10-21T20:31:47Z'
updated: '2026-10-21T21:11:20Z'
parent: yaks-472e
depends_on:
- yaks-4a53
labels:
- git
- cli
---

---
▸ 2026-10-21T20:58:21Z [delta:herd-dirs]

Claim (delta:herd-dirs, yakherd, building it myself: it ties the three landed pieces together).

**Shape:** `yaks sync [--no-commit] [--no-pull] [--no-push] [--dry-run] [-m MSG]`, new `src/sync.rs`, run in the farm worktree whatever the cwd (as `commit`). Refuses an in-tree farm (`store::farm_branch(root)` is None): "the farm is not on its own branch; commit and push it with the code".

**Steps:**
1. Register the merge driver (`mergedriver::ensure_registered`), so `.gitattributes` is in place before anything merges.
2. Commit (unless `--no-commit`): `commit::run` (which also sweeps in hand edits and the new `.gitattributes`).
3. Pull (unless `--no-pull`): remote = `branch.<b>.remote` else `origin`; no such remote -> skip with a line. `git fetch <remote> <b>`; remote branch absent -> nothing to pull. Else `git rebase <remote>/<b>` under the farm lock (writers wait); on a failed rebase: `git rebase --abort`, stop with the conflicting paths named (the driver makes this rare: non-yak files only).
4. Push (unless `--no-push`, or `yaks.private` -> skipped with the reason): `git push -u <remote> <b>:<b>` (creates the remote branch on first publish, same name). Rejected because the remote moved meanwhile: re-pull and retry once.
5. Report one line per step and the final `status::sync_state`.

**Evidence:** tests with a bare origin and two clones: first publish creates `origin/yaks` with upstream set; concurrent edits on the same yak converge from both sides with no hand git; a non-yak conflict (config.yaml) aborts cleanly leaving the branch as it was; private refuses push; `--no-push`/`--no-pull`/`--no-commit`; a race (remote moved between pull and push) retries. Real run on this repo's farm (Joel's go-ahead for the push to origin is the open question below: I will run it with `--no-push` until he says so).

---
▸ 2026-10-21T20:58:21Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T21:11:20Z [delta:herd-dirs]

Built and landed by the yakherd: cabc284 (sync) + 3bc2282 (driver registered as plain `yaks` when that is this binary on PATH; a pinned absolute path breaks every later merge once the binary moves).

Evidence:
- tests/sync.rs, 6 tests through the real binary (bare origin, two machines' clones, the second bootstrapping its farm): first publish (creates origin/yaks, upstream set, .gitattributes committed on the branch, second run a no-op); concurrent edits to ONE yak (A notes + shaves, B notes + labels) converge, both files byte-identical, both notes, one `updated:`, no markers, doctor clean, status "up to date" on both; --no-push / --no-commit (dirty farm keeps its edit, pull skipped) / --no-pull; private farm never pushed; a config.yaml conflict aborts the rebase, names the file, B's commit kept; an in-tree farm refused.
- Full `cargo test --release`: 20 suites ok.
- Real run on this repo's farm from Joel's checkout with the installed binary: `yaks sync --no-push` committed `.gitattributes` (65189c3 "yaks: merge yak files with the yaks merge driver"), `up to date with origin/yaks`, push skipped; `git config merge.yaks.driver` = `yaks merge-driver %O %A %B %P`. NOT pushed: publishing the farm is Joel's call.

Found and fixed on the way: a fresh orphan farm branch (nothing committed) read as a private farm, so the first `yaks commit`/`sync` refused (changes::read now checks store::farm_branch).

---
▸ 2026-10-21T21:11:20Z [delta:herd-dirs]

moved: shaving -> shorn
