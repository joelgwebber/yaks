---
id: yaks-28dc
title: 'sheds: say when a shed is merged with nothing outstanding, and what keeps a dirty one alive'
status: hairy
type: feature
priority: 3
created: '2026-10-16T19:53:16Z'
updated: '2026-10-16T19:53:16Z'
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
