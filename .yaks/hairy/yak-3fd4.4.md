---
id: yak-3fd4.4
title: 'View counts: memoize against index signature + capped display'
type: feature
priority: 3
created: '2026-08-15T17:25:44Z'
updated: '2026-08-15T19:35:57Z'
labels:
- perf
depends_on:
- yak-3fd4.2
parent: yak-3fd4
---

Resolves the Gmail counts question from yak-3fd4 decision 5. Given the in-memory index, exact per-view counts are cheap (O(N) filter, sub-10ms at 50k), so we keep them rather than abandoning counts like Gmail. Two safeguards: (a) MEMOIZE counts against the index signature so idle 500ms TUI polls recompute nothing (avoids O(N x V) across many pinned views); (b) CAP the displayed number for unbounded views (e.g. shorn shows NNN+) since a giant exact count is noise while small active counts stay exact. Relates to the View-list substrate yak-4473 (which renders per-view counts on the tab bar).
