---
id: yaks-48b6
title: 'Schema 11: sharding, growing ids, sources as links, prose mentions to links, updated retired'
status: hairy
type: task
priority: 2
created: '2026-11-27T02:38:26Z'
updated: '2026-11-27T02:38:54Z'
parent: yaks-7e32
depends_on:
- yaks-3b08
- yaks-8fb0
- yaks-da00
tags:
- git
---

Section 12 of [the design](/doks/design/one-document-model.md#12-migration-and-order-of-work): the one format change. Sharding by id cascading with id length (section 4, closes yaks-0f3a), ids growing with the herd, `sources:` as links, prose yak mentions converted to inline links and TUI autolinking dropped, the converged writer (quoting, key order), `updated:` retired, the mechanical-commit trailer. Rehearsed on a clone of this farm, applied here in place, other farms at the next release. Last; a human picks the moment (the coordinating skill's schema-bump order).
