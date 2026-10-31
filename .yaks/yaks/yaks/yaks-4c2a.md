---
id: yaks-4c2a
title: 'Skills: capture design-phase questions and answers as yak asks, not only chat'
status: shorn
type: task
priority: 2
created: '2026-10-31T15:57:06Z'
updated: '2026-10-31T17:24:19Z'
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

---
▸ 2026-10-31T16:45:29Z [yakherd]

Landed in `45bfc8b`:

- `yaks` skill, inbox section: design questions go on the yak they concern as asks, with options and a lean; chat links them; the pickup records a `Decision:`; one open ask per yak at a time.
- `yaks-coordinating` §7: the same rule for the yakherd's own design passes.

Evidence: `cargo test --workspace` green (skills and brief tests included). `reply_to` (several open asks per yak) stays a follow-up of the metadata design. That needs note ids, so for now the skill says to number related questions in one ask.

---
▸ 2026-10-31T16:45:29Z [yakherd]

moved: shaving -> shorn

---
▸ 2026-10-31T16:52:20Z [yakherd]
kind: landed
commit: 45bfc8b32107b8dd51485d6a3872e483f0b909cb
branch: main
---

landed: 45bfc8b yaks-4c2a, yaks-0071: skills put design Q&A on yak asks; note metadata (--meta, reserved keys, checkpo (recorded by hand: landed with a yaks binary from before `yaks-735a`)

---
▸ 2026-10-31T17:24:15Z [yakherd]
kind: landed
commit: 931d4e4af524e4e2ca47ca160ec8916e8b2763ec
branch: main
---

landed (rebased onto `c2b640f`, your checkout's `main`): 931d4e4 yaks-0071, yaks-4c2a, yaks-4728: re-apply this family's skills changes on top of the yaks-

---
▸ 2026-10-31T17:24:19Z [yakherd]
kind: landed
commit: ceb6d57c5e0c8ee7028111f4a19ab17abc194fca
branch: main
---

landed (rebased onto `c2b640f`, your checkout's `main`): ceb6d57 yaks-4c2a, yaks-0071: skills put design Q&A on yak asks; note metadata (--meta, reserved k (was `45bfc8b`)
