---
id: yaks-e39c
title: 'yaks changes <shed>: what one shed created, moved and noted since it forked, per yak, with actors'
type: feature
priority: 3
created: '2026-10-09T20:23:49Z'
updated: '2026-10-09T20:24:48Z'
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
