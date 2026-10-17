---
id: yaks-e30d
title: Render headings, rules, emphasis and strikethrough
status: hairy
type: task
priority: 3
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T04:25:58Z'
parent: yaks-ea13
depends_on:
- yaks-649d
- yaks-bc53
labels:
- markdown
- tui
---

On the renderer core: hide `*`/`_`/`**`/`~~` markers; H1/H2/H3 styled from the shared palette (H1 maybe underlined with a rule, H2 bold-colored, H3 bold), H4-H6 as H3; thematic breaks as a full-width dim `─` rule. Depends on the palette and the core.
