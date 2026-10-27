---
id: yaks-c7ba
title: Global search affordance
status: hairy
type: feature
priority: 2
created: '2026-10-27T13:23:15Z'
updated: '2026-10-27T13:23:15Z'
labels:
- tui
---

Pretty sure this is already supported in the CLI (but please double-check). But AFAICT in the tui we don't actually have a way to search globally. We have per-view _filters_ we can edit, and I often go to "Recent" to do a pseudo-global search, but this is not ideal, and I honestly don't even know whether it's truly global, or clipped, in that view.

We'll need to figure out what the right affordance is for this. `/` is already taken by the "quick" part of per-view filters, but maybe we should commandeer that.

We also need to decide whether the search results fit naturally into views like any other, and whether they can be saved as filters. It *may* already be the case that "filtering" an existing view and removing its constraints does what we'd want, in which case this may be a very small lift.
