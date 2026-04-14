---
id: yak-7cd3.3
title: Extract yaklib/model.py (core fs + YAML layer)
type: task
priority: 1
created: '2026-04-14T13:22:29Z'
updated: '2026-04-14T13:22:35Z'
depends_on:
- yak-7cd3.2
---

Biggest extraction. Move status constants, _BlockScalarDumper/dump_yaml, find_tasks_root/_auto_migrate, load_task/save_task, all_tasks, find_task_file, generate_id, parent_id, find_children, find_descendants, next_child_number, now_iso, git_head_short, load_config. Keep yak.py re-exporting every symbol so tui.py's 'import yak' still works unchanged until later step. This is the hinge — be careful.
