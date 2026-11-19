---
id: yaks-15dc
title: Ensure all the internal link types work as expected
status: hairy
type: task
priority: 3
created: '2026-11-19T22:16:40Z'
updated: '2026-11-19T23:24:58Z'
tags:
- tui
---

Various types of links -- it looks like all work except for line numbers:
- [Link](code:src/tui/actions.rs) to source
- [Link](code:src/tui/actions.rsL10) to source:line
- [Link](code:src/tui/actions.rs@58eb9a6) to source@short-hash
- [Link](code:src/tui/actions.rs@58eb9a6248e509e2e4cca2e158e1e97ca2c9a864) @hash
- [Link](code:src/tui/actions.rs@58eb9a6248e509e2e4cca2e158e1e97ca2c9a864:10) @hash:line

Though perhaps I have that syntax wrong? Let's also consider line
