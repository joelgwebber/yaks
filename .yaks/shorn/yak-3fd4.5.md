---
id: yak-3fd4.5
title: Version-gated, atomic, resumable migration framework
type: task
priority: 2
created: '2026-08-15T19:35:17Z'
updated: '2026-08-16T05:21:31Z'
labels:
- perf
parent: yak-3fd4
---

Replace the migration mechanism (foundational; prerequisite for 3fd4.6). Today find_tasks_root() runs _auto_migrate() unconditionally on every process start; _migrate_comment_blocks reads EVERY .md each time (O(N) reads per CLI call, worst for agent/CLI loops) and rewrites via non-atomic write_text (crash -> corruption). Replace with:
- Cheap version GATE (schema_version in config.yaml, or a dedicated .yaks/schema file) read once; if current, skip entirely (no O(N) scan). No marker = version 0 -> run all.
- Ordered, idempotent migration steps; retrofit existing yaml->md and comment-block migrations as versioned steps; add the dot->parent step (for 3fd4.6).
- Atomic per-file writes: add a shared atomic_write helper (temp + os.replace) and route save_task/move_task/migration/index through it (also fixes a latent non-atomic save_task corruption risk).
- Stamp the new version only after all steps succeed; resumable + idempotent.
Outcome: reliable migrations AND removal of the per-call O(N) read cost (serves the baseline-CLI-perf goal). The atomic_write helper is reused by the index (3fd4.2).

---
▸ 2026-08-16T05:21:31Z
Shorn. Implemented in model.py + commands.py + tests:
- atomic_write(path,text): temp file in same dir + os.replace, cleans up temp on error; never a torn target. Routed save_task through it (fixes a latent non-atomic save corruption risk); move_task inherits it.
- Schema versioning: CURRENT_SCHEMA_VERSION=2 with read/write_schema_version backed by a single-int .yaks/schema file (atomic write).
- _auto_migrate is now version-GATED: herd >= CURRENT returns immediately (one small read; removes the old O(N)-read-on-every-invocation cost). Otherwise run ordered steps whose version > current, stamping after EACH step (resumable). Unmarked herd = v0 -> runs all (idempotent no-ops) then stamps.
- Existing migrations retrofitted as ordered idempotent steps _MIGRATIONS = [(1, yaml->md), (2, comment-blocks)], both using atomic_write; pure _migrate_comment_blocks unchanged. v3 dot->parent reserved for 3fd4.6.
- cmd_init stamps CURRENT so fresh herds never scan.
CONTRACT CHANGE: legacy content migrates on version bumps, not on every load. Replaced the old every-load test with test_auto_migrate_runs_when_schema_behind + _skipped_when_schema_current + test_init_stamps_current_schema_version. 119 tests pass; ruff clean for touched files (pre-existing E501 in the mandate string left alone).
DECISION: schema stored in dedicated .yaks/schema file (not a config.yaml key) to avoid clobbering user config formatting.
FOLLOW-UP (3fd4.6): append the v3 dot->parent step to _MIGRATIONS and bump CURRENT to 3.
