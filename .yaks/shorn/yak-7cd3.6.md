---
id: yak-7cd3.6
title: Split tui.py pure-function pieces into yaktui/
type: task
priority: 2
created: '2026-04-14T13:22:29Z'
updated: '2026-04-14T16:28:01Z'
depends_on:
- yak-7cd3.3
commit: 2c9e6ae
parent: yak-7cd3
---

Mechanical moves of self-contained pieces: color constants + init_colors → yaktui/colors.py, TaskNode + build_tree + sort/filter helpers → yaktui/tree.py, DetailLine + build_detail_lines → yaktui/detail.py. These have no cross-references to App state. Add a TUI smoke test that imports these modules, feeds a fixture .yaks/, and asserts the build_tree/build_detail_lines output — catches ~60% of TUI regressions without touching curses.
