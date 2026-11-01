---
id: yaks-35a4
title: 'Refuse opening a yak under a finished parent: create/reparent --parent, regrow/revive/shave of a child of a shorn or dead yak'
status: shorn
type: task
priority: 2
created: '2026-11-01T16:31:51Z'
updated: '2026-11-01T17:28:01Z'
parent: yaks-9b41
labels:
- cli
---

Found shearing yaks-62f2; widened under yaks-9b41. Every move that makes a yak hairy or shaving while its parent is shorn or dead breaks the family invariant:

- `yaks create --parent <finished>`
- `yaks reparent <open> --parent <finished>` (also `bulk --reparent`, the TUI reparent)
- `yaks regrow` / `revive` / `shave` of a child whose parent is finished

## Plan

- Refuse each, naming the parent and its state and the two fixes: reopen the parent first (`yaks regrow <parent>`), or file it under an open ancestor or top-level.
- A batch that opens takes parents before children (the mirror of `children_first`), so `yaks regrow <parent> <child>` works in either order.
- One check in `Farm` used by create, reparent and transition, so the CLI and TUI share it.
- Tests that fail first; `--help`, docs/cli.md.

Decided: refuse rather than warn or auto-reopen the parent. A warning is how the state got here; auto-reopening changes someone else's yak silently.

---
▸ 2026-11-01T17:22:27Z [yakherd]
kind: move
from: hairy
to: shaving
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T17:28:01Z [yakherd]
kind: landed
sha: 95fa1e2
---

Done in the commit above (local main).

- One refusal wording, `farm::finished_parent_refusal`, at the three choke points: `Farm::create` (`CreateOutcome::ParentFinished`), `Farm::reparent` (an open yak under a finished one; `bulk --reparent` and the TUI go through it) and `Farm::check_transition` (opening moves; typed `FinishedParent`). Finished yaks still reparent freely; no-op moves are never refused.
- `Farm::check_transition` is now the whole family check (both directions), and `transition` calls it under the lock. `Farm::batch_order(ids, dest)` (was `children_first`): finishing moves deepest first, opening moves shallowest first; CLI `transition_many` and the TUI bulk use it for every dest.
- TUI: checks before writing the note, so a refused shear leaves no comment (a gap from yaks-62f2); bulk reports `· N skipped: unfinished children` / `· N skipped: parent is finished` instead of "failed"; the old bulk-slaughter pre-skip (`live_child_count`) is gone, the farm enforces it.
- Help/docs: `create --parent`, `shave`, `regrow`, `revive`, `reparent` --help; docs/cli.md "Families stay consistent".

Evidence: `cargo test --workspace --locked` green (CI-like env). New `nothing_opens_under_a_finished_parent` (create under shorn/dead, reparent refused/allowed, regrow+shave refused, no-op allowed, batch_order parents first) and TUI `refused_shear_leaves_no_comment_and_regrow_takes_parents_first`, which fails with the pre-check removed (my first version passed regardless because the TUI rows were stale; it reloads now). Real CLI: `create late --parent <shorn>` and `regrow <child>` refused with the message (exit 1); `regrow <child> <parent>` regrew both, parent first; doctor clean.

---
▸ 2026-11-01T17:28:01Z [yakherd]
kind: move
from: shaving
to: shorn
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
