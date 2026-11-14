---
id: yaks-acc0
title: 'Schema 9 migration: new layout, canonical links, note format, README, stub marker'
status: hairy
type: feature
priority: 2
created: '2026-11-14T11:21:02Z'
updated: '2026-11-14T23:35:38Z'
parent: yaks-1151
depends_on:
- yaks-6ffd
- yaks-f505
- yaks-6c65
- yaks-9f00
- yaks-7729
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

---
▸ 2026-11-14T11:21:18Z [linkplan-lead]

Dependencies: layout [yaks-6ffd](yaks-6ffd), resolver [yaks-f505](yaks-f505), note format [yaks-6c65](yaks-6c65). Docs follow in [yaks-df4e](yaks-df4e).

---
▸ 2026-11-14T22:38:06Z [linkplan-lead]

New dependency: [yaks-7729](yaks-7729) (source directory slugs, decided on [yaks-49c5](yaks-49c5)). The migration also renames each `sources/<date>-<sha8>/` to the slugged name with `git mv` and rewrites links and `sources:` entries to it, using `sources::plan_slugs` and the link-rewrite function that yak ships. Source files are snapshots: link text inside a source's own `extracted.md` is never rewritten.

---
▸ 2026-11-14T23:35:38Z [linkplan-lead]

Coordinator check after landing `7585597e`: I ran the worker's real-yak round trip over EVERY yak of four farms on this machine (`YAKS_NOTES_SAMPLE=<farm> YAKS_NOTES_LIMIT=100000`), not the first 200.

| Farm | Yaks | Converted | Left alone | Notes |
|---|---|---|---|---|
| this one | 762 | 702 | 0 | 3315 |
| hearth | 113 | 104 | 0 | 441 |
| schism | 145 | 48 | 0 | 155 |
| joel | 682 | 357 | 50 | 1051 |

Every parse was identical before and after conversion, the conversion was idempotent, and the anchors resolved in both shapes.

**The joel farm failed the worker's test, for a real reason.** 50 yaks (117 notes) there have hand-written note headers that are not timestamps (`▸ 2026-08-12 E2E VERIFIED (live headless Chrome...)`, `▸ 2026-08-13 SHORN — landed and released.`, in the `sightmap-*` yaks). The new heading is `#### <timestamp>` and cannot carry such a header, so `convert_notes` correctly returns those bodies unchanged. I fixed the test (commit after `7585597e`) to expect that and report the count; the other seven farms I scanned have none.

**Consequence for the migration (yaks-acc0), a decision for Joel:** after a schema-9 migration those 50 yaks stay in the old shape, so their notes still have no anchor on GitHub and still render as before (the reader and the writer both keep a body in the shape it has). Options, for the migration yak to put to Joel:

- Leave them. They are one farm's hand-written entries; the cost is that this farm's 50 yaks render the old way on GitHub.
- Rewrite each such header into a timestamp with the date as written at 00:00:00Z and the rest of the line moved into the note's first line, so no text is lost. A mechanical, reviewable change, but it edits what a person wrote.

Lean: leave them, and list them in the migration report so nobody is surprised.
