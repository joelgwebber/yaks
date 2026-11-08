---
id: yaks-e4f5
title: 'Research: Anthropic''s agent-memory ''dreaming'' and what it means for doks and lessons'
status: shaving
type: task
priority: 2
created: '2026-11-08T16:22:21Z'
updated: '2026-11-08T16:24:17Z'
parent: yaks-1c89
tags:
- doks
- research
---

Joel pointed at Anthropic's "The components of agent memory" (Lance Martin, 2026-11-06) and asked what to take from its "dreaming" idea and the maintenance stack (transcripts, notes, representations, context) for how the farm keeps doks and lessons salient and current.

Deliverable: a research dok (what the article says, mapped onto what yaks already has, what it doesn't), the article saved as a source, and follow-ups filed where the dok recommends work. Instruction files and skills are not edited: proposals go to Joel as asks.

---
▸ 2026-11-08T16:22:23Z [memory-1]
kind: move
from: hairy
to: shaving
thread: ksQQobtw8z0aRtm7OxvMZr8p4JLOAE33iJIBxBCg_8DSgeNASrgoPHtnOl5O
---

moved: hairy -> shaving

---
▸ 2026-11-08T16:24:17Z [memory-1]

Done. Findings in [/doks/research/agent-memory-dreaming.md](/doks/research/agent-memory-dreaming.md); the article is saved as `sources/2026-11-08-f2310691` (text only; figures not read, the two cookbooks it links not read).

What it found, in short:

- The farm already has the stack (event log, session notes, doks and links, context) and three dreaming passes (review, reflection, `yaks check`).
- Three properties are missing: a budget and demotion on the top layer (`lessons.md` can only grow), evidence-driven staleness (`check` is clock-only), and a check on whether a promoted lesson worked.
- Two instances of drift in hand: `knowledge-store.md` describes `wiki/`, `yaks page` and `import-hearth`, and `AGENTS.md`'s list of files `cargo fmt` reformats is half out of date.
- The loop isn't running: three session notes, no `lessons.md`, no recurring upkeep yaks.

Filed under yaks-1c89: [yaks-6cf2](yaks-6cf2) (check flags doks whose cited code changed), [yaks-464b](yaks-464b) (refresh knowledge-store), and [yaks-2b40](yaks-2b40) (the skill and convention changes, held on one ask to Joel).
