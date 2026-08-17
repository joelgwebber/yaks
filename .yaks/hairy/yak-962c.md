---
id: yak-962c
title: Swap Shaving emoji ✂️ -> 🪒
type: task
priority: 4
created: '2026-08-17T14:17:49Z'
updated: '2026-08-17T14:17:49Z'
parent: yak-1bef
labels:
- ui
---

🪒 (U+1FA92 razor) is less obtrusive than ✂️ in the UI. Update BOTH sources for consistency: view.py _STATUS_VIEWS shaving label ('✂️  Shaving' -> '🪒 Shaving') AND format.py STATUS_EMOJI['shaving'] (used by the detail pane, the fuzzy picker, and — once the sibling lands — per-row state emojis). Same reconcile-persisted-name wrinkle as the rename sibling for the tab label on already-customized herds.
