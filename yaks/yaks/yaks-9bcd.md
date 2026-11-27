---
id: yaks-9bcd
title: 'Derived per-user index: postings, link edges, fields, modified times (prototype storage first)'
status: hairy
type: task
priority: 2
created: '2026-11-27T02:38:24Z'
updated: '2026-11-27T02:39:09Z'
parent: yaks-7e32
depends_on:
- yaks-8fb0
- yaks-0ef9
tags:
- perf
---

Section 9 of [the design](/doks/design/one-document-model.md#9-the-index-and-modified-time): one derived per-user cache (positional postings, link edges, parsed fields, modified times from git with mechanical commits skipped). Start with the prototype: tantivy against SQLite FTS5 over the whole job, measuring a cold-cache build of this farm and a synthetic 10k one; tantivy only if it needs no second store (Joel, 2026-11-27). Takes over yaks-8f50; needs a per-user cache home (yaks-0ef9).
