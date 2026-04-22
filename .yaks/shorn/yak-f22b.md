---
id: yak-f22b
title: Revisit how we handle yak commits and hashes
type: task
priority: 1
created: '2026-04-22T18:08:56Z'
updated: '2026-04-22T19:53:17Z'
---

In most cases, we're committing yaks separately from the changes they're associated with. I think this may stem from some guidance we have about adding commit hashes to the yak itself, which creates a circular dependency. Let's toss that concept and just give strong guidance that a yak should be committed alongside the changes that complete it, whenever humanly possible.

### 2026-04-22T19:52:55Z
Dropped auto-stamp of git HEAD on shorn (cmd_shorn + TUI quick-adjust). Removed --commit flag and git_head_short() helper. Updated guidance in SKILL.md, _YAKS_MANDATE, commands/shorn.md, CLAUDE.md, and README.md: shorn when work is done; when using git, commit shorn yak alongside the code.
