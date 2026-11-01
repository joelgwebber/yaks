---
id: yaks-27c1
title: 'Dead parents: doctor checks them too, and slaughter stops treating shorn children as live'
status: shorn
type: task
priority: 2
created: '2026-11-01T16:43:15Z'
updated: '2026-11-01T17:22:27Z'
parent: yaks-9b41
labels:
- cli
---

Part of yaks-9b41.

## Doctor

`unfinished-children` covers shorn parents only (yaks-62f2). A dead yak over a hairy or shaving child is the same broken state (an abandoned umbrella still holding live work), and older farms or hand edits can hold it. Extend the check to dead parents, same code.

## slaughter

Today `slaughter` refuses a yak with any non-dead descendant, *shorn included*, and `--family` moves shorn descendants to dead, erasing that they were done. Under the invariant only open (hairy or shaving) descendants matter:

- `slaughter <id>` refuses only over open children (it already names them; add the `yaks reparent` fix like `shorn`).
- `--family` kills the open descendants and `id`, and leaves shorn ones shorn.
- The TUI's slaughter confirm and bulk skip use the same rule (`live_child_count`).

Design fork (behavior change): see the ask on yaks-9b41.

---
▸ 2026-11-01T17:16:54Z [yakherd]
kind: move
from: hairy
to: shaving
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T17:22:27Z [yakherd]
kind: landed
sha: 7a314e2
---

Done in 7a314e2 (local main).

- `Status::is_open()` (hairy or shaving) is the one definition; `farm::open_descendants` (was `live_descendants`) and the TUI's `open_child_count` use it, so shorn descendants neither block `slaughter` nor die with `--family`.
- `Farm::transition` refuses any finishing move (shorn or dead) over open children; the dead message points at `slaughter --family` (the TUI's single slaughter goes through it).
- `doctor` `unfinished-children` covers dead parents ("is dead but has unfinished children ... revive").
- Found on the way: `list --all --parent-of` dropped dead descendants (the scope ignored `--all`); fixed in `filter::apply`, test `parent_of_with_all_includes_dead_descendants` (empty before the fix).
- Docs/help: `slaughter --help`/`--family`, `doctor --help`, docs/cli.md now has one "Families stay consistent" subsection for both moves, docs/tui.md `X`, README.

Evidence: `cargo test --workspace --locked` green in the CI-like env; updated `slaughter_guards_and_family_sweeps` (shorn grandchild stays shorn, shorn-only child doesn't block, dead move refused, doctor flags hand-edited dead parent), TUI confirm snapshot now "AND its 1 unfinished descendant" (A2 is shorn). Real CLI on a scratch farm: `slaughter <umbrella>` with one shorn and one hairy child refused naming only the hairy one (exit 1); `--family` slaughtered the hairy child and the umbrella; the shorn child stayed shorn; doctor clean.

---
▸ 2026-11-01T17:22:27Z [yakherd]
kind: move
from: shaving
to: shorn
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
