---
id: yaks-b608
title: Filtering showing all yak states when herd is selected
status: shaving
type: bug
priority: 2
created: '2026-10-27T21:49:58Z'
updated: '2026-11-03T21:24:28Z'
parent: yaks-f677
tags:
- tui
---

See all the dead/shorn yaks here?

![too-many-yaks](yaks-b608/too-many-yaks.png)

But the filter shows only "hairy":

![filters-with-herd](yaks-b608/filters-with-herd.png)

---
▸ 2026-11-03T21:21:32Z [delta:starred-batch]
kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

[b608-before-hairy-herd-fullstory](yaks-b608/b608-before-hairy-herd-fullstory.txt)

[b608-after-hairy-herd-fullstory](yaks-b608/b608-after-hairy-herd-fullstory.txt)

---
▸ 2026-11-03T21:24:28Z [delta:starred-batch]

Fixed: a content filter (herd, search, type, tags, ...) narrowed the view's *anchors* to matches but searched the whole family universe for them, so a shorn or dead descendant whose herd or title matched came in bright, ignoring `status: hairy`. Now focus is the anchors that match: status stays an axis of the filter; family scope (`remaining`/`all`) still decides which descendants come along, dimmed.

- [fn build](code:src/tui/tree.rs#L68): the `universe` search is gone (it existed only for this).
- One snapshot changed on purpose: `/child` on the Hairy view no longer lists the shorn "Child A2 done".

Evidence: new test `a_content_filter_keeps_the_status_scope`; headless runs on the joel farm (Hairy view, filter drawer, herd `fullstory`), before and after attached: 59 rows with three shorn `🐑` rows -> 52 rows, none shorn. The one `💀` left is `fullstory-4b4c`, a dead parent rooting hairy children, dimmed as with no filter.
