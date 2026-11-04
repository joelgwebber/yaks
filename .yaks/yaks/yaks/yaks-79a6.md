---
id: yaks-79a6
title: 'Acceptance: shrink review-watch onto the scheduling contract'
status: hairy
type: task
priority: 2
created: '2026-11-04T22:31:57Z'
updated: '2026-11-04T22:31:57Z'
parent: yaks-7fa6
depends_on:
- yaks-b569
tags:
- skills
---

Part of the scheduling arc ([design](/doks/design/scheduling.md)). The test of whether the guidance is done.

In Joel's `joel` repo, `review-watch` drops launchd, `osascript`, `nudge` and its own `tick`, uses `create --key` for intake, and registers one recurring yak ("run the review-watch scan, triage what it made", with `wake` per the routing yak). Its `SKILL.md` should fit in about 20 lines and mostly point at `yaks-scheduling`. If it can't, the guidance has a gap: fix that, not the skill.

The skill lives in Joel's repo, uncommitted in a Delta worktree as of 2026-11-04 (`fullstory-b446` tracks it); coordinate with Joel before touching it.
