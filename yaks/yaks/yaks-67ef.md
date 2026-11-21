---
id: yaks-67ef
title: Take theme surfaces from toque
status: shaving
type: task
priority: 3
created: '2026-11-21T04:57:11Z'
updated: '2026-11-21T05:10:36Z'
tags:
- tui
---

src/tui/surface.rs moved to toque (toque feature theme, toque farm yak-4b3b) so canon's TUI can share it. Switch yaks over with no change to its snapshots or docshots; YAKS_THEME stays the override.

#### 2026-11-21T04:57:21Z

*tui-design · kind: move · from: hairy · to: shaving · thread: ksQQ8ThW0PX0SAyxtEBqIr2N1ZLOAHSsupIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-21T05:02:26Z

*tui-design*

Done in the working tree, not committed (waits on toque `ed595ee` being pushed): `src/tui/surface.rs` removed; callers use `toque::theme`; `init("YAKS_THEME")`; the direct `terminal-colorsaurus` dependency moves to toque.

Against the patched local toque: fmt clean; `cargo test` 1272 passed, 1 failed (`land::tests::a_conflicting_merge_falls_back_to_cherry_pick_and_names_the_files`, which fails identically on clean `main`); `docshots_surfaces` regenerates `docs/assets/tui-surfaces.svg` byte-identical; no snapshot changed.

#### 2026-11-21T05:10:36Z

*tui-design*

Committed and pushed against toque ed595ee (now on origin). Snapshots unchanged; tui-surfaces.svg regenerates byte-identical. The one failing test (land::...cherry_pick...) fails identically on main before this change.
