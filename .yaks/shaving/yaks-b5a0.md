---
id: yaks-b5a0
title: 'Parallel development: finish and harden coordinator/worker skills + tooling (git worktrees, Delta)'
type: task
priority: 2
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T20:27:55Z'
labels:
- agent
- skills
---

The coordination layer (the yaks-coordinating / yaks-working dev skills, public vs private farm modes, claim -> fan out -> merge) is nascent, hard to follow, and lightly tested. Goal: parallel agent work on a yaks farm that is reliable and legible in each environment we actually use: plain git worktrees today, Delta threads and subagents next. Test aggressively against real runs and refactor the skills and CLI freely as we learn.

Principles: prove it with real runs, not prose; one followable skill per environment instead of one skill full of conditionals; install/init make the right setup explicit rather than leaving agents to infer it.

Related, not reparented (decide whether to fold in): yaks-10fb (post-3901 follow-ups: 5d5c, 213b, eb5f, 70e5, 8265), yaks-5f12 (Model 3 vs git-store), yaks-c061 (answered-question pickup), yaks-0a87 (checkpoint footgun), yaks-75cc (drift as signal), yaks-77a5 (lightning-round case study).

---
▸ 2026-10-03T20:27:55Z [delta-agent]
Direction (Joel, 2026-10-03): Delta is NOT a fourth farm mode. Farm mode (public/team, private/local, out-of-tree) and environment (git worktrees, Delta) are orthogonal. Plan: (1) split the coordinator/worker skills by FARM MODE, so each is followable without the mixed public/private conditionals (yaks-5c9f); (2) a separate Delta skill that layers on top of whichever mode skill applies (yaks-2f9b); (3) skills install + init consolidate into the in-repo .agents/ by default, with init choosing the mode (yaks-3859, yaks-814a).
