---
id: yaks-37da
title: 'Schema 7: labels become tags; wiki, sources and schemas reserved'
status: hairy
type: feature
priority: 1
created: '2026-11-01T20:09:15Z'
updated: '2026-11-01T20:09:15Z'
parent: yaks-f301
labels:
- wiki
- schema
---

Implements the schema-7 format change decided on yaks-aae6 and yaks-8384, in one migration so farms migrate once:

- `labels:` becomes `tags:` in every yak; `Task.labels` becomes `tags`; CLI `--tags`/`--tag`/`--add-tag`/`--remove-tag` with the `--label*` forms as hidden aliases; JSON key `tags`; TUI and docs say tags.
- `wiki`, `sources` and `schemas` become reserved herd names.
- `SCHEMA = 7`; `yaks migrate` rewrites the key (a fixup), refusing a farm with a herd named like a reserved dir.
