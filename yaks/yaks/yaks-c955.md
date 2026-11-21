---
id: yaks-c955
title: '@rev links read LFS files: smudge the pointer so a pinned image shows'
status: shaving
type: bug
priority: 2
created: '2026-11-21T21:20:59Z'
updated: '2026-11-21T21:48:55Z'
parent: yaks-a4d8
tags:
- links
- images
- git
---

Decision 6 of [the changelog design](/doks/design/changelog.md#6-images-following-vs-pinned). `links::read_at` runs `git show <rev>:<path>`, which for an LFS-tracked file returns the 131-byte pointer. Detect a pointer and pass it through `git lfs smudge -- <path>` (probe: [lfs-probe.md](/yaks/yaks/yaks-a4d8/lfs-probe.md)). Through the one opener; check the TUI image viewer and `yaks peek` on a pinned image.

#### 2026-11-21T21:35:47Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-21T21:35:48Z

*herd-files*

Claim (herd-files): worker lfs-1 (Sonnet). Scope: src/links.rs `read_at` (and a binary-safe sibling if images are read as bytes elsewhere: find where the TUI image viewer and `yaks peek` read @rev files). Out of scope: anything else. Evidence: a test with a real LFS pointer in a temp repo (skip if git-lfs missing), and a TUI headless/peek run on this farm's pinned image. Judge: herd-files at landing.

#### 2026-11-21T21:41:36Z

*lfs-1*

Evidence link (pinned LFS image): ![manifesto page 2](/sources/2026-11-05-0272f64e-proton-engineering-manifesto-page-2-of-3/manifesto-2.png@33f5db7f)

#### 2026-11-21T21:45:42Z

*lfs-1 · kind: verify · result: pass*

verify: `cargo test --workspace` -> PASS (exit 0)

[peek-evidence](yaks-c955/peek-evidence.txt "peek of the pinned LFS image before (pointer) and after (170083-byte binary placeholder, matches git lfs smudge)")

[tui-pinned-image-frame](yaks-c955/tui-pinned-image-frame.txt "headless TUI: following the @33f5db7f link in this yak opens the real image in the image viewer (was: text pane of the pointer)")
