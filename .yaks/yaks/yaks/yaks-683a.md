---
id: yaks-683a
title: Git LFS merge issues across shed boundaries
status: dead
type: bug
priority: 3
created: '2026-10-17T18:42:13Z'
updated: '2026-10-19T04:26:22Z'
labels:
- sheds
- git
---

This came up in another thread recently, when the coordinator was trying to merge yaks from a worker. Some of the screenshots got picked up by git-lfs (which was expected for this repo). This broke its ability to merge them cleanly, so they're broken for the coordinator until the file gets copied over.

> **One fix during the merge:** the worker's three screenshots are Git LFS files, and the merge couldn't fetch them. I copied them from the worker's clone; the checksums match. When this branch is pushed, include the LFS objects too (`git push` normally does this, but this clone has never pushed LFS files).

---
▸ 2026-10-19T04:26:21Z [delta:herd-pass]

Obviated by one farm per machine (yaks-6a34, yaks-73f4): every shed reads and writes the same farm worktree, so there are no per-shed copies to look up, merge, steer or lose; LFS-tracked attachments no longer travel through code merges (the orphan farm branch has no .gitattributes). Slaughtered in the 2026-10-19 herd pass; revive if it recurs under the new model.

---
▸ 2026-10-19T04:26:22Z [delta:herd-pass]

moved: hairy -> dead
