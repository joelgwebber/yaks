---
id: yaks-87d3
title: 'source add --refresh: update a source in place, keyed by its --url'
status: shaving
type: feature
priority: 2
created: '2026-11-06T01:23:16Z'
updated: '2026-11-06T01:32:34Z'
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

---
▸ 2026-11-06T01:23:39Z [source-refresh]
kind: move
from: hairy
to: shaving
thread: ksQQcsYXd-T6RPSH25f_4VJRspLOAETlkJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T01:23:40Z [source-refresh]

Plan, from reading [sources.rs](code:src/sources.rs):

- `Request` gains `refresh: bool`; `Saved` gains `Refreshed(dir)` (unchanged content is the existing `Already`).
- `add` looks the source up by `resource == url` first when refreshing. Found and different: rewrite its payload, `extracted.md` and `source.md` in the same directory and keep id, title and `part_of`; record `previous_sha256`.
- `find` (dedupe by sha) matched on the id's `-<sha8>` suffix, which a refreshed source no longer has. It now checks each `source.md`'s `sha256:`, so "saving the same content again returns the existing source" stays true after a refresh.
- Tests first: refresh updates in place, unchanged is a no-op, no match creates, `--refresh` without `--url` is refused, a plain `add` of a refreshed source's current bytes finds it.

---
▸ 2026-11-06T01:32:34Z [source-refresh]

Done and tested.

- [add](code:src/sources.rs): `refresh` on `Request`, `Saved::Refreshed`, URL lookup, `previous_sha256`, title and `part_of` kept; unchanged content is `Already`, no match is a new source, no `--url` or an ambiguous URL is refused.
- [find](code:src/sources.rs): dedupe now reads `sha256:` from each `source.md` instead of trusting the id's suffix, which a refreshed source no longer matches.
- Help text, `docs/cli.md`, the `yaks` and `yaks-research` skills updated in the same change.

Evidence: four new unit tests (refresh in place, plain add never refreshes, `--refresh` needs `--url`, ambiguous URL), a scripted run of the built binary (Saved, Already saved, Refreshed, Already saved), and the full `cargo test` green (964 unit tests plus the integration suites).
