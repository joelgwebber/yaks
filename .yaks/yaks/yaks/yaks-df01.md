---
id: yaks-df01
title: Next/prev comment improvements
status: shaving
type: task
priority: 3
created: '2026-11-11T20:46:10Z'
updated: '2026-11-14T11:39:32Z'
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

[df01-after](yaks-df01/df01-after.txt "AFTER: same script. Frame 3 shows comment 2 whole (scroll 19); frame 5, the 25-row comment, has its header at the top (scroll 45, header pinned under the sticky rule); frame 6 Ctrl-P from comment 4's header goes to comment 3's header.")

---
▸ 2026-11-14T11:36:45Z [comments-1]

Evidence correction: the BEFORE caption cites the wrong frame. In the BEFORE file, frame 3 (Ctrl-N onto comment 2) is the one to read: the header is on the last visible row and all 7 body rows are below the fold (scroll 10). In the AFTER file the same keystroke (frame 3) shows comment 2 whole (scroll 19).

The demo farm is /tmp/df01farm (`YAKS_DIR`), binaries /tmp/yaks-before (HEAD actions.rs) and /tmp/yaks-after.

---
▸ 2026-11-14T11:36:45Z [comments-1]

Decision: Next scrolls to the block's last row via `scroll_line_into_view` (existing helper, minimal scroll), then, only if the header ended above the viewport (block taller than the page), sets scroll to the header row (clamped by `detail_max_scroll`). Rejected: a new Viewport helper (out of scope, no refactor).

Decision: the block's "end" is its last row carrying that block's `Src`, so the trailing blank separator is not required to be visible.

Decision: Previous only scrolls minimally (header into view); no whole-comment visibility for Previous (not requested).

---
▸ 2026-11-14T11:39:32Z [comments-1]
kind: verify
result: fail
---

verify: `cargo test --workspace` -> FAIL (exit 101)
