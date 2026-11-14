---
id: yaks-acc0
title: 'Schema 9 migration: new layout, canonical links, note format, README, stub marker'
status: hairy
type: feature
priority: 2
created: '2026-11-14T11:21:02Z'
updated: '2026-11-14T11:21:16Z'
parent: yaks-1151
depends_on:
- yaks-6ffd
tags:
- git
- links
---

The migration half of yaks-1151 (audit W4): one `yaks migrate` step from schema 8 to 9. A human runs it, never an agent; the order is the `yaks-coordinating` skill's schema-bump order.

## Does

- **Layout:** `git mv` everything under `.yaks/` to the branch root, in one commit. About 1,240 renames is over git's default rename limit, so rehearse on a clone and check `git show -M` and `git log --follow` (history followed the schema-6 move).
- **Strays:** move any `.yaks/**` file that arrives later (a yak from a writer who had not synced) into place. Everyone syncs before migrating.
- **Stub:** leave `.yaks/schema` holding `9`, so an old binary says "newer schema, upgrade" instead of "restore .yaks/".
- **`.gitattributes`:** rewrite the existing `.yaks/**` merge-driver and LFS rules (`mergedriver.rs`, `weight.rs`); `ensure_registered` only appends a missing line and would leave the old one.
- **Links (decisions 2 and 4):** rewrite bare yak ids and yak-note links to `/<herd>/yaks/<id>.md`, cross-yak attachments to `/<herd>/yaks/<id>/<file>`, pinned `code:` and legacy `../` forms to host URLs when the code repo's remote is a recognised host, and correct `/doks/sessions|sources/...` to the real path. Links that cannot be resolved (88 on this machine, mostly test fixtures and one farm's `../topics`) are reported, never guessed. The write side (`update --note`, `create`, the TUI editors, `attach`) canonicalises a bare id as it saves.
- **Notes:** convert notes to the new format from the note-format design yak, in the same pass.
- **README:** write the static landing `README.md` (layout and a pointer to yaks).
- Bump `store::SCHEMA` and `min_version`; readers keep accepting schema 8 shapes until migrated, as with every earlier schema.

## Depends on

The layout yak, the resolver yak and the note-format design. Do not start before the note format is decided. Rehearse on clones of this farm and the joel farm.

## Evidence

A rehearsal on this farm: `yaks doctor` clean before and after; the census's "resolves on GitHub" share before and after (script: `linkcensus.py` on yaks-0a4d); one real file checked on GitHub's rendering of a rehearsal branch pushed to a place the human names.
