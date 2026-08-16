---
id: yak-3fd4.2
title: Persistent stat-validated index (git-index-style)
type: feature
priority: 2
created: '2026-08-15T17:25:38Z'
updated: '2026-08-16T16:53:13Z'
labels:
- perf
depends_on:
- yak-3fd4.6
parent: yak-3fd4
---

The linchpin from yak-3fd4 benchmarking: a per-user, derived, never-committed index that makes cold start flat (12.6s -> 0.15s at 50k) and speeds full-scan CLI commands.

Design:
- Location: ~/.cache/yaks/<slug>/index.json (alongside the existing collapsed-ids ui-state). Atomic write (temp + rename) for TUI/CLI concurrency.
- Record per file: status, mtime_ns, size + list/filter/sort fields (id, title, type, priority, created, updated, labels, depends_on). Body/description NOT stored; reparse on demand for full-text search (revisit if search needs it in-index).
- Load path: json.load, then readdir + stat every status dir; compare (mtime_ns, size); reparse only mismatches, add new, drop missing. Handle racy-mtime edge (file mtime == index build time -> reparse), like git's index.
- Bulletproof by construction: authoritative files always win; stat runs every load; corrupt/missing index silently rebuilds; no clear-your-cache failure mode.
- Lazy consumption: only all_tasks-backed commands (list/search/next/tangled/stats/rollup) and the TUI load the index. Point commands (show/update/move/shave/shorn by id) keep using find_task_file (O(1)) and skip it. Optionally derive find_children/generate_id from the index when it is already loaded.
- Wire into the TUI _refresh_task_cache / _scan_fs / _fs_sig path (extends the existing per-scan signature to per-file).

---
▸ 2026-08-16T16:53:12Z
Done. New yaklib/index.py: git-index-shaped Index class (load/sync/tasks/write) backing model.all_tasks via a per-process singleton (_shared_index, keyed by resolved root). Cache at $XDG_CACHE_HOME/yaks/<slug>/index.json (model.cache_dir); atomic temp+rename write. Per-record: status, mtime_ns, size, and the FULL parsed task dict. Reconcile on each sync: scandir+stat all four dirs (first-wins matches find_task_file precedence), reuse cached task on mtime+size match, apply pure status renames without reparse, reparse new/modified/racy (mtime>=built_ns, git-style), drop missing. Corrupt/missing/old-version index -> silent full rebuild (bulletproof; no clear-your-cache). Laziness: only all_tasks consumers load it; point commands keep find_task_file O(1). Invalidation: save_task/move_task call _mark_index_stale so in-process mutations (CLI + unit tests) re-validate; TUI _refresh_task_cache calls refresh_index so external edits + direct deletes are caught on reload. YAKS_NO_INDEX=1 forces _all_tasks_direct (kept as reference + test oracle).

DEVIATION from the yak's 'body NOT stored' sub-decision: description IS stored in the index. FilterSpec.matches does substring search over description, so interactive/CLI search needs it in memory; the note explicitly flagged 'revisit if search needs it'. Cost is small (15.3MB vs ~10.5MB bodyless at 50k; warm load 0.31s vs predicted 0.15s) and it makes the index a transparent cache of load_task (simplest, most correct). 

BENCH (50k, realistic 93%-shorn mix, macOS APFS, with the 3fd4.3 fast parser live): direct parse-all cold 5.06s; index cold build 3.69s (writes 15.3MB); index warm load+stat-validate 0.31s => 16x vs direct, ~40x vs the original 12.6s PyYAML baseline. Flat-ish; comfortably in budget to 100k+.

Tests: tests/test_index.py (parity vs direct, description-indexed, YAKS_NO_INDEX, cache-file written, self-heal from corruption, missing-index rebuild, reconcile edit/delete, pure-rename-no-reparse, racy-reparse). conftest gained an autouse XDG_CACHE_HOME isolation fixture. Full suite 133 pass; new files ruff-clean.

FOLLOW-UP (optional, not blocking): TUI still double-stats on change (_scan_fs poll to detect + index sync to reconcile) and the 500ms poll stats all files (~90ms/50k, pre-existing). Could unify by having sync() return changed and drive the poll off it, or move to fs-events. Noted in 3fd4 as optional.
