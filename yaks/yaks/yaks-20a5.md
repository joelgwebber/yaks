---
id: yaks-20a5
title: Take the image pane from toque
status: shorn
type: task
priority: 2
created: '2026-11-21T18:27:17Z'
updated: '2026-11-22T17:50:51Z'
tags:
- tui
---

toque yak-d206 extracted imageview.rs's zoom/pan/protocol into toque's image feature (for canon's album art). yaks keeps a thin ImageView: the attachment's name and path, decoding, the header, and the key mapping; it derefs to toque::image::ImageView.

#### 2026-11-21T18:27:24Z

*tui-design · kind: move · from: hairy · to: shaving · thread: ksQQ8ThW0PX0SAyxtEBqIr2N1ZLOAHSsupIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-21T18:27:25Z

*tui-design*

Evidence: cargo test → 1269 passed, 1 failed (land::tests::a_conflicting_merge_falls_back_to_cherry_pick_and_names_the_files, fails identically on main); every image-viewer test (fit, zoom steps, pan, colours, inline images) passes on toque's code. docshots_image_* regenerate byte-identical to main's own run (cmp over all four SVGs). Geometry unit tests moved to toque with the helpers. cargo fmt --check clean.

#### 2026-11-21T18:27:26Z

*tui-design · kind: move · from: shaving · to: shorn · thread: ksQQ8ThW0PX0SAyxtEBqIr2N1ZLOAHSsupIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-22T17:50:51Z

*herd-files · kind: change · section: none*

No user-facing change.
