---
id: yaks-e39c
title: 'yaks changes <shed>: what one shed created, moved and noted since it forked, per yak, with actors'
type: feature
priority: 3
created: '2026-10-09T20:23:49Z'
updated: '2026-10-09T20:36:24Z'
parent: yaks-a3d2
labels:
- cli
- delta
---

---
▸ 2026-10-09T20:24:48Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-09T20:24:48Z [delta:Delta :Yaks (cont'd)]
Claim (coordinator). Owner: chg-1.
Goal: `yaks changes <shed>` prints what ONE shed did to its farm since it forked, per yak, with actors: a created yak (title, status), a moved yak (from -> to), new note entries (timestamp, actor, the first line of the text), and `needs:` newly set. It is the per-yak expansion of what `yaks sheds` summarizes as "farm: 2 new, 1 moved, 3 new notes". The shed is resolved with `sheds::resolve(&sheds, query)` (already built: path, Delta dir id, branch, `main`, an actor, a unique substring); print its error as is and exit 1 on no or ambiguous match. Same baseline as `sheds` (fork / sync / merge-base / vs this checkout), and say which, as `sheds` does (`vs_marker`). `--json` with one object per changed yak. A Shared/None farm prints why there is nothing to show.
Data: reuse what `sheds::inspect` already computes. `FarmDelta` has added/moved/needs and only COUNTS of notes, so you need the note entries themselves: extend `compare_farms`, or add a sibling function that returns the entries (reusing `store::parse_notes`), keeping `yaks sheds` output unchanged. Read-only in the other checkout.
Scope: src/sheds.rs (a new pub fn plus data, beside `compare_farms`/`label`; do not change `discover`/`resolve`/`shed_paths`), src/main.rs (a `Changes { shed, json }` variant and its render). Docs: docs/cli.md (a `changes` row after `sheds`), README.md, `.agents/skills/yaks/SKILL.md` (a command-table row), clap help.
Out of scope: `-C`, inbox, any write.
Evidence: tests that FAIL before your change: a worktree shed that creates one yak, moves one, and adds two notes by two actors; `changes <branch>` lists exactly those with the actors; `changes main` from that shed shows the main checkout's own changes since the fork (or "no changes of its own"); an unknown name prints resolve's candidate list and exits 1. Judge: the coordinator, at landing.

---
▸ 2026-10-09T20:26:50Z [chg-1]
Start Fri Oct  9 20:26:50 UTC 2026; path: /Users/joel/src/yaks/.delta/worktrees/99yx2fj5eh45/yaks/.yaks/shaving/yaks-e39c.md

---
▸ 2026-10-09T20:34:58Z [chg-1]
Implemented `yaks changes SHED` (+ `--json`).
- src/sheds.rs: `YakChange`, pure `yak_changes(base, theirs)` (same four rules as `compare_farms`, but carries the note entries via `store::parse_notes`), `Changes`, `changes(shed, cwd, farm_root)`, `render_changes`, `changes_json`; `vs_marker` now delegates to a shared `base_marker` (output unchanged). `compare_farms`, `discover`, `resolve`, `shed_paths` untouched.
- src/farm.rs: `Farm::shed_changes` (mirrors `Farm::sheds`). src/main.rs: `Changes { shed, json }` variant, dispatch arm.
- Tests: 4 unit tests in sheds.rs (per-yak with two actors incl. created+moved+2 notes, needs, `main` from a shed, shared farm) and tests/changes.rs (2 binary tests: branch listing with actors, unknown name -> candidate list + exit 1).
- Fail-without-change check: reverted src/{main,farm,sheds}.rs via `git apply -R`, ran `cargo test --release --test changes`: both tests FAILED; unit tests cannot compile without the change. Restored and re-verified identical.
- Docs: docs/cli.md, README.md, .agents/skills/yaks/SKILL.md (row, --json list, shed bullet), clap help.

---
▸ 2026-10-09T20:34:58Z [chg-1]
Decision: reuse the baseline `inspect` already chose (`Shed.farm_base`) and re-read the base farm with `base_farm`, rather than extend `FarmDelta`/`compare_farms` to carry entries. Rejected: widening `FarmDelta` (would change `Eq`/shape that `sheds` tests and JSON build on; risk to byte-identical `sheds` output).
Decision: `changes` is a sibling pure fn `yak_changes` duplicating the four compare rules rather than refactoring `compare_farms` onto it. Rejected: reimplementing `compare_farms` as a count over `yak_changes` (touches the function whose output must stay identical; follow-up if wanted).
Decision: notes of a CREATED yak are all listed in `changes`; `sheds` counts only notes on yaks that existed at the base (existing behavior, so "3 new notes" there vs 4 entries here). Rejected: changing `sheds`.
Decision: `--json` on a shared/none/unreadable farm prints `[]` on stdout and a `note: nothing to show: <why>` on stderr; human output prints `nothing to show: <why>`. Rejected: an error exit (the shed resolved fine) and a non-array JSON object (spec says one object per changed yak).
Decision: JSON per-yak objects repeat `vs`/`base` instead of a wrapper object, to keep the "array of one object per changed yak" shape.
Decision: note text shown is the first non-empty trimmed line; JSON carries full `text`.
Decision: `resolve` error goes to stderr as `error: <resolve's message>`, exit 1, nothing on stdout.

---
▸ 2026-10-09T20:35:44Z [chg-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-09T20:36:24Z [chg-1]
Summary: yaks changes SHED shipped (sheds.rs/farm.rs/main.rs, tests in sheds.rs + tests/changes.rs, docs). Full 'cargo test --release' green (sheds suite 491 passed, 0 failed); gate recorded PASS. Finished Fri Oct  9 20:36:24 UTC 2026.

---
▸ 2026-10-09T20:36:24Z [chg-1]
moved: shaving -> shorn
