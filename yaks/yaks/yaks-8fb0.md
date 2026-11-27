---
id: yaks-8fb0
title: 'Document substrate: Task as a typed view over Fields; one parser and writer'
status: hairy
type: task
priority: 2
created: '2026-11-27T02:38:22Z'
updated: '2026-11-27T02:38:22Z'
parent: yaks-7e32
tags:
- doks
---

Section 6 of [the design](/doks/design/one-document-model.md#6-fields-schemas-and-tags): a `Document` (path, kind, `Fields`, body) as the one representation, with `Task` a typed view over its `Fields`. One parser, one writer, one quoting policy (`store::write::needs_quotes` vs `fields::plain_safe` today), one key order. The merge driver, JSON and filter read through it.

Gates: load and save every file of a copy of this farm with **zero diff** (no byte changes until schema 11); TUI startup bench at 10k yaks no slower (yaks-e61e). Builds on yaks-8384's model. Needs a design pass by the coordinator before delegating: the `Task` API surface and the fast path.
