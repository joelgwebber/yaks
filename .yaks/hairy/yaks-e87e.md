---
id: yaks-e87e
title: 'Spike: human-in-the-loop questions from Delta workers'
type: idea
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T21:22:16Z'
parent: yaks-b5a0
labels:
- agent
- delta
---

Today a worker that hits a human decision runs yaks ask and returns control. That loses the worker's live context, and in public mode the ask sits in the worker's clone until merge. In Delta, test: (a) worker asks in the farm and also messages the parent (send_agent_message) before ending; (b) whether the parent can message an idle or finished subagent to resume it with context and worktree intact, or whether a merged worker is gone; (c) how the coordinator tracks answers it owes back (relates to yaks-c061). Decide whether a Delta workflow should keep workers parked on a question.

---
▸ 2026-10-03T21:22:16Z [delta-lead]
Findings (log O23, O18, O24): (b) YES, a finished worker can be resumed with send_agent_message and keeps its context and worktree (trial 1A: answered a scope question, it completed the brief); (a)/(c) partly: the worker's yaks ask reached me as a needs:human yak file in my working tree and showed in yaks inbox and not in next; I also learned of it from the completion message. Open: how long a finished worker stays resumable, repeated resumes, and who may clear an ask (O24). Suggested Delta workflow: ask and return, coordinator answers to the SAME worker rather than re-briefing.
