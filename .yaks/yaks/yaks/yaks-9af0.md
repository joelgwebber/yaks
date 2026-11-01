---
id: yaks-9af0
title: 'Pages: schema-7 layout, page discovery, yaks page list/show/new/set/query, doctor'
status: hairy
type: feature
priority: 1
created: '2026-11-01T18:53:10Z'
updated: '2026-11-01T20:09:16Z'
parent: yaks-f301
depends_on:
- yaks-1f34
- yaks-37da
labels:
- wiki
- cli
---

Implements yaks-aae6 on yaks-1f34. Reserved names `wiki`, `sources`, `schemas`; `wiki/**.md` and `<herd>/wiki/**.md` are pages; `yaks page list|show|new|set|query` with `--json`; doctor checks page frontmatter. Docs, help and skills in the same change.
