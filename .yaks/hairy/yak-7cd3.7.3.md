---
id: yak-7cd3.7.3
title: Extract yaktui/render.py (draw routines)
type: task
priority: 2
created: '2026-04-14T16:28:44Z'
updated: '2026-04-14T16:28:44Z'
depends_on:
- yak-7cd3.7.2
---

Pull drawing methods: draw, _draw_tabs, _draw_list, _draw_separator, _draw_detail, _safe_addstr, _highlight_matches, help bars. These are pure readers of App state — take App instance + stdscr, no mutation.
