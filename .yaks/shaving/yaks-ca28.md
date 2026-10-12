---
id: yaks-ca28
title: 'Flat farm, phase 2: sheds read both layouts (base_farm, mixed-layout comparisons)'
type: task
priority: 1
created: '2026-10-12T19:53:57Z'
updated: '2026-10-12T20:20:38Z'
parent: yaks-8b7a
depends_on:
- yaks-95be
labels:
- cli
- delta
---

Phase 2 of yaks-8b7a (plan note: D4's permanent dual read; runs after phase 1, yaks-95be). Sonnet worker.
Scope: src/sheds.rs `base_farm` (it parses `git ls-tree` paths per status dir today) so a base commit in EITHER layout materializes correctly; every cross-shed comparison between a migrated and an unmigrated checkout (`sheds`, `changes`, `show --sheds`, `inbox --sheds`, `landed`): a yak whose only difference is the layout is NOT a change; scripts/discover-fixture.sh; tests.
Out of scope: src/store.rs (phase 1 already reads both layouts from disk), migrate, git reporting, skills.
Evidence (fail first): a shed whose fork point is a v3 commit and whose working tree is v4 (it migrated) shows only its real changes; a v4 viewer and a v3 shed (not migrated) compare correctly both ways; a migration alone reads as "no changes of its own".

---
▸ 2026-10-12T20:00:39Z [delta-lead]
Amended by the transitional D4 (see yaks-8b7a): reading the FLAT layout from a git tree is the required part (after this farm migrates, every fork point is flat). Reading the old layout from a tree is transitional (phase 4 deletes it): keep it in one function. Permanent: a shed whose own farm is schema 3 reads as an old-layout clone, never as having deleted every yak.

---
▸ 2026-10-12T20:20:37Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-12T20:20:38Z [delta-lead]
Claim (coordinator). Owner: shed-1. Spec: the description, the amendment note on this yak, and yaks-8b7a's plan.
You own: src/sheds.rs, scripts/discover-fixture.sh, and the sheds tests (in src/sheds.rs and tests/ files about sheds/changes/inbox --sheds/show --sheds/answer @shed). Docs only if a sheds row describes the layout.
Phase 1 already taught `base_farm` to keep `<rel>/<id>.md` from a git tree (minimal). Your part: mixed-layout correctness and its tests: a shed whose fork point is an old-layout commit and whose working tree is flat (it migrated) shows only its real changes (a migration alone reads "no changes of its own"); a flat viewer and an unmigrated shed compare correctly both ways, through `sheds`, `changes`, `show --sheds`, `inbox --sheds` and `landed`; `answer <id>@<shed>` into an unmigrated shed refuses with phase 1's message (the shed's farm lock refuses), and says which shed. Also: mark an unmigrated shed on its farm line, e.g. `farm: … (old layout)` and JSON `farm.layout: "old"|"flat"`, so yaks-bbda only has to turn it into "old-layout clone, not read". Keep every old-layout read in `store::legacy` or one function in sheds.rs that yaks-bbda can delete.
Evidence (fail first): fixtures for each case above, built with the new binary's own writer plus hand-made old-layout trees committed in git. Judge: coordinator.
## Rules for every phase-2 worker (read before starting)
- Phase 1 (yaks-95be, landed 79f00e9) is the base. Read its shorn notes first: `store::task_path`, `store::artifact_dir`, `store::attachment_owner`, `store::path_owner`, `store::legacy` (ALL old-layout reading lives there; add any new old-layout reading there too, so yaks-bbda deletes it in one place), `store::lock` (refuses writes on an old-layout farm) and `store::lock_for_migration`.
- THIS REPO'S OWN FARM STAYS ON THE OLD LAYOUT until phase 3. Your built binary therefore refuses to write here, by design. Use the `yaks` on PATH (0.0.15) for every command on your yak; use your own `./target/release/yaks` only on temp fixtures. Never run `migrate`, or any write of the new binary, against this repo's `.yaks`.
- Finish staging is the old layout's: `git add -A -- .yaks/shaving/<id>.md .yaks/shorn/<id>.md` plus `.yaks/artifacts/<id>` if you attached something, and NEVER name a path that does not exist (git then stages nothing at all). Check `git status --short` is empty after your commit.
- File ownership is strict (three workers run at once): touch only the files your claim names. If you must touch another, `yaks ask` instead.
