---
id: yak-f688
title: Star/unstar (*) from yak detail + show starred state near frontmatter
type: feature
priority: 3
created: '2026-08-17T14:23:03Z'
updated: '2026-08-17T14:37:29Z'
parent: yak-1bef
labels:
- ui
---

Extend starring (yak-597c) into the detail pane so it works wherever you're looking at a yak, with visible state.

1) Bind * in the detail key handler (yaktui/keys_detail.py) to app._toggle_working_set(app._current_task_id()). * is free in detail (detail's v = visual-select, list's v = picker), and matches the list-mode * binding. _current_task_id() already works in detail (E/X use it).

2) Show a starred affordance near the frontmatter in the detail pane (yaktui/detail.py build_detail_lines) -- e.g. a 'Starred' line/field right after Title (mirrors the existing _error warning-line pattern). build_detail_lines needs the starred state: add a boolean 'starred' param and have the TUI's _rebuild_detail pass starred = (task id in self.working_set).

3) Refresh on toggle: _toggle_working_set currently rebuilds the list only when the working-set view is active; when toggled from detail, also call _rebuild_detail() so the affordance updates immediately. (Simplest: always _rebuild_detail() at the end of _toggle_working_set.)

Pairs with yak-693b (list-row star marker); together they give consistent starred feedback in both panes.

---
▸ 2026-08-17T14:37:29Z
Done. (1) keys_detail binds * -> app._toggle_working_set(app._current_task_id()) (free in detail; v there is visual-select). (2) build_detail_lines gained a starred param and emits a '⭐ Starred' line right after Title (beside the _error pattern); tui._rebuild_detail passes starred=(task id in self.working_set). (3) _toggle_working_set now always calls _rebuild_detail() (list rebuild still only when the working-set view is active), so the affordance updates immediately when toggled from detail. Help: added * to the Detail-pane section. Note: the signature line is pre-existing E501 (line 58), extended but not newly introduced.
