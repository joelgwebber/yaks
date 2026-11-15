---
id: yaks-acc0
title: 'Schema 9 migration: new layout, canonical links, note format, README, stub marker'
status: shaving
type: feature
priority: 2
created: '2026-11-14T11:21:02Z'
updated: '2026-11-15T06:23:54Z'
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

---
▸ 2026-11-15T05:17:45Z [linkplan-lead]

Finding before any code (checked in a scratch farm with my own build): **yaks does not read its own canonical yak link.** With `[t](/wid/yaks/wid-74a1.md)` in a yak body, `yaks refs wid-74a1 --incoming` shows no reference (the bare `wid-74a1` form is found), `yaks doctor` reports `![x](/wid/yaks/wid-74a1/shot.png)` as a broken link, and a slug note anchor on that path is not resolved. `links::parse` turns a path starting with `/` into a plain farm file and only recognises a bare id as a yak. If the migration rewrote 453 links to the canonical form first, every farm would lose its backrefs and `doctor` would flag them.

So the order is fixed: (1) teach `links::parse` that `/<herd>/yaks/<id>.md[#note]` is a yak and `/<herd>/yaks/<id>/<file>` is that yak's attachment, with tests; (2) only then the rewrite.

Also settled while reading the code, for the record:

- **Do not run `cargo install` once `store::SCHEMA` is 9.** The installed `yaks` would refuse every schema-8 farm on this machine, Joel's included, until each is migrated. The installed build stays the pre-bump one until Joel picks the moment.
- **Frontmatter references to sources** (`sources:` lists, `part_of:`, `superseded_by:`) are not rewritten: they are not clickable on GitHub and resolve through the `<date>-<sha8>` prefix lookup that landed with yaks-7729.
- **Attachment links:** an own-yak `<id>/<file>` link in a yak body already resolves on GitHub (the file sits beside the directory) and is left alone; the same link inside an attachment becomes the bare sibling name, and one from a dok or another yak becomes `/<herd>/yaks/<id>/<file>`.
- **Unpinned `code:`** stays as it is; a pinned `code:` and a legacy `../blob/<sha>/...` become the host URL when the code repo is a recognised host; a legacy unpinned `../src/x` becomes `code:src/x`. If a farm has such links and no recognised host or no code checkout, the migration refuses and says why, rather than guess.

---
▸ 2026-11-15T05:26:14Z [linkplan-lead]

Progress, `8fcd2b0d`: yaks now reads its own canonical yak link. `/<herd>/yaks/<id>.md[#note]` is a yak and `/<herd>/yaks/<id>/<file>` is its attachment, only when the id is a real yak whose herd is the directory's (a lookalike path stays a plain farm file). Checked end to end in a scratch farm: `refs --incoming` finds the backref, `doctor` accepts a real attachment at that path, `peek` reads a note by timestamp and by slug. 1226 tests pass. This had to come first, or the rewrite below would have made every backref vanish.

Build order for the rest (`migrate.rs` stays the home; each step is a pure function with its own test, run by `run` under the existing lock, with the schema marker written last):

1. `FIXUPS`: the link rewrite (`links::rewrite`, so code spans and fences are untouched), then `store::convert_notes`.
2. Source rename (`sources::plan_slugs`, refusing on `duplicate_ids`) and the links and `sources:` entries to it (`sources::rewrite_ref`).
3. Layout: only for a farm branch, only when clean and not behind its upstream. `git mv` to the branch root, rewrite `.gitattributes` and the LFS rules, README, stub `.yaks/schema`.
4. `store::SCHEMA` to 9, the README text, the report, then the rehearsals.

Bumping `SCHEMA` is the last change I make, after everything else is tested, because it makes the build I install refuse every schema-8 farm on this machine until it is migrated.

---
▸ 2026-11-15T05:31:38Z [linkplan-lead]

Decision (scope change, coordinator, within what Joel allowed on yaks-6cf2): **the migration does not rewrite `code:` links.** Pinned `code:` to a host URL needs yaks to parse its own host's URLs back into code links, which it cannot: `links::parse` is pure and does not know the remote, so a rewritten link would open in the browser instead of the new viewer, and `yaks check` would stop seeing it. Rewriting first would regress what just landed. Filed as its own yak under yaks-1151 (unify the resolution, then rewrite); until then `code:` and the legacy `../` forms stay as written, plain text on GitHub exactly as today. Everything else in the migration is unaffected: yak links, attachment links, `/doks/sessions|sources/` paths, sources, notes, layout.

This is a reduction of decision 4, so Joel: say so on yaks-1151 if you would rather have the URLs now and the parse-back in the same change; I would not, it makes the migration depend on a resolver change it does not otherwise need.

---
▸ 2026-11-15T06:12:47Z [linkplan-lead]

Hand rehearsal of the layout move, on a `git clone --no-hardlinks` of this farm's branch in `/tmp` (2,246 commits, 87 MB `.git`, 1,292 files under `.yaks/`). No code involved: `git mv .yaks/* .` and `git mv .yaks/.gitignore .gitignore`, one commit. What git did:

- **The move is clean.** The index showed all 1,292 paths as `R` (rename). `git show -M --name-status HEAD` reported 1,292 `R` and nothing else, both at the default rename limit and with `-c diff.renameLimit=100`. The audit's worry that 1,240 renames would exceed the rename limit does not bite: the limit applies to guessing renames between *changed* files, and these are all exact (100%) matches, which git pairs by blob id first. No `diff.renameLimit` setting is needed.
- **History follows.** `git log --follow` on a yak across the move gave the same 20 commits as the plain log at the old path. For a dok it gave the plain log's commits plus the one that created it under its old `wiki/` name (`ff70dba1`), and it dropped two commits the plain log shows (`0f67b22c`, `95fba654`: a multi-file commit and its undo, where `--follow` picked a different rename source). So `--follow` is not a strict superset, it is git's best guess; the plain log at the old path stays complete, and nothing was lost.

A late writer (old layout, never synced after the migration) adds one new yak, one new dok directory and edits an existing yak, then meets the migrated branch:

| How it meets it | Edit to an existing yak | New yak in a renamed directory | New file in a new directory |
|---|---|---|---|
| `git rebase migrated` (what `yaks sync` does today) | follows the rename | **conflict** (`UA`) | left under `.yaks/` |
| `git merge migrated` with `merge.directoryRenames=true` | follows the rename | follows the rename, lands in `yaks/yaks/` | left under `.yaks/doks/newdir/` |

So the late-writer rule is: after the migration, a writer on an older yaks is refused by the stub marker anyway; one who already has uncommitted or unpushed work should `yaks sync` and push *before* the migration, which is what the refusal below enforces. A stray under `.yaks/` after a merge is what the strays step in a re-run moves into place. Whether `yaks sync`'s pull should merge rather than rebase in that one situation is a separate decision I am not making here.

Consequence for the code: the layout stage can be a plain `git mv` of the tracked paths plus the untracked ones, in one commit; no rename-limit configuration, no history rewrite.

---
▸ 2026-11-15T06:19:30Z [linkplan-lead]

Interaction found while reading the code, before it bites: `mergedriver::ensure_registered` and `weight::lfs_rules` write the `.yaks/**` rules unconditionally, and `ensure_registered` runs on every `yaks sync` (and on `yaks undo`). After the layout move, a plain `yaks sync` would append `.yaks/**/*.md merge=yaks` back beside the moved `**/*.md merge=yaks`, and `yaks lfs` would write the old LFS rules again (it checks `have.contains(rule)`, which the moved rules would no longer satisfy). The migration would be undone one sync later, silently.

So both become layout-aware: the rule prefix is `.yaks/` when the farm sits under `.yaks/` (`FarmGit::prefix`) and empty at the branch root, taken from the same source as every other path conversion. `ensure_registered` also treats a rule in either form as present, so a farm mid-migration is not given a second one. Tests for both layouts, for each.

---
▸ 2026-11-15T06:23:54Z [linkplan-lead]

Progress, committed locally and not landed (it only changes behaviour once `SCHEMA` is 9, which I have not touched):

- `migrate::canonical_links` (yak links, attachment links, `/doks/sessions|sources/` paths, source directory renames), with the rule that a path already from the root is left alone, 6 tests including idempotence.
- `links::slug_note_anchor`.
- `migrate::move_attribute_rules` and `lfs_marker_text`.
- `mergedriver::attr_line(prefix)` and `weight::lfs_rules(prefix)`: both now follow the farm's layout (`""` at the branch root, `".yaks/"` under it), and `ensure_registered` treats a rule in either form as present. Tests for both layouts, and one asserting `lfs_rules("")` is exactly what the migration's rewrite of the old rules produces.

Shape of `run` after this, so review can check the code against it:

1. **Plan, read-only** (this is the dry run, and the apply runs the same plan): layout moves of schemas 6 and 8 as today; `sources::plan_slugs` (refuse on `duplicate_ids`); per yak and per dok, session and attachment document, the link rewrite, and for yaks the note conversion; the `.gitattributes` rewrite; the layout stage for a farm branch.
2. **Refusals first**, before any write: farm branch with uncommitted changes, behind or diverged from its upstream, a merge in progress, `YAKS_DIR` outside git (content stage only, said so).
3. **Apply under the farm lock**: content writes, source renames, then for a farm branch the `git mv`, `.gitattributes`, `.gitignore`, README, stub `.yaks/schema`, the `schema` marker last, and ONE commit with the `Yaks-Actor` trailer, made by `migrate` with `git commit --only` the way `yaks commit` does. Not the per-change autocommit: its snapshot is keyed to a root that moves under it.
4. **Report** with per-form counts, left-alone bodies, unresolvable links, source renames, and the commit.
