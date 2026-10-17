---
id: yaks-d2bb
title: Markdown editing improvements
status: hairy
type: task
priority: 3
created: '2026-10-17T02:41:42Z'
updated: '2026-10-17T02:48:25Z'
parent: yaks-ea13
labels:
- markdown
- rendering
- tui
---

A few basic QoL improvements for markdown editing:
- Bring the styling affordances into line with the markdown rendering. They're wildly different colors and styles now.
- We could at least replace *styling* _markers_ with spaces when the cursor's not inside them.
- It would be helpful if [links](https://example.com/) weren't quite so noisy.
  - We need to brainstorm on how best to handle them. At the very least, we can style them such that the `[link]` bit is more prominent than `(schema://the-actual-uri)`.
- The editor background is pitch black, rather than the usual background color. There's some value in changing it a little, but the contrast at present is quite startk.
- If feasible in `edtui`, it would be helpful to have some basic auto-indent handling in lists and quotes.

> It would be nice if quotes auto-continued,
> when you add a newline to the end, like most structured editors these days.
