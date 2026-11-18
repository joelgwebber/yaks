---
id: yaks-5df7
title: 'Commit view: Enter on a file unfolds its diff; the ? help names the diff keys plainly'
status: shaving
type: feature
priority: 2
created: '2026-11-18T22:42:30Z'
updated: '2026-11-18T22:48:31Z'
tags:
- tui
- git
---

Joel (2026-11-18): looking for the diff in the keyboard help, it reads as tree.toggle / tree.left / tree.right, obscure unless you know the unification. Enter on a file row should unfold its diff (as Enter folds a directory in the directory view); Enter on a diff line still opens the file at that line. Follow-up to yaks-bdab.

#### 2026-11-18T22:42:34Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-18T22:48:31Z

*sources-triage · commit: adb356bd0f7b9a6b344515d289f14284b84a0b67*

Landed on main.

- `Enter` on a file row of the commit view unfolds or folds its diff (as `Enter` folds a directory in the directory view); `Enter` on a diff line opens the file at the commit at that line, as before. `Space`, `l`/`h` unchanged. The bottom hint now reads `Enter:diff` (it was `Enter:open  Space:diff`), and the `?` popup's `pane.enter` is the key that unfolds, so the diff no longer hides behind `tree.*`.
- `docs/tui.md` (intro and key table), the `?` help line, the commit docshots refreshed.
- Evidence: gate green; tests `enter_on_a_file_unfolds_its_diff_and_on_a_diff_line_opens_the_file_at_that_commit`, `history_re_reads_the_commit_view_with_its_cursor` (now also checks the open diff comes back); headless on the real farm, a9e9430 with Enter on `src/gitrepo.rs`: [frame](yaks-5df7/enter-unfolds.txt), then Enter on its hunk header opened `src/gitrepo.rs@a9e9430` at line 212.
