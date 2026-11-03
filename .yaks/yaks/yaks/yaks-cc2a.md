---
id: yaks-cc2a
title: Shed details renders read-only, unscrollable yaks list
status: hairy
type: bug
priority: 3
created: '2026-11-03T20:53:54Z'
updated: '2026-11-03T20:53:54Z'
tags:
- tui
- sheds
---

Not quite sure what we *should* show for these -- we don't have a lot of information available for them besides the yaks list. But it feels kind of broken to see it duplicate the yaks list that's *also* available in the list/tree view, but have it be wholly non-interactive. I think I'd err on the side of keeping it simple, and leaving the yak list to the list view.
