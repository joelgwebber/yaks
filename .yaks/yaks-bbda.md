---
id: yaks-bbda
title: 'Flat farm, phase 4: remove the migration and the dual-layout read once every farm is migrated'
status: shaving
type: task
priority: 2
created: '2026-10-12T20:00:38Z'
updated: '2026-10-12T22:29:49Z'
parent: yaks-8b7a
depends_on:
- yaks-3a56
labels:
- cli
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
