---
id: yaks-649d
title: 'Renderer core: pulldown-cmark to a source-mapped row model'
status: hairy
type: feature
priority: 2
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T04:25:16Z'
parent: yaks-ea13
labels:
- markdown
- tui
---

Replace the line-at-a-time `markdown::Highlighter` in the read pane with a block renderer. Follows the decisions on yaks-e9de (CommonMark soft breaks, split with `content::parse` first, source map).

## Shape

- Add `pulldown-cmark` (pure Rust; GFM options on). Parse each content block (description, each comment) as its own document.
- Lay blocks out with a **prefix stack**: every container pushes a (first-row, continuation-row) prefix pair: `▎ ` / `▎ ` for a quote, `• ` / `  ` for a bullet, `1. ` / `   ` for an ordered item. A paragraph is word-wrapped to the pane width minus the stacked prefix width. Nesting (a quote holding a list holding code) composes with no per-combination code; this is what fixes hanging indents and the "sea of gray" quotes.
- Output rows that carry styled spans, link spans (the existing `Target`s, so Tab/Enter/jumplist keep working) and the **source line range** each row came from. Rows replace the wrap step in `detail::wrap` for body text; header fields keep their current path.
- Keep working: find (`/`, matches displayed text), Tab/Enter links, visual selection + yank (copies the *source* lines of the selected rows, not the displayed text), `E` (block-level is enough here; exact-line is its own yak), Ctrl-N/P block jumps (today found by matching `---`/`▸` in display lines, which this replaces with block indices from `content::parse`).

## In scope here

Paragraphs (soft/hard breaks), ATX + setext headings, bullet/ordered lists (start number, nesting, lazy continuation, tight vs loose), block quotes (nested, lazy), rules, inline emphasis/strong/code spanning lines, backslash escapes, entities, inline HTML as literal text. Code blocks and tables get a plain placeholder rendering here; their real treatment is a sibling yak.

## Evidence

yaks-ea13's own body is the fixture: before/after headless frames attached, an insta snapshot test of it, and a docshots scene. Tests that fail first for: hanging indent of a wrapped list item, a wrapped quote keeping its bar, emphasis across a soft break, a soft break joining lines, yank returning source.
