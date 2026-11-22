---
id: yaks-cae0
title: yaks init still writes a fresh farm under .yaks/ (schema 10 in the pre-9 layout)
status: shaving
type: bug
priority: 1
created: '2026-11-18T15:36:22Z'
updated: '2026-11-22T16:40:41Z'
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
