---
id: yaks-e87e
title: 'Spike: human-in-the-loop questions from Delta workers'
type: idea
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T20:14:31Z'
parent: yaks-b5a0
labels:
- agent
- delta
---

Today a worker that hits a human decision runs yaks ask and returns control. That loses the worker's live context, and in public mode the ask sits in the worker's clone until merge. In Delta, test: (a) worker asks in the farm and also messages the parent (send_agent_message) before ending; (b) whether the parent can message an idle or finished subagent to resume it with context and worktree intact, or whether a merged worker is gone; (c) how the coordinator tracks answers it owes back (relates to yaks-c061). Decide whether a Delta workflow should keep workers parked on a question.
