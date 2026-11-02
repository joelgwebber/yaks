---
id: yaks-9b9f
title: 'Flaky test: sheds inspecting_a_shed_changes_nothing_in_it fails under macOS fsmonitor'
status: dead
type: bug
priority: 3
created: '2026-10-31T15:05:05Z'
updated: '2026-10-31T16:52:21Z'
parent: yaks-d3ad
tags:
- sheds
- test
---

`sheds::tests::inspecting_a_shed_changes_nothing_in_it` compares a snapshot of every file's mtime before and after inspecting a shed. With git's `core.fsmonitor` enabled (the macOS default for this user), the fsmonitor daemon touches `.git/worktrees/wt/fsmonitor--daemon/cookies` in between, so the snapshot differs.

It fails the same way on `main` without any other change. Passes with:

```sh
GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.fsmonitor GIT_CONFIG_VALUE_0=false cargo test
```

Fix: have the test repo set `core.fsmonitor=false` (or exclude `fsmonitor--daemon` from the snapshot).

---
▸ 2026-10-31T15:56:25Z [delta:Cloud Runners]

Adopted into `yaks-039b` under `yaks-d3ad` (2026-10-31): the git/Delta shed discovery this concerns is being removed. Slaughter as obsolete when `yaks-d3ad` lands.

---
▸ 2026-10-31T16:47:44Z [sheds-1]

Obsolete (sheds-1): yaks-d3ad deleted the flaky test (`inspecting_a_shed_changes_nothing_in_it`) with the git-discovery code it tested. For the yakherd to slaughter.

---
▸ 2026-10-31T16:52:21Z [yakherd]

Obsolete: the code this concerns was removed in `b2e4638` (`yaks-d3ad`). Slaughtered by the yakherd.

---
▸ 2026-10-31T16:52:21Z [yakherd]

kind: move
from: hairy
to: dead
---

moved: hairy -> dead
