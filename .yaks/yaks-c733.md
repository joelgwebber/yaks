---
id: yaks-c733
title: 'Code blocks and tables: no-wrap, horizontal scroll'
status: shaving
type: feature
priority: 3
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T04:58:35Z'
parent: yaks-ea13
depends_on:
- yaks-649d
labels:
- markdown
- tui
---

On the renderer core.

- **Code blocks** (fenced and 4-space indented) never wrap: fences hidden, a dim language label (e.g. `java`) on the block's first row, optional subtle background. Mermaid is just a labelled code block.
- **Tables** (GFM) render with box-drawing borders and per-column alignment; short rows pad with empty cells (as GFM does); cells hold inline styling.
- **Horizontal scroll, per block.** When the line cursor is on a row of a no-wrap block, `h`/`l` and `←`/`→` scroll that block sideways (other blocks don't move); a dim `…`/`›` marks clipped edges. Today `h`/`←` in the detail pane mean "back to the list", so on a no-wrap block they are re-bound and `Esc` still returns to the list; at scroll 0, `h` falls through to "back to list" (decide and document). Update `help_content` and docs.
- Yank on these rows copies the full source lines.

---
▸ 2026-10-17T04:58:35Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-17T04:58:35Z [delta-lead]
Claim (yakherd delta-lead, 2026-10-17): owner **hscroll-1**.

Starting point: the renderer core (src/tui/mdrender.rs, landed 47d5d5e) already renders code blocks and tables unwrapped (`MdRow.nowrap = true`), fences hidden, tables boxed with aligned/padded cells. Read `mdrender.rs` and `detail::body_rows` first; `docs/assets/tui-md-render-2.svg` shows the current look.

Scope:
- `mdrender.rs`: a dim language label for fenced code (e.g. a right-aligned or leading `java` tag on the block's first row, or a separate label row: your call, `Decision:` note), and a way to identify which rows belong to the same no-wrap block (e.g. a block id on `MdRow` carried into `DLine`; add the field to `DLine` in detail.rs with `src: None`-style defaults, in the same shape as the `src` field).
- Horizontal scroll per block: `App` state for the h-offset of the no-wrap block under the line cursor (reset when the cursor leaves the block or the yak changes); `render_dline` / `render_detail` in src/tui/render.rs clip such rows at the offset and pane width, with a dim `‹`/`›` at a clipped edge.
- Keys in the detail pane (`handlers.rs`, the detail match): on a no-wrap row `l`/`→` scroll right and `h`/`←` scroll left; `h`/`←` at offset 0 keeps today's meaning (back to list); `Esc` unchanged. Update `help_content` and `help_hint` if it fits, and docs/tui.md.
- Yank of these rows already copies source (core); keep it.

Out of scope: syntax colouring (yaks-3020), the editor, footnotes/links. Evidence: failing-first tests (clip + markers at an offset; `l` moves the offset only on a no-wrap row; `h` at 0 returns to the list), a docshots-style SVG or headless frames of a scrolled code block and table (add to `docshots_md_render` as `tui-md-render-scrolled`, generating only your SVG), PNG via `firefox --headless --profile $(mktemp -d) --screenshot out.png --window-size=W,H file://...svg` (Chrome is not installed). Judge: coordinator at landing.
