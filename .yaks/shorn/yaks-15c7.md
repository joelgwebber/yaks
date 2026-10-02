---
id: yaks-15c7
title: Scrolling in yak details doesn't show last line
type: bug
priority: 1
created: '2026-10-02T14:16:13Z'
updated: '2026-10-02T14:19:11Z'
labels:
- ui
---

We should also ensure it will always scroll at least one line below the last, so the last line of text doesn't directly abut the bottom of the viewport.

![15c7-after](artifacts/yaks-15c7/15c7-after.txt)

---
▸ 2026-10-02T14:19:11Z [claude]
Headless frame (80x16, 40-line body, after G): last line visible with a blank row beneath it.

---
▸ 2026-10-02T14:19:11Z [claude]
Root cause: the sticky header takes the top row once detail_scroll > 0, but scroll_line_into_view/mouse wheel sized the viewport as the full detail_page — so the last line landed one row below the visible area (also yaks-a211). Fix (detail_nav.rs, mouse.rs): detail_rows_shown(scroll) (page, or page-1 once scrolled) + detail_max_scroll() (leaves a blank row under the last line; content that fits never scrolls); scroll_line_into_view, wheel scroll and nav-restore all use them. Tests: scroll_into_view_is_stable_and_minimal (rewritten for header-aware viewport), detail_last_line_is_visible_with_a_blank_row_below (G frame + j-by-j never loses the cursor). cargo test --workspace green. No keys/docs changed.
