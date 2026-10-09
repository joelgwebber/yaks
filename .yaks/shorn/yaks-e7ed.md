---
id: yaks-e7ed
title: 'yaks show <id> --sheds: this checkout''s copy, then one line per shed whose copy differs'
type: feature
priority: 3
created: '2026-10-09T20:44:57Z'
updated: '2026-10-09T20:53:54Z'
parent: yaks-a3d2
labels:
- cli
- delta
---

---
▸ 2026-10-09T20:44:58Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-09T20:44:58Z [delta:Delta :Yaks (cont'd)]
Claim (coordinator). Owner: show-1.
Goal: `yaks show <id> --sheds` prints the normal `show` for THIS checkout's copy, then a `Sheds:` section with one block per OTHER shed (as `sheds::discover` lists them) whose copy of the yak differs from ours: the shed's name (`Shed::name()`) and path, its status, its `needs:` when set, and the note entries it has that ours lacks (timestamp, actor, first line; reuse `store::parse_notes`). Sheds whose copy is identical, that lack the yak, or whose farm is Shared/None are summarized in ONE trailing line (e.g. `same in 3 sheds; absent in 1; shared farm in 0`), not listed. If the yak exists ONLY in sheds (not here), `show <id> --sheds` still succeeds: it says it is not in this checkout and shows each shed's copy; plain `show` keeps exiting 1. `--json`: the existing show object plus a `sheds` array (shed, path, status, needs, new_notes [{ts, actor, text}], same: bool, absent: bool); record the shape as a Decision: note.
Data: `Farm::sheds(cwd)` for the list, then each shed's farm via `store::discover_with(path, None)` + `store::load_task_by_id`. Read-only in other checkouts.
Scope: src/main.rs (the `Show` variant gains `--sheds`, its arm and a render fn beside render_show), src/farm.rs (a read-only fn beside `Farm::show`), src/json.rs if useful. Docs: docs/cli.md (show row), README.md, `.agents/skills/yaks/SKILL.md`, clap help.
Out of scope: writes; sheds.rs internals (another worker is refactoring compare_farms there: do not edit src/sheds.rs at all).
Evidence: tests that FAIL before (tests/show_sheds.rs, real binary, a git worktree shed as in tests/inbox_sheds.rs): a shed that added a note and moved the yak shows exactly that note and the status; an identical shed is only counted; a yak only in the shed is shown with --sheds and still exits 1 without it. Judge: coordinator, at landing.

---
▸ 2026-10-09T20:52:33Z [show-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-09T20:53:54Z [show-1]
Start: Fri Oct  9 20:46:28 UTC 2026. `yaks path yaks-e7ed`: /Users/joel/src/yaks/.delta/worktrees/tv2dwd6ghd1a/yaks/.yaks/shaving/yaks-e7ed.md
Built: `Show` gains `--sheds`; `Farm::show_sheds` (src/farm.rs, beside `show`) returns `ShedCopies`/`ShedCopy`; `render_show_sheds` beside `render_show`; `json::shed_copies_value`. Docs: docs/cli.md, README.md, skills/yaks/SKILL.md, clap help.
Evidence: tests/show_sheds.rs (3 tests, real binary, git worktree sheds). Without my src changes (git stash of main/farm/json) all 3 FAIL: `error: unexpected argument '--sheds' found`; with them pass. Gate `cargo test --release`: every suite ok (493 unit, show_sheds 3 passed, inbox_sheds 7, etc.). Note: `yaks verify` ran the yak's configured `cargo test -p yaks` (debug) -> PASS; the stated release gate was run separately by hand, all `test result: ok`.

---
▸ 2026-10-09T20:53:54Z [show-1]
Decision: JSON shape = the existing show object plus `sheds: [{shed, path, status, needs, new_notes:[{ts,actor,text}], same, absent}]`, one entry per shed with a farm of its own (all of them, including same/absent ones, flagged by booleans so scripts need no text parsing); `status`/`needs` null when absent/unset. Rejected: listing only differing sheds (hides the counts the text form shows), a separate summary object (breaks "existing object plus sheds"). Without --sheds there is no `sheds` key.
Decision: "same" = same status AND same `needs:` AND no note entry beyond ours. So a shed merely behind (we have later notes) counts as same, matching `sheds`' "merely behind shows nothing". Rejected: byte-equal file comparison (would list every behind shed because our copy moved on) and a fork-point baseline (needs sheds.rs internals, out of scope).
Decision: new notes = shed's parsed note entries not present (exact ts+actor+text) in ours, via `store::parse_notes`. Rejected: counting beyond our length (breaks when both sides diverged).
Decision: trailing line text `same in N shed(s); absent in N; no farm of its own in N shed(s)`. The spec's "shared farm in N" is not accurate for None/Unreadable sheds, so I used "no farm of its own" for Shared/None/Unreadable together; the claim note said "e.g.". Absent counts only sheds with a farm of their own that lack the yak.
Decision: yak only in sheds: prints `<id> is not in this checkout.` then the Sheds section (blocks also carry `title:`), exit 0; JSON object is `{id, sheds}`. Absent everywhere still exits 1. Plain `show` unchanged.
Decision: sheds with a different-but-unreadable farm or Shared/None are not listed individually (spec); status is printed as the status directory name (hairy/shaving/...) matching JSON, not the `{:?}` capitalised form of `show`.

---
▸ 2026-10-09T20:53:54Z [show-1]
moved: shaving -> shorn
