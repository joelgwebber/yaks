---
id: yak-7cd3.4
title: Extract yaklib/deps.py (shared dep resolution)
type: task
priority: 2
created: '2026-04-14T13:22:29Z'
updated: '2026-04-14T15:57:09Z'
depends_on:
- yak-7cd3.3
commit: be913bc
---

Consolidate three near-duplicate dep-resolution implementations: cmd_next/cmd_tangled in yak.py and _depends_on_transitively + _recompute_blocked in tui.py. Expose resolved_ids(root), is_ready(task, resolved), blocked_by(task, resolved), transitively_blocked_by(task_id, all_tasks). Add unit tests for cycle detection alongside.
