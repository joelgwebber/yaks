---
id: yaks-047a
title: 'yaks sheds and changes: count an edited yak (title, description, fields), as yaks status does'
status: shorn
type: feature
priority: 2
created: '2026-10-10T17:17:57Z'
updated: '2026-10-10T19:06:58Z'
parent: yaks-ee0a
labels:
- cli
- delta
verify: cargo test --release
---

`yaks status` classifies a yak whose title, description or other frontmatter changed as `edited`, but `yaks sheds` and `yaks changes <shed>` apply only four rules (new yak, status move, new note entries, newly set `needs:`; `yak_changes`, the one place they live). So a shed that rewrote a yak's description shows nothing for it. Seen 2026-10-10 on this thread: four rewritten yaks (38dc, 7204, 7eea, ee0a) were missing from `yaks changes` though the Delta diff and `yaks status` showed them.
Goal: a fifth rule, `edited` (the yak exists in both, same status and notes, but its title, parent, labels, deps, priority, type, source, verify or body text outside the note entries differs), in `yak_changes`, so `sheds` counts it and `changes` lists it, text and JSON, and share the classification with `yaks status` rather than copying it. Docs parity: docs/cli.md, README, the yaks skill rows.
Decide in the work: whether `updated:` alone counts (it should not).

---
▸ 2026-10-10T18:56:18Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-10T18:56:19Z [delta-lead]
Claim (coordinator). Owner: edit-1. Spec = the description above plus:
Scope by symbol (src/sheds.rs): `YakChange` (add `edited: bool`), `yak_changes` (the one rule site), the `FarmDelta`/`compare_farms` counts and the text/JSON renders of the `farm:` line and of `yaks changes` (an `[edited]` tag in the bracket list, `edited` in JSON), and their tests. Docs: docs/cli.md, README.md, `.agents/skills/yaks/SKILL.md` (the sheds/changes rows and anything that lists the four rules).
Rule: a yak present in both the baseline and the shed, with nothing else to report, is `edited` when its frontmatter fields other than `updated`/`status`-via-directory/`needs` (title, parent, labels, depends_on, priority, type, source, verify) or its description (the body before the first note block) differ. A change that only appends notes, moves status or sets `needs:` is NOT also `edited`; `updated:` alone is NOT an edit. A new yak is `new`, not `edited`.
Share the classification with `yaks status` (`describe` in src/status.rs works on text, `yak_changes` on `Task`s): prefer ONE predicate both call; if that is not clean, keep two but add a test that feeds both the same pair and requires the same verdict. Record a `Decision:` note either way.
Out of scope: src/sheds.rs baseline code (`fork_point`, `nearer_baseline`, `BaseKind`, `GitState`, the module docs on baselines: another worker owns them), `landed`/`stale`, the current-shed row. Put your new tests in their own `mod`-level block or beside the existing `yak_changes` tests, NOT at the end of the file's test module (the other worker appends there).
Evidence (tests FAIL first): a shed that only rewrote a yak's description shows it `edited` in `changes` and counts it in `farm:`; a title-only edit; an `updated:`-only change is NOT shown; notes-only/move-only changes are not also `edited`; text and JSON. Real machine: from a clone with a rewritten yak, `yaks changes <shed>` lists it. Judge: coordinator.

---
▸ 2026-10-10T18:57:42Z [delta-lead]
Coordinator: owner is now edit-2 (edit-1 died before doing anything; nothing to review or copy).

---
▸ 2026-10-10T18:59:33Z [edit-2]
Start: Sat Oct 10 18:59:33 UTC 2026
path: /Users/joel/src/yaks/.delta/worktrees/q2anm14m6tny/yaks/.yaks/shaving/yaks-047a.md
Plan: shared predicate store::edited(old,new) (fields incl created/extra, description before first note, old notes must be a prefix of new); status::describe and yak_changes both call it.

---
▸ 2026-10-10T19:04:53Z [edit-2]
Failing first: with store::edited stubbed to `false`, `cargo test --release sheds::` gave 4 failures, e.g. changes_reports_an_edited_yak:
  assertion `left == right` failed
    left: []
   right: [("yaks-aaaa", true), ("yaks-bbbb", true)]
(also changes_edited_is_reported_with_a_move_and_notes, status_and_changes_agree_on_edited, changes_edited_ignores_stamps_notes_moves_and_needs). Restored predicate: all pass. (Two of the four initially also failed for test bugs: a temp-dir tag containing "edited" and comparing list emptiness instead of the flag; fixed.)

---
▸ 2026-10-10T19:04:53Z [edit-2]
Decision: ONE predicate. `store::edited(&Task, &Task)` (+ `store::description`, and `parse_task` made pub) is called by `yak_changes` (shed vs baseline) and by `status::describe` (file at HEAD vs now, parsed to Tasks). Rejected: two predicates plus an agreement test (the claim's fallback), since status could be moved onto Tasks cleanly. status_and_changes_agree_on_edited still feeds both the same pairs.
Decision: edited = title, type, priority, created, parent, labels, depends_on, source, verify, unmodelled `extra` frontmatter lines, or description (body before first note block). NOT: updated, status, needs, notes. Rejected: counting `needs` as an edit (it is already reported as `needs:`); the claim listed `verify` etc. and I added created/extra so nothing but the four exclusions can change silently.
Decision: `updated:` alone is not an edit for sheds/changes (as specified). `yaks status` keeps its existing rule that a file whose only difference is `updated:` still reads `edited` (describe_yak, existing test only_the_updated_stamp_changing_still_reads_as_edited), because there git sees a real file change and must say something; that is a status-only fallback outside the shared predicate. Rejected: changing status.
Decision: an edit alongside a move or notes is ALSO reported edited (`[hairy -> shaving, edited]`, matching status's `notes +1, edited`); a created yak is only `new`. `edited` is a bool on YakChange, an `edited: [ids]` array in sheds JSON farm, `"edited": bool` per yak in changes JSON. Text: `farm: ... 2 edited` plus `    <id> edited` lines; changes brackets get `, edited`.
Decision: the old status `describe` compared raw text (strip_prefix of old); the new one compares parsed fields, so a whitespace-only or key-reordering rewrite of the frontmatter no longer reads edited, and a body whose description is unchanged but whose note text was altered in place is not an edit (notes are append-only by contract). A file that does not parse still reads edited.

---
▸ 2026-10-10T19:06:58Z [edit-2]
verify: `cargo test --release` -> PASS (exit 0)

---
▸ 2026-10-10T19:06:58Z [edit-2]
Real machine (./target/release/yaks, temp git repo + `git worktree add` shed; aaaa description rewritten, bbbb title changed, cccc only `updated:` bumped):
  changes: `yaks-aaaa  yak aaaa  [hairy, edited]` / `yaks-bbbb  renamed bbbb  [hairy, edited]` (cccc absent); --json edited true,true.
  sheds:   `farm: 2 edited` + `    yaks-aaaa edited` / `    yaks-bbbb edited`.
  status in the shed: aaaa, bbbb, cccc all `edited` (cccc is the documented status-only updated: fallback).
Summary: store::{description, edited} shared predicate; status::describe now parses both sides and calls it; sheds YakChange.edited, FarmDelta.edited, yak_changes/compare_farms, text+JSON renders; 5 new tests in src/sheds.rs beside the yak_changes tests; docs/cli.md, README.md, .agents/skills/yaks/SKILL.md, main.rs clap help updated (grep for 'needs:` newly set' / 'needs:` set' finds no remaining four-rule list). Full `cargo test --release` ok (505 unit + integration suites).

---
▸ 2026-10-10T19:06:58Z [edit-2]
moved: shaving -> shorn
