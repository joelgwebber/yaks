---
id: yaks-c019
title: 'Fix /doks/releases/v0.0.41.md''s hero image: drop the @sha pin'
status: shaving
type: chore
priority: 2
created: '2026-11-23T00:16:34Z'
updated: '2026-11-23T00:18:39Z'
parent: yaks-e387
tags:
- release
- images
---

The pin makes the image 404 on GitHub (the @f4211aaa is read as part of the filename) and blocks the TUI's inline draw. `yaks release changelog` already turns a plain farm image into a commit-pinned `raw/<sha>/` URL, so the pin bought nothing. Write it unpinned, keep it under its bullet (the TUI child draws list images), and check CHANGELOG.md is unchanged.

#### 2026-11-23T00:18:39Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
