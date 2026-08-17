---
id: yak-597c
title: 'Working set: user-pinned yaks (ordered)'
type: feature
priority: 2
created: '2026-08-11T20:52:19Z'
updated: '2026-08-17T01:56:28Z'
labels:
- ui
parent: yak-4473
depends_on:
- yak-6b51
---

Child of yak-4473 (deps yak-6b51 for the pin infra). Originally 'Recent + Working-set'; Recent moved to the substrate sorting tranche yak-b601 (it is nearly free once sorting exists and serves as the pinned-by-default discoverability view). This yak is now the Working set only.

Motivation: quickly get back to a small hand-curated set of yaks you're actively juggling — a 'working set' distinct from whatever the tab/filter shows.

DESIGN (locked):
- Explicit user-pinned yaks, stored as an ORDERED id list in user-specific state, NOT as labels/tags. Tags are unordered, and a pin toggle in frontmatter would bump updated: and pollute Recent. Spiritually identical to collapsed_ids, but DURABLE user intent -> lives under ~/.config/yaks/<slug>/ (with the view defs from yak-6b51), not the rebuildable ~/.cache.
- A pin/unpin affordance on a list row; a built-in 'Working set' View reads the ordered list (flat, in pin order).
- Generalization note: reserved-namespace tags + a View remain the right pattern for FUTURE shared, intrinsic, unordered booleans (e.g. needs-review); pinning does not use that pattern because it needs order + personal scope.

Honest gap (unchanged): 'looked at but did not change' recently-viewed is yak-c404, not here.

---
▸ 2026-08-17T01:56:28Z
Done. Working set — the new id-list view kind.

- views_store: working_set.json in config_dir (durable), load/save (never raise, atomic), toggle_working_set (append new stars to the end, remove on re-toggle; order preserved). NOT labels/frontmatter (unordered + would churn Recent).
- view.working_set_view() (key=working-set, builtin, pinned) added to default_views() -> a fresh TUI shows ⭐ Working set; reconcile appends it for existing users (new built-in).
- TUI loads self.working_set on startup; * key toggles the cursor yak's membership (persist + notify + rebuild if viewing it). Rendering + counting are special-cased by the working-set key (explicit ordered membership from self.working_set, flat), not by spec; the count memo key now includes the working-set signature so the tab count updates on star/unstar.
- Help documents *; membership rows are the starred ids that still exist, in star order.

Scope note: the Working set view ignores the live filter for now (shows all stars); filtering-within-working-set is a possible later nicety. A star marker in other list views is a nice follow-up (feedback is currently via the notification + the ⭐ tab).

Tests: default_views includes recent+working-set; toggle add/remove/order; working_set load/save/fallback; working-set tab count (present starred ids only). Adjusted the 4-view assumptions in test_flat/test_views_store to 5 views. Full suite 165 pass; ruff-clean (only pre-existing E402/E741/F401 in shared files). Curses UI -> wants a live smoke (* on a yak, open ⭐ Working set, restart to confirm persistence).
