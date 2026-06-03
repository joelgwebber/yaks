---
id: yak-ca70
title: Native yak editor
type: feature
priority: 3
created: '2026-04-25T16:05:57Z'
updated: '2026-06-03T20:58:56Z'
labels:
- tui
---

The $EDITOR approach was nice for as a stopgap, but now it's more of a PITA. We've shown in the filter code that we can create a decent UI with a nice mixture of discoverable and vim-style controls.

Now I think it's worth considering the same treatment for yaks + frontmatter. I don't think we need a full "real editor" implementation for raw text -- we can use our basic one-line editor implementation for individual fields and small bits of text; and keep using the $EDITOR affordance for larger chunks.
