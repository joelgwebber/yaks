---
id: yaks-1f84
title: 'Editor: auto-continue lists and quotes on Enter (edtui fork)'
status: hairy
type: feature
priority: 3
created: '2026-10-17T04:25:39Z'
updated: '2026-10-17T04:25:39Z'
parent: yaks-d2bb
labels:
- markdown
- tui
---

Enter at the end of a `> `, `- `, `* `, `1. ` (renumbered to `2. `) or `- [ ] ` line starts the next line with the same prefix and indent; Enter on an item with an empty body removes the prefix (ends the list/quote). Insert mode only.

Implement in the edtui fork (a line-break hook or a markdown-aware `LineBreak` option on the `yaks-integration` branch), not as a yaks-side key shim: yaks-182f removed those because shims break vim count composition. Then `cargo update -p edtui`.
