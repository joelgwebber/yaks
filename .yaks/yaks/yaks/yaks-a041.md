---
id: yaks-a041
title: 'Regrow routes a recurring yak to the inbox (needs: agent|human)'
status: shaving
type: feature
priority: 2
created: '2026-11-04T22:31:54Z'
updated: '2026-11-04T22:51:16Z'
parent: yaks-7fa6
tags:
- cli
---

Part of the scheduling arc ([design](/doks/design/scheduling.md)). Today `regrow-due` only makes the yak hairy; nothing tells anyone it has work.

On regrow, if the yak says who acts next, set `needs:` to it (the same field `ask`/`request` use) and write the `kind: regrow` note so `inbox` shows why. No new queue. A yak that never says stays hairy only, as now.

**Blocked on an ask** (below): how the yak says who. Don't start until it is answered.

---
▸ 2026-11-04T22:31:55Z [source-fix]
kind: ask
needs: human
---

asked: needs human

`needs:` is transient: `pickup` (agent) or `answer --done` (human) clears it, and the next regrow must set it again. So the yak has to carry who to wake somewhere durable. Joel's instinct is that regrow should simply go to `needs:`; the only question is where it reads the target from.

1. **A frontmatter field** (`--set wake=agent|human`, or a `create`/`update` flag). Explicit and greppable, shows in `show` and the TUI. Costs one field.
2. **No field: a tag decides** (say tag `agent`). Reuses an existing construct, but tags are meant to be few and broad, and this overloads one with behavior.
3. **No field: the last `request`/`ask` note decides.** Regrow repeats whoever last held it. Implicit, and a first round has nothing to repeat.

Lean: 1, one field `wake: agent|human`; absent means just hairy, as today. It is the only way a first round knows, and the smallest explicit thing.

---
▸ 2026-11-04T22:34:49Z [Joel Webber]
kind: answer
needs: agent
---

Ah, I get it. That works for me. Just wanted to be sure I'm following the logic.

---
▸ 2026-11-04T22:36:18Z [sched-1]
kind: pickup
---

picked up

Decision (Joel, 2026-11-04): option 1. A recurring yak carries `wake: agent|human`; on regrow it sets `needs:` to that value (the same field `ask`/`request` use). Absent means the yak just goes hairy, as today. Rejected: a tag that carries behavior, and inferring the target from the last ask/request note.

---
▸ 2026-11-04T22:51:16Z [sched-1]
kind: move
from: hairy
to: shaving
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
