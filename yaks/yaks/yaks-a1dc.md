---
id: yaks-a1dc
title: 'Act on the No AI Slop findings: review notes, recorded authorship, heartbeats, budgets'
status: hairy
type: idea
priority: 2
created: '2026-11-21T16:25:12Z'
updated: '2026-11-21T16:25:21Z'
tags:
- design
- skills
needs: human
---

Act on [the No AI Slop findings](/doks/research/no-ai-slop-manifesto.md#what-yaks-could-do). Six proposals, one per finding:

1. A `kind: review` note (`yaks review <id>`) and a short "shorn, not reviewed" list in `tick` / `inbox --for human`, ordered by stakes.
2. Authorship recorded, not inferred: `whoami --set` records agent vs human; no agent write falls through to the human's git name.
3. A `kind: heartbeat` note that `sheds` counts and the TUI and `show` fold, so milestone notes are the only notes a reader sees.
4. A word budget on the bundled skills.
5. Rework as the outcome measure: a derived query over regrows, `redo` reviews, reverts and bugs that link a shorn yak.
6. The human's words on the yak; agent-filed follow-ups as drafts (yaks-02ac).

Filed as one yak on purpose: split it into children only for the proposals Joel picks.

#### 2026-11-21T16:25:21Z

*noslop-1 · kind: ask · needs: human*

asked: needs human

Which of the six proposals should become yaks, and in what order?

My lean, in order:

1. **Authorship recorded (2)**: small, and it fixes a case where an agent's note is stamped as yours.
2. **Review note kind (1)**: the core of the manifesto for yaks, and the input that rework (5) and reflection need.
3. **Heartbeats folded (3)**: cuts the reading load of every yak a worker touched.
4. **Skill word budget (4)**: cheap, as a test.
5. **Rework query (5)**: after (1) exists.
6. **Human's words / drafts (6)**: a skill line now; drafts with yaks-02ac.

Two design questions inside (1):

- Should review stay optional and sampled (my lean), or be required before shear for some kinds of yak (a format or skill change)?
- Should an agent be able to write a `kind: review` at all? I lean yes, labelled as an agent's, never counted as yours.
