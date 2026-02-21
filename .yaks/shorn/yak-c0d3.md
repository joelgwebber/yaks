---
id: yak-c0d3
title: Yak descriptions should use multi-line strings in yaml
type: task
priority: 2
created: '2026-02-21T01:28:46Z'
updated: '2026-02-21T01:33:32Z'
commit: 25bf6eb
---

Otherwise, we keep generating yaks with a bunch of inline newlines and backslashes, which don't really seem to work properly.
