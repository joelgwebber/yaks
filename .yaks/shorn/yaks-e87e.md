---
id: yaks-e87e
title: 'Spike: human-in-the-loop questions from Delta workers'
type: idea
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-04T05:11:55Z'
parent: yaks-b5a0
labels:
- agent
- delta
---

Today a worker that hits a human decision runs yaks ask and returns control. That loses the worker's live context, and in public mode the ask sits in the worker's clone until merge. In Delta, test: (a) worker asks in the farm and also messages the parent (send_agent_message) before ending; (b) whether the parent can message an idle or finished subagent to resume it with context and worktree intact, or whether a merged worker is gone; (c) how the coordinator tracks answers it owes back (relates to yaks-c061). Decide whether a Delta workflow should keep workers parked on a question.

---
▸ 2026-10-03T21:22:16Z [delta-lead]
Findings (log O23, O18, O24): (b) YES, a finished worker can be resumed with send_agent_message and keeps its context and worktree (trial 1A: answered a scope question, it completed the brief); (a)/(c) partly: the worker's yaks ask reached me as a needs:human yak file in my working tree and showed in yaks inbox and not in next; I also learned of it from the completion message. Open: how long a finished worker stays resumable, repeated resumes, and who may clear an ask (O24). Suggested Delta workflow: ask and return, coordinator answers to the SAME worker rather than re-briefing.

---
▸ 2026-10-03T22:17:47Z [delta-lead]
Joel (2026-10-03): coordinators answering a worker's ask is fine and preferred over ephemeral message passing, because it leaves a durable, attributed decision in the yak. A coordinator can often resolve what a worker cannot (capability, wider context) and may leave a question unanswered, or ask another party, for the human. Skills should say: coordinator answers scope/mechanics, records that it was the coordinator, and routes real design forks to the human.

---
▸ 2026-10-04T05:11:55Z [delta-lead]
SHORN SUMMARY. The spike's three questions, answered by real runs (yaks-df61 O18, O23, O24, O28, O34, O42): (a) Does a worker that asks in the farm also need to message the parent? No. The ask is a needs:human field on the yak; in a team farm it reaches the coordinator in the worker's returned file state and in yaks inbox, and in a private farm instantly; the worker's completion message names it too. (b) Can a finished or idle worker be resumed with its context and worktree? Yes, three times (trial 1A, 3 p-div, 4B): send_agent_message to its agent id (in the spawn confirmation and the completion message's sender_id) resumes it with its notes, findings and worktree; it can redo edits it had lost. (c) How does the coordinator track answers it owes? yaks inbox and the needs field; the answer is a durable note on the yak and the message is only the nudge. Decision for the Delta workflow: park a worker on an ask (it returns with edits left in the tree) and answer the SAME worker, instead of re-briefing a new one. The coordinator answers scope and mechanics itself and records that it did; a design fork goes to the human with the coordinator's lean, via an ask left open (Joel agreed). Caveat found later: a parked worker pins the coordinator's history, so the coordinator must not rewrite its branch while one is parked, and lands a resumed worker with a cherry-pick if its base is no longer an ancestor (yaks-coordinating-delta step 7). Open, not tested: how long a finished worker stays resumable (the day-old one was not tried), and repeated resumes of one worker. Evidence: the log entries above and the three runs. Follow-up: yaks-c061 (answered-question pickup) remains separate.
