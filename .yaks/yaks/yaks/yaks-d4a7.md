---
id: yaks-d4a7
title: preflight git-state fails on a farm branch's normal uncommitted state; check the farm is committed (or skip) instead of asking to git add
status: shaving
type: bug
priority: 2
created: '2026-10-19T04:30:51Z'
updated: '2026-10-21T20:33:01Z'
parent: yaks-472e
labels:
- cli
- git
---

Seen at the 0.0.21 landing (2026-10-19): `yaks preflight --push-main` in a code checkout whose farm is on its own branch failed with `.yaks/yaks-a260.md has unstaged changes (git add it)` and `.yaks/yaks-dc63.md is untracked`. Those are the farm worktree's uncommitted yak edits; the code push does not carry them and `git add` is the wrong fix (`yaks commit` is). Rule: when the farm is a farm-branch worktree (`store::farm_branch(root)`), git-state reports "farm has N uncommitted change(s): run `yaks commit`" as a warning (or not at all for `--push-main`, which is about the code branch), never "git add it". Evidence: a scratch repo with a farm branch and a dirty yak; preflight before/after.

---
▸ 2026-10-21T20:33:01Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T20:33:01Z [delta:herd-dirs]

Claimed with yaks-bd1d by `status-1` (same files); spec is on yaks-bd1d.
