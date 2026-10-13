---
id: yaks-c41f
title: Support multiple branches per shed
status: hairy
type: feature
priority: 3
created: '2026-10-13T22:04:07Z'
updated: '2026-10-13T22:04:07Z'
labels:
- sheds
---

Explore the workflow when you have sheds performing work on a branch. We'll need to think through the workflows carefully to get this right.

This has come for me recently because I have a top-level Delta thread working on a branch. It hasn't actually pushed the branch to local/origin yet, and so far I still see all the right yak changes in the UI. But I believe that's just a side-effect of the diff happening to work out that way.

The bright-path workflow is basically "I know we're on a worktree, but I also want you committing to a branch", either because we need to create a PR, or just because we need to do a long, isolated arc without affecting anyone else.

I'm not quite sure what the CLI/TUI should show here -- it could be as simple as requiring a branch parameter for the CLI, and offering a branch lens in the shed TUI. Or it could be something more clever I haven't thought of yet.
