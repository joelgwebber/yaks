---
id: yaks-49c5
title: 'Sources: a readable slug in the path beside the date and hash?'
status: hairy
type: idea
priority: 4
created: '2026-11-14T17:52:03Z'
updated: '2026-11-14T17:52:03Z'
parent: yaks-1151
tags:
- links
- doks
---

From Joel's answer on yaks-5f70: while the farms are small, should paths carry a readable slug as well as the date and hash? Session notes already do (`sessions/YYYY/MM/<actor>-<day>-<hhmm>-<slug>.md`). Sources do not: `sources/<date>-<sha8>/` names a snapshot by date and content hash only, so a source link in prose or a GitHub tree listing says nothing about it.

## Options

- Leave as is: the id is stable and content-derived, and `source.md` carries the title.
- `sources/<date>-<sha8>-<slug>/`, the slug from `--title`. Readable on GitHub. Costs: the directory name is no longer derivable from the content, so the same bytes saved with a different title is a rename, `yaks source add` dedup by hash has to match on the prefix, `--refresh` keeps the id so the slug can go stale, and every link to a source (214 sources in this farm) changes in the migration.
- A slug in `source.md`'s title only (what is there now).

Lean: decide with the schema-9 migration, since it rewrites links anyway; if yes, do it in that same pass. Needs an ask when picked up.
