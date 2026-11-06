---
id: yaks-87d3
title: 'source add --refresh: update a source in place, keyed by its --url'
status: hairy
type: feature
priority: 2
created: '2026-11-06T01:23:16Z'
updated: '2026-11-06T01:23:16Z'
tags:
- doks
- cli
---

Today a source's identity is its content: id `<date>-<sha8>`, and `source add` dedupes by sha. That suits a snapshot, not a living outside document (a Google Doc, a wiki page): every edit is a new sha and so a second source, and every dok citing the old one goes stale.

Add an opt-in `yaks source add <path> --url <U> --refresh`: an upsert keyed on the URL.

- A source whose first `sources:` entry has `resource: <U>` is refreshed in place: same directory and id, new original, `extracted.md`, `sha256`, `size`, `retrieved`, and the old hash kept as `previous_sha256`. Title and `part_of` are kept unless given.
- Same content: reports unchanged and writes nothing.
- No such source: saved as a new one, so one command both creates and refreshes.
- Needs `--url`. More than one source with that URL is an error.

yaks stays ignorant of what the URL is: fetching, converting and canonicalising belong to skills (the first is a Google Docs one via `gws`, fullstory-5cee).

Without `--refresh` nothing changes: a saved web page you cited does not move under you.
