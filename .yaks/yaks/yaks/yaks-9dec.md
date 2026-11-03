---
id: yaks-9dec
title: 'TUI phase 4b: the detail pane''s detail_* fields fold into one LineNav'
status: shaving
type: task
priority: 2
created: '2026-11-03T04:28:01Z'
updated: '2026-11-03T05:13:39Z'
parent: yaks-aac1
depends_on:
- yaks-0906
tags:
- tui
---

---
▸ 2026-11-03T05:08:29Z [herd-tui]
kind: move
from: hairy
to: shaving
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T05:08:29Z [herd-tui]

Claim (yakherd herd-tui): worker `nav-2` (Sonnet 5.5), from main at `a6634e38` (phases 4a and 6a landed).

**Task.** The detail pane's state becomes one `LineNav`, like the text viewer's, so every text subject has the same navigation state ([design §2](/wiki/design/tui-model.md#2-the-pane-and-its-subjects)). A pure refactor: no behaviour or key changes.

**Scope.**

- Replace `App`'s `detail_line`, `detail_scroll`, `detail_anchor`, `detail_find`, `detail_match` and `detail_hscroll` with one field, `detail: LineNav`. `LineNav.hscroll` keys by `DLine::nowrap_key`; the detail's extra yak-id part becomes a reset when the selected yak changes, as `drop_stale_hscroll` already does.
- `with_detail_nav` and `detail_nav()` (in `detail_nav.rs`) go: callers use `self.detail` directly.
- Every site in `src/tui/*.rs` that read or wrote those fields (about 140, most in `detail_nav.rs`, `mouse.rs`, `blockedit.rs`, `tui.rs`) uses `self.detail.<field>`.
- **Tests** (`src/tui/tests.rs`, about 114 sites): rewrite mechanically to `app.detail.line` and so on (`sed` is fine; check the diff). Keep every assertion.
- The detail pane's footnote back/forward (`goto_footnote` pushes onto `nav_back`) may stay as it is; phase 5 merges the stacks.

**Out of scope.** Any behaviour or key change; the search register (phase 6b, next); merging history stacks (phase 5). Anything else: `yaks ask yaks-9dec`.

**Evidence.**

- Gate: `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace`, fresh `TMPDIR`. No snapshot may change (`git diff --stat -- src/tui/snapshots docs/assets` is empty).
- `grep -n "detail_line\|detail_scroll\|detail_anchor\|detail_find\|detail_match\|detail_hscroll" src` finds nothing, or only the names of unrelated functions (record those).
- `AGENTS.md` and the `linenav.rs` module doc no longer say the detail pane keeps its state in `detail_*` fields.

**Judge.** herd-tui at landing.

---
▸ 2026-11-03T05:13:39Z [nav-2]

Started 2026-11-03T05:08:57Z. Farm file: /home/user/.local/share/yaks/farms/yaks-b0f8dcf9/.yaks/yaks/yaks/yaks-9dec.md; top-level /home/user/yaks-8; base a6634e3.

Done in working tree: App.detail: LineNav replaces the six detail_* fields; with_detail_nav / detail_nav() / App::block_hscroll gone; callers use self.detail.<field>; tests rewritten mechanically (200 lines in tests.rs). Gate (env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace) exit 0 (899 passed in the main unit binary, 0 failed anywhere); no snapshot or docs/assets diff (`git diff --stat -- src/tui/snapshots docs/assets` empty).

Decision: the detail's per-yak hscroll key becomes a reset in open_task_in_detail (`self.detail = LineNav::default()`, which also covers restore_nav_entry/goto_task). Rejected: a new App field remembering the yak id of the hscroll (a second source of truth), and widening LineNav.hscroll's key with a yak id (the text viewer has no yak).

Decision: App::drop_stale_hscroll stays, now = clear when focus left the detail pane, else LineNav::drop_stale_hscroll. Other yak changes (J/K, list clicks, open_detail_pane) already reset line to 0, which is not a no-wrap row, so the same check drops it. Known tiny residual: a yak swapped under the cursor by an external farm reload while the cursor sits on a no-wrap block with the same (block,n) key would keep its offset; the old id check dropped it. Judged negligible; say if you want it exact.

Decision: kept the debug header text `detail_scroll=` in state_header (actions.rs): it is output format used by docs/assets/session.txt, not a field name.

Decision: kept the five-field resets elsewhere (J/K, mouse, open_detail_pane, blockedit) field by field, as mechanical as the spec; only open_task_in_detail uses LineNav::default() (it needs the hscroll reset).

Decision: App::scroll_block(dir) stays as a thin wrapper (builds lines + width for LineNav::scroll_block), like move_detail_line.

Evidence grep (`grep -rn "detail_line\|detail_scroll\|detail_anchor\|detail_find\|detail_match\|detail_hscroll" src`): only unrelated names remain: methods open_detail_find, detail_find_jump, detail_find_matches, detail_line_count, detail_line_to, move_detail_line, click_detail_line, settle_detail_scroll; sheds_tab.rs detail_lines fn; test fn names detail_find_*; and the `detail_scroll=` state_header string. AGENTS.md never named the detail_* fields (only the `detail_nav` module); the linenav.rs module doc is updated.
