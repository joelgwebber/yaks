---
id: yak-c393.2
title: Headroom + help-bar primitive
type: task
priority: 2
created: '2026-08-13T00:57:25Z'
updated: '2026-08-13T03:46:03Z'
---

Bump terminal to ~38 rows; drop the 'yaks demo' title banner and the pane-header row to reclaim vertical space; add the real bottom help bar (black-on-white key strip) as a shared primitive, which also narrates which keys are being pressed.

---
▸ 2026-08-13T03:46:03Z
Per review, DROPPED the bottom help bar entirely (distracting, and it stretched full-width rather than under the board). render_help_bar + HELP_* constants removed; content now uses the full height.
