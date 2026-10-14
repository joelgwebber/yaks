---
id: yaks-d64e
title: Release v0.0.18
status: shaving
type: task
priority: 2
created: '2026-10-14T02:56:44Z'
updated: '2026-10-14T02:57:52Z'
labels:
- release
---

---
▸ 2026-10-14T02:56:49Z [delta:ksQQnuQD5YCiRZqkHA9KaCSpVpLOABD6mJIBxBCg]
moved: hairy -> shaving

---
▸ 2026-10-14T02:57:52Z [delta-lead]
Version 0.0.18 (Cargo.toml, Cargo.lock, npm/yaks/package.json). 37 commits since v0.0.17: the TUI Sheds tab (yaks-98f1) with shed badges, the detail pane's Sheds section and shed-only yaks as dim rows (yaks-ed6d, yaks-9631, yaks-2883, yaks-c1f8); the Inbox includes other sheds' open asks and answers them in place (yaks-1ea3); yaks sheds about 4x faster (yaks-07fe); yaks changes folds landed yaks, plus --outstanding/--status (yaks-7422); needs: fixes (yaks-8814, yaks-29e9); a worktree of a Delta clone is a plain shed (yaks-93fe). Gate: cargo test --workspace --release: 773 passed, 0 failed (3 ignored); node npm/yaks/test-mapping.mjs ok.
