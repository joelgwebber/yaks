---
id: yaks-95be
title: 'Flat farm, phase 1: store seam (flat paths, dual-layout reads, status field, schema 4)'
type: task
priority: 1
created: '2026-10-12T19:53:34Z'
updated: '2026-10-12T20:00:47Z'
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
