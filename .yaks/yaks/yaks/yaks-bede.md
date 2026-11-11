---
id: yaks-bede
title: Links to /doks/(sessions|sources)/... don't seem to work.
status: hairy
type: bug
priority: 3
created: '2026-11-11T14:23:17Z'
updated: '2026-11-11T22:39:29Z'
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

---
▸ 2026-11-11T22:39:29Z [triage-1]

Traced under yaks-1151 ([plan](/doks/design/farm-layout-and-links.md#5-sessions-and-sources-the-doksessions-bug-yaks-bede)): the premise is wrong, sessions and sources are siblings of `doks/`, so `/sessions/...` already works and `/doks/sessions/...` is a wrong path. The real bugs:

- `/sources/<id>` names a directory and neither `yaks peek` nor the TUI viewer opens its `source.md`.
- `yaks doctor` skips extensionless links (`links::exists`), so a broken `/sources/<id>` is never reported.

Fix those two (small, independent of the layout work) and correct the one link here.
