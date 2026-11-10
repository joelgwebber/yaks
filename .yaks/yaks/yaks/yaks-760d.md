---
id: yaks-760d
title: 'yaks land: record the landing after a conflicting cherry-pick is resolved'
status: hairy
type: bug
priority: 3
created: '2026-11-10T17:56:51Z'
updated: '2026-11-10T17:56:51Z'
tags:
- cli
- git
---

From the first reflection round (yaks-2b40): when `yaks land` falls back to a cherry-pick that stops on conflicts, the coordinator resolves and runs `git cherry-pick --continue`, and nothing records the `kind: landed` notes; herd-upkeep-2 added them by hand ([session](/sessions/2026/11/herd-upkeep-2-2026-11-06-1904-wrapping-up-yaks-1c89-tools-refs.md)).

Options: `yaks land --continue` (runs the continue and records), or `yaks land <sha>` on an already-applied pick recognises it and records only. Lean: `--continue`, mirroring git.
