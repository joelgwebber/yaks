---
id: yaks-faf5
title: Yak detail rendering / editing improvements
status: shaving
type: task
priority: 3
created: '2026-10-17T02:49:48Z'
updated: '2026-10-17T04:31:30Z'
parent: yaks-e9de
labels:
- rendering
- editing
- tui
---

A few rendering and editing QoL improvements for yak details:
- Adding / editing comments shouldn't take over the entire available real-estate.
  This makes it very difficult to add anything to a comment if you need context from the rest.
- When the header scrolls off the top, we collapse it into a single line with no border or buffer.
  It would be a lot clearer if it got at least a blank line underneath, or a line / bottom-border.
- There's no yak-state icon in the detail editor, until the header panel collapses to a single line.

---
▸ 2026-10-17T02:50:19Z [Joel Webber]

More clearly delineated comments would improve readability significantly.
Preferably with a touch of color to make the delineation jump out more.

---
▸ 2026-10-17T03:03:27Z [Joel Webber]

And comments like these could definitely do with a lot better date rendering.
Preferably human-friendly with relative shorthand for nearby date/times, like "1 hour ago".

---
▸ 2026-10-17T04:26:52Z [delta-lead]

Restructured 2026-10-17 under yaks-e9de. Children: yaks-66a3 (pinned-header rule + status icon), yaks-93cd (comment headers, shared with the edit form), yaks-2362 (markdown-friendly comment format, question open), yaks-ef82 (dates), yaks-a893 (design: unify view and edit, which covers the "comment editor takes over the pane" item), yaks-c6f4 (accordion overflow; may be obviated by yaks-a893).

---
▸ 2026-10-17T04:31:30Z [delta-lead]

moved: hairy -> shaving
