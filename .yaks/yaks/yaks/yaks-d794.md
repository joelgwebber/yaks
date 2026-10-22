---
id: yaks-d794
title: yaks path --all prints the farm root instead of every yak
status: hairy
type: task
priority: 3
created: '2026-10-22T00:42:06Z'
updated: '2026-10-22T00:42:06Z'
labels:
- cli
- bug
---

Found by cold-read-1 (yaks-b048): `yaks path --all` prints the farm root, because `--all` alone is not counted as a filter, so it falls into the no-arguments case. Expected: every yak's path, dead included. `yaks path --status shorn` works.
