---
id: yaks-5c9f
title: Restructure coordinator/worker skills by farm mode (public / private / out-of-tree)
type: task
priority: 2
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T20:27:55Z'
parent: yaks-b5a0
depends_on:
- yaks-80d5
labels:
- skills
---

Replace today's mixed public/private, harness-agnostic prose with a small shared core (claim, lanes, evidence, ask/answer) plus one coordinator + worker pair per environment (git worktrees; Delta), each followable without reading the others. Fold in pending guidance from yaks-0a87 and yaks-75cc. Embed and test-guard them like the yaks skill.

---
▸ 2026-10-03T20:27:55Z [delta-agent]
Retitled per b5a0 direction: split by farm mode, not environment; Delta is a separate layered skill (yaks-2f9b).
