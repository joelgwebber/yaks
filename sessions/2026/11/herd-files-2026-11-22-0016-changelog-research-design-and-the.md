---
type: session-review
title: 'Changelog: research, design and the release commands'
source: session-note
actor: herd-files
person: Joel Webber
host: joel
harness: delta
thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-22
written: '2026-11-22T00:16:07Z'
good: 2
partial: 1
---

## Summary

Joel asked for research and design on yaks-a4d8, the changelog yak he had parked for weeks, and then said "LFG". The research ([/doks/research/changelogs.md](/doks/research/changelogs.md)) surveyed Keep a Changelog, Changesets, towncrier, reno, git-cliff and GitHub's generated notes. It concluded that a shorn yak already is a fragment with depth attached. What was missing was a line written for a reader and a computed release membership. The design ([/doks/design/changelog.md](/doks/design/changelog.md)) settled seven decisions with Joel over two rounds of comments:

- a `kind: change` note at shear;
- membership computed from commit ranges, reno-style, never stored;
- `yaks release changes|dok|changelog`;
- a release dok per version with a generated block;
- `CHANGELOG.md` as a per-branch projection;
- pinned images that follow `@rev`;
- a best-effort backfill.

Five Sonnet workers built the pieces (yaks-bcb8, yaks-c955, yaks-045c, yaks-c708, yaks-7278), and I did the backfill (yaks-3696). Every tag now has a release dok, and v0.0.41's is written with headlines. Joel's LFS question turned into a probe. LFS images at a commit work locally (smudge) and on GitHub via `github.com/<o>/<r>/raw/<sha>/<path>`, but not via `raw.githubusercontent.com`.

## Turns

- good: the opening discussion. My first reply mapped Joel's six points onto existing tool families and proposed the change line, computed membership and the projection. He agreed and sharpened it: the branch point (changelog per branch from a branchless farm), the shear-note question and the "what came of a dok" archaeology.
- good: "Sounds like a plan. LFG!" Research dok with six sources, design dok, eight yaks filed, then four batches of workers landed and reproduced, and the backfill done.
- partial: the LFS render test page for Joel's browser check is on origin, but his verdict on relative image links is still pending.

## Lessons

- A cheap worker's feature passed its tests and still had a hole only a real run showed, twice. `shorn --change` on an already-shorn yak dropped the line silently (fixed in c20ad225). `release changelog --release` planned all 40 tags to print one (fixed in 6ae40a60). Reproducing each landing with my own build is the review, as AGENTS.md says.
- Change lines written right after the work are cheap. Writing the ten for this session's own yaks took one command each, because I knew what shipped. For the 500 older yaks it would be archaeology. That is the case for asking at shear, not at release.
- `GIT_TRACE=<file>` with a count of `built-in:` lines finds git-spawn hot spots in seconds. It found both the planning cost and the per-link `web_url` cost (yaks-bfb2).
- Headless TUI navigation by counting Tab presses is fragile. Workers and I each scripted a loop that searches for the right count via the frame's `doc=` / `viewing` line. A "follow link by text" headless action would save that (two workers reported the same friction).
- `yaks commit` sometimes errors "git commit failed ... nothing to commit" right after an autocommit: filed yaks-879e.

## Saved

- Doks: [/doks/research/changelogs.md](/doks/research/changelogs.md), [/doks/design/changelog.md](/doks/design/changelog.md), `doks/releases/v0.0.1.md` to [v0.0.41](/doks/releases/v0.0.41.md), `schemas/release.md`.
- Sources: six, cited by the research dok.
- Yaks: shorn yaks-a4d8 with its family (yaks-bcb8, yaks-c955, yaks-045c, yaks-c708, yaks-7278, yaks-3696); filed yaks-bfb2, yaks-6554, yaks-879e, and yaks-bda5 and yaks-c059 under yaks-64c4. Change lines written for this session's ten earlier yaks and `--no-change` for 21 release yaks.
- Code: landed on local main d2a9936c through 6ae40a60.
