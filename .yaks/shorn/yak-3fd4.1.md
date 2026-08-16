---
id: yak-3fd4.1
title: Benchmark fs cost model (scan/parse/stat/index)
type: task
priority: 2
created: '2026-08-15T17:25:30Z'
updated: '2026-08-15T17:26:07Z'
labels:
- perf
---

Measure the operations that dominate CLI/TUI cost so storage decisions rest on numbers. bench/fs_bench.py generates synthetic corpora (1k/10k/50k, realistic 93%-shorn mix) and times: all_tasks scan+parse, readdir+stat only, PyYAML vs libyaml vs hand-rolled parse, single-id find_task_file, children glob, and persistent-index build + load+validate. Results + interpretation recorded on the parent yak-3fd4. Headline: parsing dominates; a stat-validated index cuts 50k cold start 12.6s -> 0.15s; find_task_file is O(1); sharding + symlinks unnecessary at realistic scale.

---
▸ 2026-08-15T17:26:07Z
Shorn: built bench/fs_bench.py and measured 1k/10k/50k on macOS APFS/arm64. Full results + interpretation live on parent yak-3fd4. Key outputs that drove the design: (1) parsing is ~100% of cost, readdir+stat negligible; (2) stat-validated index -> 0.15s cold start at 50k vs 12.6s (~80x); (3) find_task_file is O(1); (4) libyaml/fast-path parse 2-6x; (5) sharding + symlinks unnecessary at realistic scale. Spawned children 3fd4.2 (index), 3fd4.3 (parser), 3fd4.4 (counts). Bench script retained for re-measurement.
