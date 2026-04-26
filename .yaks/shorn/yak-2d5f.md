---
id: yak-2d5f
title: (Y)ank on a yak or source link in details to copy the target/url
type: feature
priority: 2
created: '2026-04-18T23:27:27Z'
updated: '2026-04-19T00:11:58Z'
commit: dcd1721
---

This would be particularly helpful when dealing with source links and you don't want to open them in the default browser.

---
▸ 2026-04-19T00:11:36Z
Updated 'y' handler in keys_detail.py: when cursor is on a link line (inline yak-id span, dep/parent/child line, or artifact/source URL line), copies the link target instead of the current task ID. Falls back to current task ID when not on a link.
