---
id: yaks-c6f4
title: Window/scroll the edit-form content accordion when blocks overflow
status: dead
type: task
priority: 4
created: '2026-08-26T12:53:40Z'
updated: '2026-10-17T18:01:25Z'
parent: yaks-faf5
depends_on:
- yaks-92e5
labels:
- ui
- editing
---

Follow-up to the block accordion in the edit form (yaks-127e). With many comments, the fixed 1-line separators above the focused block can push the focused block's editor off the bottom of a short pane (the single Min(0) editor gets little/no height and later separators clip). Confirmed reproducible on a task with enough comments.

Fix: window or scroll the content stack so the focused block stays fully visible -- e.g. render only a range of blocks around the focused one, or offset the stack so the focused separator + its editor are in view, with a small indicator (…) when blocks are hidden above/below. Low priority: typical comment counts render fine; this only bites with long comment stacks on short viewports.

---
▸ 2026-10-17T04:26:52Z [delta-lead]

May be obviated by the view/edit unification design (yaks-a893); revisit once that design is agreed.

---
▸ 2026-10-17T17:02:15Z [delta-lead]

Obviated by yaks-92e5 (the accordion goes away). Slaughter when that lands.

---
▸ 2026-10-17T18:01:25Z [delta-lead]

Obviated: yaks-92e5 (landed c2b20a0) removed the edit form's content-block accordion this yak was about.

---
▸ 2026-10-17T18:01:25Z [delta-lead]

moved: hairy -> dead
