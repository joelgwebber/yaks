---
id: yaks-c733
title: 'Code blocks and tables: no-wrap, horizontal scroll'
status: hairy
type: feature
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

- **Code blocks** (fenced and 4-space indented) never wrap: fences hidden, a dim language label (e.g. `java`) on the block's first row, optional subtle background. Mermaid is just a labelled code block.
- **Tables** (GFM) render with box-drawing borders and per-column alignment; short rows pad with empty cells (as GFM does); cells hold inline styling.
- **Horizontal scroll, per block.** When the line cursor is on a row of a no-wrap block, `h`/`l` and `←`/`→` scroll that block sideways (other blocks don't move); a dim `…`/`›` marks clipped edges. Today `h`/`←` in the detail pane mean "back to the list", so on a no-wrap block they are re-bound and `Esc` still returns to the list; at scroll 0, `h` falls through to "back to list" (decide and document). Update `help_content` and docs.
- Yank on these rows copies the full source lines.
