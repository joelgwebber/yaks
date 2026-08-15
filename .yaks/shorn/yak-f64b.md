---
id: yak-f64b
title: Shared terminal-rendering abstraction (surface/style) for TUI + demo
type: feature
priority: 3
created: '2026-08-14T03:13:51Z'
updated: '2026-08-15T03:27:27Z'
---

Kill demo<->TUI drift by rendering both through one abstraction instead of maintaining two parallel painters. Today demo/yakscreen.py reimplements the layout of scripts/yaktui/render.py + detail.py (list rows, detail lines, tabs) and re-expresses colors.py as ANSI. It already imports the pure bits (status_char, parent_id, humanize_date) but forks the layout, so UI changes need manual demo updates.\n\nKey observation: the TUI already has a clean seam. The MODEL layer is backend-agnostic (tree.build_tree returns (status,task,depth,ghost); detail.build_detail_lines returns DetailLine(text,kind,links)) and render.py functions are pure readers of App state. The demo forks the model (Board.tree_rows, its own build_detail_lines) mainly because the real ones take a filesystem root + embed status glyphs.\n\nProposed two stages:\n1. MODEL sharing: refactor build_tree/build_detail_lines to take a backend-agnostic task source (protocol) instead of a Path root; have the demo feed its in-memory Board. Single-sources WHAT appears (rows, detail sections, links). Modest, low-risk.\n2. PAINTER sharing: introduce a Surface protocol (put(y,x,text,Style), hline/vline, dims) + a backend-agnostic Style (fg/bg/bold/dim/...). Port render.py painters (draw_tabs/draw_list/draw_detail/help) to draw against Surface. Backends: CursesSurface (real app) + GridSurface (demo -> cast). Single-sources the exact layout + geometry. Bigger, needs care + tests since it touches the shipping TUI. Width stays backend-owned (curses uses terminal wcwidth; demo uses avt-accurate char_width) so the draw code is width-agnostic.\n\nDon't block the demo on this; it's an investment that pays off with frequent UI changes / more demos. Decompose into children once we commit.

---
▸ 2026-08-14T03:37:14Z
Stage 1 (model sharing) landed via yak-f64b.1 and is likely sufficient per the 'thin bifurcated painters over shared models' insight. Stage 2 (a full Surface/Style painter abstraction so the demo runs the real render.py painters too) is DEFERRED - only worth it if we do many demos or frequent restyles. Leaving this open as the tracking home for that optional stage.
