---
id: yaks-d0b0
title: Yaks links from docs aren't followed
status: hairy
type: bug
priority: 3
created: '2026-11-03T20:46:20Z'
updated: '2026-11-03T21:13:47Z'
parent: yaks-f677
tags:
- tui
---

Trying to follow a link like yaks-37da from within a doc gives `wiki/howto/yaks-37da: not in this farm`.
Looks like it's trying to follow them as though they're relative resources, when yaks are absolute by definition.
