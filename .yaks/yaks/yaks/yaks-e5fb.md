---
id: yaks-e5fb
title: 'Port worker landing (land.sh) into the binary: yaks land <sha>'
status: hairy
type: feature
priority: 2
created: '2026-10-18T15:54:02Z'
updated: '2026-10-18T15:54:02Z'
parent: yaks-6a34
labels:
- cli
- git
---

Joel (2026-10-18): shell scripts for landing are stinky, especially across platforms. Move `.agents/skills/yaks-coordinating-delta/land.sh` into the binary as `yaks land <sha> [--dry-run] [--force-discard]`: pending-state report, ancestry check (shallow-aware), `merge-tree --write-tree`, the unexplained-changes refusal, clearing Delta's applied state, the squash commit (the linear landing from its sibling yak), the tree check, and, for a live farm, the farm landing (`yaks commit --land`) in the same command. git stays a subprocess (no libgit2 dependency); the selftest scenarios become Rust tests with real temp repos. Then land.sh is deleted from the skill and from `BUNDLED`, and the Delta/worktrees skills say `yaks land`. Feasible: every step is git plumbing the script already calls.
