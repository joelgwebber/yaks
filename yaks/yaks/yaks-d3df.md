---
id: yaks-d3df
title: Cut v0.0.41
status: hairy
type: task
priority: 1
created: '2026-11-22T16:42:25Z'
updated: '2026-11-22T16:42:25Z'
tags:
- release
---

After the pre-release batch (yaks-cae0, yaks-daa4, yaks-6554, yaks-879e): `yaks release changes --check`, rerun `yaks release dok v0.0.41` and refresh its headlines (pin a docshot), bump Cargo.toml/Cargo.lock/npm package.json to 0.0.41, `yaks release changelog -o CHANGELOG.md` committed with the bump, then (Joel's go-ahead) push main + tag v0.0.41 to origin; GitHub Release body from `yaks release changelog --release v0.0.41`.
