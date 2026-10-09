---
id: yaks-a3d2
title: 'Design pass: CLI affordances and skill/workflow changes for agents using shed discovery (before the UI)'
type: task
priority: 3
created: '2026-10-09T20:12:45Z'
updated: '2026-10-09T20:44:15Z'
parent: yaks-ee0a
labels:
- cli
- skills
- delta
- design
needs: human
---

Joel (2026-10-09): before the UI, pause to work through the CLI and skill/workflow changes agents need to use shed discovery well. Inputs: yaks-ee0a (proposed per-shed commands: `yaks changes <shed>`, `yaks inbox --sheds`, `yaks show <id> --sheds`, `yaks answer <id>@<shed>`), yaks-7eea (names and the -C/--shed selector; visit vs full access), and the facts from yaks-c635 (from inside any checkout an agent sees every shed, including a worker that has not committed yet; a clone on a shared machine cannot see the human's checkout).
Questions to settle with Joel:
1. Which skill steps should now call `yaks sheds` (spawn, while a worker runs, a worker that is overdue, landing), and where `yaks discover` belongs (diagnosis only)?
2. How does a shed get a human-readable name? The thread title is in DELTA_THREAD_TITLE only inside that thread; a worker's YAKS_ACTOR name shows in `who:` only after its first note.
3. The selector: -C <path> and/or --shed <name|dir|actor>; which commands take it first?
4. A worker on a shared machine cannot see the human's checkout: does any skill step assume it can?
Output: a short design note on this yak and one yak per agreed change.

---
▸ 2026-10-09T20:23:49Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-09T20:23:49Z [delta:Delta :Yaks (cont'd)]
Design note (coordinator, 2026-10-09). Build order, each piece read-only unless marked:
0. DONE in the prep commit: `sheds::resolve(sheds, query)` resolves a shed by exact path, Delta dir id, branch (`main` = the primary), an actor in `who`, then a unique case-insensitive substring; ambiguity and no-match errors list the candidates. `Shed` gains `dir` (Delta dir id), `primary`, and `name()` (main, else the first `who`, else dir, else branch, else the dir name); `yaks sheds` prints and emits `name`, `dir` and `primary`.
1. Global `yaks -C <path>`: identical to cd-ing there first (full access, like `git -C`). Worker yak.
2. `yaks inbox --sheds`: open asks across sheds with each shed's name and the question text, read from each shed's working tree. Worker yak.
3. `yaks changes <shed>`: per yak, what that shed created, moved and noted since it forked (from `FarmDelta` plus the note entries), with actors. Worker yak.
4. Next wave: `yaks show <id> --sheds`; `yaks answer <id>@<shed>` (the one cross-shed write, under that shed's farm lock); `--shed` on read commands; then a skills pass that wires `sheds`/`changes`/`inbox --sheds` into yaks-coordinating(-delta) and yaks-working, followed by a cold read.
Questions for Joel are two asks on this yak (names; what --shed may do).

---
▸ 2026-10-09T20:23:49Z [delta:Delta :Yaks (cont'd)]
asked: needs human

Q1 NAMES. A thread's title exists only inside that thread (DELTA_THREAD_TITLE). To show it from other sheds, yaks has to record it. My lean: any yaks command run inside a Delta clone writes `yaks.shed.title` (and the thread id) into that clone's OWN git config (never committed, rebuildable), and `yaks sheds` reads it through the pin chain; the name falls back to who:/dir/branch as now. The cost: an ordinary command then writes to that clone's git config. Alternative: no recording; names stay who:/dir. Q2 WHAT --shed MAY DO. My lean: `--shed <name>` is a VISIT: read commands, plus answer/ask, with a one-line banner naming the shed; `-C <path>` stays the deliberate full-access route. Alternative: --shed is just -C by name (full access). Please answer each Q (yes, no, or another option).

---
▸ 2026-10-09T20:26:07Z [delta:Delta :Yaks (cont'd)]
Spawned (base 192af4b, all claude-sonnet-5-5): cflag-1 83ecd85713f64fc7 -> yaks-4e39; inbox-1 69ae675719914f1d -> yaks-2917; chg-1 92c1a81063e44ac0 -> yaks-e39c.

---
▸ 2026-10-09T20:44:15Z [delta:Delta :Yaks (cont'd)]
Landed wave 1: yaks-4e39 (-C, d763638), yaks-2917 (inbox --sheds, 6fb56a2), yaks-e39c (changes, f0e6f0b). Review fixes in the next commit: inbox --sheds repeated every ask whose shed forked before our later note (seen live: 3 copies), and 'main' was ambiguous because Delta clones are on branch main. Gate after all three plus fixes: 12 suites ok, 493 unit tests.
