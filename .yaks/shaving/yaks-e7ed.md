---
id: yaks-e7ed
title: 'yaks show <id> --sheds: this checkout''s copy, then one line per shed whose copy differs'
type: feature
priority: 3
created: '2026-10-09T20:44:57Z'
updated: '2026-10-09T20:44:58Z'
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
