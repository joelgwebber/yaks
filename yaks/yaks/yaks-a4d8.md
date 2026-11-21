---
id: yaks-a4d8
title: Yaks changelog
status: shaving
type: feature
priority: 3
created: '2026-10-10T13:49:16Z'
updated: '2026-11-21T21:36:36Z'
---

Add tools for rolling up information about recent changes, for an agent to boil down into a changelog.

Specify the workflow in the yaks skills; make sure it's shed-aware.

#### 2026-11-21T15:51:23Z

*sched-1*

For some other projects, we've been using an existing "changelog" tool (I forget which, but I think there are a few). Before deciding on a specific path for yaks, let's do a little research on the ones already out there, and decide whether to adopt an existing one (and just add tools to feed it from yak history), or just roll one directly into the yaks tooling.

#### 2026-11-21T21:14:23Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-21T21:21:10Z

*herd-files*

Decision (Joel, 2026-11-21): build it into yaks rather than adopt Changesets/towncrier, taking their conventions: a `kind: change` note written at shear, release membership computed from commit ranges (reno's model, never stored), a release dok per tag with a generated block, and CHANGELOG.md as a per-branch projection. Research: [/doks/research/changelogs.md](/doks/research/changelogs.md) (six sources saved, LFS-at-commit probe attached as [lfs-probe.md](yaks-a4d8/lfs-probe.md)). Design: [/doks/design/changelog.md](/doks/design/changelog.md). Pieces filed as children: yaks-bcb8 (change lines), yaks-045c (release changes + check), yaks-c708 (release doks), yaks-7278 (CHANGELOG.md), yaks-c955 (LFS @rev), yaks-3696 (backfill). Archaeology follow-ups under yaks-64c4: yaks-bda5, yaks-c059.

#### 2026-11-21T21:35:17Z

*herd-files*

Decision (Joel, 2026-11-21): the commands are `yaks release changes|dok|changelog` (not `yaks changes`); the TUI shear pre-fills the change line with the yak's title for the human to edit.

#### 2026-11-21T21:36:36Z

*herd-files*

Batch 1 (herd-files, base 412882e4): change-1 on yaks-bcb8 (claude-sonnet-5-5, thread ksQQ7I2z8nJeT6a6hDJ07WHX45LOAHb1KJIBxBCA0aLJyrhJPreUvVwLftHG); lfs-1 on yaks-c955 (claude-sonnet-5-5, thread ksQQ7I2z8nJeT6a6hDJ07WHX45LOAHb29ZIBxBCA0aLJyrhJPreUvVwLftHG). Next: yaks-045c after bcb8 lands. LFS render test page for Joel's browser check: yaks-a4d8/lfs-render-test.md (farm 1d0a4386, not yet synced).
