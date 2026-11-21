---
id: yaks-20a5
title: Take the image pane from toque
status: hairy
type: task
priority: 2
created: '2026-11-21T18:27:17Z'
updated: '2026-11-21T18:27:17Z'
tags:
- tui
---

toque yak-d206 extracted imageview.rs's zoom/pan/protocol into toque's image feature (for canon's album art). yaks keeps a thin ImageView: the attachment's name and path, decoding, the header, and the key mapping; it derefs to toque::image::ImageView.
