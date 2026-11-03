---
id: yaks-c5d7
title: Absolute document/image refs not resolving
status: shaving
type: task
priority: 3
created: '2026-11-03T04:06:14Z'
updated: '2026-11-03T21:21:26Z'
parent: yaks-f677
tags:
- tui
---

For example:
> /sources/2026-10-26-cc589301/image-28.png: no such attachment

---
▸ 2026-11-03T21:17:23Z [delta:starred-batch]
kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T21:21:26Z [delta:starred-batch]

Fixed with yaks-91d9: a markdown image is now classified by the link grammar like any link. A farm path (`/sources/<id>/image-28.png`, or page-relative in a page) is a farm document; it opens in the image viewer and is drawn in place when alone. Before, every image was an "attachment", so `/sources/...` failed with "no such attachment".

In a yak a relative image is still an attachment path, as before.

Evidence: the yaks-91d9 test covers `/sources/...`; the attached frame there is this exact image from the report.
