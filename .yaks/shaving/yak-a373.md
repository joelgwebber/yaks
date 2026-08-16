---
id: yak-a373
title: 'Saved views: pinnable, sortable filter presets'
type: feature
priority: 3
created: '2026-08-12T16:28:50Z'
updated: '2026-08-15T16:13:42Z'
labels:
- ui
depends_on:
- yak-4473
---

Would be nice to be able to pin the results of more complex searches, so that they're easily accessible and updated live in the UI.

---
▸ 2026-08-15T16:13:37Z
DESIGN (locked). Reframed from 'saved search lists' to Saved Views, built on the View-list substrate yak-4473.
- A saved view is a named FilterSpec (+ sort/limit) persisted to user-specific storage (decision: user-specific first; team/shared later).
- Creation flow: build a filter with f or /, then 'save current filter as view' and name it. A view is just a named snapshot of the live filter, so it re-evaluates from disk each reload and updates live for free.
- Editing while in a view forks an ephemeral 'modified' view (* on the chip); save updates the view or forks a new one; Esc reverts to the saved spec.
- Management (pin/unpin, reorder, rename, delete) happens in the v picker; pinned views appear on the tab bar with pin order = tab order.
- Open sub-decision (implementation-level, non-blocking): which user-specific location, a personal file under ~/.config/yaks vs the per-project cache under ~/.cache/yaks. Overlaps with yak-9835 (in-app settings/config editor).
