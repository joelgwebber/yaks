---
id: yak-c725
title: 'TUI: add/remove dependencies in-UI'
type: task
priority: 2
created: '2026-04-05T14:34:18Z'
updated: '2026-04-05T14:43:43Z'
commit: c6e1079
---

Currently deps can only be managed via slash commands or $EDITOR. Add quick affordances:
- 'b' to add a dep (prompt for task id, or picker)
- 'B' to remove a dep (picker over current depends_on list)
Update the current task's depends_on and bump 'updated'.
