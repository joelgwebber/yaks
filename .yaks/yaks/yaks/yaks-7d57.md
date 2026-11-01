---
id: yaks-7d57
title: 'Trial: cheaper-model workers follow the sync, heartbeat, actor and branch-checkpoint conventions'
status: hairy
type: task
priority: 2
created: '2026-10-31T15:57:06Z'
updated: '2026-11-01T01:10:23Z'
parent: yaks-039b
depends_on:
- yaks-0071
labels:
- skills
- agent
---

Joel (2026-10-31): before building tooling such as `yaks checkpoint`, run experiments with workers on cheaper models (e.g. Sonnet) and see whether they follow the conventions by hand.

## What to observe

- Do they `yaks sync` after every batch of yak writes, especially on a cloud runner or another machine?
- Do they write heartbeat notes often enough for `stale` to mean dead?
- Do they set `YAKS_ACTOR`, so each thread writes under one stable name?
- Do they push their worker branch to `origin` and note the branch and sha, in the order push, record, sync?

## Shape

- At least one run on a Delta cloud runner and one local, a few workers each, on real yaks.
- Score each convention per worker from the farm and the remote afterwards: `yaks log --by <worker>`, sync times against write times, branches on origin against the shas the notes name.
- Record a `Decision:` per convention: rule is enough, or needs a tool.

---
▸ 2026-10-31T16:55:10Z [yakherd]

First observations from the 2026-10-31 local run (three LiteLLM Sonnet workers) are on `yaks-039b`'s wave-2 note. Short version: stable `YAKS_ACTOR` names everywhere; notes at milestones rather than on a clock; sync untested (same machine). A cloud-runner run is still needed for the sync and checkpoint conventions.

---
▸ 2026-10-31T17:12:34Z [yakherd]

Pre-trial finding (`yaks-4728` end-to-end check): the obvious branch naming `yaks/<worker>/...` fails outright beside a branch farm. The skills now warn and `update --meta branch=` warns, so the trial should check that workers pick a different prefix.

---
▸ 2026-11-01T01:10:23Z [yakherd]

Trial run started by Joel, 2026-11-01: memcraft cloud runner thread `ksQQOn_lPggNQrKkBKNvXOzaN5LOADYRe5IBxBCA0aLJyrhJPreUvVwLftHG`, new yaks `mem-4fcd`, `mem-f4c5`, `mem-f5cf`.

Confounds to read results against:

- **Old skills.** memcraft commits its skills at 0.0.24 (`yaks skills status` there says stale). The runner loads skills from the repo, not the binary, so it is following the pre-`yaks-0071` rules: no `YAKS_ACTOR` rule, sheds described as checkouts, sync only at milestones. Project-local skills are never auto-synced. Its binary comes from `npm i -g @j15r/yaks`, so 0.0.27 after the wipe.
- **The runner was wiped a third time** at the start of this run. Farm writes made before its next `yaks sync` are lost with each wipe.
- **Bootstrap.** The farm worktree on the runner is made by hand in `scripts/runner-setup.sh`, because yaks refuses to on a bare host (filed separately).
