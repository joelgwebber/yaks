---
id: yaks-85d9
title: yaks sheds counts a created yak's notes too (matches yaks changes)
type: task
priority: 3
created: '2026-10-09T22:35:47Z'
updated: '2026-10-09T22:35:48Z'
parent: yaks-a3d2
labels:
- cli
---

---
▸ 2026-10-09T22:35:48Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-09T22:35:48Z [delta:Delta :Yaks (cont'd)]
Claim (coordinator). Owner: notes-1. Decided by Joel (2026-10-09, "SGTM" on dedupe-1's recommendation in yaks-5138): `yaks sheds` counts a created yak's notes as new notes, so its `farm:` line agrees with `yaks changes`.
Change: in `sheds::compare_farms`, drop the `else` that skips a created yak's notes (count `c.notes.len()` for added yaks too), update the doc comment, update every test assertion that encoded the old rule (yaks-5138's report names `changes_lists_one_shed_per_yak_with_actors`), and the docs that describe `sheds`' notes count (docs/cli.md sheds row says "yaks with new notes (count)"; check README.md and the yaks skill row for any wording that the change makes wrong).
Scope: src/sheds.rs (compare_farms and tests only), tests/changes.rs if it asserts the old count, docs. Out of scope: everything else; another worker is editing Shed/name/resolve in src/sheds.rs right now, so touch only compare_farms and the test assertions.
Evidence: a unit test where a created yak with 2 notes counts as 2 new notes (fails before). Judge: coordinator.
WALKTHROUGH STEP (contrived on purpose, for a demo of the cross-shed ask/answer loop): before you change any code, run `yaks ask <this yak> --note "Mechanics check: the JSON notes[] for a created yak will now list it with its count. OK to change that assertion too?"` and return immediately with a final message that says you are blocked on that ask. Do not wait. You will be woken; then run `yaks inbox --for agent`, `yaks pickup`, and do the work.
