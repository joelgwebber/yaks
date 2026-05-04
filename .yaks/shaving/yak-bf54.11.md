---
id: yak-bf54.11
title: Support bidirectional sync
type: feature
priority: 1
created: '2026-05-01T15:40:38Z'
updated: '2026-05-03T16:39:22Z'
---

After using the external issue tracker sync for a while, I'm finding that I really do need
bidirectional sync. Let's work out a plan for doing this cleanly on top of our existing sync
workflow. The resolution UI works well; we just need bidirectional options, and perhaps a manual
edit affordance for merging by hand.

---
▸ 2026-05-03T16:39:22Z
Design doc landed at docs/design/sync.md — covers full bidirectional flow, capability matrix, mutation gating, and per-tracker hints. Implementation order: Phase 0 (sync_caps) → 4 (plan-time notes) → 1 (TUI direction toggle) → 2 (TUI merged_value edit) → 3 (skill apply) → 5 (mutation gating).
