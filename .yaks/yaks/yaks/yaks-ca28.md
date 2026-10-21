---
id: yaks-ca28
title: 'Flat farm, phase 2: sheds read both layouts (base_farm, mixed-layout comparisons)'
status: shorn
type: task
priority: 1
created: '2026-10-12T19:53:57Z'
updated: '2026-10-12T20:58:10Z'
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

---
▸ 2026-10-12T20:23:53Z [shed-2]

shed-2 starting (replaces shed-1). date -u: Mon Oct 12 20:23:49 UTC 2026
yaks path yaks-ca28: /Users/joel/src/yaks/.delta/worktrees/bm7tfggpbs0m/yaks/.yaks/shaving/yaks-ca28.md

---
▸ 2026-10-12T20:55:55Z [shed-2]

Decision: no layout-specific comparison code. Every farm read in src/sheds.rs goes through one `load_farm` (store::load already reads both layouts; status comes from `status:` or the directory), so a base commit in either layout, a flat or old working tree, and either viewer compare through the same `yak_changes`. Rejected: normalising both sides to a canonical layout before comparing, or a layout-aware diff; neither is needed because `Task` has no layout.
Decision: `load_farm` also rewrites `](<id>/` to `](<id>/` in the compared bodies (what `yaks migrate` does to bodies). Without it every migrated yak with an attachment reads as `edited` (the description differs), so a migration alone would not read "no changes of its own". Checked fail-first: with the line commented out, the three migration tests fail. A link into `artifacts/` that is not inside an id dir is left alone, as migrate leaves it.
Decision: all old-layout knowledge stays in one section of src/sheds.rs headed "old layout (transitional: yaks-bbda deletes this section)": `is_old_layout`, `old_tree_task`, `load_farm`/`flat_attachment_links`, plus the `layout_marker`/`layout_str` renderers. Everything old reads through `store::legacy` (files, is_status_dir). Rejected: adding to store::legacy (store.rs is out of scope).
Decision: a shed is "old layout" when its farm is schema < 4 OR a yak file still sits in a status dir (same test as store::lock's write refusal, via `legacy::files`). Rejected: schema alone (misses an old binary writing into a v4 farm).
Decision: surface is `farm: ... (old layout)` at the end of the farm line (also on the `yaks changes` header, e.g. `changes (old layout):`) and JSON `farm.layout` ("old"|"flat") in `sheds --json`, plus a per-yak `layout` key in `changes --json`. Rejected: a new ShedFarm variant (every match in farm.rs/main.rs would change; farm.rs is not mine). The flag is a plain `Shed.old_layout` / `Changes.old_layout` bool yaks-bbda can turn into "old-layout clone, not read".
Decision: `answer <id>@<shed>` into an unmigrated shed needed no code: Farm::answer takes the shed farm's `store::lock`, which refuses with phase 1's message (names the shed farm's path, schema v3, `yaks migrate`). The message names the shed's farm path, not the shed's slug; making it say the slug would be an edit in src/farm.rs (answer_in_shed), which is not in my files. Test asserts the path and that nothing was written. If you want the slug, that is a one-line follow-up in farm.rs.
Decision: unit-test fixtures in src/sheds.rs (`task_file`, `set_status`, `commit_move`, `rewrite`) now write FLAT farms (with a `schema` file, as init does); old-layout trees are made by hand with `old_task_file`. Rejected: leaving old-layout fixtures, whose rows now correctly read `(old layout)` and broke 5 tests.
Evidence: tests/sheds_layouts.rs (new, 5 tests, binary-driven, flat side written by the binary, old side hand-made and committed): migrated-uncommitted and migrated-committed shed = "no changes of its own" from both viewers, then only the real move + note after work; flat viewer vs unmigrated shed both ways through sheds, changes, show --sheds, inbox --sheds; landed across layouts; answer refusal writes nothing. Unit tests: base_farm same tasks from an old and a flat commit, is_old_layout, flat_attachment_links.
Also: scripts/discover-fixture.sh builds a flat farm (schema 4 + `.yaks/s-0001.md` with status:); ran it, discover output unchanged in shape. Docs: docs/cli.md sheds + changes rows, README.md sheds + changes rows (layout wording, `(old layout)`, farm.layout, changes --json layout). cargo fmt run on my files only; actor.rs/docshots.rs/tests.rs restored.

---
▸ 2026-10-12T20:56:55Z [shed-2]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-12T20:58:10Z [shed-2]

moved: shaving -> shorn
