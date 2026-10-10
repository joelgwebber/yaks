---
id: yaks-047a
title: 'yaks sheds and changes: count an edited yak (title, description, fields), as yaks status does'
type: feature
priority: 2
created: '2026-10-10T17:17:57Z'
updated: '2026-10-10T18:57:42Z'
parent: yaks-ee0a
labels:
- cli
- delta
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
