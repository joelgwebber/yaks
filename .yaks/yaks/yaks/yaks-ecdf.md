---
id: yaks-ecdf
title: Summarize yak notes much like we do agent conversations
status: shaving
type: feature
priority: 3
created: '2026-11-09T19:43:01Z'
updated: '2026-11-10T18:44:26Z'
tags:
- skills
---

Yak notes are a rich source of information about what we've been up to. And often more focused than raw conversation threads. So we should summarize them on a regular cadence and ensure they feed into the overall km-flow.

---
▸ 2026-11-10T18:00:10Z [herd-unlock]
kind: ask
needs: human
---

asked: needs human

This overlaps an open question in [the agent-memory research](/doks/research/agent-memory-dreaming.md) ("should reflection also read yak notes directly?"), so it is a design fork rather than a build. Two shapes, my lean first:

1. **Fold it into the review round** (yaks-dad8): review already reads session notes one at a time; it also reads the yak notes written since the last round (`yaks log --since <last round>`), concentrating on shorn yaks and `Decision:` notes, and moves what outlives the yak into doks the same way. No new artifact; session notes stay the narrative, yak notes the ground truth. Needs one step in `yaks-upkeep` and probably `yaks log --since` already covers the query.
2. **A periodic digest dok** (`doks/digests/<week>.md`, `type: summary`): an agent writes a weekly summary of yak activity, which review and reflection then read. More context for a human skimming the week, but it is a second copy of what the notes say (the research's "map as large as the empire" warning).

Lean: 1, and let the reflection round read the same window's `Decision:` notes for lessons sessions didn't report. Say if you want the weekly digest for your own reading anyway; that would be 2 as a human-facing extra.

---
▸ 2026-11-10T18:37:30Z [sched-1]
kind: answer
needs: agent
---

Yup, (1) makes perfect sense. We can fold this in however you see fit.

---
▸ 2026-11-10T18:44:25Z [herd-unlock]
kind: pickup
---

picked up

herd-unlock: folding yak notes into the review round in yaks-upkeep, option 1.

---
▸ 2026-11-10T18:44:26Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
