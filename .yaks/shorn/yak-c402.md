---
id: yak-c402
title: Not all child yaks showing in list view
type: bug
priority: 2
created: '2026-04-06T23:20:36Z'
updated: '2026-04-06T23:29:25Z'
commit: a31a608
---

In some cases, baby yaks don't all show up in the list view. E.g., when you're in the hairy view,
have a parent being shaved, with children in other states, only the hairy children are displayed.
This may be a specific instance of a more general problem, but we should ensure that all child yaks
are shown in all cases, if the parent's being viewed.
