---
id: yaks-8b7a
title: 'Decision: status as a frontmatter field (one stable path per yak) vs status directories'
status: shorn
type: task
priority: 1
created: '2026-10-12T16:05:28Z'
updated: '2026-10-12T23:01:57Z'
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

---
▸ 2026-10-12T20:00:38Z [delta-lead]

Amendments (Joel, 2026-10-12, on the plan above). These supersede D1, D4, D5 where they differ.
- D1 confirmed: `.yaks/<id>.md` in the farm root.
- D10 (new, Joel's idea, checked): attachments move from `.yaks/artifacts/<id>/` to `.yaks/<id>/`, beside the yak, in the SAME migration. Checked on this farm: 50 artifact dirs, none orphaned; 81 links in bodies, all of the one form `](<id>/<file>)`, so the rewrite to `](<id>/<file>)` is mechanical. Bonus: today those links are relative to the farm root, so an editor's markdown preview of `.yaks/shorn/x.md` resolves them to `.yaks/shorn/artifacts/...` and breaks; from `.yaks/<id>.md` the new form resolves in any viewer. Costs: about 90 `artifacts` references in src (attach, rename-attachment, merge, rename, preflight's untracked check, the git reporting's artifact classification, the TUI's link resolution, brief); the root scan skips directories; staging a yak is now `.yaks/<id>.md` plus `.yaks/<id>/` (`yaks path` should print both, or a pathspec `.yaks/<id>*`, safe here because no legacy dotted ids remain).
- D4 changed: the dual-layout read is TRANSITIONAL, not permanent. It exists so `yaks migrate` can read the old layout and so an unmigrated farm stays readable meanwhile.
- D5 confirmed, plus a cleanup: once Joel's farms are migrated, a phase 4 removes `yaks migrate`, the dual read, the legacy git classification and the both-layouts doctor check, and every transitional doc line. What stays for good: a schema-3 farm refuses to open with a message naming the last release that can migrate it, and `yaks sheds` reports a shed whose farm is schema 3 as an old-layout clone (not as one that deleted every yak).

---
▸ 2026-10-12T20:22:37Z [delta-lead]

Phase 2 respawn: mig-1/git-1/shed-1 were started on anthropic/claude-opus-5.5 (the worker profile's default changed; I did not pin a model though Joel asked for Sonnet) and were told to stop clean within a minute. Respawned as mig-2 (yaks-86f4), git-2 (yaks-9f91), shed-2 (yaks-ca28) on claude-sonnet-5-5. The claim notes name the -1 owners; the -2 workers replace them.

---
▸ 2026-10-12T20:23:52Z [delta-lead]

Spawned phase 2 (base 41b0053, model custom:LiteLLM/claude-sonnet-5-5 per the spawn confirmations): mig-2 76b2756b137644f6 -> yaks-86f4; git-2 4ba8b1c72a394316 -> yaks-9f91; shed-2 62c228b3013a476b -> yaks-ca28. Phase 1 (yaks-95be) landed as 79f00e9: gate 649 passed, 0 failed.

---
▸ 2026-10-12T20:36:54Z [delta-lead]

Landed yaks-9f91 (git-2 4ba8b1c72a394316): b9de271, merge by SHA, clean. Gate on this machine 16 suites, 656 passed, 0 failed, 3 ignored (the pre-existing three; phase 1's handed-off status test is un-ignored and passes). Real machine (temp flat farm): shave + note reads 'moved hairy -> shaving, notes +1' and yaks commit writes 'yaks: shaving <id>', status clean after. Waiting on mig-2 and shed-2.

[migrate-real-farm-copy](yaks-8b7a/migrate-real-farm-copy.txt)

---
▸ 2026-10-12T20:58:49Z [delta-lead]

Review of yaks-86f4 + yaks-9f91 together: yaks migrate on a COPY of this repo's real farm in a scratch git repo (never the real one).

---
▸ 2026-10-12T20:58:49Z [delta-lead]

Landed yaks-86f4 (mig-2 76b2756b137644f6): e2edbd9, merge by SHA (main.rs auto-merged with git-2's change; one unexplained comment edit in src/tui/detail.rs from Delta's applied state discarded, it was not in the worker's commit). Gate on this machine after both landings: 16 suites, 670 passed, 0 failed, 3 ignored (pre-existing).
Real-machine review (attached, migrate-real-farm-copy.txt): on a copy of this repo's farm, `migrate --dry-run` wrote nothing; `migrate` moved 404 yaks and 51 attachment dirs; `list --all` byte-identical before and after; `yaks status` one line `farm layout migrated: 404 yaks`; `doctor` clean; `yaks commit` made one commit of 485 renames (git detected them, `log --follow` crosses it); a write works afterwards. The one `](artifacts/` left is prose inside a note (backticked), correctly untouched.
Found and fixed in review (preflight.rs, my commit): `preflight` on a STAGED migration put every yak in the verify scope, so all 263 never-verified shorn yaks failed and the migration commit could not pass preflight. Fix: a yak whose only change is the migration (changes::classify's `migrated`) is not in the change's scope; explicit ids are still checked. New test fails without the fix, passes with it; preflight on the staged real-farm copy now prints `preflight: ok`.

---
▸ 2026-10-12T21:02:26Z [delta-lead]

Landed yaks-ca28 (shed-2 62c228b3013a476b): 3cc4712, merge by SHA, clean. Phase 2 complete. Gate on this machine with all three: 17 suites, 678 passed, 0 failed, 3 ignored (pre-existing). Real machine: the new binary's 'yaks sheds' on this machine tags the still-unmigrated farm '(old layout)' and compares the live sheds normally. Next: phase 3 (yaks-3a56).

---
▸ 2026-10-12T22:32:05Z [delta-lead]

Phase 4 started (yaks-bbda). Spawned bye-1 8d2d630764264357 (custom:LiteLLM/claude-sonnet-5-5, base 5fbd1a5) for the code removal (src/, tests/, docs/cli.md, README.md). Coordinator's half, done in parallel on disjoint files (commit below):
- yaks skill: `migrate` row removed; the pre-0.0.16 note now says such a farm refuses to open and names the fix (`npx -y @j15r/yaks@0.0.16 migrate`, commit).
- AGENTS.md first invariant: the transitional sentence becomes the permanent refusal.
- .agents/skills/README.md: the cold-read brief's path is `.yaks/yaks-5c9f/cold-read-brief.md` (it moved with the migration).
- .gitignore: the `!.yaks/artifacts/` un-ignore lines are dead (attachments now live in `.yaks/<id>/`, which the broad `artifacts/` rule does not match); removed. Checked: `git check-ignore` does not ignore `.yaks/yaks-5c9f/cold-read-brief.md`.
- docs/assets/tui-detail-pinned.svg refreshed (stale since before the flat work); rasterized with headless Chrome and looked at: the pinned header, then the scrolled body with the detail cursor on the last note, matching docs/tui.md's caption.
- Re-check of .agents/skills, docs/README.md, docs/tui.md, AGENTS.md for status-dir / artifacts/ / relocation / glob wording: nothing else left.

[phase4-real-machine](yaks-8b7a/phase4-real-machine.txt)

---
▸ 2026-10-12T23:01:15Z [delta-lead]

Phase 4 real-machine check (after landing yaks-bbda).

---
▸ 2026-10-12T23:01:15Z [delta-lead]

Landed yaks-bbda (bye-1 8d2d630764264357): 42d209e, merge by SHA, clean; 22 files, +665 -1807. Gate on this machine: 17 suites, 667 passed, 0 failed, 3 ignored (fewer than before: the old-layout tests went with the code). Leftover grep (legacy::, lock_for_migration, Migrated, legacy_pair, yaks-bbda, is_old_layout, flat_attachment_links, Command::Migrate) over src/ and tests/: empty.
Real machine (attached): this farm works and doctor is clean; a read-only `yaks list` in a pre-0.0.16 clone (git-2's) refuses with exit 1 and the message naming `npx -y @j15r/yaks@0.0.16 migrate`, writing nothing there; `yaks sheds` shows the six old-layout clones as `farm unreadable: old layout (pre-0.0.16 clone)`, never as empty farms.
All phases done: 1 yaks-95be, 2 yaks-86f4/9f91/ca28, 3 yaks-3a56 (0.0.16 released), 4 yaks-bbda (ships as 0.0.17). Follow-ups filed along the way: yaks-8814 (landed vs a needs: the reference resolved), yaks-07fe (shed scan speed). Shearing the umbrella.

---
▸ 2026-10-12T23:01:57Z [delta-lead]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-12T23:01:57Z [delta-lead]

moved: shaving -> shorn
