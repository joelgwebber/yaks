---
id: yaks-9dec
title: 'TUI phase 4b: the detail pane''s detail_* fields fold into one LineNav'
status: shaving
type: task
priority: 2
created: '2026-11-03T04:28:01Z'
updated: '2026-11-03T05:08:29Z'
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
