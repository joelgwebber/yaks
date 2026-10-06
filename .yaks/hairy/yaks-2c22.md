---
id: yaks-2c22
title: 'Process rails: yaks brief (worker brief from a yak) and yaks preflight (landing checks)'
type: feature
priority: 2
created: '2026-10-03T21:13:29Z'
updated: '2026-10-06T03:40:48Z'
parent: yaks-b5a0
labels:
- cli
- skills
- agent
---

Prose rules in the skills have not kept agents on the rails (yaks-df61 O5, O6, O9). Candidate commands that take the checklist out of prose: (1) yaks brief <id> --as <actor>: print a worker brief from the yak itself (id, how to get a binary, actor, scope from notes, evidence contract = verify command or required attachments, forbidden moves, commit and return format); (2) yaks preflight [<id>...]: before landing, check each shorn yak has its verify last PASS, no untracked or unstaged files under .yaks/ (evidence not staged), the yak file and code are in the same commit, and the commit message names the id; exit non-zero with one actionable line per failure. Design test (yaks-coordinating): both are expressible as queries over files plus git status, so they may live in the tool; anything that assumes Delta or worktrees stays in the skill. Open: whether these generalise beyond Delta, and whether brief should be a template skill instead. Decide with Joel before building.

---
▸ 2026-10-03T22:48:20Z [delta-lead]
Design fork: build 'yaks brief <id>' and 'yaks preflight' in the tool, or keep the checklist in a skill? My lean: build 'preflight' first (pure file + git-status queries: verify last PASS, nothing untracked or unstaged under .yaks/, duplicate status dirs, yak move and code in one commit, id in the message), since trials 1-3 showed agents and coordinators miss exactly those checks and doctor already catches duplicate status dirs. Defer 'brief' until the skills restructure (yaks-5c9f) settles what a brief contains, because the template is still changing. Alternative: skill-only. Answer: preflight now / brief later / skill-only.

---
▸ 2026-10-06T03:40:48Z [Joel Webber]
I'm good with putting these into the tools now. Can't always trust those sub-agents.
