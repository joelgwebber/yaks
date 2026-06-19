---
id: yak-343a
title: Cursor/highlight affordances in "edit" mode missing the actual cursor
type: bug
priority: 2
created: '2026-06-19T20:18:34Z'
updated: '2026-06-19T20:42:51Z'
---

The dialogs we use in things like the filter overlay are more legible than the create/edit UI, at least in part because they show an actual text cursor.
We should fix this, but also look into whether we've inadvertently re-implemented a separate UI/dialog for each of these parts of the UI, and if so unify them sensibly.

---
▸ 2026-06-19T20:42:51Z
Fixed + unified. Cursor bug: task_form positioned the caret mid-render inside _draw_text_field, then drew the content zone + footer afterward, leaving the hardware cursor parked at the bottom of the screen (so the block cursor never appeared in the active field). Fix: _draw_text_field now RETURNS the caret (y,x); _draw_meta_zone and draw_task_form thread it up; the main loop places it via stdscr.move() as the final op before refresh() — same pattern dialogs already use. Unification: extracted dialogs.line_editor_window(ed, inner_w) -> (visible_text, cursor_col), the single-line windowing+caret math that task_form._draw_text_field and dialogs.edit_prompt each had duplicated; both now call it. Left alone deliberately: fuzzy_pick_task (different interaction, caret-at-end) and editor.py (multiline editor). 2 new unit tests for the helper; 112 tests pass. Visual confirmation of the live cursor still wants a human eye in a real terminal.
