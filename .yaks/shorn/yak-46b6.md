---
id: yak-46b6
title: User comment affordance
type: feature
priority: 2
created: '2026-04-08T13:53:29Z'
updated: '2026-04-09T21:36:20Z'
commit: 4b2764c
---

It would be useful to add a "comment" feature that just adds a comment to the end of a yak's
description, without opening the entire yak for editing. It could use the same structure as the
existing "note" in the skills API (I think that's the right one we have agents using?). We can use
these affordances to loosely enforce structure, which we can then pull from to build feedback loops
(e.g., where the agent looks for new user comments to act on).
