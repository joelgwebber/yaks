---
id: yak-3fd4.6
title: Drop filename dot-hierarchy; parent as frontmatter field (Path A)
type: feature
priority: 2
created: '2026-08-15T19:35:17Z'
updated: '2026-08-16T14:59:55Z'
labels:
- perf
depends_on:
- yak-3fd4.5
parent: yak-3fd4
---

Path A layout change: drop the filename dot-trick; represent parentage with a 'parent:' frontmatter field. Status stays directory-partitioned.
- Keep existing IDs verbatim; dots stop meaning hierarchy (inert, stable tokens). New children get a fresh flat id + parent field. No reference rewriting; IDs never churn on reparent.
- Reparent = rewrite one parent field (O(1)); descendants unaffected.
- model changes: parent_id/find_children/next_child_number read the parent field, not the id string (need the loaded task). Ripples: yaklib.deps, yaktui.tree, TUI _navigate_to ancestor expansion, create --parent (flat id + set parent), reparent command.
- find_children becomes a field scan: O(children) via the index, O(N) without it (mild regression vs today O(entries) glob; fine at current scale; resolved by the index).
- Migration performed via the framework in 3fd4.5 (a versioned dot->parent step). Depends on 3fd4.5.

---
▸ 2026-08-16T14:59:55Z
Shorn. Parent-as-field refactor landed atomically:
- model: parent_of(task) reads the frontmatter field; find_children + new descendant_ids walk parent pointers; removed dot-based parent_id/next_child_number/find_descendants; create gives children flat IDs + a parent field.
- v3 migration (_migrate_v3_dot_to_parent; CURRENT_SCHEMA_VERSION=3) backfills parent from legacy dotted IDs, idempotent, IDs left untouched. Ran on this herd: 72 dotted children backfilled, schema -> 3.
- reparent.py collapsed from ~180 lines to a single parent-field rewrite + cycle check (ReparentResult); no more ID cascade / link / artifact rewrites. IDs are stable across reparent.
- consumers moved to parent pointers: tree.py (ancestors, descendant-universe, --parent-of scope, child-linking, sibling sort; apply_collapse is now depth-based and ID-scheme-independent), tui (_toggle_collapse/_navigate_to), rollup (effective_source takes a parent map), filter (--parent-of via descendant_ids in filter_tasks/build_tree, out of the per-task predicate), commands + mutate (create/show/reparent).
- tests: rewrote test_reparent for field semantics; updated test_rollup effective_source signature, test_filter --parent-of integration, test_cli_workflow child-parent-field; added a v3 migration test. 116 pass; ruff clean on touched files.
Intended behavior changes: new children get flat IDs (not parent.N); reparent no longer changes IDs/cascades and prints 'Reparented X under Y' / 'Promoted X to top-level'; --parent-of preserved via parent pointers.
