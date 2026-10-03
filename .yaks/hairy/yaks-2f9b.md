---
id: yaks-2f9b
title: 'Delta environment: coordinator + worker skill'
type: task
priority: 2
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T20:14:31Z'
parent: yaks-b5a0
depends_on:
- yaks-5c9f
labels:
- skills
- delta
---

Delta-specific guidance, starting from the case study: the parent thread is the coordinator and claims (shaves) before spawn_subagent; the worker task names the yak and its evidence contract; workers never commit path dependencies or other machine-local config; the coordinator re-runs verification before accepting a worker's 'tests pass'; one landing point (the coordinator pushes to local and/or origin, chosen in context); extra repos are attached as Delta worktrees, not nested clones; how to resync a parent left stale by landing elsewhere.
