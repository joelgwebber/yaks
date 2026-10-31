---
id: yaks-4c2a
title: 'Skills: capture design-phase questions and answers as yak asks, not only chat'
status: shaving
type: task
priority: 2
created: '2026-10-31T15:57:06Z'
updated: '2026-10-31T16:42:41Z'
parent: yaks-039b
depends_on:
- yaks-6a11
labels:
- skills
---

Joel (2026-10-31): design-phase questions and answers belong in yaks, not only in chat. Capturing them as yak asks and answers gives a more structured design process and better provenance than chat context.

The `yaks` skill already says to "ask to record a real design fork", but in practice agents ask in chat. The 2026-10-31 sheds design (`yaks-039b`) ran every question through chat; the answers were copied into the yaks afterwards as `Decision:` notes.

## Guidance to write

- During design, each open question goes on the yak it concerns as `yaks ask <id> --note -`, with the options and a stated lean. Chat links to the asks rather than restating them.
- The human answers with `yaks answer` (or `a` in the TUI). The agent picks it up and records the outcome as a decision.
- Several questions on one yak: one ask per question, once note metadata supports more than one open ask per yak (`reply_to`); until then, one numbered ask.
- Where this goes: `yaks` (Notes and descriptions, the inbox), `yaks-coordinating` (design passes), the `brief` text if relevant.

Depends on `yaks-6a11`: explicit `ask`/`answer` kinds and `reply_to` are what make several questions per yak workable.

---
▸ 2026-10-31T16:42:41Z [yakherd]

moved: hairy -> shaving
