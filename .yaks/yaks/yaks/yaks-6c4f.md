---
id: yaks-6c4f
title: Source links in yaks resolve against the farm's parent, which is the farm worktree under a live/branch farm
status: shorn
type: bug
priority: 2
created: '2026-10-18T19:35:20Z'
updated: '2026-10-18T19:59:19Z'
parent: yaks-6a34
labels:
- tui
---

`App::open_source` (src/tui/source.rs) resolves `[x](../src/foo.rs#L12)` against `farm.root().parent()`. Since the live farm (yaks-6a34) that is `~/src/yaks-farm`, a sparse worktree with no source, so links say "no such file"; under an orphan farm branch (yaks-73f4) it can never work. Resolve against the code checkout instead: the git top-level of the directory the TUI was started in (when it is not the farm worktree), else the host's main checkout. Pinned links (`../blob/<sha>/...`) need the code repository too. Also decide what `../` means on GitHub when the yak is viewed on the farm branch (it points into the farm branch's tree): maybe write such links as plain `src/foo.rs` and resolve them repo-relative.

---
▸ 2026-10-18T19:57:54Z [delta-farm]

moved: hairy -> shaving

---
▸ 2026-10-18T19:59:19Z [delta-farm]

Fixed (delta-farm): `source_base` (src/tui/source.rs) resolves source links in the farm's parent unless the farm is live; then in the git top-level of the TUI's cwd, else the repository's main checkout (`sheds::host_checkout`), never the farm worktree. Test `a_live_farm_resolves_source_links_in_the_code_checkout_not_the_farm_worktree`. Real machine: a probe yak linking `../src/commit.rs#L40`, followed from ~/src/yaks with a stub editor: the old ~/go/bin build said 'no such file', the fix queued `src/commit.rs:L40`. Full `cargo test --release` green. The GitHub-side meaning of `../` on a farm branch is left to yaks-73f4's design.

---
▸ 2026-10-18T19:59:19Z [delta-farm]

moved: shaving -> shorn
