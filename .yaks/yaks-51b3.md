---
id: yaks-51b3
title: Parent/child references in yak details broken across sheds
status: dead
type: bug
priority: 2
created: '2026-10-17T14:03:33Z'
updated: '2026-10-19T04:26:22Z'
labels:
- tui
- sheds
- cli
---

When yaks are created and statuses are updated across sheds, parent and child references in the detail view can get broken. I've seen a few different cases that are a little hard to quickly characterize, so we'll probably need to research a taxonomy of failure modes to deal with.

As always (and per yaks-991c), please ensure all lib/CLI affordances take these issues into account in their own interfaces.

---
▸ 2026-10-19T04:26:21Z [delta:herd-pass]

Obviated by one farm per machine (yaks-6a34, yaks-73f4): every shed reads and writes the same farm worktree, so there are no per-shed copies to look up, merge, steer or lose; LFS-tracked attachments no longer travel through code merges (the orphan farm branch has no .gitattributes). Slaughtered in the 2026-10-19 herd pass; revive if it recurs under the new model.

---
▸ 2026-10-19T04:26:22Z [delta:herd-pass]

moved: hairy -> dead
