---
id: yaks-0b10
title: Two machines can mint the same yak id; the merge driver then merges two yaks into one file
status: hairy
type: bug
priority: 2
created: '2026-11-27T02:09:09Z'
updated: '2026-11-27T02:09:09Z'
parent: yaks-7e32
tags:
- git
---

`store::generate_id` checks a random 16-bit id against local files only. Two machines that each create k yaks between syncs collide with probability about k²/65536 (0.6% at 20 each, 3.8% at 50). On a collision the merge driver sees both sides add `<id>.md`, uses an empty base ([mergedriver.rs](https://github.com/joelgwebber/yaks/blob/7c1452bd5712911a15ba5e244c9079af914a5c76/src/mergedriver.rs#L263-L285)), and merges two unrelated yaks into one file, with a title clash note.

Design (section 4 of the dok): `yaks sync` fetches, then before rebasing compares the yak files each side added since the merge base; the same path with a different `created:` is two yaks, and the local one is renamed through the rewrite engine in its own commit before the rebase. Safety net: the merge driver refuses to merge two added files whose `created:` differ and leaves a conflict. Longer ids (the growth rule) make it rarer, not impossible.
