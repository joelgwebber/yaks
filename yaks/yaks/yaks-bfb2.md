---
id: yaks-bfb2
title: 'release changelog: one web_url costs ~12 git calls; cache per-run remote facts so a 41-release CHANGELOG.md isn''t 64 s'
status: shaving
type: task
priority: 3
created: '2026-11-22T00:12:59Z'
updated: '2026-11-22T23:12:42Z'
tags:
- perf
- release
- links
---

Measured at 6ae40a60 with every release dok backfilled: `yaks release changelog -o` takes 64 s; `--release v0.0.6` (51 yak links) runs 630 git processes (rev-parse --verify 144, config --get 116, remote get-url 106, config --bool 102, then per link is-bare-repository, cat-file -e, branch -r). `locate::Space::web_url` recomputes the host repo, farm top, branch and pushed HEAD for every link. Cache those once per `Space` use in a batch (keep one seam: a memo inside `web_url`'s helpers or a batch handle, not a second URL builder), keeping the per-file 'is it on the remote' check. Gate: `a_write_spawns_few_git_processes`-style spawn-count test for the changelog.

#### 2026-11-22T00:15:36Z

*herd-files*

Graduated from yaks-a4d8 (the changelog umbrella) when it was shorn: a performance follow-up, not part of the feature.

#### 2026-11-22T23:12:42Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
