---
id: yak-bf54.11
title: Support bidirectional sync
type: feature
priority: 1
created: '2026-05-01T15:40:38Z'
updated: '2026-06-19T16:47:10Z'
---

After using the external issue tracker sync for a while, I'm finding that I really do need
bidirectional sync. Let's work out a plan for doing this cleanly on top of our existing sync
workflow. The resolution UI works well; we just need bidirectional options, and perhaps a manual
edit affordance for merging by hand.

---
▸ 2026-05-03T16:39:22Z
Design doc landed at docs/design/sync.md — covers full bidirectional flow, capability matrix, mutation gating, and per-tracker hints. Implementation order: Phase 0 (sync_caps) → 4 (plan-time notes) → 1 (TUI direction toggle) → 2 (TUI merged_value edit) → 3 (skill apply) → 5 (mutation gating).

---
▸ 2026-05-04T03:40:44Z
Implementation landed across Phases 0/4/1/2/5/3. All 166 tests pass. Plugin 0.1.71. End-to-end: capability matrix (sync_caps.py) → plan-time notes (SKILL recipes) → TUI dialog with direction toggle/bucket nav/capability column → manual edit (e → merged_value/merged_body) → mutation gating (warn-and-re-plan with slaughter/last_synced carve-outs) → skill upstream-push cookbook with comment-up provenance round-trip.

---
▸ 2026-05-04T15:30:44Z
Live-tracker validation 2026-05-04: GH (label down + comment up + provenance dedup), Linear (title up via merged_value + comment up + provenance dedup), mutation gating refusal + --force-discard-pending + --last-synced carve-out + slaughter carve-out — all green on real /tmp/yaks-sync-{gh,linear} scratch repos. Closing bf54.11.
