---
id: yaks-28dc
title: 'sheds: say when a shed is merged with nothing outstanding, and what keeps a dirty one alive'
status: shorn
type: feature
priority: 3
created: '2026-10-16T19:53:16Z'
updated: '2026-10-16T20:43:43Z'
parent: yaks-5662
labels:
- cli
- delta
---

Once a shed's work has landed, `yaks sheds` reads `idle` plus `no changes of its own` / `landed (N)`, with no hint whether anything still keeps it alive. In surveyor, 4nqz8n8wn5z6 and tdc89esj1r0d each carried the same discarded `go/bench/refine.go` experiment (uncommitted) and pvm560rcetw7 had untracked `.sightmap/` output; the only clue was the DIRTY count in the table header row.

Change (Joel, 2026-10-16 SGTM):
- When the shed's HEAD is an ancestor of the primary's HEAD (its commits are in) and its farm has nothing outstanding, say so plainly, e.g. `merged: nothing outstanding`.
- When it is dirty, say what keeps it: `uncommitted: N modified, M untracked` (counts from the `status --porcelain` already run), so a human can tell a shed safe to delete from one holding work.

Settle and record: exact wording, whether "merged" checks against the primary or the reference ref, the JSON fields (e.g. `merged: bool`, `dirty: {modified, untracked}`), and the TUI Sheds tab row (`shed_row` in src/tui/sheds_tab.rs). Update docs/cli.md, README.md, the yaks skill, the `Sheds` help and the Sheds tab help.

Evidence: tests for merged+clean, merged+dirty, not merged; a toque frame of the Sheds tab attached; real `yaks sheds` from surveyor.

---
▸ 2026-10-16T20:36:20Z [delta]

moved: hairy -> shaving

[sheds-tab-surveyor](yaks-28dc/sheds-tab-surveyor.txt)

[sheds-detail-merged-dirty](yaks-28dc/sheds-detail-merged-dirty.txt)

---
▸ 2026-10-16T20:43:43Z [delta]

Decision: "merged" means the shed's HEAD is an ancestor of the PRIMARY's HEAD (not the viewer's, not the reference ref), so every viewer gets the same answer, and it matches what Joel lands into. It's `null` for the primary or when the primary isn't visible. Text: a `state:` line just above `farm:`:
- `merged, nothing outstanding` when merged, nothing outstanding in the farm, and a clean tree;
- otherwise `merged` and/or `uncommitted: N modified, M untracked`;
- nothing at all for a shed that isn't merged and is clean (the ahead count already says it has commits).

JSON adds `merged` (bool|null) and `untracked` (the `??` part of `dirty`). TUI: the detail pane gets a `state` field. The list row appends `merged, nothing outstanding` or `merged · uncommitted files` only for a merged shed with nothing outstanding, since `✎N` already counts dirty files. Rejected: a separate row tag column (too wide for 80 columns).

---
▸ 2026-10-16T20:43:43Z [delta]

Evidence. New test `merged_and_uncommitted_say_what_keeps_a_shed`: merged+clean, merged with 1 modified and 1 untracked, not merged+clean, and the primary never merged. Three Sheds-tab insta snapshots gained the `state  uncommitted: N modified` line on their dirty fixture shed (reviewed; that's the only change). Full `cargo test --release`: green.

Real machine, from ~/src/fs/subtext/surveyor, with two scratch worktrees added then removed (HEAD~1 clean, HEAD~3 plus an untracked file):
```
heuristics-models  state: uncommitted: 7 modified, 6 untracked
yaks-28dc-clean    state: merged, nothing outstanding
yaks-28dc-merged   state: merged · uncommitted: 1 untracked
```
Headless TUI frames are attached: the Sheds tab list and the detail pane of the merged+dirty shed. Docs: docs/cli.md, docs/tui.md, README.md, the yaks skill, `Sheds --help`. No new key, so the `?` overlay is unchanged.

---
▸ 2026-10-16T20:43:43Z [delta]

moved: shaving -> shorn
