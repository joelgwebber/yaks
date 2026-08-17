---
id: yak-b601
title: Sorting capability (flat vs tree) + Recent built-in view
type: feature
priority: 2
created: '2026-08-16T23:15:41Z'
updated: '2026-08-16T23:59:44Z'
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

---
▸ 2026-08-16T23:59:44Z
Done. General sorting + the Recent built-in view.

- View gained sort_by / sort_dir / limit + an is_flat property (yaktui/view.py); SORT_FIELDS = updated/created/priority/title/id. A View with sort_by set is FLAT; status Views leave it None and stay tree.
- yaktui/tree.py build_flat(): flat, sorted rows (status, task, 0, False) with no parent/child nesting. Reuses FilterSpec for matching (content + status), applies --parent-of as a descendant-set membership test over the in-memory items, sorts by _sort_key (priority numeric, rest string; ISO dates sort chronologically), applies limit. build_tree is untouched.
- tui._rebuild_task_list branches on the active view's is_flat: flat -> build_flat (no collapse); tree -> build_tree + apply_collapse as before.
- Recent view (view.recent_view()): all tasks, updated desc, flat, limit=RECENT_LIMIT (50). Pure derivation from updated: (navigating TO a yak doesn't bump updated, so no churn). default_views() = 3 status Views + Recent, so a fresh TUI ships Recent pinned in the tab bar (substrate discoverability). tui seeds self.views = default_views().
- view_counts counts Recent by its spec (all tasks; capped display); limit is presentation-only, not reflected in the count.

BEHAVIOR NOTE (regression from T3, now mitigated): build_tree's tab_status==SHORN root-sort-by-updated is no longer reached from the TUI (T3 passes tab_status=None so status can be a removable filter axis). The Shorn tab now sorts structurally (priority/id) like the other status tabs; recency browsing is the Recent view's job (updated desc across all statuses, shorn included). A dedicated 'recently shorn' flat view becomes a one-liner once Saved Views (yak-a373) lands. build_tree's tab_status param is unchanged (still used by tests).

Tests: tests/test_flat.py (sort desc+limit, priority asc, spec filtering, --parent-of scope, recent_view/default_views shape). Full suite 150 pass; view.py/tree.py/test_flat ruff-clean; import smoke shows [Hairy, Shaving, Shorn, 🕒 Recent]. Curses still blocks live TUI instantiation in tests -> wants a live smoke (esp. Recent ordering + that switching to Recent renders flat).
