---
id: yaks-e9de
title: Readable markdown + detail/edit QoL
status: shaving
type: feature
priority: 2
created: '2026-10-17T04:23:51Z'
updated: '2026-10-17T04:43:37Z'
labels:
- markdown
- tui
---

Umbrella for the markdown rendering, markdown editing and detail-pane QoL work planned in the Delta thread "Markdown rendering / editing QoL" (2026-10-17). Children carry the work; this yak carries the decisions every child must follow.

## Decisions (agreed with Joel, 2026-10-17)

- **CommonMark line breaks (option B).** A single newline is a soft break and renders as a space; only a blank line starts a paragraph. A hard break is two trailing spaces or a trailing backslash. Legacy bodies written one-item-per-line will run together; that is accepted (they get fixed by hand, or later by yaks-33c8), and the skills must teach agents to use lists or blank lines.
- **Parser: `pulldown-cmark`** (pure Rust, keeps the C-free release matrix). GFM tables, task lists, strikethrough, footnotes and alerts on. Its offset iterator is the source map.
- **Split comments before parsing.** The body is split with `content::parse` first; the description and each comment render as separate markdown documents (no state leaks between them, e.g. an unclosed fence).
- **Rendering may collapse, the editor never does.** The read pane may hide markers, collapse links to their text, reflow paragraphs and draw tables. The editor keeps display width == text width: styling, dimming and `Modifier::HIDDEN` only. Anything that needs a display mapping in the editor (collapsed links, wrap hanging indents, rendered tables) is past the edtui breakpoint and out of scope.
- **Every rendered row knows its source lines.** Yank copies source markdown, find searches displayed text, `E` opens the editor at the exact source line.
- **No-wrap blocks (code, tables) scroll horizontally**, per block, when the line cursor is on them; nothing else in the pane scrolls sideways.
- **One style table** for the read pane and the editor; body headings (3 distinct levels) must not look like the pane's own cyan-bold labels.
- **DRY:** one comment-header design (shared with the edit form's `render_block_separator`), one parser, one palette.
- **Skipped:** math (revisit only with Kitty-graphics images), emoji shortcodes, definition lists. Mermaid renders as a labelled code block. Inline HTML renders as literal text.

## Evidence convention

Every UI child attaches before/after headless frames (`yaks tui --headless`, plus a docshots SVG for durable states). yaks-ea13's body is the torture-test fixture.

## Order

1. Quick wins (independent): editor background, pinned-header rule + status icon, dates, link `)` bug, comment headers from `content::parse`.
2. Shared style palette (design pass).
3. Renderer core (pulldown-cmark → row model with source map, prefix-stack layout).
4. Features on the core: headings/inline, code + tables with h-scroll, links + attachments, GFM extras, exact-line `E`.
5. Editor: hidden markers, quieter links, highlights from the shared parser, auto-continue (in the edtui fork).
6. View/edit unification (design pass first).
7. Skills guidance (yaks-382b) once the core lands; each feature updates skills/help/docs in its own change.

---
▸ 2026-10-17T04:31:30Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-17T04:33:16Z [delta-lead]
Batch 1 spawned 2026-10-17 from base ce4aeba, all on claude-sonnet-5-5 (worker profile, isolated clones):
- bg-1 → yaks-5408, thread ksQQqt3mx3gRRTWsmE1GMTX1T5LOABiKcJIBxBCg_8DSgeNASrgoPHtnOl5O
- hdr-1 → yaks-66a3, thread ksQQqt3mx3gRRTWsmE1GMTX1T5LOABiLPZIBxBCg_8DSgeNASrgoPHtnOl5O
- links-1 → yaks-f67e, thread ksQQqt3mx3gRRTWsmE1GMTX1T5LOABiMf5IBxBCg_8DSgeNASrgoPHtnOl5O
- dates-1 → yaks-ef82, thread ksQQqt3mx3gRRTWsmE1GMTX1T5LOABiOt5IBxBCg_8DSgeNASrgoPHtnOl5O
- palette-1 → yaks-bc53, thread ksQQqt3mx3gRRTWsmE1GMTX1T5LOABiTKpIBxBCg_8DSgeNASrgoPHtnOl5O
The lead (delta-lead) builds yaks-649d (renderer core) in parallel, in a new module.

---
▸ 2026-10-17T04:37:18Z [delta-lead]
Joel answered yaks-2362 and yaks-3b8d (both yes, both as a schema migration in the next release): new yak yaks-a870 (schema 5 migration) carries both fixups; migrate-1 spawned on it.

---
▸ 2026-10-17T04:43:37Z [delta-lead]
Batch 1 landed 2026-10-17 by the yakherd (land.sh, merge by SHA, each identical to the computed merge; `cargo test --release` 623+ pass, doctor clean):
- links-1 e7f2d34 (yaks-f67e) → d430c17
- dates-1 7684791 (yaks-ef82) → cfa4bd1
- hdr-1 2e9ba38 (yaks-66a3) → da1a2d2
- bg-1 d9f3493 (yaks-5408) → bc1deaf
- palette-1 2a6faf4 (yaks-bc53, sign-off ask open to Joel) → b7c9cea
Checked on the real build: headless detail shows `Status: 🪒 shaving`, `Created: 19m ago`, the pinned header with its rule row. Lead's mdrender module (yaks-649d, not yet wired) committed 9e986e6.
