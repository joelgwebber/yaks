---
id: yaks-2c42
title: 'Sources: yaks source add (file, dir, --url, --part-of), sha256 dedup, yaks promote from attachments'
status: hairy
type: feature
priority: 2
created: '2026-11-01T18:53:11Z'
updated: '2026-11-01T18:53:19Z'
parent: yaks-f301
depends_on:
- yaks-9af0
labels:
- wiki
- cli
---

Implements the sources half of yaks-aae6. Port of Hearth's local-file and directory snapshots (`internal/sources`), minus fetching and extraction. `yaks promote <id>/<file> --source|--page PATH` moves an attachment and rewrites the yak's link.
