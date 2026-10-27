---
id: yaks-bbda
title: 'Flat farm, phase 4: remove the migration and the dual-layout read once every farm is migrated'
status: shorn
type: task
priority: 2
created: '2026-10-12T20:00:38Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-8b7a
depends_on:
- yaks-3a56
labels:
- cli
needs: agent
---

Phase 4 of yaks-8b7a (after phase 3's release and after Joel's farms are migrated: ~/src/yaks, ~/src/j15r, and any private/pointer farms). Remove the transition: `yaks migrate`, the dual-layout read in src/store.rs, the legacy deleted-plus-added classification in src/changes.rs, the both-layouts doctor/preflight check, and every transitional line in docs and skills. Keep: a schema-3 farm refuses to open with a message naming the last release that can migrate it; `yaks sheds` shows a schema-3 shed's farm as "old layout (pre-0.0.16 clone)", never as an empty farm. Release 0.0.17.

---
▸ 2026-10-12T21:22:59Z [delta-lead]

Joel (2026-10-12): as part of phase 4, re-check every skill and doc for wording the flat layout made stale (status directories, artifacts/<id>/, moves relocating a file, staging globs), beyond the transitional lines this phase removes: grep .agents/skills, docs/, README.md, AGENTS.md and src/ --help text. Also refresh docs/assets/tui-detail-pinned.svg (stale since before the flat work: docshots re-render it differently from b25a5b9 too; one 'cargo test -p yaks docshots -- --ignored' and commit).

---
▸ 2026-10-12T22:29:49Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-12T22:29:49Z [delta-lead]

Claim (coordinator). Owner: bye-1 (code); the coordinator does the prose sweep in parallel. Joel (2026-10-12): every farm is migrated, so proceed. Base: 0.0.16 (released). Spec = the description plus this note.

## Remove (all transitional, every place marked yaks-bbda / "transitional" / "old layout")
- `yaks migrate`: the `Command::Migrate` variant, its dispatch and help (src/main.rs), the "yaks migrate" section of src/farm.rs (`Farm::migrate`, `MigrateOutcome`, helpers), its tests (src/farm.rs, tests/cli.rs).
- `store::legacy` entirely, and every caller: `load`, `find_task_file`, `all_ids`, `resolve_farm_root` read the flat layout only; `lock` loses the write blocker; `lock_for_migration` goes. `attachment_owner`/`path_owner` lose the `artifacts/<id>` and `<status>/<id>.md` branches; `preflight::yak_id_of` loses its legacy fallback.
- Git reporting: `changes::legacy_pair`, `Kind::Migrated`, `Migration`, `Classified::migrated`, the migration line and commit message (src/status.rs, src/commit.rs), the `migrated` key in `status --json`, the legacy attachment branch; preflight's migration scope exclusion.
- Doctor/preflight: the `duplicate-status` and `both-layouts` checks. KEEP `invalid-status`. KEEP a duplicate-id check, reworded for the flat layout (two files whose frontmatter has the same `id:`).
- src/farm.rs: the `artifacts/<id>/` fallbacks (attach/rename-attachment/merge/rename).
- src/sheds.rs: the old-layout section (`is_old_layout`, `old_tree_task`, `flat_attachment_links` and its use in `load_farm`, the `old_layout` field), the `farm.layout` JSON key and `changes --json`'s `layout` key, and the `(old layout)` markers.
- `Status::dir()` is now a misnomer: rename it to `Status::as_str()` (like `ShedKind::as_str`), every call site; its doc comment loses the directory sentence.

## Keep for good (new, permanent; tests fail first)
- A schema-3 farm (the `schema` file says 3, or any `hairy/ shaving/ shorn/ dead/` directory holds a `.md`) refuses to OPEN, for reads too, with: `<farm> uses the old yak layout (status directories, schema 3), which this yaks no longer reads: migrate it once with yaks 0.0.16 (npx -y @j15r/yaks@0.0.16 migrate), commit, then use this yaks again`. One check, in `Farm::open` (and wherever another farm is opened: `merge`'s source, `answer @shed`'s target).
- `yaks sheds`: a shed whose farm is schema 3 is `ShedFarm::Unreadable` with a message naming it an old-layout clone from before 0.0.16, never read as an empty farm (which would look like it deleted every yak). Same in `changes`, `show --sheds`, `inbox --sheds`.
- A base commit from before a farm migrated has no flat files: `base_farm` then finds nothing and the existing fallback applies; make sure that is a fallback, not a "deleted every yak" delta, and test it.

## Scope and docs
You own src/, tests/, docs/cli.md, README.md. The coordinator owns .agents/skills, docs/README.md, docs/tui.md, AGENTS.md and docs/assets: do NOT edit those. In docs/cli.md and README.md: remove the `migrate` rows and every transitional sentence (old layout, both layouts, duplicate-status, `(old layout)`, `farm.layout`, `layout`, `farm layout migrated`), and describe the schema-3 refusal once (the `doctor`/farm or `init` area, your call; record it).
Evidence: the two permanent behaviors above fail first; `grep -rnE "legacy|migrat|artifacts/|old layout|both.layouts|duplicate.status|yaks-bbda" src tests docs/cli.md README.md` shows only the refusal message and its tests (paste it); full gate `cargo test --release` green. Judge: coordinator.

---
▸ 2026-10-12T22:30:25Z [bye-1]

Start: Mon Oct 12 22:30:19 UTC 2026 (bye-1)
yaks path: /Users/joel/src/yaks/.delta/worktrees/h4n200yc9791/yaks/.yaks/yaks-bbda.md

---
▸ 2026-10-12T22:45:51Z [bye-1]

Failing first (before any removal), `cargo test --release --test cli --test sheds_layouts`:
- tests/cli.rs `a_schema_3_farm_refuses_to_open_and_names_the_release_that_migrates_it` FAILED
- tests/sheds_layouts.rs: all 5 FAILED: an_old_layout_shed_is_named_as_such_never_read_as_an_empty_farm, changes_show_and_inbox_name_an_old_layout_shed_and_skip_it, answering_into_an_old_layout_shed_refuses_and_writes_nothing, a_flat_shed_forked_before_the_migration_is_compared_to_the_checkout_not_to_nothing, an_old_layout_checkout_refuses_every_command_naming_the_release_that_migrates_it
Plan: one `store::old_layout(root)` predicate; `Farm::open` + merge source + answer@shed target refuse with `OpenError::OldLayout`/bail; `sheds::load_farm` errors on it so every shed read lands in ShedFarm::Unreadable; base_farm keeps flat files only (old base commit -> None -> existing fallback).

---
▸ 2026-10-12T22:57:10Z [bye-1]

Decision: one predicate and one message. `store::old_layout(root)` (schema marker < 4, or any `hairy/ shaving/ shorn/ dead/` dir holding a `.md`; an empty status dir does not count) plus `store::old_layout_refusal` / `OLD_LAYOUT_FIX`. Every refusal (open, merge source, sheds) builds on them. Rejected: separate checks per call site; keeping `schema_warning` (removed with the field).
Decision: `Farm::open` returns a new `OpenError::OldLayout(msg)`, printed as `error: <msg>`, exit 1, for reads and writes alike. Rejected: reusing `NoFarm` (loses the distinct cause) and a warning-only read.
Decision: `store::resolve_farm_root` still recognises an old-layout farm as a farm (via `old_layout`), so discovery finds it and `open` refuses with the migrate message instead of "no .yaks farm". `yaks init` over one says "already exists, leaving it untouched" (unchanged, checked by hand).
Decision: `answer <id>@<shed>` into an old-layout shed needs no second check: `sheds::load_farm` errors on an old-layout farm, so the shed is `ShedFarm::Unreadable("old layout (pre-0.0.16 clone): ... not read; <fix>")` and `answer` already bails "shed X's farm is unreadable: ...". Same path gives `changes` ("nothing to show"), `show --sheds` (counted under "no farm of its own") and `inbox --sheds` (skipped). Rejected: a second `old_layout` test in `answer` (dead code).
Decision: `merge` with an old-layout SOURCE bails with the refusal (an error, exit 1), not `MergeOutcome::NoSource`; a flat source is copied byte for byte.
Decision: `base_farm` keeps flat `<id>.md` files only. A commit from before the migration has none, so it returns `None` and the caller's existing fallback runs (fork -> merge-base -> this checkout). Tests: forked-before-migration shed shows `vs: merge-base` with only its real note; with every base old-layout it shows `vs: checkout` and the note alone. Never "deleted every yak" or "every yak new".
Decision: doctor keeps `DuplicateId`, reworded for the flat layout (same frontmatter `id:` in two yak files, message names both files) and `invalid-status`; preflight gets a matching `duplicate-id` check code (`Check::DuplicateId`) in place of `duplicate-status`/`both-layouts`. Rejected: dropping duplicate detection.
Decision: `changes::classify`: a yak with both a deleted and an added path is now read as `Updated` (one path per yak, so it is only odd history), not dropped or paired as a move. Rejected: deleting the arm (the match must stay exhaustive).
Decision: `Status::dir()` -> `Status::as_str()`, all call sites.
Decision: the schema-3 refusal is documented once, in the `init` row of docs/cli.md (the farm-admin table), with the exact message; the `sheds`/`changes` rows say how an old-layout clone shows. README.md has only the `sheds` sentence (no init-area prose to hang it on).
Decision: tests/fixtures/farm (schema 3, status dirs) converted to the flat layout with the 0.0.16 `yaks migrate` on a temp copy; golden snapshots unchanged (no snapshot edits). status/commit unit fixtures converted to flat files with `status:` edits; their old-layout-move tests removed with the code they covered.

---
▸ 2026-10-12T22:58:09Z [bye-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-12T22:58:28Z [bye-1]

Summary (bye-1): removed the migration and the dual-layout read; added the permanent schema-3 refusal.
- Removed: `yaks migrate` (+ `Farm::migrate`, plan/outcome, link rewrite, `tracked_by_git`), `store::legacy`, `lock_for_migration`, `Status::dir` (now `as_str`), `schema_warning`; changes: `legacy_pair`, `Kind::Migrated`, `Migration`; status/commit migration line, message and `migrated` JSON key; preflight migration scope + `duplicate-status`/`both-layouts`; doctor's `DuplicateStatus`/`BothLayouts`; sheds: `old_layout` field, `is_old_layout`, `old_tree_task`, `flat_attachment_links`, `farm.layout`/`layout` JSON keys, `(old layout)` markers; `artifacts/<id>` branches in attachment_owner/path_owner/yak_id_of/merge.
- Kept/new: `store::old_layout` + refusal message (`<farm> uses the old yak layout (status directories, schema 3), which this yaks no longer reads: migrate it once with yaks 0.0.16 (npx -y @j15r/yaks@0.0.16 migrate), commit, then use this yaks again`) on `Farm::open` (all commands, reads too) and the merge source; an old-layout shed is `ShedFarm::Unreadable("old layout (pre-0.0.16 clone) ... not read ...")` in sheds/changes/show --sheds/inbox --sheds/answer@shed; a pre-migration base commit falls back to the next base. doctor/preflight keep `invalid-status` and a flat `duplicate-id`.
- Tests failed first (6 new, listed in an earlier note), now pass: tests/cli.rs refusal test, tests/sheds_layouts.rs (6 tests, rewritten around the two permanent behaviours), store/farm/sheds/preflight unit tests. tests/fixtures/farm converted to flat (snapshots unchanged).
- Gate: `cargo test --release` green, every suite (541 unit + 17 integration binaries), run after the last code edit and after `cargo fmt`; `yaks verify` PASS above.
- Docs: docs/cli.md and README.md updated (migrate rows, layout/old-layout/both-layouts/duplicate-status/migration sentences removed; refusal documented in the `init` row). Not touched (coordinator): .agents/skills/yaks/SKILL.md (lines 188, 209), AGENTS.md (12-13), .agents/skills/README.md:42 still mention `yaks migrate`/old layout; docs/tui.md and docs/README.md had no hit.
- Not done: docs/assets (coordinator), release 0.0.17 (no version bump: Cargo.toml is forbidden).

---
▸ 2026-10-12T22:58:28Z [bye-1]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-bbda` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-bbda --drop-request`, which removes this request note, so no new note is left behind.
