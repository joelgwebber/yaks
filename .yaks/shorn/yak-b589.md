---
id: yak-b589
title: Ctrl n/p not consistent across dialogs
type: bug
priority: 2
created: '2026-06-21T14:59:01Z'
updated: '2026-06-24T18:05:10Z'
---

Maybe we're implementing these dialogs with different code? They should be sharing all the keyboard mapping, and most of the same rendering and other behaviors.

---
▸ 2026-06-24T18:05:05Z
Root cause: each dialog hardcoded its own nav key set. fuzzy_pick_task bound Ctrl-N/P (14/16) but task_form only had Tab/Shift-Tab/arrows/jk, so Ctrl-N/P did nothing in the create/edit form. Fix: extracted shared NAV_NEXT_KEYS/NAV_PREV_KEYS in dialogs.py (Down/Ctrl-N/Tab and Up/Ctrl-P/Shift-Tab); fuzzy_pick_task and all six task_form row handlers now reference them. Also added Shift-Tab as prev in the fuzzy picker for symmetry. New test_nav_keys_shared_across_dialogs pins the contract. 113 tests pass.
