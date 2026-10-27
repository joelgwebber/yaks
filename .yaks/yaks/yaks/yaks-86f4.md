---
id: yaks-86f4
title: 'Flat farm, phase 2: yaks migrate + doctor/preflight checks'
status: shorn
type: task
priority: 1
created: '2026-10-12T19:53:57Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-8b7a
depends_on:
- yaks-95be
labels:
- cli
needs: agent
---

Phase 2 of yaks-8b7a (plan note: D5, D8; runs after phase 1, yaks-95be). Sonnet worker.
Scope: a `yaks migrate [--dry-run]` command (src/main.rs: one new `Command` variant and its dispatch; src/farm.rs: `Farm::migrate`), the doctor and preflight integrity checks (an id in both layouts replaces "two status dirs"; an invalid `status:` value), and the docs for exactly these: docs/cli.md rows (`migrate`, `doctor`, `preflight`, `init`), README.md rows, and AGENTS.md's first invariant (status is now a frontmatter field; one file per yak at `.yaks/<id>.md`).
Migrate: under the farm lock; refuse, listing the ids, when any id exists in both layouts; move each legacy `<status>/<id>.md` to `<id>.md` with `status:` set; remove the emptied status dirs (never a non-empty one); write `schema` 4; print one summary line and the suggested commit. `--dry-run` writes nothing and prints what would move. Idempotent on a v4 farm ("already flat"). Works on a private or pointer farm (no git involved).
Out of scope: src/store.rs (phase 1, done), src/changes.rs/status.rs/commit.rs and src/sheds.rs (other workers), skills (coordinator).
Evidence (fail first): a v3 fixture migrates to flat with every status preserved and notes byte-identical; a collision refuses and changes nothing; dry-run changes nothing; doctor flags a hand-made both-layouts farm.

---
▸ 2026-10-12T20:00:39Z [delta-lead]

Amended by D10 (see yaks-8b7a): migrate also moves `.yaks/artifacts/<id>/` to `.yaks/<id>/` and rewrites every `](<id>/` link in every body to `](<id>/`, then removes the emptied `artifacts/`. This worker also owns converting the artifact code paths to phase 1's helper: src/farm.rs (attach, rename_attachment, merge's artifact copy, rename/rename_many's artifact dir move), src/preflight.rs (untracked-attachment check), src/tui/handlers.rs and src/tui/editor.rs (link resolution, rename), and the `attach`/`rename-attachment` docs rows. The migrate command is transitional (phase 4 deletes it): keep it in one place.

---
▸ 2026-10-12T20:20:37Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-12T20:20:38Z [delta-lead]

Claim (coordinator). Owner: mig-1. Spec: the description, the amendment note on this yak, and yaks-8b7a's plan (D5, D8, D10) and amendments.
You own: src/farm.rs (`Farm::migrate`; doctor; attach, rename_attachment, merge's artifact copy, rename/rename_many moving `<old>/` to `<new>/`; the merge source may be either layout), src/preflight.rs (the both-layouts check replacing "two status dirs"; the untracked-attachment check for `.yaks/<id>/`), src/main.rs (ONE new `Command::Migrate` variant, its dispatch, and the `--help` text of `migrate`/`init`/`doctor`/`preflight`/`attach`/`rename-attachment`), src/tui/handlers.rs and src/tui/editor.rs (attachment links are now `<id>/<file>`, resolved from the farm root), and the docs rows for exactly those commands in docs/cli.md and README.md, plus AGENTS.md's first invariant.
Migrate details: under `store::lock_for_migration`; refuse, listing ids, when any id is in both layouts; per old-layout file write `<id>.md` with `status:` (reuse `store::write::save` on the parsed task, status from its directory) then remove the old file; move `artifacts/<id>/` to `<id>/` and rewrite every `](<id>/` in EVERY yak's body to `](<id>/`; remove emptied status dirs and an emptied `artifacts/` (never a non-empty dir); write `schema` 4 last; print `migrated N yaks (M attachment dirs) to the flat layout; commit with: git add -A .yaks && git commit -m "yaks: migrate the farm to the flat layout"`. `--dry-run` prints the plan and writes nothing; on a flat farm, "already flat". A private/pointer farm works the same (no git involved).
Doctor/preflight: "id in both layouts" (a flat `<id>.md` plus an old-layout copy), keep the old "two status dirs" detection working for an unmigrated farm, and "invalid status value" (read the file text: a `status:` line whose value is not one of the four).
Note: `rename`'s id rewrite already rewrites id tokens inside link text; check it covers `](<old>/file)` and add a test.
Evidence (fail first): a v3 fixture (status dirs, two attachments, a body linking another yak's attachment) migrates with every status, note and attachment preserved and every link resolving; a collision refuses and changes nothing; dry-run changes nothing; doctor flags both-layouts and an invalid status; attach writes `<id>/<file>` and links `](<id>/<file>)`; rename moves the attachment dir and fixes links; merge from a v3 source lands flat with its attachments. Judge: coordinator.
## Rules for every phase-2 worker (read before starting)
- Phase 1 (yaks-95be, landed 79f00e9) is the base. Read its shorn notes first: `store::task_path`, `store::artifact_dir`, `store::attachment_owner`, `store::path_owner`, `store::legacy` (ALL old-layout reading lives there; add any new old-layout reading there too, so yaks-bbda deletes it in one place), `store::lock` (refuses writes on an old-layout farm) and `store::lock_for_migration`.
- THIS REPO'S OWN FARM STAYS ON THE OLD LAYOUT until phase 3. Your built binary therefore refuses to write here, by design. Use the `yaks` on PATH (0.0.15) for every command on your yak; use your own `./target/release/yaks` only on temp fixtures. Never run `migrate`, or any write of the new binary, against this repo's `.yaks`.
- Finish staging is the old layout's: `git add -A -- .yaks/shaving/<id>.md .yaks/shorn/<id>.md` plus `.yaks/artifacts/<id>` if you attached something, and NEVER name a path that does not exist (git then stages nothing at all). Check `git status --short` is empty after your commit.
- File ownership is strict (three workers run at once): touch only the files your claim names. If you must touch another, `yaks ask` instead.

---
▸ 2026-10-12T20:23:54Z [mig-2]

mig-2 started (replaces mig-1). date -u: Mon Oct 12 20:23:48 UTC 2026
yaks path yaks-86f4: /Users/joel/src/yaks/.delta/worktrees/n5p2egqfm27v/yaks/.yaks/shaving/yaks-86f4.md

---
▸ 2026-10-12T20:51:40Z [mig-2]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-12T20:52:47Z [mig-2]

Decision: `Farm::migrate` and its helpers (`rewrite_artifact_links`, `MigratePlan`, `MigrateOutcome`, `tracked_by_git`) live in one marked section of src/farm.rs ("yaks migrate (transitional: yaks-bbda deletes this section)"). Rejected: a new src/migrate.rs (not in my file scope; the claim names farm.rs).
Decision: link rewrite only touches `](<seg>/` where `<seg>` is followed by `/` (a link into a yak dir). Prose such as "artifacts/ in general", `[x](artifacts/)` and bare `artifacts/yak-1/a.png` mentions outside a markdown link are left alone. Rejected: a blanket `artifacts/` replace (would corrupt prose); also rejected rewriting bare path mentions (not links, and the claim says `](<id>/`).
Decision: refusal covers three cases, all listing ids and writing nothing: id in both layouts, id in two old status dirs, and an existing `.yaks/<id>/` that `artifacts/<id>/` would land on. Rejected: merging into an existing dir (silent clobber risk).
Decision: a non-empty old dir (status dir or `artifacts/`) stays, with a stderr `note:`; remove_dir (never remove_dir_all) is the only removal. Migration still succeeds and writes schema 4.
Decision: the schema-3 warning on open is suppressed for `migrate` only (it is the cure for that warning). Rejected: leaving it printed (noisy, misleading on a successful run).
Decision: the commit hint prints only when git tracks something under the farm (`git ls-files -- .`), so private/pointer farms get the plain summary line with no git step. Rejected: always printing it.
Decision: doctor/preflight: new IssueKinds `BothLayouts` (`both-layouts`) and `InvalidStatus` (`invalid-status`) plus preflight Checks of the same names; `duplicate-status` (two old status dirs) is kept for unmigrated farms. Invalid status is read from file text (`invalid_status_value`) because the loader falls back to hairy silently. Rejected: changing the loader to report (store.rs is out of scope).
Decision: `rename` moves `.yaks/<old>/` to `.yaks/<new>/` through a temp name (swap/chain-safe) and refuses when `.yaks/<new>/` already exists unclaimed. The id-token rewrite already covers `](<old>/file)` links; tested.
Decision: `merge` from an old-layout source rewrites `](<id>/` links in the incoming bodies (rewritten flat files only; a flat source is still copied byte-for-byte) and copies `<id>/` or, failing that, `artifacts/<id>/`.
Decision: TUI link resolution needed only small changes: the rename-attachment parser reads `<id>/<file>` (no `artifacts/` prefix), the paste-name collision check uses `store::artifact_dir`; `open_external` already joins the farm root. src/tui/detail.rs untouched (image-link scanning is path-agnostic).

Summary: `yaks migrate [--dry-run]` (Farm::migrate, one Command::Migrate variant + report_migrate). Attach, rename-attachment, merge, rename/rename_many, preflight's untracked check and the TUI now use `<id>/` via store::artifact_dir. Doctor/preflight gained both-layouts and invalid-status. Docs: docs/cli.md rows (migrate, doctor, preflight, init, attach, rename-attachment, rename, merge), README rows (doctor, preflight, migrate), AGENTS.md first invariant + intro line, clap --help for migrate/init/doctor/preflight/attach/rename-attachment.
Evidence (new tests, all passing): farm::tests migrate_converts_a_v3_farm_to_flat (statuses, notes byte-identical, attachments, every link resolves, doctor clean, rerun = already flat), migrate_refuses_when_an_id_is_in_both_layouts_and_changes_nothing, migrate_refuses_when_an_attachment_dir_would_collide, migrate_dry_run_writes_nothing, migrate_leaves_a_non_empty_old_dir_alone, migrate_on_a_flat_farm_is_already_flat, artifact_links_are_rewritten_only_when_they_name_a_yak_dir, rename_moves_the_attachment_dir_and_fixes_links, merge_from_a_v3_source_lands_flat_with_attachments, doctor_flags_both_layouts_and_invalid_status; preflight yak_in_both_layouts_fails, invalid_status_value_fails; tests/cli.rs migrate_converts_a_v3_farm_via_the_cli (write refuses before, dry-run, collision refuses, migrate, already flat, writes work after).
Real machine: ran ./target/release/yaks on a COPY of this repo's .yaks (in a scratch git repo, never on the real one): dry-run listed moves and wrote nothing; migrate printed `migrated 404 yaks (51 attachment dirs) to the flat layout; commit with: ...`; doctor "All clear"; second run "already flat"; git -M saw renames; no `](artifacts/` links left in any yak; yaks-95be/phase1-real-machine.txt present beside its yak.
Gate: `cargo test --release`: 16 suites, 662 passed, 0 failed, 4 ignored (the same 4 as phase 1; the yaks-9f91 one is that worker's). Not touched/known: src/brief.rs, commit.rs, status.rs, changes.rs, sheds.rs still carry `artifacts/` strings (other owners / coordinator).

---
▸ 2026-10-12T20:52:47Z [mig-2]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-86f4` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-86f4 --drop-request`, which removes this request note, so no new note is left behind.
