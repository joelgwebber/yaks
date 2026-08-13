---
id: yak-c393
title: 'Demo foundation: variable-focus divider, headroom, help bar'
type: feature
priority: 2
created: '2026-08-13T00:57:25Z'
updated: '2026-08-13T01:25:00Z'
---

Foundational rendering primitives for the demo, agreed with the user. Unblocks the full screenplay + modal primitives. Three parts as children.

---
▸ 2026-08-13T01:01:21Z
Built all three foundation pieces in one Layout refactor. (.1) Variable focus divider: Layout.compose takes per-frame agent_frac; Director.focus_to(target) tweens the divider over N frames. Stances FOCUS_AGENT=0.60 / FOCUS_BALANCED=0.42 / FOCUS_BOARD=0.0 (agent hidden -> board gets full terminal). (.2) Headroom+help bar: 100x30->120x38, dropped the title banner + pane-header rows, added render_help_bar (black-on-white key strip, HELP_LIST/HELP_DETAIL mirrored from draw_help_bar) pinned to the bottom row. (.3) Board-focused split: render_board_pane renders the real app layout when wide — tabs full-width, list(~1/3)|detail(~2/3) with an inner separator (starts below tabs), else falls back to full-pane detail or list. Verified via plain-grid dumps for all three stances; cast valid 120x38 w/ 5 markers; modules compile.
