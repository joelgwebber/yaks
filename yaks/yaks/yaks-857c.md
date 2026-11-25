---
id: yaks-857c
title: 'Research: Vercel Board compared with yaks, and ideas to take'
status: shaving
type: task
priority: 3
created: '2026-11-25T04:22:56Z'
updated: '2026-11-25T04:25:43Z'
tags:
- agent
- research
---

Compare Vercel Board (a hosted message board and task queue for agents, announced 2026-11-18) with yaks, and mine it for ideas, including what is out of scope or a poor fit.

Findings: [Vercel Board compared with yaks](/doks/research/vercel-board.md).

Idea yaks filed from it: yaks-ba2f, yaks-7a25, yaks-b1bb, yaks-5bc9, yaks-f1e8, yaks-1407, yaks-4a0d, yaks-980b, and the design question yaks-a621.

#### 2026-11-25T04:25:43Z

*board-research*

Done. Findings in [Vercel Board compared with yaks](/doks/research/vercel-board.md), citing nine saved sources (blog, docs overview, tasks, CLI reference, swarm guide, onboarding playbook, and from `vercel/board` at `6d49814`: SPEC.md, the local backend doc, the agent skill).

Filed:

- yaks-ba2f plan files (`create --file`), p2
- yaks-7a25 exclusive claims on one machine, p2
- yaks-b1bb a coordinator's wait from a cursor, p2
- yaks-5bc9 conditional writes on yaks, p3
- yaks-f1e8 machine-readable error reasons, p3
- yaks-1407 agent onboarding page and orientation command, p3
- yaks-4a0d validated note kinds, p4
- yaks-980b skill lessons from the swarm guide, p3
- yaks-a621 design: `verify` runs a command another actor wrote, p2

Checks: `yaks check` has no finding on the new dok, sources or yaks; `yaks doctor`'s three findings predate this (yaks-5e8d, yaks-d262, yaks-5250).
