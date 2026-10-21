---
id: yaks-78ff
title: Think through yak shed commit/worktree flows
status: dead
type: task
priority: 3
created: '2026-10-17T16:44:57Z'
updated: '2026-10-19T04:26:23Z'
labels:
- sheds
- git
---

There are a number of complex cases that are starting to rear their ugly heads around the relative state of yaks spread across worktrees/sheds, and when they're committed. We need to think these through comprehensively to form a clear taxonomy of how we want each such case to work in practice.

---
▸ 2026-10-17T16:48:59Z [Joel Webber]

# Uncommitted on coordinator; worker shed can't edit

Steps:
- Created a fresh yak on main.
- Spun up a delta thread to work on it.
- Worker diagnosed the issue and formulated a plan, but...

> Your yak-48d6 is only in your primary checkout, so I haven't written these findings onto it.

Relatedly:
> This worktree shows `.yaks/yaks-bc53.md` as modified. I didn't touch it and left it alone.

In both of these cases, the worker was either unable (in the first) or unwilling (the second) to make changes to a yak on its worktree.

---
▸ 2026-10-17T17:20:15Z [Joel Webber]

One other avenue to consider -- to what extent should the yaks tools and TUI be committing yak changes directly?

There are clearly cases where we don't want yaks to automatically commit -- eg on main. But when on a local branch or worktree it's probably net better to aggressively commit, for both history and visibility to other sheds.

Similarly, we might consider doing something similar for merging (of the .yaks only) changes across worktrees. We'll want to think that all through carefully, though -- there could be a lot of weird edge cases to work through.

---
▸ 2026-10-19T04:26:21Z [delta:herd-pass]

Obviated in its main question (the cross-shed state taxonomy) by one farm per machine (yaks-6a34, yaks-73f4). Joel's auto-commit question (2026-10-17 note) survives in a new form, auto-committing the farm branch, and is carried on yaks-4a53 (yaks sync). Slaughtered in the 2026-10-19 herd pass.

---
▸ 2026-10-19T04:26:23Z [delta:herd-pass]

moved: hairy -> dead
