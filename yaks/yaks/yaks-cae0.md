---
id: yaks-cae0
title: yaks init still writes a fresh farm under .yaks/ (schema 10 in the pre-9 layout)
status: shaving
type: bug
priority: 1
created: '2026-11-18T15:36:22Z'
updated: '2026-11-22T17:23:41Z'
tags:
- git
- init
---

Found while checking yaks-3515 (2026-11-18). In a brand-new repo, `yaks init --no-skills` (0.0.38 and main) creates the orphan `yaks` branch with `.yaks/.gitignore`, `.yaks/config.yaml`, `.yaks/schema` (= 10); `yaks path` prints `<worktree>/.yaks` and yaks/attachments go under `.yaks/<herd>/yaks/`.

Schema 9 moved the farm to the branch root (yaks-6ffd, yaks-acc0), and yaks-6ffd says "`yaks init` keeps writing `.yaks/` until the migration yak flips the default". The flip never happened: [init.rs](code:src/init.rs#L388-L390) still defaults to `worktree.join(".yaks")`.

Fix: a fresh farm is written at the worktree root; an existing branch keeps its layout. Check `store::init`'s `legacy_dir` test and the init tests, and that `doctor` / `stray-layout` stay quiet on a fresh farm.

#### 2026-11-22T16:40:41Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-22T16:41:00Z

*herd-files*

Claim (herd-files): worker init-1 (Sonnet), base 6ae40a60, batch before v0.0.41. Scope: src/init.rs and whatever writes a fresh farm's files (store::farm_root_in / the root-vs-.yaks choice), tests. Judge: herd-files at landing.

#### 2026-11-22T16:43:04Z

*init-1*

Start 2026-11-22T16:43:03Z; checkout /Users/joel/src/yaks/.delta/worktrees/zmnswq3xa7y3/yaks; farm file /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-cae0.md

#### 2026-11-22T17:23:40Z

*init-1*

Decision: fresh farm root is `farm_root_in(&worktree).unwrap_or_else(|| worktree.clone())` in src/init.rs (ensure_branch_farm). That one line covers every path that creates a farm: own farm, `--branch`, and an existing branch (farm_root_in finds it, so a pre-9 `.yaks/` branch keeps its layout). Rejected: a flag or config key for the layout (the layout is schema 9, not a choice); touching store::init's `legacy_dir` test (it only fires for a root named `.yaks`, which init no longer passes, and migration/Farm::at still use it for old farms). `--farm` (shared farm) never creates a farm, and first-use bootstrap only adds a worktree for an existing branch, so nothing else writes a fresh farm.

#### 2026-11-22T17:23:40Z

*init-1*

Decision: updated existing tests that encoded the old layout (init_modes, autocommit, cli create json path, sync, weight) rather than leaving them: their helpers did `path.parent()` or joined `.yaks/` to reach the worktree. Left alone: tests that build `.yaks/` farms by hand (migration, preflight, links, status, ...), which test pre-9 behaviour.

#### 2026-11-22T17:23:41Z

*init-1*

Evidence: new test tests/init_modes.rs::init_writes_a_fresh_farm_at_the_branch_root. Proved it fails before the fix (stashed src/init.rs): panicked listing `.yaks/.gitignore .yaks/config.yaml .yaks/schema`. Passes after. Scratch run with my binary, fresh repo, `init --no-skills`, `create`:
  ls -a $(yaks path): . .. .git .gitignore .lock config.yaml schema yak
  git ls-tree -r --name-only yaks: .gitignore config.yaml schema yak/yaks/yak-2db6.md
  yaks doctor: All clear: no farm-integrity issues found.  yaks check: All clear.
Docs parity: grep `.yaks` over docs/ README.md .agents/skills/ src/init.rs help: remaining mentions describe pre-9 farms/migration, nothing calls a fresh farm `.yaks/`; no doc change needed. Flaky under parallel load (not touched by this change, passed on rerun): needs_loop, commits.
