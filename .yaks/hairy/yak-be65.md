---
id: yak-be65
title: Make force-repaint actually work
type: bug
priority: 3
created: '2026-07-26T16:03:16Z'
updated: '2026-08-15T05:10:09Z'
labels:
- ui
---

We have a force-repaint key, but it doesn't seem to do much. Especially when the app's terminal gets inadvertently cleared, and when the terminal resizes.
Repainting doesn't *always* fail when the terminal resizes, but sometimes it seems to go unnoticed, forcing a quit/restart to fix it.
This happens with some regularity in Zed's embedded terminal.
