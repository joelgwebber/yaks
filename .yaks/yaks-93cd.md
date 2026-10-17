---
id: yaks-93cd
title: Comment headers in the detail pane, shared with the edit form
status: shorn
type: feature
priority: 2
created: '2026-10-17T04:25:39Z'
updated: '2026-10-17T04:58:11Z'
parent: yaks-faf5
depends_on:
- yaks-ef82
labels:
- markdown
- tui
---

Split the body with `content::parse` and render each comment under a header row instead of the raw `---` / `▸ <ts> [actor]` lines: a colored marker, the actor, the relative date (from the dates yak), and a rule to the right edge. One function draws this header for both the detail pane and the edit form (today's `render_block_separator` in src/tui/create.rs), so there is one design.

Ctrl-N/P block jumps and `E` must keep finding blocks: derive block indices from `content::parse` rather than `block_index_per_line`'s display-text match. Can land before the renderer core (body lines inside each block keep today's highlighter until the core replaces it).

---
▸ 2026-10-17T04:58:04Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-17T04:58:08Z [delta-lead]
Done in yaks-649d (commit 47d5d5e) by delta-lead: the detail pane splits the body with `content::parse` and draws each comment under a `▸ author · when ────` header row (relative/local time from dates-1's `Clock`); `detail::comment_label` + `palette::comment_header`/`rule` are shared with the edit form's `render_block_separator` (one design, DRY). Block indices come from `DLine.src`, not a `---`/`▸` text match, so Ctrl-N/P and `E` keep working (tests `e_on_comment_line_opens_form_focused_on_that_comment`, `ctrl_n_p_navigate_content_blocks_in_detail`, `comments_render_under_a_header_with_their_block_index`). The deps on yaks-2362/yaks-a870 were removed: rendering does not depend on the on-disk shape change, and `content::parse` will accept both shapes there. Evidence: docs/assets/tui-md-render-2.svg (PNG on yaks-649d) shows the header.

---
▸ 2026-10-17T04:58:11Z [delta-lead]
verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-17T04:58:11Z [delta-lead]
moved: shaving -> shorn
