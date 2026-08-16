---
id: yak-7cd3.7.1
title: Extract yaktui/dialogs.py (prompts, picks, fuzzy, edit_prompt)
type: task
priority: 2
created: '2026-04-14T16:28:44Z'
updated: '2026-04-14T16:31:28Z'
commit: 35b19d0
parent: yak-7cd3.7
---

Pull the self-contained input helpers out of the App class: _input_prompt, _edit_prompt, _pick, _confirm, _pick_type_for_create, _fuzzy_pick_task. Convert to module-level functions that take stdscr (and whatever state they need) as arguments. These only depend on curses primitives + _safe_addstr — the cleanest piece of App to carve off first.
