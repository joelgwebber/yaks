---
id: yak-7cd3.7
title: Carve App class into focused modules
type: task
priority: 3
created: '2026-04-14T13:22:29Z'
updated: '2026-04-14T13:22:35Z'
depends_on:
- yak-7cd3.6
---

Highest-risk step. Pull dialogs (input_prompt, edit_prompt, pick, confirm, fuzzy_pick) → yaktui/dialogs.py. Pull state mutations (_create_task, _edit_task, _delete_task, quick-adjusts, _add_dependency, _attach_file) → yaktui/mutate.py. Pull drawing (draw, _draw_tabs, _draw_list, _draw_separator, _draw_detail) → yaktui/render.py. Pull key dispatch → yaktui/keys_list.py + yaktui/keys_detail.py. App class in yaktui/app.py becomes a thin coordinator holding state + wiring. Pass App instance into helpers rather than methods-on-App to avoid circular imports.
