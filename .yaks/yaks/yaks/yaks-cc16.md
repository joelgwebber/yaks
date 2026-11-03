---
id: yaks-cc16
title: 'TUI phase 1: notifications clear on the next key'
status: shaving
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T03:47:40Z'
parent: yaks-947a
tags:
- tui
---

Phase 1 of [the design](/wiki/design/tui-model.md#phases) (yaks-953f, its phase 1): a notification lasts until the next key, and the search commit stops writing a `filter: q` toast. Today `App.notification` is never cleared, so a stale `filter: widget` toast outlives the search it describes.

---
▸ 2026-11-03T03:47:40Z [herd-tui]
kind: move
from: hairy
to: shaving
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
