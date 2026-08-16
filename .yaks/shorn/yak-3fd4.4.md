---
id: yak-3fd4.4
title: 'View counts: memoize against index signature + capped display'
type: feature
priority: 3
created: '2026-08-15T17:25:44Z'
updated: '2026-08-16T16:56:46Z'
labels:
- perf
depends_on:
- yak-3fd4.2
parent: yak-3fd4
---

Resolves the Gmail counts question from yak-3fd4 decision 5. Given the in-memory index, exact per-view counts are cheap (O(N) filter, sub-10ms at 50k), so we keep them rather than abandoning counts like Gmail. Two safeguards: (a) MEMOIZE counts against the index signature so idle 500ms TUI polls recompute nothing (avoids O(N x V) across many pinned views); (b) CAP the displayed number for unbounded views (e.g. shorn shows NNN+) since a giant exact count is noise while small active counts stay exact. Relates to the View-list substrate yak-4473 (which renders per-view counts on the tab bar).

---
▸ 2026-08-16T16:56:46Z
Done. render.tab_counts is now memoized against (task-cache version, filter-spec snapshot) and computes from the in-memory cache instead of globbing the fs every frame. Previously the empty-filter path ran list(dir.glob('*.md')) per status per render (O(N) per frame — ~46k-file glob on the shorn tab at 50k, on every keypress + 500ms poll); now empty-spec is a straight per-status tally over app._task_cache and the active-spec path reuses build_tree over the cache. The memo makes idle polls and plain re-renders O(1) (avoids the O(N x V) recompute the yak flagged); it invalidates when the TUI bumps _task_cache_version on reload or when filter_spec changes. Both count call sites (draw_tabs, _position_inline_search_cursor) hit the memo, so cursor placement stays consistent. Added format_count(n, cap=999) -> 'NNN+' for unbounded views (chiefly shorn); applied at all three display sites (tabs, inline-search widths, filter total) so widths agree; small active counts stay exact. Computation stays exact + cheap, presentation is capped. Tests: tests/test_counts.py (cap, empty-spec tally, memo hit + invalidation on version/spec change, active-spec match count). Full suite 137 pass; new code ruff-clean.
