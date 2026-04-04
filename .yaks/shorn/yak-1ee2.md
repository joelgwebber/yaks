---
id: yak-1ee2
title: TUI auto-refresh on filesystem change
type: feature
priority: 2
created: '2026-04-04T22:07:13Z'
updated: '2026-04-04T22:08:51Z'
commit: 2e13ec6
---

Poll .yaks/ directories every 500ms. If file count or mtimes changed, reload while preserving the current task selection by id.
