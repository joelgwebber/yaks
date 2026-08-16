---
id: yak-3fd4.6
title: Drop filename dot-hierarchy; parent as frontmatter field (Path A)
type: feature
priority: 2
created: '2026-08-15T19:35:17Z'
updated: '2026-08-15T19:35:57Z'
labels:
- perf
depends_on:
- yak-3fd4.5
---

Path A layout change: drop the filename dot-trick; represent parentage with a 'parent:' frontmatter field. Status stays directory-partitioned.
- Keep existing IDs verbatim; dots stop meaning hierarchy (inert, stable tokens). New children get a fresh flat id + parent field. No reference rewriting; IDs never churn on reparent.
- Reparent = rewrite one parent field (O(1)); descendants unaffected.
- model changes: parent_id/find_children/next_child_number read the parent field, not the id string (need the loaded task). Ripples: yaklib.deps, yaktui.tree, TUI _navigate_to ancestor expansion, create --parent (flat id + set parent), reparent command.
- find_children becomes a field scan: O(children) via the index, O(N) without it (mild regression vs today O(entries) glob; fine at current scale; resolved by the index).
- Migration performed via the framework in 3fd4.5 (a versioned dot->parent step). Depends on 3fd4.5.
