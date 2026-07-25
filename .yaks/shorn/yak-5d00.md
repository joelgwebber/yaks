---
id: yak-5d00
title: Update skills to ensure that yaks don't leak into external PRs, issues, etc.
type: task
priority: 2
created: '2026-07-25T21:02:52Z'
updated: '2026-07-25T21:59:02Z'
---

Check both skills to ensure they're clear on this. In practice, repos with external issue trackers and such don't use the yaks themselves in a team setting. If the user wants to mention yaks in their PR descs that's fine (eg, I'll do it for *this* repo), but as a general rule we should avoid that.

---
▸ 2026-07-25T21:59:02Z
Done. yak-tracker skill already airtight (hard rule against [yaks:...]/yak-ID markers upstream; 'yaks rollup --keys' PR helper; 'we never write yak IDs anywhere') — no change. Fixed the gap in the yak skill: the old Team bullet said 'referencing yak IDs in commit messages and PRs is welcome', which leaks when a broader audience/external tracker is involved. Reworked it and added a 'Keep yaks out of external-facing surfaces' subsection: default = keep yak IDs/[yaks:...] out of PR titles/descriptions and external trackers; use the external key from 'yaks rollup --keys' instead; referencing yaks externally is opt-in only (e.g. this yaks-native repo). Bumped 0.1.78->0.1.79 in lockstep (skill = shipped payload). 117 tests pass.
