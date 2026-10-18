---
id: yaks-6c4f
title: Source links in yaks resolve against the farm's parent, which is the farm worktree under a live/branch farm
status: hairy
type: bug
priority: 2
created: '2026-10-18T19:35:20Z'
updated: '2026-10-18T19:35:20Z'
parent: yaks-6a34
labels:
- tui
---

`App::open_source` (src/tui/source.rs) resolves `[x](../src/foo.rs#L12)` against `farm.root().parent()`. Since the live farm (yaks-6a34) that is `~/src/yaks-farm`, a sparse worktree with no source, so links say "no such file"; under an orphan farm branch (yaks-73f4) it can never work. Resolve against the code checkout instead: the git top-level of the directory the TUI was started in (when it is not the farm worktree), else the host's main checkout. Pinned links (`../blob/<sha>/...`) need the code repository too. Also decide what `../` means on GitHub when the yak is viewed on the farm branch (it points into the farm branch's tree): maybe write such links as plain `src/foo.rs` and resolve them repo-relative.
