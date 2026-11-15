---
id: yaks-acc0
title: 'Schema 9 migration: new layout, canonical links, note format, README, stub marker'
status: shaving
type: feature
priority: 2
created: '2026-11-14T11:21:02Z'
updated: '2026-11-15T05:13:33Z'
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

---
▸ 2026-11-15T05:01:02Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-15T05:13:33Z [linkplan-lead]

Design for the build, from reading `migrate.rs`, `store::lock`, `autocommit`, `commit.rs`, `mergedriver::ensure_registered` and `weight::enable_lfs` (no code yet).

## Shape

`yaks migrate` today is `migrate::run(root, dry_run)` over a farm *root* (a directory), writing files in place and the `schema` marker last, "never touches git". Schema 9 moves the root itself, so it is a second, git-aware stage after the in-place one:

1. **Content stage** (new fixups in `FIXUPS`, run in place on the old root, dry-run capable): yak ids and yak notes to `/<herd>/yaks/<id>.md`, cross-yak attachments to `/<herd>/yaks/<id>/<file>`, `/doks/sessions|sources/...` to the real path, pinned `code:` and legacy `../` forms to host URLs when the code repo's remote is a recognised host, `sources:` and `part_of:` references and links to the slugged source directory, notes to the new shape (`store::convert_notes`). Never inside a code fence or a source's own text.
2. **Source rename** (`sources::plan_slugs`, with `duplicate_ids` checked first): the directory renames, applied before the layout move so one `git mv` pass sees the final names.
3. **Layout stage** (git): `git mv .yaks/* .` on the farm branch's worktree, `.gitattributes` and LFS rules rewritten from `.yaks/**` to `**`, `.gitignore` carried over, a static `README.md`, a stub `.yaks/schema` holding `9`, then the `schema` marker. One migration commit holding all of it, made by `migrate` itself the way `yaks commit` does (`--only` the farm paths), not by the per-change autocommit, whose snapshot is keyed to a root that moves under it.

## Why git-aware, and the refusals

- Only a farm that is a farm branch (`store::farm_git`) is moved; a `YAKS_DIR` farm outside git keeps `.yaks/` (nothing to host) and gets the content stage only. Say so in the report.
- Refuse unless the farm worktree is clean and in sync with its upstream (the audit's "everyone pushes first": a stranded `.yaks/**` file from a late writer is the failure to avoid). `yaks migrate --dry-run` reports the same refusals without writing.
- A concurrent writer on an older yaks is refused after the migration by the stub marker (`newer schema, upgrade`); a late `.yaks/**` file from one is moved into place by a re-run (the strays step), not lost.

## Decisions I am taking, each reversible in review

- **Dry run is the report.** It lists, per farm: yaks to move, links rewritten by form, unresolvable links, sources renamed (and any slug collision), notes converted, bodies left in the old shape (the joel farm's 50 hand-headed yaks: a note header that is not a timestamp), and the layout refusals. Same numbers as the apply, so the rehearsal's before/after is the real thing.
- **Unresolvable links are reported, never guessed.** A link the old rules could not resolve stays as written.
- **Idempotent.** A re-run on a schema-9 farm is a no-op; a re-run after an interrupted content stage finishes (the marker is written last, as before).
- **Rehearse on clones only.** `git clone --no-hardlinks` the farm branch of this farm, joel, hearth and schism into `/tmp`, run it there, never on a live farm.

## Evidence I will gather before asking Joel to run it anywhere

Per rehearsal clone: `yaks doctor` clean before and after; the census share of links resolving on GitHub before and after (`linkcensus.py`); `git log --follow` over a sample of moved yaks and `git show -M --stat` for the rename detection; the stub marker refusing an older binary (the installed 0.0.36 build is the "older" one once SCHEMA is 9); one real file viewed on GitHub's rendering, which needs a branch pushed somewhere Joel names (not done without his say-so).
