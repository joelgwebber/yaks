---
id: yaks-fb15
title: 'source list: show part-of and citation status'
status: hairy
type: feature
priority: 3
created: '2026-11-07T15:58:16Z'
updated: '2026-11-07T15:58:16Z'
tags:
- cli
- doks
---

Found while ingesting sources in yaks-3610 (0.0.33).

`yaks source list` prints one flat row per source (id, media type, title, url). A source saved with `--part-of` looks the same as a top-level one, so I had to read each `source.md` to confirm my three manifesto screenshots hung under the tweet. `yaks check` knows about parts and about uncited sources, but `list` shows neither.

Add to the text output: indent a part under its parent, and mark uncited sources (and `uncited:` / `superseded_by:` marks). `--json` already carries `fields.part_of`.

Small and local: the `source list` arm and its listing function.
