---
id: yaks-bc53
title: Shared markdown style palette for read pane + editor
status: hairy
type: task
priority: 2
created: '2026-10-17T04:24:29Z'
updated: '2026-10-17T04:24:29Z'
parent: yaks-e9de
labels:
- markdown
- tui
---

Design pass, then a small code change. One table of styles used by both the read pane and the editor (today `markdown.rs` has its own private styles and the read pane layers its own link/label colors on top).

Must cover: H1/H2/H3 (three visibly distinct levels that do not read like the pane's own cyan-bold labels `Children:` / `Depends on:`), emphasis, strong, strikethrough, inline code, code block (fg + optional subtle bg), quote bar, list bullets per depth (•, ◦, ▪), rules, links (text vs dimmed URL), task boxes, alert kinds (NOTE/TIP/IMPORTANT/WARNING/CAUTION), comment headers, dimmed markers (editor).

Evidence: a docshots scene rendering every style against the default and a light terminal palette; attach the frames. Get Joel's sign-off on the frame before the renderer core consumes it.
