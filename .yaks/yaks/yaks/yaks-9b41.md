---
id: yaks-9b41
title: 'Families: a finished yak never has open descendants (moves enforce it, doctor checks it, skills say it once)'
status: shorn
type: feature
priority: 1
created: '2026-10-13T18:04:15Z'
updated: '2026-11-01T17:31:48Z'
tags:
- doctor
- skills
---

Joel (2026-10-13): "It creates great confusion, and runs contrary to a sane workflow. We should probably also add more skill guidance on the subject -- my agents keep doing it!"

Raised again on 2026-11-01 by an Indent cloud runner that left many families with a shorn parent over hairy or shaving children. This is the arc for that, to finish before cutting 0.0.28.

## The invariant

A shorn or dead yak has no hairy or shaving children. (Children are enough: if every finished yak's children are finished, no descendant is open.) A shorn child under a dead parent is fine: the umbrella was abandoned, that part was done.

## Every move that can break it

| Move | Today (0.0.27) | Arc |
|---|---|---|
| `shorn` a parent over open children | allowed | refused (yaks-62f2, landed ac5c6e2) |
| `slaughter` a parent over open children | refused; also refuses over *shorn* children, and `--family` kills them | refuse only over open ones (yaks-27c1) |
| `create --parent` / `reparent --parent` onto a finished yak | allowed | refused (yaks-35a4) |
| `regrow` / `revive` / `shave` a child of a finished yak | allowed | refused (yaks-35a4) |
| hand edits, older farms | not checked | `doctor` `unfinished-children`, shorn *and* dead parents (yaks-62f2 for shorn, yaks-27c1 for dead) |

The weaker convention, "a shaving child's parent is shaving", stays a convention: yaks-cfa6 decides whether `shave` takes hairy ancestors along.

## Skills

One "Families" section in the `yaks` skill (the invariant, graduating a child, where a follow-up goes), and the scattered lines replaced by a pointer (yaks-93c5).

## Not doing

- `shorn --family`: shears unfinished work as done without evidence for it.
- `shorn --force`: the escape hatch is graduating the child (`yaks reparent`), which keeps the farm true. Add one only if a real need appears (Joel's lean on yaks-62f2).
- A doctor error for "shaving under hairy": see yaks-cfa6.

---
▸ 2026-11-01T16:43:28Z [yakherd]

kind: move
from: hairy
to: shaving
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T16:43:28Z [yakherd]

kind: ask
needs: human
---

asked: needs human

Two behavior changes in this arc need your call. Everything else (yaks-35a4, the doctor half of yaks-27c1, yaks-93c5) follows from the invariant, so I'll go ahead with those unless you object.

1. **`slaughter` and shorn children** (yaks-27c1). Today it refuses over shorn children and `slaughter --family` turns them dead. Lean: only open children block it, and `--family` leaves shorn ones shorn, so abandoning an umbrella never erases work that was done. Alternative: keep today's behavior (dead means the whole subtree is gone).

2. **`shave` takes hairy ancestors along** (yaks-cfa6). Lean: yes, and drop "and its parent if hairy" from the skills, rather than the runner's proposed doctor error for a shaving child under a hairy parent. Cost: the umbrella's move carries the worker's name, so `yaks sheds` shows it under that worker (as today when the worker follows the skill). Alternative: leave it as skill guidance, with no check.

Not doing, from the runner's list: `shorn --family` (shears unfinished work without evidence) and `shorn --force` (graduating the child with `yaks reparent` is the escape hatch; your lean on yaks-62f2 was `--force` only if a real need appears).

---
▸ 2026-11-01T17:16:17Z [Joel Webber]

kind: answer
needs: agent
---

1. That makes sense to me. I don't think it's necessary to slaughter all the children in this case.
2. That also makes sense; it's a similarly weird case to say a child is shaving, but its parent hasn't started yet.

---
▸ 2026-11-01T17:16:53Z [yakherd]

kind: pickup
---

picked up

Decision: both leans taken (Joel, answer above). slaughter blocks only on hairy/shaving children and --family leaves shorn descendants shorn (yaks-27c1); shave takes hairy ancestors along and the skills drop 'and its parent if hairy' (yaks-cfa6). Rejected: today's slaughter-kills-the-subtree, and a doctor error for shaving-under-hairy. Order: yaks-27c1, yaks-35a4, yaks-cfa6, then yaks-93c5 (skills), then cut 0.0.28.

---
▸ 2026-11-01T17:31:48Z [yakherd]

Arc done: a finished yak never has open descendants, and the commands, the TUI, doctor and the skills agree.

- yaks-62f2 (ac5c6e2): `shorn` refuses over open children; doctor `unfinished-children`.
- yaks-27c1 (7a314e2): dead parents too; `slaughter` ignores shorn children and `--family` leaves them shorn; `list --all --parent-of` shows dead descendants.
- yaks-35a4 (95fa1e2): nothing opens under a finished parent (create, reparent, regrow, revive, shave); `Farm::batch_order`; the TUI checks before writing a shear comment.
- yaks-cfa6 (76aeae2): `shave` takes hairy ancestors along (`child:` on their entries).
- yaks-93c5 (ea50761): one Families section in the skills.

Evidence is on each child; the gate (`cargo test --workspace --locked`, CI-like env) was green after each commit.

---
▸ 2026-11-01T17:31:48Z [yakherd]

kind: move
from: shaving
to: shorn
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
