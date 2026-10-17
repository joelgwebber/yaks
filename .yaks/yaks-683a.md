---
id: yaks-683a
title: Git LFS merge issues across shed boundaries
status: hairy
type: bug
priority: 3
created: '2026-10-17T18:42:13Z'
updated: '2026-10-17T18:42:13Z'
labels:
- sheds
- git
---

This came up in another thread recently, when the coordinator was trying to merge yaks from a worker. Some of the screenshots got picked up by git-lfs (which was expected for this repo). This broke its ability to merge them cleanly, so they're broken for the coordinator until the file gets copied over.

> **One fix during the merge:** the worker's three screenshots are Git LFS files, and the merge couldn't fetch them. I copied them from the worker's clone; the checksums match. When this branch is pushed, include the LFS objects too (`git push` normally does this, but this clone has never pushed LFS files).
