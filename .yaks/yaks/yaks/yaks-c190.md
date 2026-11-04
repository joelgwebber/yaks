---
id: yaks-c190
title: Tidy the Surveyor API sources in joel into one
status: shorn
type: chore
priority: 4
created: '2026-11-04T17:23:24Z'
updated: '2026-11-04T17:29:23Z'
parent: yaks-dedc
depends_on:
- yaks-d19c
tags:
- wiki
---

In the `joel` farm, `sources/2026-11-03-bea6d0de` (Confluence HTML) and `sources/2026-11-03-14a63718` (its markdown conversion, hand-fixed in `extracted.md`) are the case that started yaks-dedc.

Once the tool fix is in (needs `--text`), replace them with one source: the markdown as `extracted.md`, `--url` the Confluence page, title "Surveyor service: API (Confluence, draft v10)". Drop the raw HTML. Check first what cites them (`yaks refs sources/<id>/source.md --incoming` on each, and `yaks find` for the ids) and repoint those; this touches Joel's farm content, so say what was changed in a note. Never edit sources in place otherwise: they are write-once, so this is a new source and a removal, not a rewrite.

---
▸ 2026-11-04T17:29:06Z [source-fix]

kind: move
from: hairy
to: shaving
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-04T17:29:23Z [source-fix]

Done in the `joel` farm. Nothing cited either old source (only `part_of` linked them to each other). Saved the hand-fixed markdown once with `--url` and the Confluence title as `sources/2026-11-04-491a47d1` (no HTML, no temp paths), then removed `2026-11-03-bea6d0de` and `2026-11-03-14a63718` (farm commit 36cc494; not synced). `yaks check` now reports the new source as uncited, as the old ones were.

---
▸ 2026-11-04T17:29:23Z [source-fix]

kind: move
from: shaving
to: shorn
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
