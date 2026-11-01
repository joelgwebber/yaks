---
id: yaks-ebb2
title: 'TUI: a Wiki tab listing pages and sources, with rendered page view'
status: shaving
type: feature
priority: 2
created: '2026-11-01T18:53:12Z'
updated: '2026-11-01T23:04:46Z'
parent: yaks-200f
depends_on:
- yaks-9af0
- yaks-84e0
labels:
- wiki
- tui
---

A view of `wiki/`, herd wikis and `sources/`, opening a page in the rendered markdown view with the detail pane's navigation (yaks-84e0).

---
▸ 2026-11-01T23:04:46Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T23:04:46Z [delta:farm-wiki]

Claim (yakherd delta:farm-wiki): worker `wiki-1`, on branch `wiki` at 20f6bc6. It runs the TUI arc serially in one worktree: yaks-ebb2, then yaks-7395, then yaks-1532, with one commit each.

**yaks-ebb2, the Wiki tab.** A new top-level tab, after Starred and before Sheds, listing the farm's pages:

- Sources: `pages::load_all` covers `wiki/` and herd wikis; `sources::list` covers sources.
- Grouping: a tree by folder, or a flat list with paths. Record the choice as a `Decision:`.
- Columns: title, kind, tags.
- Enter opens a page in the rendered markdown viewer that yaks-84e0 just built on the shared `linenav` component (`src/tui/linenav.rs`; the `AttachView` viewer in `handlers.rs`/`render.rs`).
- The page's frontmatter shows as a header block above the body.
- Sources open their `source.md` plus `extracted.md`.
- `/` filtering works like the other tabs.
- The tab count is the page count.
