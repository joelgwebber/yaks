---
id: yak-b601
title: Sorting capability (flat vs tree) + Recent built-in view
type: feature
priority: 2
created: '2026-08-16T23:15:41Z'
updated: '2026-08-16T23:16:43Z'
parent: yak-4473
labels:
- ui
depends_on:
- yak-1b89
---

Tranche 4 of yak-4473 (deps yak-1b89). General sorting + the first non-status View.

Sorting is a general View capability: sort_by in {updated, created, priority, title, id} + sort_dir. A View that sorts renders FLAT (depth 0, no collapse tree), because a sort order cannot coexist with the parent/child tree; status Views stay tree views. build_tree gains a flat path (or a sibling flat builder).

Ships the Recent built-in View (moved here from yak-597c, which is now Working-set-only): all tasks, sort updated desc, limit N, flat; pinned by default — this also satisfies the substrate's day-one discoverability requirement. Recent is derived purely from the updated: frontmatter field; navigating TO a yak does NOT bump updated, so the list does not churn under you. It captures human edits and programmatic moves (shave/shorn/dep) alike. Honest gap by design: Recent is not 'recently viewed' — that is yak-c404.

Open (minor): default for limit N; relative 'updated within Nd' chip is a possible later add (absolute date-range filtering stays out of scope — painful in curses).
