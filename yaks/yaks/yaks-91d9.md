---
id: yaks-91d9
title: Documents don't render inline images
status: shorn
type: bug
priority: 3
created: '2026-11-03T03:59:12Z'
updated: '2026-11-03T21:21:27Z'
parent: yaks-f677
tags:
- tui
---

Inline images render perfectly well in yak details, but not in documents. See the attached image for an example.

---
![inline-images](yaks-91d9/inline-images.png)

---
▸ 2026-11-03T21:17:24Z [delta:starred-batch]

kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

[doc-inline-image-frame](yaks-91d9/doc-inline-image-frame.txt)

---
▸ 2026-11-03T21:21:25Z [delta:starred-batch]

Fixed: the document viewer now lays out markdown through the same function as the detail pane, so a lone image is drawn in place there too.

- [fn md_rows](code:src/tui/detail.rs) is the one layout both use (extracted from `body_rows`); `AttachView::rows` takes the image env and passes it on (none for a pinned `@sha` document, whose images on disk may be another version).
- `render_inline_images` takes the pane's scroll, so the viewer paints with its own.

Evidence: new test `a_page_in_the_viewer_draws_attachment_and_farm_root_images_in_place` (rows reserved and pixels drawn, for `<id>/shot.png` and `/sources/.../shot.png`); a headless frame of the real `wiki/topics/scribe-capture-and-mining.md` in the joel farm, with the image drawn under `🖼 Capture in progress` (attached).

---
▸ 2026-11-03T21:21:27Z [delta:starred-batch]

kind: move
from: shaving
to: shorn
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
