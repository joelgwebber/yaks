---
id: yaks-837d
title: A recurring yak sheared after its due day regrows at the next tick for an occurrence the late round covered
status: hairy
type: bug
priority: 3
created: '2026-11-09T15:37:41Z'
updated: '2026-11-09T15:37:41Z'
tags:
- cli
---

Seen 2026-11-09 in cowpaths/joel: `fullstory-cc8e` (`every: P1D`, regrown 2026-11-07, due 2026-11-08) was worked and sheared on 2026-11-09. `yaks tick --dry-run` then says it would regrow "for 2026-11-09", so the next session the same day runs a second round. Moving `due:` to 2026-11-09 doesn't help, since `regrow-due` counts from `regrown:`.

The skill says "shear it on its due day and it waits for the next one"; there is no way to say "this late round covered today". Options: count from the shear time when it is after the latest occurrence, or a `yaks shorn --covers <date>` style flag. Snooze applies only to open yaks.
