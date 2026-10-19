---
id: yaks-5cea
title: Lookup-by-id should find cross-shed yaks
status: dead
type: bug
priority: 2
created: '2026-10-13T21:21:42Z'
updated: '2026-10-19T04:26:22Z'
labels:
- sheds
- cli
- tui
---

At present it only finds yaks if they're *also* in the cwd's shed. If they're net-new in an unmerged shed, you can't find them by id.

---
▸ 2026-10-19T04:26:21Z [delta:herd-pass]

Obviated by one farm per machine (yaks-6a34, yaks-73f4): every shed reads and writes the same farm worktree, so there are no per-shed copies to look up, merge, steer or lose; LFS-tracked attachments no longer travel through code merges (the orphan farm branch has no .gitattributes). Slaughtered in the 2026-10-19 herd pass; revive if it recurs under the new model.

---
▸ 2026-10-19T04:26:22Z [delta:herd-pass]

moved: hairy -> dead
