---
id: yaks-e853
title: '[/] selection of links in details should scroll horizontally into view when necessary'
status: shaving
type: bug
priority: 3
created: '2026-11-09T23:51:02Z'
updated: '2026-11-14T11:38:24Z'
tags:
- tui
---

This only comes up in a handful of block types (maybe just tables?), but is quite confusing when it occurs.

---
▸ 2026-11-14T11:32:23Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T11:32:25Z [linkplan-lead]

Claim: worker `hscroll-1` (Sonnet tier). Owns `tui/linenav.rs` (`block_hscroll`, `scroll_block`) and the call sites in `tui/detail_nav.rs` and the viewer's link jump. Out of scope: keymap, link resolution (`follow.rs`, `links.rs`). Evidence: a snapshot of a wide table after Tab onto a link that starts off screen.

---
▸ 2026-11-14T11:33:05Z [hscroll-1]

Start: Sat Nov 14 11:33:04 UTC 2026; farm: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-e853.md; checkout: /Users/joel/src/yaks/.delta/worktrees/8ke1kyy8b0ks/yaks

---
▸ 2026-11-14T11:38:24Z [hscroll-1]

Decision: a link's `col` is measured in the row's UNSCROLLED text, not after the block's current horizontal scroll. `DLine::links` holds char columns of `DLine::text`; `render_dline` styles them first and only then `clip_nowrap` cuts the row by `hscroll` (render.rs). So the target offset is computed from the row's text minus the gutter, never from the current offset. Settled by reading `render_dline`/`clip_nowrap` and by the test `tab_onto_an_off_screen_table_link_scrolls_it_into_view_and_back` (Tab lands on `one`, then on `two`, Shift-Tab back to `one`: offsets 20 -> 28 -> back down).

Decision: scroll as little as it takes (scroll right until the link end is just left of the `›` marker; scroll left until its start is just right of `‹`; a link wider than the pane gets its start). Rejected: centring the link, or always putting it at the left edge (jumpy when moving between links on one row).

Decision: the link span is measured in display columns (wide chars via `unicode_width`), with the gutter (quote bar, list marker) taken off, as `clip_nowrap` splits it. A link inside the gutter is ignored.

Milestone: implemented `LineNav::reveal_link_sideways` called at the end of `jump_link` (new `width` arg, passed from `detail_nav.rs` via `detail_width` and from the viewer's `run_viewer_cmd` via `v.width`). Tests: detail pane + viewer. Docs line added in docs/tui.md. Rendered frames before/after looked at (46x12): before = Tab onto `one` left the offset 0 (link off screen); after = `one` visible after the `‹`, Tab -> `two` scrolls on, Shift-Tab scrolls back.
