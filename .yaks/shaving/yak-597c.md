---
id: yak-597c
title: Recent + Working-set views
type: feature
priority: 2
created: '2026-08-11T20:52:19Z'
updated: '2026-08-15T16:13:42Z'
labels:
- ui
depends_on:
- yak-4473
---

It would be really helpful to be able to quickly get back to "recently viewed" yaks. Eg, when you do some searching, find something, go back to what you were doing, and need to find it again, it's a PITA to do the search again.

Bonus points: additional affordance to "pin" yaks to the top of the history, for a kind of "working set".

---
▸ 2026-08-15T16:13:41Z
DESIGN (locked). Split the original 'viewing history' idea into two built-in Views on the View-list substrate yak-4473, and spun the true 'recently viewed' history out to yak-c404 (global back/forward nav).
- Recent view: derived purely from the updated: frontmatter field. Query = all tasks, sort updated desc, limit N, flat. Zero new persistence; navigating TO a yak does NOT bump updated, so the list does not churn under you. Captures human edits and programmatic moves (shave/shorn/dep/etc.) alike. Pinned by default (also covers substrate discoverability).
- Working set: explicit user-pinned yaks, stored as an ORDERED id list in user-specific UI state, NOT as labels/tags (tags are unordered, and a pin toggle in frontmatter would bump updated and pollute Recent). Pin/unpin affordance on a list row; a built-in 'Working set' view reads the list. Spiritually identical to collapsed_ids.
- Honest gap by design: Recent is not recently-viewed. 'Looked at but did not change' is covered by yak-c404, not here.
- Generalization note: reserved-namespace tags + a view remain the right pattern for FUTURE shared, intrinsic, unordered booleans (e.g. needs-review); pinning does not use that pattern because it needs order + personal scope.
