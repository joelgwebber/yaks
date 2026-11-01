---
id: yaks-8e75
title: 'Schemas as data: schemas/<type>.md, warn-only checks, defaults, yak field extensions'
status: hairy
type: feature
priority: 2
created: '2026-11-01T18:53:10Z'
updated: '2026-11-01T18:53:19Z'
parent: yaks-f301
depends_on:
- yaks-1f34
labels:
- wiki
- schema
---

Implements yaks-8384: `schemas/<type>.md` (Hearth's format: `fields: {name: {type, description, default}}`), warnings from `page set`/`doctor`, defaults on `page new`, and `schemas/yak.md` extending the yak's built-in fields.
