---
id: yak-d369
title: 'TUI: blocked indicator in list view'
type: task
priority: 2
created: '2026-04-05T14:34:24Z'
updated: '2026-04-05T14:41:34Z'
commit: 78d88a9
---

Currently you have to toggle the 't' (tangled) filter to see which hairy tasks are blocked. Add a subtle visual cue on the list row itself — e.g., a chain/lock glyph after the id, or a color tweak — so blocked tasks are recognizable at a glance without filtering. Only applies to hairy tasks with at least one unshorn dep.
