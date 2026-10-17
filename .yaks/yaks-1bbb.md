---
id: yaks-1bbb
title: 'GFM extras: task lists, alerts, footnotes'
status: hairy
type: task
priority: 3
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T04:25:58Z'
parent: yaks-ea13
depends_on:
- yaks-649d
labels:
- markdown
- tui
---

On the renderer core.

- Task lists: `- [ ]` / `- [x]` render as ☐ / ☑ (toggling from the detail pane is a possible follow-up, not in scope).
- Alerts: `> [!NOTE]` / TIP / IMPORTANT / WARNING / CAUTION render as a quote with a colored bar and a title row (palette).
- Footnotes: references render as superscript-ish `[1]` links that jump to the definition; definitions collect at the end of their content block (description or comment) under a dim rule.
