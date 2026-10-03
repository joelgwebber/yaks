---
id: yaks-df61
title: 'Delta observation log: how threads, worktrees, clones and the farm interact'
type: task
priority: 2
created: '2026-10-03T20:35:54Z'
updated: '2026-10-03T20:40:03Z'
parent: yaks-b5a0
labels:
- delta
- agent
---

Running log (append-only artifact, observation-log.md) of what we learn running yaks work through Delta: threads, subagents, subthreads, worktrees, clones, remotes, and the farm. Each entry: what happened, evidence, and what it implies (a skill change, a CLI change, or an open question). Feeds yaks-2f9b and the other children of yaks-b5a0. Not a verification yak: its evidence is the log itself.

![observation-log](artifacts/yaks-df61/observation-log.md)

---
▸ 2026-10-03T20:36:49Z [delta-lead]
Created the log with O1-O8 from the first Delta run and the resync. Append new entries to the attached artifact in place; do not re-attach.

---
▸ 2026-10-03T20:39:13Z [delta-lead]
Added O9 result, O10-O13 (private-mode walk-up in Delta, landing into the user's checkout, multi-worktree file tools). Log stays open while yaks-b5a0 runs.

---
▸ 2026-10-03T20:40:03Z [delta-lead]
O12 resolved: push to local main is refused while the user's checkout has unstaged changes (human yak drift counts). Options and recommendation in the log.
