---
id: yaks-85d9
title: yaks sheds counts a created yak's notes too (matches yaks changes)
type: task
priority: 3
created: '2026-10-09T22:35:47Z'
updated: '2026-10-09T22:45:17Z'
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

---
▸ 2026-10-09T22:38:36Z [notes-1]
asked: needs human

Mechanics check: the JSON notes[] for a created yak will now list it with its count. OK to change that assertion too?

---
▸ 2026-10-09T22:38:41Z [notes-1]
Started: Fri Oct  9 22:38:33 UTC 2026
yaks path: /Users/joel/src/yaks/.delta/worktrees/2wqa4188f15r/yaks/.yaks/shaving/yaks-85d9.md

---
▸ 2026-10-09T22:39:51Z [delta-lead]
Yes, as the yakherd: update that assertion; the JSON notes[] for a created yak now carries its count.

---
▸ 2026-10-09T22:40:18Z [notes-1]
picked up

Drop the else in compare_farms, update doc + test assertions + docs

---
▸ 2026-10-09T22:43:00Z [notes-1]
Decision: drop the `else` in `compare_farms` (`if c.added {..}` then an independent `if !c.notes.is_empty()`), rather than adding a special case or changing `yak_changes`; the counts then follow `yak_changes` exactly. Rejected: filtering in render only (JSON `notes[]` would still disagree with `changes`).
Decision: docs — only docs/cli.md `sheds` row needed a clause (README, skills say "new notes" generically and stay true). Rejected: rewording README/skills (no wrong wording).
Decision: updated `changes_lists_one_shed_per_yak_with_actors` ("3 new notes" -> "4 new notes") and `compare_farms_is_yak_changes_counted` (ffff's 1 note in `d.notes`); added `compare_farms_counts_a_created_yaks_notes` (created yak with 2 notes counts 2).

---
▸ 2026-10-09T22:44:27Z [notes-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-09T22:45:17Z [notes-1]
Summary: `compare_farms` now counts a created yak's notes (independent `if`, no `else`), so `yaks sheds` `farm:` and JSON `notes[]` agree with `yaks changes`. Tests: new `compare_farms_counts_a_created_yaks_notes`; updated `changes_lists_one_shed_per_yak_with_actors` (3 -> 4 new notes) and `compare_farms_is_yak_changes_counted`. Docs: docs/cli.md sheds row. Fail-first check: restored the `else` temporarily, the new test failed (left [] right [("yaks-bbbb", 2)]), then restored the fix. Gate: `cargo test --release` all suites ok (495 unit + integration); `yaks verify` PASS.

---
▸ 2026-10-09T22:45:17Z [notes-1]
moved: shaving -> shorn
