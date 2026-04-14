---
id: yak-7cd3.1
title: Extract yaklib/format.py (humanize_date + status-char map)
type: task
priority: 2
created: '2026-04-14T13:22:28Z'
updated: '2026-04-14T14:22:54Z'
commit: 3819ae9
---

Pull _humanize_date from tui.py and STATUS_CHAR mapping {HAIRY:'H',SHAVING:'S',SHORN:'N'} into yaklib/format.py. Update tui.py call sites (5 for STATUS_CHAR, 2 for humanize_date). Smallest extraction — proves out the package layout and sys.path handling for entry-point scripts before we tackle bigger pieces.
