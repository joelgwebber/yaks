---
id: yaks-67ef
title: Take theme surfaces from toque
status: hairy
type: task
priority: 3
created: '2026-11-21T04:57:11Z'
updated: '2026-11-21T04:57:11Z'
tags:
- ui
---

src/tui/surface.rs moved to toque (toque feature theme, toque farm yak-4b3b) so canon's TUI can share it. Switch yaks over with no change to its snapshots or docshots; YAKS_THEME stays the override.
