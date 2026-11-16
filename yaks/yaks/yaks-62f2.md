---
id: yaks-62f2
title: 'Invariant: a shorn yak has no unfinished children (doctor check, shorn refuses, skills rule)'
status: shorn
type: task
priority: 2
created: '2026-10-31T19:28:04Z'
updated: '2026-11-01T16:42:39Z'
parent: yaks-9b41
tags:
- cli
- skills
---

Joel (2026-10-31): a yak is shorn only when all its children are shorn (or dead). If the parent really is done enough, graduate the unfinished children first: reparent them to another family, or unparent them.

Found when `yaks-da23` was shorn while it still held `yaks-f1e6` and `yaks-19c3` (hairy); fixed by moving them to `yaks-039b`.

## Scope

- **`yaks doctor`:** a check, "shorn parent with unfinished children", listing each parent and the children that are not shorn or dead.
- **`yaks shorn <id>`:** refuse (or at least warn) when the yak has children that are not shorn or dead, naming them and the fix (`yaks reparent <child> --parent <other>` / `--unparent`). Decide refuse vs warn; the lean is refuse, with `--force` only if a real need appears.
- **Skills:** the `yaks` skill's Workflow "Done" step already says to shear the parent when its last child is shorn. Add the converse as a hard rule: never shear a yak over unfinished children; graduate them first. The same goes for `yaks-working`'s finish step and the coordinating skill's reconcile step.
- **Docs and help:** `docs/cli.md` (`shorn`, `doctor`) and `--help`.

---
▸ 2026-11-01T16:23:55Z [yakherd]

kind: move
from: hairy
to: shaving
---

moved: hairy -> shaving

---
▸ 2026-11-01T16:31:37Z [yakherd]

kind: landed
sha: ac5c6e2
---

Done in ac5c6e2 (main, local only, not pushed).

- `Farm::transition` refuses Shorn when the yak has hairy/shaving children (typed `farm::UnfinishedChildren`, names them and the fix: shear them too, or `yaks reparent <child> --parent <other>` / `--unparent`). Dead and shorn children don't count; re-shearing an already-shorn yak is untouched.
- Batches take children before parents (`Farm::children_first`, deepest first, stable): `yaks shorn <parent> <child>` works in any order, and the TUI's bulk shear of a marked family shears it whole (it walked marks in id order, so a parent sorting first would have been refused).
- `yaks doctor`: new `unfinished-children` issue (rank 9) for farms from before or hand edits.
- Docs/help: `shorn --help`, `doctor --help`, docs/cli.md (state transitions + doctor), the hard rule in the yaks skill Done step, yaks-working Finish, yaks-coordinating Reconcile. Also fixed the stale `--as` help on update/shave/shorn/regrow/slaughter/revive (missed in yaks-5d1c: now names `git config yaks.actor`).

Evidence:
- Unit `farm::tests::shorn_refuses_a_parent_over_unfinished_children` (refusal, dead child ignored, children_first order with a grandchild, doctor flags a hand edit).
- TUI `bulk_shear_takes_children_before_their_parent`: fails with the reorder removed (a0 stays Hairy), passes with it.
- Real CLI on a scratch farm: `yaks shorn P` -> "error: yak-6e38 has unfinished children: yak-ee7f (hairy). Shear them first (or with it: ...)", exit 1; `yaks shorn P C` -> "Shorn! yak-ee7f / Shorn! yak-6e38", exit 0; doctor clean.
- `yaks doctor` on this farm and memcraft's: clean (no existing violations).
- Gate: `cargo test --workspace --locked` in a CI-like env (no Delta env, useConfigOnly git config, fresh TMPDIR): all green. One existing fixture (`doctor_clean_farm_has_no_issues`) was itself a shorn parent over a hairy child; its parent is now hairy.

Follow-up filed: create/reparent onto a shorn parent still produces the state (doctor catches it).

---
▸ 2026-11-01T16:31:37Z [yakherd]

kind: move
from: shaving
to: shorn
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
