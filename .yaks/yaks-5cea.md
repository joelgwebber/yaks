---
id: yaks-5cea
title: Lookup-by-id should find cross-shed yaks
status: hairy
type: bug
priority: 2
created: '2026-10-13T21:21:42Z'
updated: '2026-10-13T21:21:42Z'
source: sheds
---

At present it only finds yaks if they're *also* in the cwd's shed. If they're net-new in an unmerged shed, you can't find them by id.
