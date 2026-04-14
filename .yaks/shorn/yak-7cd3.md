---
id: yak-7cd3
title: Refactor monolithic scripts into yaklib/yaktui
type: feature
priority: 2
created: '2026-04-14T13:21:58Z'
updated: '2026-04-14T16:41:34Z'
commit: df730db
---

Split scripts/yak.py (1274L) and scripts/tui.py (2313L) into sensible modules under scripts/yaklib/ and scripts/yaktui/. Entry files stay in place (plugin invokes them by name); modules hold the guts. See child yaks for each sequenced step. yak-140e (test harness) is the safety net.
