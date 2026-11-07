---
id: yaks-bdc3
title: 'Ticks on two machines: one regrow per occurrence after a merge'
status: hairy
type: bug
priority: 3
created: '2026-11-07T16:57:33Z'
updated: '2026-11-07T16:57:33Z'
parent: yaks-44b3
tags:
- git
---

Hearth will run `yaks tick` every minute on its server (hearth-cfac in the Hearth farm), while a person may also run `yaks tick` on a laptop against the same farm. If both regrow one recurring yak for the same occurrence before syncing, the merge driver unions the notes, so the yak carries two `kind: regrow` notes and two `asked`/`requested` notes for one round. [The scheduling design](/doks/design/scheduling.md) already lists this two-machine case as needing a test before `tick --sync`.

Proposal: the merge driver keeps one `kind: regrow` note per `occurrence:` value, and one wake note following it, when both sides added them; the later duplicate is dropped. A test: two clones tick the same yak, sync both, and the yak reads as one regrow.
