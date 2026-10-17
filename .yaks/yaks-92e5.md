---
id: yaks-92e5
title: Edit form keeps only header fields (drop content blocks and the accordion)
status: hairy
type: task
priority: 3
created: '2026-10-17T17:02:15Z'
updated: '2026-10-17T17:02:15Z'
parent: yaks-a893
depends_on:
- yaks-597d
labels:
- tui
---

Second half of the yaks-a893 design: with body editing in place, `E` on a header row opens the form with title/type/priority/labels/source/herd only. Remove the per-comment `ContentBlock`s in edit mode, `render_content_stack`'s accordion, `EditTarget::Content` routing to the form, and yaks-c6f4's problem with them. Create (`c`/`C`) keeps its single description block. Depends on the in-place editing yak landing first.
