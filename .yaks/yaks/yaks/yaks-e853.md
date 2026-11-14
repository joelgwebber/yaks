---
id: yaks-e853
title: '[/] selection of links in details should scroll horizontally into view when necessary'
status: shaving
type: bug
priority: 3
created: '2026-11-09T23:51:02Z'
updated: '2026-11-14T11:32:25Z'
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
