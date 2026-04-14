---
id: yak-7cd3.8
title: Drop yak.py back-compat shim, update tui.py imports
type: task
priority: 3
created: '2026-04-14T13:22:29Z'
updated: '2026-04-14T16:41:34Z'
depends_on:
- yak-7cd3.7
- yak-7cd3.5
commit: df730db
---

Switch tui.py (or yaktui/app.py) to import directly from yaklib instead of going through 'import yak'. Remove the re-exports left in yak.py. yak.py is purely a CLI entry point after this. Final cleanup pass: remove dead code, run ruff/black, update CLAUDE.md with the new layout.
