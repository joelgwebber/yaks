---
id: yak-1b89
title: Activate/edit/save view; Esc reverts to saved (one filter)
type: feature
priority: 2
created: '2026-08-16T23:15:40Z'
updated: '2026-08-16T23:50:02Z'
parent: yak-4473
labels:
- ui
depends_on:
- yak-5892
---

Tranche 3 of yak-4473 (deps yak-5892). Wire the single live filter to Views — no base+overlay stack.

Activating a View loads its spec into the one live filter (the thing f and / already edit). Editing forks an ephemeral 'modified' View, shown with a * on the chip; you may loosen OR tighten any criterion, including removing base criteria (status is just another removable axis). 'Save' persists: update the active View, or fork a new named one. Esc reverts the live filter to the active View's saved spec (change from today, where Esc clears outright); 'empty' is simply the spec of an All view.

Reuses the entire existing filter path (FilterSpec + build_tree); the only new state is 'which View is active' + 'is it modified'. Status widening forks a clearly-marked ephemeral cross-status View (the * + Esc-revert are the guardrail).

Done when: switching Views swaps the live filter; edit/save/Esc-revert behave as specified.

---
▸ 2026-08-16T23:50:02Z
Done. Wired the single live filter to Views (one-filter model, no base+overlay stack).

- View gained a spec: FilterSpec field (yaktui/view.py); builtin_status_views() seeds each status View with spec=FilterSpec(statuses={status}). Status is now just a (removable) axis of the live filter.
- tui.py: filter_spec initializes from the active view's spec. New helpers _set_view (load a view's spec into the live filter), _activate_view (=_set_view+_reset_list), _is_view_modified (live filter != active view's saved spec), _revert_filter_to_view. _switch_tab/click activate views; _navigate_to loads the target status View's spec; mutate.create_task uses _set_view(0).
- _rebuild_task_list now drives entirely off the spec: build_tree(root, None, filter_spec) — status comes from the spec (all statuses when unset, so removing status widens). Collapse gating fixed to use the CONTENT spec (status stripped) so a bare status scope still folds like before (build_tree already used content_spec internally).
- Esc reverts the live filter to the active view's spec (was: clear outright).
- render.py: the active tab shows a trailing * when modified; the filter chip shows only when modified (since the spec now always carries a status), with the live match count.
- COUNT SEMANTICS CHANGED: view_counts now counts each View by its OWN spec, independent of the live filter (stable per-view sizes), because activating a View replaces the live filter, so the old cross-tab live-preview no longer matches what you'd see. Memo now keys on (data version, views signature), not the filter -> editing the filter never recomputes counts.

SCOPE NOTE: delivered activate/edit/Esc-revert (the in-memory one-filter semantics). 'Save as a named view' / persistence is deferred to yak-a373 (which owns the ~/.config storage from yak-6b51); there is nothing to persist for the built-in status Views. This unblocks yak-b601 (sorting + Recent), which needs 'Views carry a spec and activating loads it'.

Tests: tests/test_counts.py rewritten for the new semantics (per-view counts, filter-independent memo, *-on-modified-active). Full suite 144 pass; view.py/test_counts ruff-clean; no new lint in shared files; import smoke green. Curses blocks full TUI instantiation in tests, so runtime behavior wants a live smoke by the user.
