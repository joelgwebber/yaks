---
id: yaks-aac1
title: 'TUI phase 4: pane subjects; the viewer and image viewer leave Overlay'
status: hairy
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T03:46:57Z'
parent: yaks-947a
depends_on:
- yaks-5d08
tags:
- tui
---

Phase 4 of [the design](/wiki/design/tui-model.md#2-the-pane-and-its-subjects): the pane shows a subject (yak, document, shed). The text viewer and image viewer stop being overlays, the pane is detached from the list row (decision 5), and one `LineNav` serves every text subject.
