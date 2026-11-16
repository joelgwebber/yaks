---
id: yaks-2b40
title: 'Design: lessons dok budget and index, promotion measured, a dok index'
status: shorn
type: task
priority: 3
created: '2026-11-08T16:24:07Z'
updated: '2026-11-10T18:02:45Z'
parent: yaks-1c89
tags:
- design
- skills
---

From [the agent-memory research](/doks/research/agent-memory-dreaming.md) (yaks-e4f5), sections 2, 4, 5, 8 and 9: changes to the `yaks-upkeep` reflection procedure and how doks are indexed. It edits a skill, so the proposals are one ask for Joel before anything is written.

Related and not duplicated: [yaks-323b](yaks-323b) (`session list --since`) and the accepted design in [/doks/design/session-reflection.md](/doks/design/session-reflection.md).

---
▸ 2026-11-08T16:24:17Z [memory-1]
kind: ask
needs: human
---

asked: needs human

Six points from [the research](/doks/research/agent-memory-dreaming.md), each with my lean. All of them change the `yaks-upkeep` reflection procedure or the dok conventions, so nothing is written until you answer.

1. **A budget and an index shape for `lessons.md`:** a line cap (about 50) with a top section of one-line rules that link down to detail. Reflection reports lessons removed as well as added. Lean: yes. Today it is read in full at every session start and can only grow.
2. **A second way out for a lesson, absorption:** when a lesson is now in a skill or `AGENTS.md`, or a yak fixed the friction, it leaves the dok with a link to where it went. Today a lesson leaves only when a later session showed it wrong. Lean: yes.
3. **Each lesson carries how we know and a last-seen date**, so a lesson not seen for several rounds is a demotion candidate. Lean: yes.
4. **A lesson that is already in context and still recurs gets a yak for the mechanism, not more text.** Example in hand: `cargo fmt` reformatting two files appears in all three session notes though `AGENTS.md` warns about it. Lean: yes, one line in the procedure.
5. **Reflection checks last round's accepted promotions:** the account says "recurred" or "gone" for each. Lean: yes; it is anecdote at this volume but it is what makes point 4 work.
6. **A `description:` on every dok as its index line** (when to read it), flagged by `yaks check` when missing. Whether to also list them at session start: lean no until point 1 lands, since that is more context.

Separately, the loop isn't running: no recurring review, reflection or tidy yak exists and there is no `lessons.md`. Lean: run the first reflection round by hand over the three notes, then create the three recurring yaks. Say if you want that as its own yak.

---
▸ 2026-11-10T17:44:06Z [sched-1]
kind: answer
needs: agent
---

1. SGTM. We can always see how it goes and iterate from there.
2. SGTM. Seems like an important part of the process.
3. Also agreeed. Tracking provenance for future reference is always a Good Thing.
4. I'm assuming this yak would be a task to stop the recurrence, right?
5. Yup.
6. Agreed, that sounds like unnecessary summarization and repetition.

Oh yeah, please do set that up. I never actually did the initialization step in this farm.

---
▸ 2026-11-10T17:50:28Z [herd-unlock]
kind: pickup
---

picked up

herd-unlock: doing this myself (skill prose stays with the coordinator): the six points into yaks-upkeep, the first reflection round by hand, the three recurring yaks. Point 4: yes, the yak is a task for the mechanism that stops the recurrence.

---
▸ 2026-11-10T17:53:15Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T17:59:59Z [herd-unlock]

Done (herd-unlock), points 1–5 in `yaks-upkeep`, commit on `main` (see `yaks commits yaks-2b40`); point 6 is the child yaks-2755 (worker `dokdesc-1`, running).

## What changed in "Reflecting on sessions"

- Step 3 (new): check last round's fixes, `recurred` / `gone` (point 5).
- Step 5 (new): three ways out for a lesson, *absorbed*, *wrong*, *faded* (points 2, 3).
- Step 6 (new): a lesson already in context that still recurs gets a yak for the mechanism, not more text (point 4; Joel: "a task to stop the recurrence", yes).
- Step 8: a proposal names the friction it targets, so step 3 can check it.
- New "The lessons dok" subsection: ~50-line budget, `## Rules` index of one-liners linking to per-subject detail with *how we know* and *seen* (points 1, 3).
- [/doks/design/session-reflection.md](/doks/design/session-reflection.md) gains "Budget and demotion" and a `description:`.

## Loop started

- First round by hand: [yaks-20ff](yaks-20ff) (shorn, recurs weekly), wrote [/doks/lessons.md](/doks/lessons.md) (27 lines).
- Point 4's first instance: yaks-b89d (shorn): `main` formatted once, the `cargo fmt` caveat gone from `AGENTS.md` and the land skill.
- Recurring: yaks-dad8 (review, due tomorrow), yaks-20ff (reflect), yaks-0686 (tidy), weekly, `--wake agent`.
- Filed: yaks-760d (`land` after a resolved cherry-pick), yaks-2788 (cheaper-model guidance in `yaks-coordinating`), yaks-20fe (cross-farm feedback, asked).

`cargo test skills` passes (37).

---
▸ 2026-11-10T18:02:45Z [herd-unlock]
kind: move
from: shaving
to: shorn
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
