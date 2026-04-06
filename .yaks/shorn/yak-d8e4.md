---
id: yak-d8e4
title: In hairy/shaving list view, push shorn baby yaks to the bottom
type: task
priority: 2
created: '2026-04-06T23:08:58Z'
updated: '2026-04-06T23:29:26Z'
commit: a31a608
---

We display baby yaks in list view that are in a different state than the parent, so that you can see
all a yak's children. When this happens within hairy/shaving views, we should drop shorn yaks to the
bottom. The net effect is that, e.g., if you are looking at a parent being shaved, and you see a
mixture of hairy/shaving/shorn children, the children remaining to be done bubble up to the top.
This sorting should take precedent over the priority, so that "done p1" things still drop to the
bottom.
