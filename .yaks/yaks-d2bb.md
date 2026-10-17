---
id: yaks-d2bb
title: Markdown editing improvements
status: shaving
type: task
priority: 3
created: '2026-10-17T02:41:42Z'
updated: '2026-10-17T04:31:30Z'
parent: yaks-e9de
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

---
▸ 2026-10-17T04:26:52Z [delta-lead]

Restructured 2026-10-17 under yaks-e9de. Children: yaks-5408 (black background), yaks-9425 (dimmed/hidden markers, quieter links, shared highlights), yaks-1f84 (auto-continue lists/quotes in the edtui fork). "Bring the styling into line with rendering" is yaks-bc53 (shared palette) plus yaks-9425. Out of scope by decision: anything that changes display width in the editor (collapsed links, wrap hanging indents, rendered tables).

---
▸ 2026-10-17T04:31:30Z [delta-lead]

moved: hairy -> shaving
