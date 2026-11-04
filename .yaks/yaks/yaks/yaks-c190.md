---
id: yaks-c190
title: Tidy the Surveyor API sources in joel into one
status: hairy
type: chore
priority: 4
created: '2026-11-04T17:23:24Z'
updated: '2026-11-04T17:23:24Z'
parent: yaks-dedc
depends_on:
- yaks-d19c
tags:
- wiki
---

In the `joel` farm, `sources/2026-11-03-bea6d0de` (Confluence HTML) and `sources/2026-11-03-14a63718` (its markdown conversion, hand-fixed in `extracted.md`) are the case that started yaks-dedc.

Once the tool fix is in (needs `--text`), replace them with one source: the markdown as `extracted.md`, `--url` the Confluence page, title "Surveyor service: API (Confluence, draft v10)". Drop the raw HTML. Check first what cites them (`yaks refs sources/<id>/source.md --incoming` on each, and `yaks find` for the ids) and repoint those; this touches Joel's farm content, so say what was changed in a note. Never edit sources in place otherwise: they are write-once, so this is a new source and a removal, not a rewrite.
