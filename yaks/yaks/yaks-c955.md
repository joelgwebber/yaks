---
id: yaks-c955
title: '@rev links read LFS files: smudge the pointer so a pinned image shows'
status: hairy
type: bug
priority: 2
created: '2026-11-21T21:20:59Z'
updated: '2026-11-21T21:20:59Z'
parent: yaks-a4d8
tags:
- links
- images
- git
---

Decision 6 of [the changelog design](/doks/design/changelog.md#6-images-following-vs-pinned). `links::read_at` runs `git show <rev>:<path>`, which for an LFS-tracked file returns the 131-byte pointer. Detect a pointer and pass it through `git lfs smudge -- <path>` (probe: [lfs-probe.md](/yaks/yaks/yaks-a4d8/lfs-probe.md)). Through the one opener; check the TUI image viewer and `yaks peek` on a pinned image.
