---
id: yak-93dc
title: Ctrl+i/o navigation aren't working in details
type: bug
priority: 1
created: '2026-04-05T14:55:17Z'
updated: '2026-04-05T15:16:57Z'
---

Ctrl+o doesn't seem to do anything. Ctrl+i seems to just switch among the children. My intent was
that, after following a parent/child/block/dep link, that the navigation state would form a stack
that we could navigate in/out of. The stack doesn't need to outlive the current detail context. It
should reset each time you leave and re-enter details.
