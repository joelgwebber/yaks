---
id: yak-3fd4.3
title: 'Faster frontmatter parse: libyaml/fast-path with fallback'
type: task
priority: 3
created: '2026-08-15T17:25:41Z'
updated: '2026-08-16T16:40:27Z'
labels:
- perf
parent: yak-3fd4
---

Complementary to the index (yak-3fd4 decision 4). PyYAML SafeLoader dominates parse cost; libyaml CSafeLoader is 2-4x and a hand-rolled frontmatter parser 2-6x. Task frontmatter is simple scalars + short lists, so a fast-path parser is viable with a PyYAML fallback on any anomaly. Speeds index build, reparse-of-changed, and point reads. Change is localized to model.load_task. Does NOT replace the index (still 6-7s at 50k alone), so land it either before or alongside the index.

---
▸ 2026-08-16T16:40:27Z
Done. Two-tier in model.load_task: (1) CSafeLoader (libyaml) as the loader via _yaml_load, with SafeLoader fallback when libyaml isn't built; (2) a conservative hand-rolled _fast_frontmatter for the exact subset dump_yaml emits (top-level key: scalar + key:/- item block lists, plain or single-quoted scalars). Scalar typing reuses PyYAML's own implicit resolver so accepted results are byte-identical to safe_load; anything outside the subset (folds, block/flow scalars, nesting, comments, bool/null/float/timestamp/exotic-int) returns None and defers to the loader. Also routed load_config through _yaml_load. Bench on 213-task repo corpus (frontmatter only): PyYAML 151us/task, CSafeLoader 17.6us (8.6x), fast-path 4.4us (34x); fast path covers 211/213 (2 wrapped-title bails now hit CSafeLoader). Tests: tests/test_fast_parse.py (differential vs safe_load, bail cases, roundtrip, live-corpus guard requiring >90% fast-path hit). Full suite 123 pass, ruff clean.
