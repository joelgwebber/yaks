---
id: yaks-59bf
title: yaks commits fails on a branch farm with no commits yet
status: hairy
type: bug
priority: 3
created: '2026-10-31T17:12:34Z'
updated: '2026-10-31T17:12:34Z'
parent: yaks-039b
tags:
- git
- cli
---

Found by `branches-1` (2026-10-31): on a brand-new branch farm whose `yaks` branch has no commits yet, `yaks commits <id>` fails outright:

```text
git log failed: ... 'yaks' does not have any commits yet
```

It comes from the farm-history `git log --follow`, which predates `yaks-4728`. The fix shape: treat "no commits yet" as an empty farm history, not an error.
