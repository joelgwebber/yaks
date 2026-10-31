---
id: yaks-9b9f
title: 'Flaky test: sheds inspecting_a_shed_changes_nothing_in_it fails under macOS fsmonitor'
status: hairy
type: bug
priority: 3
created: '2026-10-31T15:05:05Z'
updated: '2026-10-31T15:05:05Z'
labels:
- sheds
- test
---

`sheds::tests::inspecting_a_shed_changes_nothing_in_it` compares a snapshot of every file's mtime before and after inspecting a shed. With git's `core.fsmonitor` enabled (the macOS default for this user), the fsmonitor daemon touches `.git/worktrees/wt/fsmonitor--daemon/cookies` in between, so the snapshot differs.

It fails the same way on `main` without any other change. Passes with:

```sh
GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.fsmonitor GIT_CONFIG_VALUE_0=false cargo test
```

Fix: have the test repo set `core.fsmonitor=false` (or exclude `fsmonitor--daemon` from the snapshot).
