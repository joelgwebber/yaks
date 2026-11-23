---
id: yaks-bc4a
title: 'TUI draws an image wherever GitHub does: list items, quotes, pinned and host-URL images'
status: shaving
type: feature
priority: 2
created: '2026-11-23T00:18:31Z'
updated: '2026-11-23T00:19:09Z'
parent: yaks-e387
tags:
- tui
- images
- markdown
---

Under yaks-e387 (TUI and GitHub resolve links the same way).

GitHub draws every markdown image: in a paragraph, a list item, a quote, a table. The TUI draws one in place only when it is a local attachment or an unpinned farm file in a top-level paragraph ([image_slot](code:src/tui/detail.rs#L777-L780), [end_image](code:src/tui/mdrender.rs#L707-L717)); everything else is a `🖼 alt` link row. Found on [/doks/releases/v0.0.41.md](/doks/releases/v0.0.41.md): a pinned image under a bullet. yaks-0bf0 and yaks-0370 deferred "images inside list items and quotes (need the container gutter as a draw offset)".

Want:

- An image in a list item or a quote (nested too) is drawn in place, indented to its container's gutter, with its caption, as a top-level one is now. A table cell stays the link row (a grid row cannot hold a picture): say so in docs/tui.md.
- An image's target resolves through the one resolver (link -> `Place`, applying `Space::read_url`) instead of the string forms in mdrender, so whatever a link can name, an image can show: an attachment, a farm file, a pinned farm file (`@sha`, and the host-URL form a sibling yak adds to `read_url`). A pinned image reads its bytes at the commit (`links::read_bytes_at`, LFS smudged, as `farm_image_at` does), so the inline cache key must include the rev, not only path and mtime.
- A pinned document's own images (the viewer of `x.md@sha`) are drawn at that commit, which lifts the "pinned document draws no images" rule in `handlers.rs`.
- docs/tui.md's inline-image section; a docshot or an attached frame of a list image and a pinned image drawn.

#### 2026-11-23T00:19:09Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
