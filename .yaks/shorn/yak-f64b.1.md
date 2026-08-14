---
id: yak-f64b.1
title: 'Stage 1: TaskRepo protocol; demo consumes shared build_tree + build_detail_lines'
type: task
priority: 3
created: '2026-08-14T03:28:20Z'
updated: '2026-08-14T03:37:14Z'
---

Decouple build_tree + build_detail_lines from the filesystem via a small TaskRepo protocol (find/children/resolve_link_spans/artifacts + resolved_ids). FsTaskRepo wraps root; a demo BoardRepo serves the in-memory Board. Then the demo calls the REAL builders and deletes its forked tree_rows + build_detail_lines. Painters stay bifurcated (curses vs virtual grid) but consume the same view-model. Keep the shipping TUI green (run pytest).

---
▸ 2026-08-14T03:37:14Z
Landed. New yaklib/repo.py: TaskRepo protocol + FsTaskRepo(root). detail.build_detail_lines now reads via a repo (accepts a Path for back-compat -> wraps in FsTaskRepo, so tui.py + all tests unchanged) and takes status_glyph (emoji for TUI, ASCII for demo). build_tree needed NO change - it was already cache-decoupled. Demo: added BoardRepo + _yak_to_task + board_reverse_deps; render_board_pane now calls the real build_tree + build_detail_lines; deleted the demo's forked tree_rows/children_of/blockers_of/build_detail_lines/DetailLine. Painters (render_list/render_detail) stay local but consume the shared view-model. Verified: 117 tests green (shipping intact), demo output byte-identical to before, cast valid.
