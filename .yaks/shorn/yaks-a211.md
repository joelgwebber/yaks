---
id: yaks-a211
title: Detail page height counts the sticky title row, so Tab can put the cursor one row below the viewport
type: bug
priority: 3
created: '2026-09-23T21:43:01Z'
updated: '2026-10-02T14:19:11Z'
labels:
- ui
---

Found by the yaks-28b4 lane: detail_page = h-3 counts the pinned/sticky title row as a body row, so scroll_line_into_view lets the cursor sit one row past the visible area (Tab to a link near the bottom). Frame in yaks-28b4's artifact navpos-back-forward-frames.txt (the 'A' frame).

---
▸ 2026-10-02T14:19:11Z [claude]
Fixed by yaks-15c7 (same off-by-one: pinned header row not subtracted from the viewport).
