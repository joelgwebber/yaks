---
id: yak-378c
title: Update skill guidance to delineate gitignored / checked-in workflows
type: task
priority: 1
created: '2026-06-24T17:57:25Z'
updated: '2026-06-24T18:07:53Z'
---

I've seen agents get confused about checking in yaks, even when .yaks is gitignored. Let's be more explicit about the fact that there are two workflows -- one local, one team.
Then we can also use this as a basis for other advice, like *not* mentioning yaks anywhere external, when on the local-only workflow.

---
▸ 2026-06-24T18:07:50Z
Delineated the two workflows everywhere agents read. skills/yak/SKILL.md: new 'Two workflows: local or team' section (detect via git check-ignore/ls-files; default local-only when ambiguous) + made hard rule 2 mode-aware (commit shorn yak in team mode, never commit in local-only). Local-only guidance explicitly forbids leaking yak IDs into commits/PRs/comments/external trackers. Mirrored the mode-aware commit rule into the injected mandate (_YAKS_MANDATE in commands.py) and both README copies (manual mandate block + intro framing + new 'Local or team' How-it-works bullet). No code-path change; 113 tests pass, JSON valid. Bumped plugin 0.1.74->0.1.75.
