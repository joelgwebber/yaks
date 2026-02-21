---
id: yak-4a23
title: Parent/child task hierarchy via dot-suffixed IDs
type: feature
priority: 2
created: '2026-02-20T03:39:26Z'
updated: '2026-02-20T03:41:57Z'
commit: 2e6db8f
---

Implicit parent/child relationships based on ID convention (foo-abc, foo-abc.1, foo-abc.2). Includes: helper functions for parsing parent/child relationships, create --parent flag for child task creation, show enrichment with parent/children display, no-dots validation on prefix at init time. Arbitrary depth supported.
