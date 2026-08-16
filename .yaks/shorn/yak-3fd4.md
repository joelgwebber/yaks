---
id: yak-3fd4
title: Filesystem storage & performance design at scale
type: task
priority: 2
created: '2026-08-15T16:13:12Z'
updated: '2026-08-16T16:57:58Z'
labels:
- perf
---

Umbrella to think through the fs-native storage model and its performance as a herd grows large, WITHOUT ever needing an external database. To be broken into research (benchmarks) and design-change children as needed.

Context / conclusions so far:
- Views, sorting, filtering, pins, and counts all operate on the in-memory task cache, so they add negligible cost. The real cost is the pre-existing read-plus-YAML-parse-every-file-on-reload model (_refresh_task_cache), plus the one directory that grows without bound: shorn/. Active work (hairy/shaving) stays bounded by human attention.
- Principle to preserve: AUTHORITATIVE state lives in files (status=dir, id/parent in filename, frontmatter; grep-able, editable, CLI-scannable). DERIVED/personal state (view defs, pins, collapse, sort, any recency index) lives in rebuildable per-user cache. No second source of truth, no DB.

Candidate children:
- RESEARCH: benchmark cold scan + parse at 1k / 10k / 50k / 100k files on modern fs (APFS/ext4-htree/XFS) to fix the 'within reason' threshold on real numbers; separate readdir+stat cost from read+parse cost.
- BUILD: incremental cache keyed on per-file (mtime,size) — extend the existing _fs_sig idea so steady-state reload is O(changed) not O(N); stat-only enumeration, re-parse only changed files. Highest-leverage, pure fs primitives.
- DESIGN (conditional on benchmarks): time-shard shorn/ (e.g. shorn/YYYY/MM) to keep any single dir bounded and co-locate the recency hot-set. Costs: find_task_file searches more dirs; cross-status moves pick a shard. Do NOT do preemptively.
- DESIGN: memoize view counts against the fs signature (live counts on many pinned tabs are an O(N x views) pass otherwise).
- NOTE: id-to-path lookup is already O(dirs x entries) because a task's path depends on its mutable status; the TUI hides this behind its cache but the CLI pays it per command. Consider an id-to-path memo if large-N CLI use hurts.
- GUARDRAIL: any recency/updated index is a rebuildable memoization in the cache dir, guarded by the fs signature, never authoritative. Last resort, only if measured.

---
▸ 2026-08-15T17:25:23Z
BENCHMARK RESULTS (bench/fs_bench.py; macOS APFS, M-series arm64, py3.12, libyaml present). Synthetic corpora, realistic mix (93% shorn). Times in seconds unless noted:

metric                    1k      10k     50k
all_tasks scan+parse     0.18    1.77   12.57
readdir+stat only        0.002   0.016   0.092
parse pyyaml             0.17    1.71   12.91
parse cyaml libyaml      0.05    0.45    7.41
parse naive handrolled   0.03    0.26    6.16
index build full parse   0.18    1.83   14.06
index load + validate    0.002   0.029   0.150
find_task_file           16us    16us    22us   (flat, O(1))
find_children glob       0.7ms   4.2ms   20ms   (linear, O(entries))
index size               0.2MB   2.1MB   10.5MB

INTERPRETATION:
- Cost is ~100% YAML PARSING; readdir+stat is negligible (0.09s at 50k). all_tasks == parse_pyyaml.
- CORRECTION: find_task_file is O(1) (~16-22us, flat) via exact-name stat probes, NOT O(entries). Earlier note was wrong. Only glob/prefix ops (find_children, all_tasks, generate_id) are O(entries)/O(N).
- Un-indexed cold start is fine to ~5-10k (<2s) but untenable past it (12.6s at 50k; ~25-30s extrapolated at 100k). Frequent TUI restarts make this the core pain point.

DECISIONS (evidence-based):
1. PERSISTENT STAT-VALIDATED INDEX (linchpin; own child). 12.6s -> 0.15s at 50k (~80x); flat-ish to 100k+. Git-index-style, bulletproof:
   - Per-user cache (~/.cache/yaks/<slug>/index.json), derived, NEVER committed.
   - Per file: status, mtime_ns, size + list/filter/sort fields (id, title, type, priority, created, updated, labels, depends_on). Body NOT stored (reparse for full-text search; tradeoff to revisit).
   - Every load: readdir + stat each status dir, compare (mtime_ns,size); reparse mismatches, add new, drop missing; handle racy-mtime edge (file mtime == build time -> reparse) like git.
   - Self-healing: files are truth; stat runs every load; corrupt/missing index -> silent full rebuild. No clear-your-cache failure mode.
   - Atomic write (temp+rename) for TUI/CLI concurrency.
   - Lazy: only full-scan commands + TUI load it; point commands (show/update/move/shave/shorn by id) use find_task_file and skip it.
2. SHARDING shorn/ NOT needed at realistic scale (readdir+stat 50k = 0.09s; index removes find_children cost). Revisit only well past 100k or for git-diff hygiene, not perf.
3. SYMLINK indexes REJECTED. Problem already solved by in-memory index (sub-10ms filter/50k). Committed symlink index = second source of truth -> drift + merge conflicts (fails bulletproof). Uncommitted one is dominated by single-file JSON index (atomic write vs many-inode hazard). Replacing status dirs with symlinks loses cheap atomic rename for no gain.
4. FASTER PARSE (own child, complementary): libyaml 2-4x, fast-path frontmatter parser 2-6x vs PyYAML. Speeds index build + reparse-changed + point reads. Frontmatter is simple scalars/lists so a fast path with PyYAML fallback is viable. Does NOT replace the index.
5. VIEW COUNTS (Gmail question; own child): given the index, exact counts are cheap (sub-10ms/50k), so we need not abandon them. But (a) MEMOIZE against the index signature so idle polls recompute nothing (avoids O(N x V)); (b) CAP the display for unbounded views (NNN+) since giant exact numbers are noise. Computation cheap; presentation capped.

WITHIN-REASON THRESHOLD: un-indexed OK to ~5-10k; indexed OK to 100k+ with margin (likely ~500k before O(N) constants hit 1-2s). No external DB, no sharding, ever, for any realistic single-project herd.

Bench script kept at bench/fs_bench.py for re-measurement.

---
▸ 2026-08-15T18:09:34Z
MICRO-BENCHMARK (bench/fs_micro.py; same machine; warm-cache SSD; 50k files, 24.9MB):
- scandir only (names) 0.019s; scandir + stat 0.100s. Enumeration is genuinely ~100ms at 50k; stat is ~80ms of it. Confirmed real.
- read all bytes (Python) 1.296s; read+decode 1.359s; native 'find -exec cat' 1.399s. => Touching 50k small files costs ~1.3s REGARDLESS of language (per-file open/close syscall overhead, ~26us/file; not bandwidth, not parse). So the tightest-native full-rebuild FLOOR at 50k is ~1.3s (parse ~free on top), NOT the 6s Python naive-parse figure.
- Index load: JSON 7.2MB -> 0.032s; TSV 5.9MB -> 0.020s; packed read+split 0.025s. => Reading ONE compact file is ~40x faster than the native read floor and ~400x faster than full parse. JSON is only 32ms here; the JSON-is-slow worry does not bite at this scale. Serialization format is an implementation detail; the architecture (single derived, stat-validated, atomically-written file) is what matters.
- ls -1 sorted 0.066s; ls -1f unsorted 0.052s; ls -la (stat) 0.282s. => 'ls feels slow' is -l stat + terminal rendering + cold-cache/network, not readdir; warm-SSD enumeration is fast.

REFINED DECISIONS:
- KEEP status subdirs + file-per-task. Enumeration is ~free; subdirs give zero-cost status (encoded in path, read during readdir), atomic O(1) crash-safe status change via rename (no content rewrite), and human browsability. Dropping them solves a non-problem and forfeits these. Key the index by id and store status; a pure-rename status change is then detectable without reparse.
- The index delivers packed-store speed (one-file load) WITHOUT giving up file-per-task / grep / edit-in-place. A true packed store would kill the ~1.3s per-file-open floor but forfeits the core value props; not worth it.
- In-memory records ARE the query engine: once loaded, all filter/sort/count run in memory (sub-10ms/50k); files are touched only for stat-validation + reparse-of-changed. The on-disk index is just a fast-loading, cheaply-validated serialization of that cache (git-index shape; JSON now, binary/mmap later if wanted).
- RUST leverage (eventual): native parse ~free (rebuild floor drops to the ~1.3s open-bound, less if the 50k opens are parallelized); a single static binary removes Python interpreter+import startup paid on EVERY CLI call (a baseline-CLI cost the index cannot help); native fs-events (FSEvents/inotify) could replace stat-scan polling for invalidation. Keep the index format-agnostic so a Rust impl can swap in a binary/mmap layout. Do not block current work on Rust.

---
▸ 2026-08-15T19:35:09Z
LAYOUT DECISION: Path A. Status stays directory-partitioned (hairy/shaving/shorn/dead); parentage moves to a frontmatter 'parent:' field. Rationale: status is the most frequent QUERY (tabs) and MUTATION (shave/shorn), so keep it a cheap readdir + atomic rename that degrades gracefully when the index is cold/absent (notably every fresh clone / CI run in team mode, since the index is per-user + uncommitted). Hierarchy is shallow and secondary; put its index-dependence on the rarer axis. Path B (hierarchy=dirs, status=field) rejected: makes the common status query O(N) without the index, is a bigger/riskier migration, and its wins (arc-as-unit, archival, attachment co-location) are recovered by the in-memory index or addable later. status and hierarchy are orthogonal (a shorn child can sit under a shaving parent) and a file lives in one dir, so exactly one axis can be physical; give it to status.

PLAN = 3 features + 1 foundational fix (children):
- 3fd4.6 version-gated/atomic/resumable migration framework (NEW; foundational).
- 3fd4.5 parent-as-field, drop the dot-trick (NEW; needs 3fd4.6).
- 3fd4.2 index (against the final schema).
- 3fd4.3 faster parser.
- 3fd4.4 counts (memoize + cap).

PARENT-AS-FIELD SPECIFICS (locked):
- Keep existing IDs verbatim; dots stop meaning hierarchy (inert, stable). New children get a fresh flat id + parent field. NO reference rewriting; IDs never churn on reparent (fixes depends_on / source / external-ref breakage).
- Reparent = rewrite one parent field (O(1)); descendants unaffected (point at the stable id).
- model API change: parent no longer derivable from the id string; parent_id/find_children/next_child_number read the field (need the loaded task). Ripples: yaklib.deps, yaktui.tree, TUI _navigate_to ancestor expansion, create --parent, reparent.
- find_children becomes a field scan: O(children) via the index, O(N) without it (mild regression vs today O(entries) glob; fine at current scale; resolved by the index).

MIGRATION MECHANISM: REPLACE (see 3fd4.6). Today find_tasks_root runs _auto_migrate unconditionally every process start; _migrate_comment_blocks reads EVERY .md each call (O(N) reads/CLI call) and rewrites non-atomically (crash -> corruption). Replace with cheap version gate + ordered idempotent steps + atomic writes + stamp-on-success.

RESOLVED: Path A (not B); keep existing ids + inert dots (minimal migration, no ref rewriting); JSON index to start (format is impl detail); atomic writes everywhere. STILL OPEN (impl-level): saved-view storage (~/.config vs ~/.cache) [a373]; index file path under ~/.cache/yaks/<slug>/; schema_version storage (config.yaml key vs dedicated .yaks/schema file).

---
▸ 2026-08-15T19:35:53Z
NUMBERING CORRECTION: children came out as 3fd4.5 = migration framework, 3fd4.6 = parent-as-field (my prior note had the two numbers swapped). Dependency chain is: 3fd4.6 (parent-as-field) depends on 3fd4.5 (framework); 3fd4.2 (index) depends on 3fd4.6; 3fd4.4 (counts) depends on 3fd4.2. 3fd4.3 (parser) is independent.

---
▸ 2026-08-16T16:57:58Z
ARC COMPLETE — all children shorn; shearing the umbrella. The fs-native storage model is validated at scale with no external DB, no sharding, no symlinks. What landed:
- .1 benchmarks: parsing was ~100% of cost; readdir+stat negligible; find_task_file O(1). Drove every decision below.
- .5 migration framework: version-gated (.yaks/schema), ordered idempotent steps, atomic writes, resumable. Replaced the O(N)-per-invocation _auto_migrate.
- .6 parent-as-field (Path A): status stays directory-partitioned; hierarchy moved to a frontmatter parent: field; IDs now flat + stable; reparent is a one-field rewrite. Inert legacy dots.
- .3 faster parse: libyaml CSafeLoader + conservative hand-rolled fast-path (resolver-typed, PyYAML fallback). ~34x vs pure PyYAML on real corpus; covers 211/213.
- .2 persistent stat-validated index (linchpin): per-user ~/.cache/yaks/<slug>/index.json, git-index-shaped, self-healing, atomic, lazy, description stored so search stays in-memory. 50k cold start 5.06s->0.31s warm (~40x vs original 12.6s baseline).
- .4 view counts: memoized against cache version + spec; removed per-render fs glob; capped display (NNN+).
WITHIN-REASON verdict holds: comfortable to 100k+; revisit sharding/Rust only well beyond. Follow-ups noted (not blocking): unify TUI _scan_fs poll with index.sync()/fs-events; optional Rust rewrite (yak-2219). UI arc (yak-4473 View-list substrate + a373/597c/c404) is unblocked and can build on the flat-id + index foundation.
