---
id: yaks-6a9b
title: 'Per-herd directories: <herd>/yaks/<id>.md (schema 6), room for per-herd and global docs'
status: shaving
type: feature
priority: 2
created: '2026-10-21T03:30:37Z'
updated: '2026-10-21T15:48:37Z'
labels:
- design
---

Raised in yaks-b32e's question list ("Do we separate herds at the top-level, even if just for clarity?") but never decided on its own: b32e's decision (keep the flat layout) was about scale, and herd dirs were only measured as part of the hashed `<herd>/3/a/8/` layout. Joel asked again 2026-10-19 before putting his multi-herd personal farm (~/src/fs/.yaks: fullstory, sightmap, sightkick, jev, potemkin) under source control.

Lean (delta:herd-pass): not now. Costs:
- Breaks the stable-path invariant: `rename-herd` and a cross-herd move (yaks-54e9) would MOVE files, so `git log --follow`, `yaks commits` (file-follow), editor tabs and any link to the file break; today they only rewrite text.
- The herd is already in the id; the directory would be a second copy of it, so a file in the wrong dir needs a rule (which wins?) and a doctor check.
- Schema bump + migration, and every path-reading site changes (store, sheds farm-diff, preflight's path->id, status/commit messages, attachments).
- What it would buy is mostly available already: `ls .yaks/jev-*`, `git log -- '.yaks/jev-*'`, `yaks list --herd jev`; extracting a herd is `yaks merge` from a filtered copy.

Revisit if: one farm routinely holds herds owned by different people (per-dir CODEOWNERS / sparse checkout would then earn it), or a herd needs to be split out often.

---
▸ 2026-10-21T15:27:07Z [delta:herd-dirs]

Decision (Joel, 2026-10-19): do it. Reasons beyond my "not now" lean above: wiki-like structures outside individual yaks are coming (they need room that is not the farm root), and the multi-owner case. Joel corrected one of my costs: a rename or cross-herd move ALREADY renames the file today (the filename is the id), so herd dirs do not make paths less stable. He accepts another migration (all farms are on his two machines).

## Layout (schema 6)
```
.yaks/
  config.yaml  schema  .gitignore  .lock      farm files stay at the root
  <herd>/
    yaks/
      <id>.md                                  the yak (full id as filename)
      <id>/                                    its attachments, beside it
    docs/                                      reserved, not built (per-herd docs)
  docs/                                        reserved, not built (farm-wide docs)
```
- **Filename keeps the full id** (`jev/yaks/jev-1a2b.md`, not `jev/yaks/1a2b.md`). Attachment links in bodies are relative (`[x](jev-1a2b/log.txt)`) and attachments move with the file, so NO body rewrite; editor tabs and `find -name` stay unambiguous across herds; an id copied from text greps straight to its file. Fuzzy find is unaffected (`jev/…1a2b` matches either way).
- **The frontmatter `id:` stays authoritative.** `store::task_path(root, id)` = `<prefix>/yaks/<id>.md` where prefix = id before the first `-`; `artifact_dir` = `<prefix>/yaks/<id>/`. A yak file anywhere else is MISPLACED: `doctor` reports it, `yaks migrate` moves it.
- **Discovery of yak files:** for each top-level dir with a `yaks/` subdir, read `yaks/*.md`. Fixed depth, one `read_dir` per herd.
- **Reserved:** `docs` is not a valid herd name (create / rename-herd / config refuse it). Dot-files and the root farm files are never herds.
- **Source links** `../src/foo.rs` keep their meaning (a path from the repo root, resolved by yaks); they just no longer happen to resolve as plain relative links in other renderers (they already did not on a farm branch).

## Migration (`yaks migrate`, schema 5 -> 6)
A layout step before the body fixups: move every `<id>.md` and `<id>/` at the root into `<herd>/yaks/`, then write `schema` last (an interrupted run is re-run; moves are idempotent). `--dry-run` reports the counts per herd. Plain renames, so git records 100%-similarity renames and `git log --follow` keeps working. Schema-5 farms are refused by every other command (the existing `migration_refusal`); 0.0.21 refuses a schema-6 farm as newer.

## Code (all through the two path helpers)
- `store`: `task_path`, `artifact_dir`, `path_owner`/`attachment_owner` (paths relative to the farm root: `<h>/yaks/<id>.md`, `<h>/yaks/<id>/...`), `load`, `all_ids`, writes create the herd dir; `farm::flat_files` (doctor) + a misplaced-file check; `migrate` layout step; `SCHEMA = 6`.
- Path-printing and git-tree readers: `brief` (finish line), `main` (attach/rename-attachment messages), `sheds::base_files` (accept both the old flat and new shape: a base commit can predate the migration), `changes`/`status`/`preflight` (via `path_owner`).
- Tests: helpers, loader, migration (dry run, real run, idempotent re-run, attachments move, bodies byte-identical), misplaced-file doctor check, a farm with two herds.

## Surfaces
docs (cli/tui/README), the bundled skills' "Task format" and every `.yaks/<id>.md`, AGENTS.md invariants, `--help` of migrate/path/attach. Then migrate this repo's farm and Joel's other farms (hearth, j15r, canon, fs) with the released binary.

---
▸ 2026-10-21T15:27:07Z [delta:herd-dirs]

moved: hairy -> shaving

[schema6-tui-frame](yaks-6a9b/schema6-tui-frame.txt)

---
▸ 2026-10-21T15:48:37Z [delta:herd-dirs]

Built (uncommitted in the delta thread's worktree; not landed yet):

- `store`: `herd_of`, `herd_dir`, `herd_dirs`, `yak_files` (one `read_dir` per herd, fixed depth); `task_path`/`artifact_dir` = `<herd>/yaks/<id>(.md)`; `path_owner`/`attachment_owner` read the new shape (a loose root file, `docs/…`, dot-paths and a herd's `docs/` are never yaks); `save` creates the herd dir; `SCHEMA = 6`; `RESERVED_HERD_NAMES = ["docs"]` (create and rename-herd refuse it).
- `migrate`: a `herd-directories` layout step before the body fixups (plain renames, refuses on an existing destination or a `docs` herd, report per herd).
- `doctor`: new `misplaced-file` (wrong herd dir, wrong name, or loose at the root); a second copy beside the real file stays `duplicate-id`.
- `sheds`: a base commit from before the migration is materialized at the new paths; an unmigrated shed is reported, never read as empty. `merge` creates herd dirs. `brief`, `attach`/`rename-attachment` messages print the new paths.
- Docs/help/skills: cli.md (migrate, init, doctor, path, attach), docs/README.md layout tree, tui.md source-link note, AGENTS.md invariants, yaks skill "Task format", yaks-working, coordinating-team, `--help` for migrate/init/path/attach, RELEASING.md.

Evidence:
- `cargo test --release`: all 18 suites green (747 unit tests), including the previously failing tests/attachment_sheds.rs (fixed its macOS /var vs /private/var compare).
- New tests: path helpers (incl. docs/, dot, loose files), migrate 4->6 with two herds + an attachment dir + non-yak root files (dry run moves nothing, real run, idempotent re-run, doctor clean), doctor misplaced-file + `docs` refused as a herd, merge refuses a schema-5 source.
- Real farm: a copy of this repo's farm (491 yaks, 82 attachment dirs, herds `yaks` + `test`) migrated in 0.08 s; doctor clean; second run "nothing to migrate"; `list --all` 491 before and after; git sees 633 renames (100%) + the schema line; `yaks path`, `yaks attachment`, `show` work; headless TUI frame attached (detail pane resolves cross-herd children).
