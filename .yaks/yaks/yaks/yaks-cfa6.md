---
id: yaks-cfa6
title: shave takes hairy ancestors along (replacing the skills' 'and its parent if hairy')
status: shorn
type: task
priority: 3
created: '2026-11-01T16:43:15Z'
updated: '2026-11-01T17:30:52Z'
parent: yaks-9b41
labels:
- cli
---

Part of yaks-9b41.

The `yaks` and `yaks-working` skills tell an agent to `yaks shave <id>`, "and its parent if still hairy". Agents skip it, leaving a shaving child under a hairy umbrella; the Indent runner proposed a doctor error for that.

Lean: make it true by construction instead. `yaks shave <child>` also shaves each hairy ancestor (printing `Shaving <parent> (parent of <child>)`, its move note saying why), the TUI's shave does the same, and the skills drop the clause. No doctor check: a hairy parent over a shaving child is untidy, not wrong, and nothing reads it as a contradiction.

The cost: the ancestor's move is stamped with the worker's name, so `yaks sheds` lists the umbrella under that worker (as it already does when the worker follows the skill). Design fork: see the ask on yaks-9b41.

---
▸ 2026-11-01T17:28:01Z [yakherd]
kind: move
from: hairy
to: shaving
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T17:30:51Z [yakherd]
kind: landed
sha: 76aeae2
---

Done (see the landed sha).

- `Farm::transition_lifting` (what `transition` now wraps): after the family check, a move to shaving also moves each hairy ancestor (`Farm::shave_lifts`: parent, grandparent, ... up to the first non-hairy one, cycle-safe), outermost first in the result. Each lifted entry carries `child: <id>` via the new `store::move_task_with(.., extra)`; it is stamped with the same actor.
- CLI prints `Shaving <parent> (parent of <id>)` per lifted yak; the TUI's single move says `<id> → shaving (with <ids>)`. A bulk shave walks parents first (batch_order), so it lifts nothing it already moves.
- No doctor check for shaving-under-hairy (Joel, yaks-9b41 ask, 2).
- Help/docs: `shave --help`, docs/cli.md (Families section and the note-metadata `child` field), docs/tui.md state row. The skills' "and its parent if hairy" goes in yaks-93c5.

Evidence: `cargo test --workspace --locked` green (CI-like env); `shave_lifts_hairy_ancestors` (two hairy ancestors lifted outermost first, stops at a shaving one, `child:` and actor on the entry, no-op lifts nothing). Real CLI: `yaks shave <leaf>` under hairy mid under hairy top printed `Shaving yak-ff1c`, `Shaving yak-0899 (parent of yak-ff1c)`, `Shaving yak-5b34 (parent of yak-ff1c)`; top's last entry shows `child: yak-ff1c` and the `thread:`.

---
▸ 2026-11-01T17:30:52Z [yakherd]
kind: move
from: shaving
to: shorn
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
