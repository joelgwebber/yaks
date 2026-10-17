---
id: yaks-a893
title: 'Design: unify view and edit for the description and comments'
status: hairy
type: feature
priority: 2
created: '2026-10-17T04:25:39Z'
updated: '2026-10-17T04:25:39Z'
parent: yaks-faf5
labels:
- tui
---

Design pass first; implementation yaks follow from it.

Problems today:
- `M` (add a comment) opens `Overlay::Edit`, which replaces the whole detail pane: you lose the context you're replying to.
- `E` on a comment opens the edit form, whose content accordion collapses every other block to a one-line separator (yaks-127e; overflow bug yaks-c6f4). Switching between view and edit moves everything around.
- The accordion was a workaround for editing/rendering bugs that were hard to pin down (Joel's recollection; check yaks-127e's notes and git history for what they were).

Direction to evaluate: editing happens *in place* in the detail pane. The block being edited becomes an editor at the same position and the rest stays rendered around it, scrolling as usual; a new comment opens as an empty editor at the end of the comments. Header fields stay in the form, or get their own in-place editors. Questions to settle: what a key does inside vs outside the editor, how scrolling works with an editor taller than the viewport, whether the form's header rows stay a separate surface, how this interacts with the exact-line `E`.

Deliverable: a short design note on this yak with mock frames (headless renders of a prototype, or hand-drawn text frames), agreed with Joel, then implementation yaks.
