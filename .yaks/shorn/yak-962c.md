---
id: yak-962c
title: Swap Shaving emoji ✂️ -> 🪒
type: task
priority: 4
created: '2026-08-17T14:17:49Z'
updated: '2026-08-17T14:41:29Z'
parent: yak-1bef
labels:
- ui
---

🪒 (U+1FA92 razor) is less obtrusive than ✂️ in the UI. Update BOTH sources for consistency: view.py _STATUS_VIEWS shaving label ('✂️  Shaving' -> '🪒 Shaving') AND format.py STATUS_EMOJI['shaving'] (used by the detail pane, the fuzzy picker, and — once the sibling lands — per-row state emojis). Same reconcile-persisted-name wrinkle as the rename sibling for the tab label on already-customized herds.

---
▸ 2026-08-17T14:41:29Z
Done. Shaving emoji ✂️ -> 🪒 (U+1FA92) in both sources: view._STATUS_VIEWS shaving label and format.STATUS_EMOJI['shaving'] (detail pane, fuzzy picker, and the new always-on per-row state emoji). Propagates to customized herds via the reconcile fix.
