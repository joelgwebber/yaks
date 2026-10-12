---
id: yaks-8b7a
title: 'Decision: status as a frontmatter field (one stable path per yak) vs status directories'
type: task
priority: 1
created: '2026-10-12T16:05:28Z'
updated: '2026-10-12T19:53:34Z'
labels:
- design
- cli
---

Joel (2026-10-12): revisit, hopefully for the last time, whether a yak's status should stay implicit in its directory (`.yaks/{hairy,shaving,shorn,dead}/<id>.md`) or move into the frontmatter with one stable path per yak. Decide before the yaks-38dc TUI work, because the cross-shed code (src/sheds.rs base_farm/yak_changes, src/changes.rs classify) would be rewritten by it.

## Evidence the directories cost us (trawled 2026-10-12)
- Staging misses new paths: `git add -u` after a status move stages only the deletion. Hit twice in one session (yaks-ad6d claim, fixed in 5070350; yaks-ed5e claim b721f72, which blocked worker land-1). `yaks path` (yaks-aa49) exists because of it, and the skills recommend `':(glob).yaks/*/<id>.md'` pathspecs.
- A yak in two status dirs at once: its own integrity class. `yaks doctor` (yaks-39d4) and `yaks preflight` (yaks-06be) check for it; seen in yaks-df61 O22 (a double landing) and again this session (yaks-ad6d's landing left a stale `shaving/` copy).
- Merges: a status move on one side plus an appended note on the other is a modify/delete conflict, the hardest kind for git. With a status line in the frontmatter, the two edits sit in different hunks and merge cleanly. Two sides creating the same moved path are add/add conflicts (three of them this session, in .yaks only).
- Diffs and history: every transition is a delete plus an add, so moves are reconstructed by pairing D and A per id (src/changes.rs classify) and provenance follows the file with `git log --follow` (yaks-213b), which is similarity-based and can miss.
- Links drift when a yak moves (yaks-248d).
- Cross-shed comparison materializes a base farm by parsing `ls-tree` paths per status dir (src/sheds.rs base_farm); with one path per yak, "this yak in that shed" is a single blob lookup.

## What the directories buy
- Status is visible at a glance in any file browser (Zed's and Delta's tree), and a move is a plain `mv`.
- Queries can skip a status by not reading its directory. Measured 2026-10-12 on this farm (393 yaks): `yaks list` and `yaks list --all` both take ~0.08 s, and default queries already read hairy+shaving+shorn (94% of the files), so the dirs only prune `dead/` and explicit `--status` filters. A large farm needs an index (yaks-8f50) either way.

## Costs of changing
- Schema bump (the farm already carries `schema: 3` and refuses a newer one), a migration command, and old binaries erroring on a migrated farm: a worker brief that names a 0.0.15 `yaks` would fail loudly, not corrupt anything.
- Sheds forked before the migration keep the old layout, so the cross-shed reader has to read both layouts for a while.
- About 30 `Status::dir()` call sites, the TUI, AGENTS.md's first invariant ("status is implicit from the directory"), docs and every skill.

## Options
A. Keep the directories; keep paying with tooling (path, doctor, preflight, commit).
B. Flat: one directory, `status:` in the frontmatter, dead included (one rule).
C. Hybrid: flat for hairy/shaving/shorn, keep `dead/` as the one subdirectory (slaughter/revive stay moves; dead stays out of the browse view).

---
▸ 2026-10-12T16:06:25Z [delta-lead]
More evidence, found while filing this: the third staging miss in two days, and the first to reach origin. yaks-7faf's shear commit b25a5b9 recorded only the deletion of shaving/yaks-7faf.md; the new shorn/ file stayed untracked, so origin/main had the release yak missing entirely until 5d2ba3f. yaks doctor did not catch it (a yak that is simply absent is not a two-dir duplicate) and preflight's git-state check only runs on demand.

---
▸ 2026-10-12T19:44:39Z [Joel Webber]
moved: hairy -> shaving

---
▸ 2026-10-12T19:53:34Z [delta-lead]
Decision (Joel, 2026-10-12): go FLAT, dead included, one rule; and do it now, before any of the yaks-38dc TUI work (then a thin Sheds tab first, then the overlay). "I'd rather have to back out of that due to some serious unforeseen problem, than take on the pain of a hybrid approach."

## Transition plan (coordinator; D1, D4 and D5 marked [confirm] need Joel's OK before code)

### Decisions
- D1 [confirm] Location: `.yaks/<id>.md`, directly in the farm root beside `config.yaml`, `schema` and `artifacts/`. Alternative: a subdirectory (`.yaks/yaks/<id>.md`). Root gives the shortest stable links (yaks-248d).
- D2 Field: `status: hairy|shaving|shorn|dead` in the frontmatter, written right after `title:`. A root file with no `status:` reads as hairy (like a missing `type:` reads as task).
- D3 Dead yaks live flat too; default queries already exclude them by status.
- D4 [confirm] Schema 4. The new binary READS both layouts, permanently (old commits, fork points and sheds created before a farm migrated keep the old layout), and WRITES only the flat one. On a v3 farm every write command refuses: `this farm is schema v3 (status directories); run 'yaks migrate' and commit the result`. No dual-write path to maintain. Old binaries already refuse a v4 farm (SchemaTooNew), loudly, before reading anything.
- D5 [confirm] `yaks migrate [--dry-run]`: explicit, takes the farm lock, refuses (listing them) when an id exists in both layouts, moves each `<status>/<id>.md` to `<id>.md` with `status:` inserted, removes the emptied status dirs, writes `schema` 4, and prints a one-line summary plus the commit to make. It never commits by itself, like every other command.
- D6 Status moves become an in-place edit of `status:`; the `moved: <from> -> <to>` note entry stays exactly as it is, so `yaks log` and attribution are unchanged.
- D7 git-side reporting (`yaks status`, `yaks commit` messages, src/changes.rs): a status change is a modified file whose `status:` changed. The old pairing of a deleted and an added path per id stays for history read across the migration, and the migration itself reports as one line (`farm layout migrated: N yaks`), not N moves.
- D8 Integrity: the "yak in two status dirs" check becomes "id in both layouts" (doctor, preflight); add "invalid status value".
- D9 `yaks path` stays; its docs stop saying a move relocates the file. The worker brief's finish step becomes `git add $(yaks path <id>)` alone.

### Transition hazards (the parts that need care)
- Migrate at a quiet point: no live worker sheds with yak changes (check `yaks sheds`). A shed forked before the migration that later moves a yak produces a rename/rename conflict on merge.
- Quit every running `yaks tui` (and any old binary) before migrating: an old process that opened the farm before the migration does not re-check the schema, would see empty status dirs, and a write from it would recreate `hairy/` beside the flat files. D8's doctor check catches the result.
- The migration commit renames every yak file plus a one-line add; git's rename detection follows it, so `yaks commits` (`git log --follow`) keeps the history.
- Other farms (~/src/j15r, private and pointer farms) migrate when Joel next writes in them with the new binary: the refusal message says how.

### Phases
1. Store seam (coordinator, prep commit; everything else builds on it): flat path helper, a loader that reads both layouts, find/save/move/load-by-id flat, schema 4, the write refusal on v3, init without status dirs; gate green.
2. Three parallel Sonnet workers on disjoint files, after phase 1 lands:
   - migrate: `yaks migrate`, doctor/preflight checks (D5, D8), their docs rows and AGENTS.md's first invariant.
   - git reporting: src/changes.rs, src/status.rs, src/commit.rs (D7), their docs rows.
   - sheds: src/sheds.rs base_farm reading both layouts from a git tree, cross-shed comparison between a migrated and an unmigrated checkout, scripts/discover-fixture.sh.
3. Coordinator: brief.rs and the skills/docs sweep (they must agree), migrate this repo's own farm in one commit, real-machine checks (the TUI looks unchanged in docshots; `yaks sheds` against a clone forked before the migration; a `--dry-run` on ~/src/j15r), then release 0.0.16.
