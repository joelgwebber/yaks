---
id: yaks-101d
title: 'Links: one resolver for yaks, farm paths, code: and @rev pins; refs and doctor over pages; yaks peek'
status: hairy
type: feature
priority: 1
created: '2026-11-01T18:53:11Z'
updated: '2026-11-01T18:53:19Z'
parent: yaks-f301
depends_on:
- yaks-9af0
labels:
- wiki
- links
---

Implements yaks-ccb1. A shared resolver used by refs, doctor, peek and the TUI; old `../src` and `../blob/<sha>` forms stay valid; `yaks refs` accepts a page path; `yaks peek <link>` prints the span; `yaks mv` moves a page and rewrites links to it.
