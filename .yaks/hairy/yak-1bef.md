---
id: yak-1bef
title: TUI list display polish (View-list follow-ups)
type: task
priority: 3
created: '2026-08-17T14:17:19Z'
updated: '2026-08-17T14:17:19Z'
labels:
- ui
---

A little herd of display/naming tweaks that fell out of living with the View-list arc (yak-4473). All small, independent, non-urgent.

Design property to PRESERVE (nice emergent behavior): unstarring a yak from within the Starred/Working-set view makes it vanish immediately (correct), and because Recent is derived purely from updated: (not from nav or membership), the just-unstarred yak is NOT bumped — but you can still get back to it instantly since it's whatever you were last looking at. More importantly, re-starring is easy to find. Keep Recent decoupled from membership/nav so this stays true.
