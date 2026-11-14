---
id: yaks-df01
title: Next/prev comment improvements
status: shaving
type: task
priority: 3
created: '2026-11-11T20:46:10Z'
updated: '2026-11-14T11:32:22Z'
tags:
- tui
---

A few small tweaks:
- Next comment should both move the cursor to the comment header (as currently), *and* attempt to ensure the full comment's visible by ensuring the bottom is also in the viewport. If that's not possible because of its length, we should default to putting the comment header at the top of the viewport.
- Previous comment should focus the _current_ comment's header, unless the header's already selected.

---
▸ 2026-11-14T11:32:22Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
