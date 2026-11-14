---
id: yaks-df01
title: Next/prev comment improvements
status: shaving
type: task
priority: 3
created: '2026-11-11T20:46:10Z'
updated: '2026-11-14T11:36:33Z'
tags:
- tui
---

A few small tweaks:
- Next comment should both move the cursor to the comment header (as currently), *and* attempt to ensure the full comment's visible by ensuring the bottom is also in the viewport. If that's not possible because of its length, we should default to putting the comment header at the top of the viewport.
- Previous comment should focus the _current_ comment's header, unless the header's already selected.

---
▸ 2026-11-14T11:32:22Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T11:32:24Z [linkplan-lead]

Claim: worker `comments-1` (Sonnet tier). Owns `tui/actions.rs` (next/previous comment) and its tests. Out of scope: keymap, rendering, other files. Evidence: tests asserting the scroll offsets, and a toque frame before and after.

---
▸ 2026-11-14T11:32:57Z [comments-1]

Start: Sat Nov 14 11:32:56 UTC 2026. yaks path: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-df01.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/y4b40t5p4bcd/yaks.

---
▸ 2026-11-14T11:35:30Z [comments-1]

Implemented jump_block change in actions.rs + 3 tests in tests.rs (short comment, too-long comment, previous header/inside); targeted tests pass. Next: toque before/after frames, gate.

[df01-before](yaks-df01/df01-before.txt "BEFORE (main e4d0ff5a): headless toque frames, 70x16, Enter then Ctrl-N x4 then Ctrl-P x2 on a yak with comments of 4/7/10/25 rows. Frame 4 (Ctrl-N to comment 2): only 3 of its 7 rows show, the rest is below the fold.")
