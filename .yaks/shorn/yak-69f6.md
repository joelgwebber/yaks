---
id: yak-69f6
title: Change dependency keys
type: task
priority: 2
created: '2026-04-15T01:16:07Z'
updated: '2026-04-15T04:47:28Z'
depends_on:
- yak-8cef
commit: 60adc1a
---

'd' is too easy to hit by accident, and there's no obvious reason to remove dependencies from the
list view, where you can't see them. I think it's better to push dependency deletions solely
into the detail editor, where you can use the cursor to point at them. We can use 'D' for all cases:
- Add dependency in list view
- Remove dependency when dep selected in the detail view
- Add dependency in when *no dep* selected in detail view
