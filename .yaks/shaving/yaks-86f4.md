---
id: yaks-86f4
title: 'Flat farm, phase 2: yaks migrate + doctor/preflight checks'
type: task
priority: 1
created: '2026-10-12T19:53:57Z'
updated: '2026-10-12T20:20:38Z'
parent: yaks-8b7a
depends_on:
- yaks-95be
labels:
- cli
---

Phase 2 of yaks-8b7a (plan note: D5, D8; runs after phase 1, yaks-95be). Sonnet worker.
Scope: a `yaks migrate [--dry-run]` command (src/main.rs: one new `Command` variant and its dispatch; src/farm.rs: `Farm::migrate`), the doctor and preflight integrity checks (an id in both layouts replaces "two status dirs"; an invalid `status:` value), and the docs for exactly these: docs/cli.md rows (`migrate`, `doctor`, `preflight`, `init`), README.md rows, and AGENTS.md's first invariant (status is now a frontmatter field; one file per yak at `.yaks/<id>.md`).
Migrate: under the farm lock; refuse, listing the ids, when any id exists in both layouts; move each legacy `<status>/<id>.md` to `<id>.md` with `status:` set; remove the emptied status dirs (never a non-empty one); write `schema` 4; print one summary line and the suggested commit. `--dry-run` writes nothing and prints what would move. Idempotent on a v4 farm ("already flat"). Works on a private or pointer farm (no git involved).
Out of scope: src/store.rs (phase 1, done), src/changes.rs/status.rs/commit.rs and src/sheds.rs (other workers), skills (coordinator).
Evidence (fail first): a v3 fixture migrates to flat with every status preserved and notes byte-identical; a collision refuses and changes nothing; dry-run changes nothing; doctor flags a hand-made both-layouts farm.

---
▸ 2026-10-12T20:00:39Z [delta-lead]
Amended by D10 (see yaks-8b7a): migrate also moves `.yaks/artifacts/<id>/` to `.yaks/<id>/` and rewrites every `](artifacts/<id>/` link in every body to `](<id>/`, then removes the emptied `artifacts/`. This worker also owns converting the artifact code paths to phase 1's helper: src/farm.rs (attach, rename_attachment, merge's artifact copy, rename/rename_many's artifact dir move), src/preflight.rs (untracked-attachment check), src/tui/handlers.rs and src/tui/editor.rs (link resolution, rename), and the `attach`/`rename-attachment` docs rows. The migrate command is transitional (phase 4 deletes it): keep it in one place.

---
▸ 2026-10-12T20:20:37Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-12T20:20:38Z [delta-lead]
Claim (coordinator). Owner: mig-1. Spec: the description, the amendment note on this yak, and yaks-8b7a's plan (D5, D8, D10) and amendments.
You own: src/farm.rs (`Farm::migrate`; doctor; attach, rename_attachment, merge's artifact copy, rename/rename_many moving `<old>/` to `<new>/`; the merge source may be either layout), src/preflight.rs (the both-layouts check replacing "two status dirs"; the untracked-attachment check for `.yaks/<id>/`), src/main.rs (ONE new `Command::Migrate` variant, its dispatch, and the `--help` text of `migrate`/`init`/`doctor`/`preflight`/`attach`/`rename-attachment`), src/tui/handlers.rs and src/tui/editor.rs (attachment links are now `<id>/<file>`, resolved from the farm root), and the docs rows for exactly those commands in docs/cli.md and README.md, plus AGENTS.md's first invariant.
Migrate details: under `store::lock_for_migration`; refuse, listing ids, when any id is in both layouts; per old-layout file write `<id>.md` with `status:` (reuse `store::write::save` on the parsed task, status from its directory) then remove the old file; move `artifacts/<id>/` to `<id>/` and rewrite every `](artifacts/<id>/` in EVERY yak's body to `](<id>/`; remove emptied status dirs and an emptied `artifacts/` (never a non-empty dir); write `schema` 4 last; print `migrated N yaks (M attachment dirs) to the flat layout; commit with: git add -A .yaks && git commit -m "yaks: migrate the farm to the flat layout"`. `--dry-run` prints the plan and writes nothing; on a flat farm, "already flat". A private/pointer farm works the same (no git involved).
Doctor/preflight: "id in both layouts" (a flat `<id>.md` plus an old-layout copy), keep the old "two status dirs" detection working for an unmigrated farm, and "invalid status value" (read the file text: a `status:` line whose value is not one of the four).
Note: `rename`'s id rewrite already rewrites id tokens inside link text; check it covers `](<old>/file)` and add a test.
Evidence (fail first): a v3 fixture (status dirs, two attachments, a body linking another yak's attachment) migrates with every status, note and attachment preserved and every link resolving; a collision refuses and changes nothing; dry-run changes nothing; doctor flags both-layouts and an invalid status; attach writes `<id>/<file>` and links `](<id>/<file>)`; rename moves the attachment dir and fixes links; merge from a v3 source lands flat with its attachments. Judge: coordinator.
## Rules for every phase-2 worker (read before starting)
- Phase 1 (yaks-95be, landed 79f00e9) is the base. Read its shorn notes first: `store::task_path`, `store::artifact_dir`, `store::attachment_owner`, `store::path_owner`, `store::legacy` (ALL old-layout reading lives there; add any new old-layout reading there too, so yaks-bbda deletes it in one place), `store::lock` (refuses writes on an old-layout farm) and `store::lock_for_migration`.
- THIS REPO'S OWN FARM STAYS ON THE OLD LAYOUT until phase 3. Your built binary therefore refuses to write here, by design. Use the `yaks` on PATH (0.0.15) for every command on your yak; use your own `./target/release/yaks` only on temp fixtures. Never run `migrate`, or any write of the new binary, against this repo's `.yaks`.
- Finish staging is the old layout's: `git add -A -- .yaks/shaving/<id>.md .yaks/shorn/<id>.md` plus `.yaks/artifacts/<id>` if you attached something, and NEVER name a path that does not exist (git then stages nothing at all). Check `git status --short` is empty after your commit.
- File ownership is strict (three workers run at once): touch only the files your claim names. If you must touch another, `yaks ask` instead.
