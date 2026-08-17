---
id: yak-33e9
title: Rename 'Working set' -> 'Starred' (label only)
type: task
priority: 3
created: '2026-08-17T14:17:49Z'
updated: '2026-08-17T14:41:29Z'
parent: yak-1bef
labels:
- ui
---

s/Working set/Starred/ for clarity + tab real-estate. LABEL ONLY — keep the internal view key 'working-set' (persistence + special-casing key on it; changing the key would break existing views.json reconcile).

Touch: view.working_set_view() name '⭐ Working set' -> '⭐ Starred'; tui._toggle_working_set notification text ('added to/removed from working set' -> 'starred'/'unstarred'); render.py help line.

WRINKLE (shared with the emoji-swap sibling): views_store.reconcile prefers the PERSISTED name over the code default (replace(base, name=e.get('name', base.name))), so users who've already written views.json via a picker action won't see the new label. Fix cleanly by only persisting a built-in's name when the user actually renamed it (store name omitted/None for un-renamed built-ins so the code default shows), or migrate. Fresh installs are unaffected.

---
▸ 2026-08-17T14:41:29Z
Done. Renamed the Working set View label -> '⭐ Starred' (view.working_set_view); internal key stays 'working-set'. Notification now 'starred'/'unstarred'; help lines say 'Star / unstar (Starred view)'. Shipped with the reconcile propagation fix (below) so the new label reaches already-customized herds.
