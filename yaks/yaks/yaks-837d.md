---
id: yaks-837d
title: A recurring yak sheared after its due day regrows at the next tick for an occurrence the late round covered
status: hairy
type: bug
priority: 3
created: '2026-11-09T15:37:41Z'
updated: '2026-11-18T14:43:02Z'
tags:
- cli
---

Seen 2026-11-09 in cowpaths/joel: `fullstory-cc8e` (`every: P1D`, regrown 2026-11-07, due 2026-11-08) was worked and sheared on 2026-11-09. `yaks tick --dry-run` then says it would regrow "for 2026-11-09", so the next session the same day runs a second round. Moving `due:` to 2026-11-09 doesn't help, since `regrow-due` counts from `regrown:`.

The skill says "shear it on its due day and it waits for the next one"; there is no way to say "this late round covered today". Options: count from the shear time when it is after the latest occurrence, or a `yaks shorn --covers <date>` style flag. Snooze applies only to open yaks.

#### 2026-11-18T14:43:02Z

*feedback-triage*

Seen again in cowpaths/joel: its feedback dok's entry `2026-11-18: a recurring yak regrows right after it is sheared` (fullstory-cc8e and two others, 11-16/17/18). Checked against the farm's history and `recur::plan`:

- The daily round regrows in the afternoon (11-17 16:17Z, `due: 2026-11-18`); the agent works it the next morning and shears it at 13:43Z on 11-18, the due day. A regrown yak counts from `due:` with k from 0, so the 11-18 occurrence is already past and the next tick regrows it at once. Each day's round is one behind, which is this yak exactly.
- The entry says `due:` "was already tomorrow" when sheared; the commits show `due: 2026-11-18` at the shear, and 11-19 only after the regrow.
- The `yaks-scheduling` skill's "shear it on its due day and it waits for the next one" holds for a never-regrown yak (k from 1) and not for a regrown one (k from 0). Whatever the fix, the rule and the skill must say the same thing: that inconsistency is what the reporter hit.
