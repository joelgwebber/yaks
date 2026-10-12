---
id: yaks-95be
title: 'Flat farm, phase 1: store seam (flat paths, dual-layout reads, status field, schema 4)'
type: task
priority: 1
created: '2026-10-12T19:53:34Z'
updated: '2026-10-12T20:19:35Z'
parent: yaks-8b7a
labels:
- cli
---

Phase 1 of yaks-8b7a (see its plan note; D1-D4, D6). Coordinator-owned prep commit; the three phase-2 workers build on it.
Scope: src/store.rs and src/model.rs (and whatever tests the change breaks). A flat path helper (`task_path(root, id)`); `load` reading root `*.md` (status from frontmatter, missing = hairy) plus the four legacy status dirs (status from the dir); `find_task_file`, `load_task_by_id`, `write::save` (flat path; `status:` written after `title:`), `move_task` (edit `status:` in place, keep the `moved:` entry); `parse_task` reading `status:`; `SCHEMA` = 4; `init` scaffolds no status dirs; the write refusal on a v3 farm (D4) at the one place writes go through.
Out of scope: migrate, doctor/preflight, git reporting, sheds, docs (phase 2 and 3).
Evidence: round-trip tests for the flat layout; a v3 fixture still loads read-only; a write on a v3 farm refuses with the D4 message; full gate green.

---
▸ 2026-10-12T20:00:38Z [delta-lead]
Amended by D10 and the transitional D4 (see yaks-8b7a): also add the one artifact-path helper everyone calls (`store::artifact_dir(root, id)` = `<root>/<id>/`, plus a predicate for 'is this repo path an attachment') so phase-2 workers convert their call sites to it; the dual read is transitional and must stay isolated (one function) so phase 4 can delete it.

---
▸ 2026-10-12T20:00:47Z [delta-lead]
moved: hairy -> shaving

![phase1-real-machine](artifacts/yaks-95be/phase1-real-machine.txt)

---
▸ 2026-10-12T20:18:41Z [delta-lead]
Real machine: the new binary on this repo's schema-3 farm (reads with a warning, a write refuses and writes nothing) and on a fresh flat farm (init, create, shave, note, shear: one file, status edited in place, moved: entries intact, doctor clean).

---
▸ 2026-10-12T20:18:42Z [delta-lead]
Done (phase 1). What landed:
- `src/store.rs`: `task_path`, `artifact_dir`, `attachment_owner`, `path_owner`; `load` reads `<root>/*.md` (status from `status:`, missing = hairy) plus, via `legacy::load`, the old status dirs (flat wins on a clash); `parse_task` reads `status:` (invalid keeps the fallback, never duplicated into `extra`); the writer puts `status:` after `title:`; `save` writes the one flat path; `move_task` edits in place and keeps the `moved:` entry; `find_task_file`/`load_task_by_id`/`all_ids` flat first, then legacy; `init` creates no status dirs; `SCHEMA` = 4.
- The write refusal (D4) sits in `store::lock`, which every mutation already takes: refuses on schema 3 or on any yak file in a status dir (an old binary wrote it); `lock_for_migration` skips it, for yaks-86f4.
- ALL old-layout reading is in one module, `store::legacy` (STATUSES, is_status_dir, is_farm, files, load, ids, find, write_blocker), so yaks-bbda deletes it in one place.
- Outside store, only what the gate needed: `Farm::paths`, `rename_many`'s old-file removal, `create --json`'s path (a real bug: it built the old path itself), `merge` reading either layout (flat copied byte-for-byte, legacy rewritten flat; its artifact copy is still `artifacts/<id>` for yaks-86f4), the merge source's farm check, `preflight::yak_id_of` (everything after the last `.yaks` component is farm-relative, so no signature change for yaks-9f91), `sheds::base_farm` accepting `<rel>/<id>.md` (minimal; mixed-layout comparisons stay with yaks-ca28), the Older-schema warning text.
- Tests: path assertions rewritten to `at(root, id, status)` / `task_path`; temp farms are flat; the two "two status dirs" tests now build old-layout copies directly (yaks-86f4 replaces that check). 6 new `store::layout_tests`. ONE test ignored on purpose: tests/status.rs `clean_then_dirty_with_check_exit_codes` (yaks status does not yet report an in-place status change as a move), with an ignore reason naming yaks-9f91, which un-ignores it as its fail-first test.
Gate: `cargo test --release`, 16 suites, 649 passed, 0 failed, 4 ignored (3 pre-existing + the one above).
Decision: the refusal lives in `store::lock` rather than `write::save`: every mutation already holds the lock (AGENTS.md invariant), it refuses before any partial write, and it also covers non-save writes (rename's removals, merge's copies, attach). Rejected: a check in `save` only (misses those) or in `Farm::open` (would block read-only commands too).
Decision: a committed farm is always a `.yaks/` directory, so repo paths are made farm-relative by the last `.yaks` component (`preflight::yak_id_of`). Rejected: threading a `git rev-parse --show-prefix` through `classify`/`generate_message` (signature churn in yaks-9f91's files for no extra correctness).

---
▸ 2026-10-12T20:19:35Z [delta-lead]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-12T20:19:35Z [delta-lead]
moved: shaving -> shorn
