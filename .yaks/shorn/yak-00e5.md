---
id: yak-00e5
title: 'Demo rendering tweaks: robust divider, detail pane, tree/ghosts, bigger term'
type: task
priority: 2
created: '2026-07-26T02:43:14Z'
updated: '2026-07-26T02:43:29Z'
---

Follow-up polish on the demo scaffolding (yak-f9a2) from review feedback.

---
▸ 2026-07-26T02:43:29Z
Addressed 4 review items. (1) DIVIDER DRIFT: root cause was emitting each frame row as one flowing string joined by CRLF and relying on cumulative display-width — a glyph the emulator sizes differently, or a write landing on the right margin (auto-wrap), cascades and shoves the divider + everything after it. Rewrote castkit.Screen.render_frame to a new primitive: per-row absolute cursor addressing (CUP \x1b[r;1H + clear-line) and per-run absolute column (CHA \x1b[cG). Cells are 1:1 with display columns (wide glyph reserves a continuation cell), so structure can't drift regardless of emulator width disagreements; worst case is a 1-col cosmetic gap around a mismatched glyph. Bonus: only non-blank runs are painted, so the cast shrank 82KB->63KB. (2) DETAIL PANE: added DetailLine + build_detail_lines (demo port of detail.build_detail_lines) + render_detail — header/fields/Source/Depends on/Parent/Children/Blocks/Description with kind-based styling mirroring draw_detail; pulls humanize_date from source. (3) LIST TREE: Board.tree_rows ports render.build_tree (anchor=focus in active status, ancestors+descendants as dimmed ghosts), render_list does depth indent, blocked '*' lead, priority/type/id colors, ghost dim + status-emoji badge (ghost_badge_attr mirror). Imports parent_id from source. (4) SIZE: terminal 100x30 -> 120x34; demo.html player fit=width, terminalFontSize=small, container widened to 1140px. Verified: divider aligned across emoji tab row via plain-grid dump; cast valid v2 w/ 5 markers; CUP+CHA confirmed in output; assets serve 200. No version bump (docs/+demo/ not shipped).
