---
id: yaks-d3df
title: Cut v0.0.41
status: shaving
type: task
priority: 1
created: '2026-11-22T16:42:25Z'
updated: '2026-11-22T17:43:53Z'
tags:
- release
---

After the pre-release batch (yaks-cae0, yaks-daa4, yaks-6554, yaks-879e): `yaks release changes --check`, rerun `yaks release dok v0.0.41` and refresh its headlines (pin a docshot), bump Cargo.toml/Cargo.lock/npm package.json to 0.0.41, `yaks release changelog -o CHANGELOG.md` committed with the bump, then (Joel's go-ahead) push main + tag v0.0.41 to origin; GitHub Release body from `yaks release changelog --release v0.0.41`.

#### 2026-11-22T16:42:27Z

*herd-files*

Batch (herd-files, base 6ae40a60, all claude-sonnet-5-5): init-1 yaks-cae0 (thread ksQQ7I2z8nJeT6a6hDJ07WHX45LOAHgO5JIBxBCA0aLJyrhJPreUvVwLftHG); shots-1 yaks-daa4 (…HgPcJ…); html-1 yaks-6554 (…HgP4Z…); commit-1 yaks-879e (…HgQcZ…). Removed Joel's stale 0.0.22 global npm yaks from Homebrew's node (Joel: uninstall it).

#### 2026-11-22T17:34:00Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-22T17:43:53Z

*herd-files*

Prepared (herd-files): batch landed (eed4dd5e, c763a393, 876170b5, fe2b473d) plus 668232a1 (the release being cut leads CHANGELOG.md with its dok's date); `release changes --check` ok; v0.0.41 dok has headlines and a pinned archive-browser image (farm f4211aaa, on origin; raw URL serves image/png); bump + CHANGELOG.md committed as 7986845c, pushed to local main, installed (yaks 0.0.41). Gate: fmt clean, 1587 passed, 0 failed. Waiting on Joel's go-ahead to push main and tag v0.0.41 to origin (publishes to npm).
