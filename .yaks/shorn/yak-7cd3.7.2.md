---
id: yak-7cd3.7.2
title: Extract yaktui/mutate.py (task CRUD + quick-adjusts + attach)
type: task
priority: 2
created: '2026-04-14T16:28:44Z'
updated: '2026-04-14T16:34:25Z'
depends_on:
- yak-7cd3.7.1
commit: dcc2e1d
---

Pull task-mutating methods: _create_task, _edit_task, _delete_task, _quick_adjust_priority, _quick_adjust_type, _quick_adjust_title, _quick_adjust_labels, _add_dependency, _remove_dependency, _attach_file, _add_comment, _build_template, _parse_template, _edit_file_in_editor. These all read/write via yaklib + trigger self.reload()/self._rebuild_detail(), so they'll be free functions that take an App instance as their first arg.
