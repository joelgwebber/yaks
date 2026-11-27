---
id: yaks-362d
title: Kind registry, schemas for every kind, one farm-wide tag vocabulary
status: hairy
type: feature
priority: 2
created: '2026-11-27T02:38:24Z'
updated: '2026-11-27T02:38:49Z'
parent: yaks-7e32
depends_on:
- yaks-8fb0
tags:
- doks
---

Sections 2 and 6 of [the design](/doks/design/one-document-model.md#2-principles): kinds registered in Rust (yak, dok, session, source, release) with path rule, identity, capabilities and default views; a field with behaviour on a kind without the capability warns, never silently inert. Schemas for every kind (built-in yak fields documented; session and source schemas applied to their files). One farm-wide tag vocabulary (its own schema file, descriptions for agents), normalized and warned for every kind, one display style (closes yaks-bd59), `yaks tags` to print it.
