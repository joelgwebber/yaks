---
id: yaks-cfa6
title: shave takes hairy ancestors along (replacing the skills' 'and its parent if hairy')
status: hairy
type: task
priority: 3
created: '2026-11-01T16:43:15Z'
updated: '2026-11-01T16:43:15Z'
parent: yaks-9b41
labels:
- cli
---

Part of yaks-9b41.

The `yaks` and `yaks-working` skills tell an agent to `yaks shave <id>`, "and its parent if still hairy". Agents skip it, leaving a shaving child under a hairy umbrella; the Indent runner proposed a doctor error for that.

Lean: make it true by construction instead. `yaks shave <child>` also shaves each hairy ancestor (printing `Shaving <parent> (parent of <child>)`, its move note saying why), the TUI's shave does the same, and the skills drop the clause. No doctor check: a hairy parent over a shaving child is untidy, not wrong, and nothing reads it as a contradiction.

The cost: the ancestor's move is stamped with the worker's name, so `yaks sheds` lists the umbrella under that worker (as it already does when the worker follows the skill). Design fork: see the ask on yaks-9b41.
