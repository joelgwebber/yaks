---
id: yak-3fd4.3
title: 'Faster frontmatter parse: libyaml/fast-path with fallback'
type: task
priority: 3
created: '2026-08-15T17:25:41Z'
updated: '2026-08-15T17:25:41Z'
labels:
- perf
---

Complementary to the index (yak-3fd4 decision 4). PyYAML SafeLoader dominates parse cost; libyaml CSafeLoader is 2-4x and a hand-rolled frontmatter parser 2-6x. Task frontmatter is simple scalars + short lists, so a fast-path parser is viable with a PyYAML fallback on any anomaly. Speeds index build, reparse-of-changed, and point reads. Change is localized to model.load_task. Does NOT replace the index (still 6-7s at 50k alone), so land it either before or alongside the index.
