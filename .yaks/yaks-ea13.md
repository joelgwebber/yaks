---
id: yaks-ea13
title: Markdown rendering improvements
status: hairy
type: task
priority: 3
created: '2026-10-05T22:15:30Z'
updated: '2026-10-16T21:22:42Z'
labels:
- markdown
---

Our markdown rendering is often hard to read. We should aim for a more complete markdown->TUI (or console) implementation that takes into account all the common markdown idioms that apply to text rendering. A few examples:

- We could remove markers _like_ *these* in the rendering output.
- [Links](https://example.com/) could be rendered more simply.
-
