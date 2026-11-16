---
id: yaks-bede
title: Links to /doks/(sessions|sources)/... don't seem to work.
status: shorn
type: bug
priority: 3
created: '2026-11-11T14:23:17Z'
updated: '2026-11-14T12:01:05Z'
parent: yaks-1151
tags:
- tui
---

Eg:
- [dok](doks/design/knowledge-store.md) works.
- a session at `/doks/sessions/2026/11/herd-stars-...md` does not (the real path is `/sessions/2026/11/...`).
- nor a source at `/doks/sources/2026-11-03-81364278` (the real path is `/sources/2026-11-03-81364278`, a directory that needs to open its `source.md`).

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

---
▸ 2026-11-14T12:01:04Z [linkplan-lead]

The two real bugs are fixed by [yaks-f505](yaks-f505) (landed `298b91ac`): `/sources/<id>` opens that source's `source.md` in `yaks peek`, `doctor` and the TUI viewer, and `doctor` now reports a broken extensionless `/sources/...` or `/doks/sessions/...` link (it reported the session link only, before). I re-ran `yaks doctor` here: it listed both wrong links in this yak's description, which I then corrected to the real paths. `doctor` is clean now. Layout questions stay with [yaks-1151](yaks-1151).

---
▸ 2026-11-14T12:01:05Z [linkplan-lead]
kind: move
from: hairy
to: shorn
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shorn
