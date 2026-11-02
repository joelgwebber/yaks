---
id: yaks-ab81
title: Checkpoint notes verify the commit is pushed (runners lost work twice to local-only checkpoints)
status: hairy
type: task
priority: 2
created: '2026-11-01T22:53:27Z'
updated: '2026-11-01T22:53:27Z'
tags:
- agent
- cli
---

Found observing the memcraft runners across a cloud → local → cloud switch (2026-11-01, yaks-7d57). Work was lost twice because checkpoints never reached `origin`:

- paint-2 and menu-1 died with the runner; neither had a `wip/` branch on origin, so their work was rebuilt from scratch (paint-3, menu-2).
- The replacements repeat it: paint-3's note says "Checkpoint commit local"; menu-2's wip commit `6762cae` is on no remote branch.
- None of their notes carry `kind: checkpoint`, so `yaks sheds` shows no branch for them and the coordinator had nothing to recover from.

The skills already say "commit, push your work branch, note it, then sync, in that order". Agents still skip the push. Lean: make the tool check it. `yaks update --meta kind=checkpoint --meta commit=<sha> --meta branch=<b>` verifies the commit is reachable from `<remote>/<branch>` (a `git branch -r --contains` after a fetch of that branch) and refuses, naming the push command, when it isn't. `yaks sheds` could also mark a checkpoint whose commit is not on the remote.

Open: how to handle no network or no remote (warn rather than refuse?).
