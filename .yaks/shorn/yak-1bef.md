---
id: yak-1bef
title: TUI list display polish (View-list follow-ups)
type: task
priority: 3
created: '2026-08-17T14:17:19Z'
updated: '2026-08-17T14:41:47Z'
labels:
- ui
---

A little herd of display/naming tweaks that fell out of living with the View-list arc (yak-4473). All small, independent, non-urgent.

Design property to PRESERVE (nice emergent behavior): unstarring a yak from within the Starred/Working-set view makes it vanish immediately (correct), and because Recent is derived purely from updated: (not from nav or membership), the just-unstarred yak is NOT bumped — but you can still get back to it instantly since it's whatever you were last looking at. More importantly, re-starring is easy to find. Keep Recent decoupled from membership/nav so this stays true.

---
▸ 2026-08-17T14:41:29Z
Shipped an enabling fix with 33e9/962c: views_store now stores a NULL name for un-renamed built-in Views (save_views) and reconcile falls back to the code default for null names, so built-in label/emoji changes (Starred rename, 🪒) reach herds that already wrote views.json via the picker — not just fresh installs. Explicit user renames are still preserved.

---
▸ 2026-08-17T14:41:47Z
HERD COMPLETE — all five children shorn. Shipped: always-on per-row state emoji (8e44), ⭐ star marker on starred rows (693b), Working set -> Starred rename (33e9), ✂️ -> 🪒 (962c), and * + starred affordance in the detail pane (f688) — plus the reconcile null-name propagation fix. Tab bar now: 🦬 Hairy | 🪒 Shaving | 🐑 Shorn | 🕒 Recent | ⭐ Starred.
