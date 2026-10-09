---
id: yaks-a3d2
title: 'Design pass: CLI affordances and skill/workflow changes for agents using shed discovery (before the UI)'
type: task
priority: 3
created: '2026-10-09T20:12:45Z'
updated: '2026-10-09T20:12:45Z'
parent: yaks-ee0a
labels:
- cli
- skills
- delta
- design
---

Joel (2026-10-09): before the UI, pause to work through the CLI and skill/workflow changes agents need to use shed discovery well. Inputs: yaks-ee0a (proposed per-shed commands: `yaks changes <shed>`, `yaks inbox --sheds`, `yaks show <id> --sheds`, `yaks answer <id>@<shed>`), yaks-7eea (names and the -C/--shed selector; visit vs full access), and the facts from yaks-c635 (from inside any checkout an agent sees every shed, including a worker that has not committed yet; a clone on a shared machine cannot see the human's checkout).
Questions to settle with Joel:
1. Which skill steps should now call `yaks sheds` (spawn, while a worker runs, a worker that is overdue, landing), and where `yaks discover` belongs (diagnosis only)?
2. How does a shed get a human-readable name? The thread title is in DELTA_THREAD_TITLE only inside that thread; a worker's YAKS_ACTOR name shows in `who:` only after its first note.
3. The selector: -C <path> and/or --shed <name|dir|actor>; which commands take it first?
4. A worker on a shared machine cannot see the human's checkout: does any skill step assume it can?
Output: a short design note on this yak and one yak per agreed change.
