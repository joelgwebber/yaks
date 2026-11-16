---
id: yaks-79a6
title: 'Acceptance: shrink review-watch onto the scheduling contract'
status: shorn
type: task
priority: 2
created: '2026-11-04T22:31:57Z'
updated: '2026-11-05T04:42:41Z'
parent: yaks-7fa6
depends_on:
- yaks-b569
tags:
- skills
---

Part of the scheduling arc ([design](/doks/design/scheduling.md)). The test of whether the guidance is done.

In Joel's `joel` repo, `review-watch` drops launchd, `osascript`, `nudge` and its own `tick`, uses `create --key` for intake, and registers one recurring yak ("run the review-watch scan, triage what it made", with `wake` per the routing yak). Its `SKILL.md` should fit in about 20 lines and mostly point at `yaks-scheduling`. If it can't, the guidance has a gap: fix that, not the skill.

The skill lives in Joel's repo, uncommitted in a Delta worktree as of 2026-11-04 (`fullstory-b446` tracks it); coordinate with Joel before touching it.

---
▸ 2026-11-05T04:42:40Z [sched-1]

Done. `review-watch` (Joel's `joel` repo, uncommitted in the Delta worktree) rewritten onto the `yaks-scheduling` contract.

- `SKILL.md`: 58 lines to 29, and it now mostly points at `yaks-scheduling`. The gap check passed: nothing in the guidance had to be invented, and the only scheduling text left is the one `yaks create ... --every P1D --wake agent` command that registers the recurring yak.
- `scripts/review_watch.py`: 262 lines to 176. Gone: `nudge` (osascript), `tick` (a second one), `launchd` (plist printing), `plistlib`, the date math (it passes `--due +2d`). Intake is `create --key <PR url> --json`, which also makes a yak already holding the URL in any state count as tracked rather than duplicated.
- Evidence: a stubbed `gh` on a scratch farm (not Joel's): scan 1 made one yak, scan 2 changed nothing, a hand-sheared yak stayed shorn while still requested, a merged PR cleared with a note, a re-request regrew it with a fresh `due:`, a dead yak was not recreated (one yak total throughout), and `yaks tick` printed one report. `scan --dry-run` against the real repos and farm listed 6 new, 1 cleared, 5 tracked, as expected. I did not run a real `scan`, and did not create the recurring yak, in Joel's farm.

Not done: the skill can't be loaded through the `skill` tool this turn (the catalog is fixed at turn start), so I checked its frontmatter by script instead. The first real use is the cold read.

---
▸ 2026-11-05T04:42:41Z [sched-1]
kind: move
from: hairy
to: shorn
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shorn
