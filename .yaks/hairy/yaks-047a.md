---
id: yaks-047a
title: 'yaks sheds and changes: count an edited yak (title, description, fields), as yaks status does'
type: feature
priority: 2
created: '2026-10-10T17:17:57Z'
updated: '2026-10-10T17:17:57Z'
parent: yaks-ee0a
labels:
- cli
- delta
---

`yaks status` classifies a yak whose title, description or other frontmatter changed as `edited`, but `yaks sheds` and `yaks changes <shed>` apply only four rules (new yak, status move, new note entries, newly set `needs:`; `yak_changes`, the one place they live). So a shed that rewrote a yak's description shows nothing for it. Seen 2026-10-10 on this thread: four rewritten yaks (38dc, 7204, 7eea, ee0a) were missing from `yaks changes` though the Delta diff and `yaks status` showed them.
Goal: a fifth rule, `edited` (the yak exists in both, same status and notes, but its title, parent, labels, deps, priority, type, source, verify or body text outside the note entries differs), in `yak_changes`, so `sheds` counts it and `changes` lists it, text and JSON, and share the classification with `yaks status` rather than copying it. Docs parity: docs/cli.md, README, the yaks skill rows.
Decide in the work: whether `updated:` alone counts (it should not).
