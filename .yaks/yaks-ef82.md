---
id: yaks-ef82
title: Human-friendly timestamp rendering
status: hairy
type: feature
priority: 3
created: '2026-08-27T16:15:00Z'
updated: '2026-10-17T04:26:52Z'
parent: yaks-faf5
---

In header fields like created/updated; but especially comment timestamps, which are currently raw RFC/UTC.
Bonus points for rendering in the local timezone.

---
▸ 2026-10-17T04:26:52Z [delta-lead]
Scope agreed 2026-10-17 (yaks-e9de): relative shorthand for nearby times ("5m ago", "3h ago", "yesterday"), an absolute local-time date beyond about a week, with the timezone shown where absolute; used for comment headers (yaks-93cd) and the Created/Updated fields. "Now" must be injectable (an `App` clock, a fixed instant in tests) so insta snapshots and docshots stay deterministic. Today `humanize_date` (src/tui/detail.rs) formats UTC with no zone label.
