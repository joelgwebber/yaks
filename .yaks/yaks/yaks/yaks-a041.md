---
id: yaks-a041
title: 'Regrow routes a recurring yak to the inbox (needs: agent|human)'
status: hairy
type: feature
priority: 2
created: '2026-11-04T22:31:54Z'
updated: '2026-11-04T22:31:54Z'
parent: yaks-7fa6
tags:
- cli
---

Part of the scheduling arc ([design](/doks/design/scheduling.md)). Today `regrow-due` only makes the yak hairy; nothing tells anyone it has work.

On regrow, if the yak says who acts next, set `needs:` to it (the same field `ask`/`request` use) and write the `kind: regrow` note so `inbox` shows why. No new queue. A yak that never says stays hairy only, as now.

**Blocked on an ask** (below): how the yak says who. Don't start until it is answered.
