---
id: yaks-89fc
title: Image size syntax and default fit in markdown
status: hairy
type: feature
priority: 3
created: '2026-10-22T20:35:29Z'
updated: '2026-10-22T20:35:29Z'
labels:
- tui
- images
- markdown
---

Follow-up to yaks-4305. Decide how a yak author says how big an image shows, and what the default is, once images render inline (and in the viewer). Research findings so far, so we pick a syntax that does not break other viewers:

**There is no CommonMark standard.** The spec has no image dimensions; the thread on talk.commonmark.org (2014) never settled one. What exists:

| Syntax | Where | Notes |
|---|---|---|
| `![alt](img.png){width=50%}` | pandoc `link_attributes`, kramdown, markdown-it-attrs, PHP Markdown Extra | The most widespread generic attribute form. Units: none means px, or `px`, `cm`, `in`, `%`. Renders as plain `![alt](img.png)` plus literal `{width=50%}` text in a viewer that lacks the extension (GitHub). |
| `![alt =300x200](img.png)` | markdown-it `img-size` "new syntax" | Valid CommonMark (size lives in the alt text), degrades to alt text containing `=300x200`. Either dimension optional, `%` allowed. |
| `![alt\|300x200](img.png)` | Obsidian | Same trick with a pipe. |
| `![alt](img.png =300x200)` | legacy markdown-it | Breaks image parsing on GitHub: avoid. |
| `<img src width>` | HTML | GitHub's only supported way; yaks bodies have no inline HTML, so not an option. |

**Lean:** the alt-text forms are the only ones that stay valid CommonMark and GitHub-safe (they only pollute the alt text). The `{width=50%}` attribute form is the most conventional but needs `pulldown-cmark`'s attribute extension turned on (check: it has `ENABLE_*` flags for heading attributes; confirm for images before assuming). Whichever we pick, `pulldown-cmark` gives us the image's alt text and destination, so parsing the size out of the alt text is cheap and needs no parser change.

**Units in a terminal:** px has no meaning without a cell size (we have it from `Picker::font_size()`); `%` of pane width is the natural unit; maybe also `cells`.

**Defaults to decide:**
- Fit-width, never enlarge (what the viewer does now, `Resize::Fit`)? Or natural size capped at pane width? Most viewers (GitHub) show natural size capped at the container width, i.e. exactly this.
- A height cap, so one tall screenshot cannot take a whole pane (inline only; the viewer uses the whole pane).
- Ways to ask for the other behaviours: natural size, fill width (scale up), an explicit column or row count.

Writes touch: `src/tui/mdrender.rs` (the image link row), `docs/tui.md`, the `yaks` skill's markdown notes (what authors should write), and the `yaks attach` link form only if it should emit a size by default (probably not). Sits beside the inline-image yak: that one needs the row count this one decides.
