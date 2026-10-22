---
id: yaks-0bf0
title: Show images inline in yak bodies and comments
status: hairy
type: feature
priority: 3
created: '2026-10-22T20:35:28Z'
updated: '2026-10-22T20:35:28Z'
labels:
- tui
- images
- markdown
---

Follow-up to yaks-4305, which only shows an image in a pane of its own after Enter. An image link in a yak body (`![name](<id>/<file>)`) renders today as a `🖼 name` link row; it should draw the picture in place, in the detail pane, as GitHub does.

Things to work out (this is mostly a layout problem, not a decoding one):

- The detail pane is a list of text rows (`detail::DLine`) with a row-indexed cursor, scroll, find and link jumps. An image occupies N rows of cells and, with a graphics protocol, is not text at all. Give it N placeholder rows so every index stays valid, and paint the image over them after the text.
- ratatui-image draws with `StatefulImage` (resize and encode when the area changes), and a protocol image must be clipped or skipped when partly scrolled off. Its docs warn that rendering blocks the UI thread unless encoding runs on a background thread (`ThreadProtocol`). Measure that for a large PNG before committing to the simple path.
- The row count must be known before the image is decoded, or the pane reflows when it loads. Read the header (dimensions) first, decode lazily, and cache by path and size.
- Kitty graphics persist across redraws, so make sure scrolling and overlays (help, pickers) clear them. This is where terminal bugs live. Keep half-block as the safe fallback, and consider a config switch to turn inline images off.
- Image size and fit: see the sizing yak (it decides how many rows an image takes).
- Remote (`http://`) images stay links.

Depends on the viewer from yaks-4305 (`src/tui/imageview.rs`, the shared picker on `App`). Acceptance: scroll a yak body past an image in Ghostty with no artifacts and no flicker, plus a toque frame with the half-block image in the pane.
