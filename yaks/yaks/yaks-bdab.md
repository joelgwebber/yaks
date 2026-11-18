---
id: yaks-bdab
title: 'Commits phase 5: an inline diff in the commit view'
status: shorn
type: feature
priority: 3
created: '2026-11-18T03:03:10Z'
updated: '2026-11-18T16:26:21Z'
parent: yaks-3515
depends_on:
- yaks-3f90
tags:
- tui
- git
---

Phase 5 of yaks-3515; needs phase 2. In the commit view, a file's diff shown inline (coloured `+`/`-`, the hunks foldable), as well as opening the file at that commit. Later; plan it once phases 1 to 3 are in use.

#### 2026-11-18T16:16:18Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-18T16:26:20Z

*sources-triage · commit: a9e94302333defc9cac6e8a6698ccc07b54c60e2*

Landed on main.

- `Space` (or `l`/`→`) on a file row of the commit view unfolds its patch under it: hunk headers cyan, `+` green, `-` red, three lines of context, against the first parent for a merge (`gitrepo::commit_file_diff`). `h`/`←` folds it back and puts the cursor on its file, also from a diff line. `▸`/`▾` marks each file.
- `Enter` on a diff line opens the file at the commit at that line (a deleted line: where it was; `commitview::patch_lines` numbers each line on the new side).
- The patch is read from git once, when first unfolded. The open folds are part of the history location, so `o` comes back to the same unfolded diff.
- Keys in `BINDINGS` (`Ctx::Commit`); `docs/tui.md` key table and intro; new docshot `docs/assets/tui-commit-diff.svg` (the other commit docshots refreshed: they predated the `Space:diff` hint and the `Code:` row).
- Evidence: gate green; tests `space_unfolds_a_files_diff_inline_and_enter_on_a_line_opens_it_there` (+ snapshot `commit_view_diff`), `patch_lines_number_each_line_on_the_new_side`; headless on the real farm, 65fa57e with `src/land.rs` unfolded: [frame](yaks-bdab/commit-diff-real-farm.txt); [docshot](yaks-bdab/tui-commit-diff.svg).

#### 2026-11-18T16:26:21Z

*sources-triage · kind: move · from: shaving · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn
