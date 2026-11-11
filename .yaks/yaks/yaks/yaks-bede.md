---
id: yaks-bede
title: Links to /doks/(sessions|sources)/... don't seem to work.
status: hairy
type: bug
priority: 3
created: '2026-11-11T14:23:17Z'
updated: '2026-11-11T21:19:41Z'
parent: yaks-1151
tags:
- tui
---

Eg:
- [dok](doks/design/knowledge-store.md) works.
- [session](/doks/sessions/2026/11/herd-stars-2026-11-07-2025-starred-batch-13-yaks-shorn-with-sonnet.md) does not.
- nor [source](/doks/sources/2026-11-03-81364278).

Likely because the filesystem layout is:
- .yaks/
	- [herd]/
		- docs?
		- sessions?
		- sources?
		- yaks/
	- doks/
  - sessions/
  - sources/

I can't recall whether we decided to embed docs, et al optionally inside a herd, or just globally.
