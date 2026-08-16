---
id: yak-3fd4.2
title: Persistent stat-validated index (git-index-style)
type: feature
priority: 2
created: '2026-08-15T17:25:38Z'
updated: '2026-08-15T19:35:57Z'
labels:
- perf
depends_on:
- yak-3fd4.6
---

The linchpin from yak-3fd4 benchmarking: a per-user, derived, never-committed index that makes cold start flat (12.6s -> 0.15s at 50k) and speeds full-scan CLI commands.

Design:
- Location: ~/.cache/yaks/<slug>/index.json (alongside the existing collapsed-ids ui-state). Atomic write (temp + rename) for TUI/CLI concurrency.
- Record per file: status, mtime_ns, size + list/filter/sort fields (id, title, type, priority, created, updated, labels, depends_on). Body/description NOT stored; reparse on demand for full-text search (revisit if search needs it in-index).
- Load path: json.load, then readdir + stat every status dir; compare (mtime_ns, size); reparse only mismatches, add new, drop missing. Handle racy-mtime edge (file mtime == index build time -> reparse), like git's index.
- Bulletproof by construction: authoritative files always win; stat runs every load; corrupt/missing index silently rebuilds; no clear-your-cache failure mode.
- Lazy consumption: only all_tasks-backed commands (list/search/next/tangled/stats/rollup) and the TUI load the index. Point commands (show/update/move/shave/shorn by id) keep using find_task_file (O(1)) and skip it. Optionally derive find_children/generate_id from the index when it is already loaded.
- Wire into the TUI _refresh_task_cache / _scan_fs / _fs_sig path (extends the existing per-scan signature to per-file).
