---
id: yaks-23c9
title: One query tree and one matcher for search, filters, find, bulk and the TUI
status: hairy
type: feature
priority: 2
created: '2026-11-27T02:38:23Z'
updated: '2026-11-27T02:38:48Z'
parent: yaks-7e32
depends_on:
- yaks-8fb0
tags:
- cli
- search
---

Section 8 of [the design](/doks/design/one-document-model.md#8-queries-and-search): one query tree (text terms, phrases, `-word`, field scopes, `--where` operators, relations ready/tangled/parent-of) and one matcher (NFC, case folding, accent folding with offset maps via `unicode-normalization` / `unicode-segmentation`; identifier-aware tokens). Behind `--search`, `find`, `list --kind/--where`, `bulk` on any kind, `dok query`, the TUI `/`, pickers and completion. A dotted `--where` path is an error (from yaks-dca5). Absorbs yaks-71c2. Runs by scanning until the index lands.
