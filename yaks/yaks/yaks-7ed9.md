---
id: yaks-7ed9
title: Archive member and directory headers show the farm path (yak/yaks/yak-1a2b/code.zip/...) where an attachment shows yak-1a2b/file
status: hairy
type: bug
priority: 3
created: '2026-11-22T17:20:00Z'
updated: '2026-11-22T17:20:00Z'
tags:
- tui
---

Seen in the docshots refreshed by yaks-daa4 (docs/assets/tui-archive-member.svg, tui-archive-tree.svg vs tui-line-range.svg): the viewer header and the 'viewing' line name an archive member as `yak/yaks/yak-1a2b/code.zip/src/lib.rs` and the directory view as `yak/yaks/yak-1a2b/`, while an attachment reads `yak-1a2b/run.log`. The link form is `<id>/code.zip/src/lib.rs`; headers should use it too (one naming rule for places).
