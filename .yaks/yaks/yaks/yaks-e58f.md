---
id: yaks-e58f
title: 'Session review mechanics: list unreviewed session notes, stamp promoted:'
status: hairy
type: feature
priority: 2
created: '2026-11-06T04:28:30Z'
updated: '2026-11-06T04:28:34Z'
parent: yaks-1c89
depends_on:
- yaks-17f5
tags:
- cli
- doks
---

The mechanical half of Hearth's local `session-review` job (`hearth-d740`): find session notes nobody has reviewed yet, and mark one reviewed.

- List notes without `promoted:`, oldest first (`dok query` reaches neither `sessions/` nor a missing field today).
- Stamp `promoted: <date>` on a note once its durable facts are in doks, in one commit per review round.

The judgement (what is worth keeping, which dok it goes in) is the upkeep skill's job.
