---
id: yaks-9f91
title: 'Flat farm, phase 2: git reporting (yaks status, yaks commit, changes.rs) for in-place status changes'
status: shorn
type: task
priority: 1
created: '2026-10-12T19:53:57Z'
updated: '2026-10-12T20:34:16Z'
parent: yaks-8b7a
depends_on:
- yaks-95be
labels:
- cli
- git
---

Phase 2 of yaks-8b7a (plan note: D7; runs after phase 1, yaks-95be). Sonnet worker.
Scope: src/changes.rs (`classify`), src/status.rs (`describe_yak`, the `yaks status` lines and message), src/commit.rs (`yaks commit`'s generated message), their tests, and the docs rows for `status`/`commit` in docs/cli.md and README.md.
Rule: in a flat farm a status change is a modified `<id>.md` whose `status:` changed between HEAD and the working tree: report it as `moved <from> -> <to>` exactly as today, and `yaks commit` says `shorn <id>` etc. as today. Keep understanding a deleted plus an added path for the same id (legacy history, and a v3 farm read by this binary). The migration itself (every yak deleted from a status dir and added at the root with the same status) reports as ONE line, `farm layout migrated: N yaks`, not N moves, and `yaks commit` uses that as its message.
Out of scope: src/store.rs, migrate, doctor, sheds, skills.
Evidence (fail first): a flat-farm status change reports `moved`; a combined status change plus note reports both; a staged migration reports the one line; the existing v3-shaped tests still pass.

---
▸ 2026-10-12T20:00:39Z [delta-lead]

Amended by D10 (see yaks-8b7a): an attachment is now any file under `.yaks/<id>/` (use phase 1's predicate), not `.yaks/artifacts/<id>/`; and the migration's one summary line also covers the attachments moving. The legacy deleted-plus-added pairing is transitional (phase 4 deletes it): keep it in one function.

---
▸ 2026-10-12T20:20:37Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-12T20:20:38Z [delta-lead]

Claim (coordinator). Owner: git-1. Spec: the description, the amendment note on this yak, and yaks-8b7a's D7.
You own: src/changes.rs, src/status.rs, src/commit.rs, their tests, tests/status.rs (UN-IGNORE `clean_then_dirty_with_check_exit_codes`: it is your fail-first test; phase 1 ignored it with a reason naming you), and the `status`/`commit` rows in docs/cli.md and README.md (plus the `///` help of the `Status`/`Commit` variants in src/main.rs only: another worker owns the rest of main.rs).
Rules: a status change in the flat layout is a MODIFIED `<id>.md` whose `status:` differs between HEAD and the working tree: report `moved <from> -> <to>` and let `yaks commit` say `shaving <id>` / `shorn <id>` exactly as today. Attachments are files under `.yaks/<id>/` (use `store::attachment_owner` on the farm-relative path; `preflight::yak_id_of` already makes repo paths farm-relative). Keep the old deleted-plus-added pairing for an unmigrated farm, in ONE function (yaks-bbda deletes it). The migration itself (old-layout files deleted, flat files added with the same statuses, `artifacts/<id>/` moved to `<id>/`) reads as one line `farm layout migrated: N yaks` in `yaks status`, and `yaks commit`'s message is `yaks: migrate the farm to the flat layout (N yaks)`.
Evidence (fail first): the un-ignored test passes; a status change plus a note reports both; a flat attachment reports `artifacts`; a staged migration reports the one line and the commit message; existing old-layout tests still pass. Judge: coordinator.
## Rules for every phase-2 worker (read before starting)
- Phase 1 (yaks-95be, landed 79f00e9) is the base. Read its shorn notes first: `store::task_path`, `store::artifact_dir`, `store::attachment_owner`, `store::path_owner`, `store::legacy` (ALL old-layout reading lives there; add any new old-layout reading there too, so yaks-bbda deletes it in one place), `store::lock` (refuses writes on an old-layout farm) and `store::lock_for_migration`.
- THIS REPO'S OWN FARM STAYS ON THE OLD LAYOUT until phase 3. Your built binary therefore refuses to write here, by design. Use the `yaks` on PATH (0.0.15) for every command on your yak; use your own `./target/release/yaks` only on temp fixtures. Never run `migrate`, or any write of the new binary, against this repo's `.yaks`.
- Finish staging is the old layout's: `git add -A -- .yaks/shaving/<id>.md .yaks/shorn/<id>.md` plus `.yaks/artifacts/<id>` if you attached something, and NEVER name a path that does not exist (git then stages nothing at all). Check `git status --short` is empty after your commit.
- File ownership is strict (three workers run at once): touch only the files your claim names. If you must touch another, `yaks ask` instead.

---
▸ 2026-10-12T20:23:54Z [git-2]

git-2 started: Mon Oct 12 20:23:49 UTC 2026
yaks path yaks-9f91: /Users/joel/src/yaks/.delta/worktrees/qzztv7bg68rg/yaks/.yaks/shaving/yaks-9f91.md

---
▸ 2026-10-12T20:33:09Z [git-2]

Decision: the status transition is read in `changes::read` (HEAD's and the working tree's `status:` stored on each `Change` as `status_before`/`status_after`), so `classify` and `generate_message` stay pure functions of `&[Change]` and `yaks status` and `yaks commit` cannot disagree. Rejected: having `classify` shell out to git (breaks the pure unit tests, runs git per call) and doing it only in `status.rs` (commit would say `updated` where status says `moved`).
Decision: the old deleted-plus-added pairing is ONE function, `changes::legacy_pair` (with its helpers `status_at`, `is_legacy_path`), marked transitional for yaks-bbda. It also decides `Migrated`: an old-layout path deleted and a flat `<id>.md` ADDED with the same status. Rejected: a separate pass over all changes for the migration (two places for yaks-bbda to find).
Decision: the migration is `Kind::Migrated` per yak, folded by `classify` into `Classified::migrated` (count + staged); the `schema` file change is folded in too, so the migration is exactly one line and nothing else. A yak whose old file was deleted and flat file added with a DIFFERENT status stays a `moved`, even in the same commit as a migration. Rejected: counting by directory (would also swallow unrelated moves) and leaving `schema` as an `other` line (a second line for one event).
Decision: commit message is `yaks: migrate the farm to the flat layout (N yaks)` (the claim note's text; the yak description's looser wording was `farm layout migrated: N yaks` for status only), joined with other phrases by `; ` when the commit also has them. `--json` gains `migrated: null | {yaks, staged}`.
Decision: `head_text` moved from status.rs to changes.rs (pub) so `read` can use it; one git-show helper.
Note: `yaks attach` in the new binary still writes `.yaks/artifacts/<id>/` (another worker's scope); `attachment_owner` accepts both shapes, so status reports `artifacts` either way (tested flat `<id>/`; real-machine run saw the legacy shape).

---
▸ 2026-10-12T20:34:07Z [git-2]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-12T20:34:16Z [git-2]

Done. `yaks status`/`yaks commit` now report an in-place `status:` change in a flat farm as `moved <from> -> <to>` / `shaving <id>` etc.; a flat attachment reports `artifacts`; a staged migration is one line `farm layout migrated: N yaks` and commit message `yaks: migrate the farm to the flat layout (N yaks)`. Old deleted+added pairing kept in `changes::legacy_pair`. tests/status.rs `clean_then_dirty_with_check_exit_codes` un-ignored and passing. New tests in src/status.rs: flat move, move+note, edit-not-a-move, flat attachment, staged migration, old-layout move. Docs: docs/cli.md and README.md status/commit rows, `///` help of Status/Commit in main.rs. Gate: `cargo test --release` all suites green (537 unit, 3 pre-existing ignored).

---
▸ 2026-10-12T20:34:16Z [git-2]

moved: shaving -> shorn
