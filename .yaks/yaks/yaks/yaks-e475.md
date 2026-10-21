---
id: yaks-e475
title: answer <id>@<shed> can be lost when the worker syncs with git before committing it
status: dead
type: bug
priority: 2
created: '2026-10-12T16:04:58Z'
updated: '2026-10-19T04:26:22Z'
parent: yaks-ee0a
labels:
- cli
- delta
---

Seen 2026-10-12 on yaks-ed5e: `yaks answer <id>@<shed>` writes the answer into the shed's working tree and leaves it uncommitted (by design: the worker commits it with its yak file). The worker then had to sync (`git fetch local && git merge local/main`) and the uncommitted answer was lost: the landed yak file has the ask, no answer, no pickup, and still `needs: human`, so it showed in Joel's inbox after it was shorn.
Any cross-shed write that is left uncommitted is exposed to whatever the worker does next with git (merge, checkout, reset, stash). This matters more if a second cross-shed write (`note @shed`, guidance) is added.
Options to weigh: the brief tells a woken worker to commit or pickup the answer before any git operation; `pickup` refuses or warns when the yak file has uncommitted cross-shed entries that are not yet in HEAD; the answer is written somewhere git does not touch until pickup (breaks one-file-per-yak); or the cross-shed write commits itself on the shed's branch (rejected earlier because the worker may be mid-edit). Also: `yaks doctor`/`preflight` could flag a shorn yak that still has `needs:` set.

---
▸ 2026-10-19T04:26:21Z [delta:herd-pass]

Obviated by one farm per machine (yaks-6a34, yaks-73f4): every shed reads and writes the same farm worktree, so there are no per-shed copies to look up, merge, steer or lose; LFS-tracked attachments no longer travel through code merges (the orphan farm branch has no .gitattributes). Slaughtered in the 2026-10-19 herd pass; revive if it recurs under the new model.

---
▸ 2026-10-19T04:26:22Z [delta:herd-pass]

moved: hairy -> dead
