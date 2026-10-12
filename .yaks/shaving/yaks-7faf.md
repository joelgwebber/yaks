---
id: yaks-7faf
title: Release v0.0.15
type: task
priority: 3
created: '2026-10-12T01:54:23Z'
updated: '2026-10-12T01:54:35Z'
labels:
- release
---

---
▸ 2026-10-12T01:54:35Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-12T01:54:35Z [delta-lead]
Version 0.0.15 (Cargo.toml, Cargo.lock, npm/yaks/package.json in lockstep). Plan: push origin main (done, a0f6489), bump + commit, manual workflow dispatch dry run first (yaks-3479's lesson: a dry run does not exercise the registry's per-package auth, but it does catch a bad build/version lockstep before tagging), then tag v0.0.15 and push the tag for the real publish.
