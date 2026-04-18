---
id: yak-6f9e.10
title: Fix invisible badge color against background
type: bug
priority: 2
created: '2026-04-18T00:15:17Z'
updated: '2026-04-18T01:10:02Z'
commit: ab70a7e
---

One or more badge colors are invisible or near-invisible against the background. Need to check all badge variants and ensure sufficient contrast.

### 2026-04-18T00:42:48Z
Fixed in v2 polish pass.

### 2026-04-18T01:09:49Z
Root cause: same pico button variable issue. Fixed task-id wrapping with white-space: nowrap instead of fixed width.

![Child IDs no longer wrap](artifacts/yak-6f9e.10/wrapping-fixed.png)
