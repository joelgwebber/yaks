---
id: yaks-b6e3
title: 'session note: report every missing required section at once, not one per run'
status: hairy
type: bug
priority: 3
created: '2026-11-22T23:51:14Z'
updated: '2026-11-22T23:51:14Z'
tags:
- cli
- doks
---

Writing a session note on 2026-11-22, `yaks session note` refused three times in a row:
1. A positional title (`yaks session note "<title>" --body-file f`): it takes only `--title`.
2. `the note needs a ## Summary`.
3. `the note needs a ## Turns section`.

Each run reported only the first problem, so an agent fixes one and runs again. Fix: validate the whole body and list every missing or malformed section (Summary, Turns verdict lines, Lessons) in one error. Optionally accept the title as a positional too, since every other create-style command (`yaks create "<title>"`) does.

Shows it: /sessions/2026/11/herd-files-2026-11-22-2348-v0-0-41-release-and-changelog-link-memo.md
