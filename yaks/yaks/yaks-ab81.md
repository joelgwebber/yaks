---
id: yaks-ab81
title: Checkpoint notes verify the commit is pushed (runners lost work twice to local-only checkpoints)
status: shorn
type: task
priority: 2
created: '2026-11-01T22:53:27Z'
updated: '2026-11-18T15:00:35Z'
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

#### 2026-11-18T03:02:47Z

*sources-triage*

Folded into  (`yaks update --commit` checks the commit is pushed), per Joel's answer on yaks-3515.

#### 2026-11-18T03:03:15Z

*sources-triage*

Folded into yaks-f009 (`yaks update --commit` checks the commit is pushed), per Joel's answer on yaks-3515.

#### 2026-11-18T14:56:45Z

*sources-triage · kind: landed · commit: 65fa57e4ab9db0590d17167ea13e8db902d5bca4 · source: a82a78a14f08c244967d4ace8e85cadf4c7ec776 · branch: main*

landed: 65fa57e4 yaks-f009: land writes host commit links; update --commit records a pushed commit (also yaks-ab81)

#### 2026-11-18T15:00:34Z

*sources-triage*

Done in yaks-f009 (65fa57e4): `yaks update <id> --commit [<rev>]` refuses a commit the remote lacks as last fetched, naming the push command; warns with no remote. The skills' checkpoint convention now uses --commit.

#### 2026-11-18T15:00:35Z

*sources-triage · kind: move · from: hairy · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shorn
