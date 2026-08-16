---
id: yak-6b51
title: View picker + pin/reorder/rename/delete + persistence
type: feature
priority: 2
created: '2026-08-16T23:15:40Z'
updated: '2026-08-16T23:16:25Z'
parent: yak-4473
labels:
- ui
depends_on:
- yak-5892
---

Tranche 2 of yak-4473 (deps yak-5892). The View management surface + persistence.

Key v opens a picker listing every View. Pin/unpin, reorder, rename, delete happen there (unlimited space). Pinned Views = the tab bar; pin order = tab order, so 'reorder tabs' and 'manage pins' are one operation. Non-pinned Views live only in the picker; the picker is also the tab-strip overflow affordance. Built-in status Views are un-deletable (pin/reorder only).

Persistence: pins + order + any custom View defs go to USER-SPECIFIC DURABLE storage under $XDG_CONFIG_HOME/yaks/<slug>/ (default ~/.config/yaks). Rationale: these are user-authored intent (losing them loses work), unlike the rebuildable ~/.cache (index, collapsed_ids). Atomic temp+rename writes, mirroring the index.

Done when: status Views can be reordered/pinned via v and the order persists across restarts.
