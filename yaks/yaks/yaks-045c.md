---
id: yaks-045c
title: 'yaks release changes: compute a release''s yaks from its commit range, --check for missing change lines; land warns'
status: shaving
type: feature
priority: 2
created: '2026-11-21T21:20:57Z'
updated: '2026-11-21T22:02:42Z'
parent: yaks-a4d8
depends_on:
- yaks-bcb8
tags:
- cli
- release
- git
---

Decisions 2 and 3 of [the changelog design](/doks/design/changelog.md#2-release-membership-is-computed-never-stored). Membership: a commit in the range (previous release tag..tag, on the checked-out branch) names the yak by id or a note's `commit:` (same rule as src/yakcommits.rs). Tags `v*` unless `git config yaks.releaseTags`. Output groups: shorn with a change line, shorn without (gaps; an ancestor's line in the same release covers a child), open/dead yaks named, commits naming no yak. `--json`; `--check` exits non-zero on gaps. `yaks land` warns when it lands a commit naming a shorn yak without a change line. Never stored.

#### 2026-11-21T22:02:42Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
