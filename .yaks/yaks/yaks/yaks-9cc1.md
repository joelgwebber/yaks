---
id: yaks-9cc1
title: 'Design: recurring yaks (regrow on a schedule) and due dates'
status: shorn
type: idea
priority: 3
created: '2026-11-01T18:53:14Z'
updated: '2026-11-02T01:29:28Z'
parent: yaks-44b3
depends_on:
- yaks-1f34
tags:
- wiki
- design
---

Layer 5. Joel (2026-11-01): reminders often want extra data (a link to the plumber's page), and recurring yaks are useful beyond Hearth: maintenance, weekly metric checks. Starting idea: a yak that regrows on a schedule. Needs the field model (yaks-8384).

---
▸ 2026-11-01T22:26:46Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T22:26:47Z [delta:farm-wiki]

## Proposal: due dates and recurring yaks (layer 5)

Both use the field model (yaks-1f34), so neither needs a new file format: they are two schema-described yak fields plus one command. Nothing runs on a clock inside yaks. A harness (Hearth, cron, a TUI tick) calls `yaks regrow-due` when it likes; yaks only answers "what is due" and "regrow what is due".

### Fields (declared in `schemas/yak.md`, checked by doctor)

- `due: 2026-11-15` (a date, or a datetime). The yak should be done by then.
- `every: P1W`: an ISO 8601 duration (`P1D`, `P2W`, `P1M`, `P3M`, `P1Y`), or a simple cron-ish word (`monday`, `1st`). This makes the yak recurring.
- `regrown:` holds the last time it regrew (written by yaks, not by hand).

### Behaviour

- `yaks list --due [--before <date>]`, plus a `due` column in `list`/`next`. A `due:` in the past shows as **overdue** in `list`, `next` and the TUI (a red date chip).
- **Recurring:** when you shear a yak that has `every:`, it stays the same yak. Later, `yaks regrow-due` moves every shorn recurring yak whose next occurrence has arrived back to hairy:
  - It adds a `kind: regrow` note ("regrew for the 2026-11-10 occurrence").
  - It sets `due:` to that occurrence.
  - It writes `regrown:`.
- The yak's notes keep each round's history, so "check the water softener salt" accumulates one shear note per month. That is the evidence trail Joel wanted, and links on it (the plumber's page) stay put.
- `yaks regrow-due --dry-run` lists what would regrow. The TUI could run it on load and on its sync tick.
- One rule for the schedule: the next occurrence is the first `regrown + every`, or `due + every` when the yak was never regrown, that is not in the future. Missed occurrences don't stack up: one regrow covers them all and the note says how many were skipped.

### Rejected

- **A new yak per occurrence (copying the template):** it multiplies yaks and scatters the history.
- **A daemon inside yaks:** the clock belongs to the harness, as decided on yaks-44b3.

Implementation would be one worker-sized yak once the ask below is answered: fields in `schemas/yak.md`, `list --due`, overdue rendering, `regrow-due`, and docs.

---
▸ 2026-11-01T22:26:47Z [delta:farm-wiki]

kind: ask
needs: human
---

asked: needs human

1. **One yak that regrows** (history accumulates in its notes, `due:` advances), not a copy per occurrence. Lean: regrow the same yak.
2. **The clock lives outside yaks:** `yaks regrow-due` is idempotent, and the harness, cron or TUI tick calls it. Lean: yes, with the TUI running it on load and on each sync tick.
3. **Schedule syntax:** ISO 8601 durations (`P1W`, `P1M`) only, or also words (`monday`, `1st`, `weekdays`)? Lean: durations now, words later if you find you want them.

---
▸ 2026-11-01T22:50:24Z [Joel Webber]

kind: answer
needs: agent
---

1. Yes, exactly. Living yaks don't always stay shorn!
2. That makes sense, and works well in either offline or agent-harness mode.
3. Let's keep the schedule syntax simple and predictable. Agents will have no trouble with this, and we can layer on CLI/TUI convenience affordances later if needed.

---
▸ 2026-11-02T00:36:10Z [delta:farm-wiki]

kind: pickup
---

picked up

Decision (Joel, 2026-11-01): all three leans. One yak regrows (its history accumulates); the clock lives outside yaks (`yaks regrow-due` is idempotent and called by a harness, cron, or the TUI on load and each sync tick); schedules are ISO 8601 durations only. Implementation is yaks-9cc1's child, below.

---
▸ 2026-11-02T01:29:27Z [delta:farm-wiki]

Implemented by yaks-2fc5 (landed 7560285).

---
▸ 2026-11-02T01:29:28Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
