---
id: yaks-2c22
title: 'Process rails: yaks brief (worker brief from a yak) and yaks preflight (landing checks)'
type: feature
priority: 2
created: '2026-10-03T21:13:29Z'
updated: '2026-10-05T22:44:37Z'
parent: yaks-b5a0
labels:
- cli
- skills
- agent
needs: human
---

Prose rules in the skills have not kept agents on the rails (yaks-df61 O5, O6, O9). Candidate commands that take the checklist out of prose: (1) yaks brief <id> --as <actor>: print a worker brief from the yak itself (id, how to get a binary, actor, scope from notes, evidence contract = verify command or required attachments, forbidden moves, commit and return format); (2) yaks preflight [<id>...]: before landing, check each shorn yak has its verify last PASS, no untracked or unstaged files under .yaks/ (evidence not staged), the yak file and code are in the same commit, and the commit message names the id; exit non-zero with one actionable line per failure. Design test (yaks-coordinating): both are expressible as queries over files plus git status, so they may live in the tool; anything that assumes Delta or worktrees stays in the skill. Open: whether these generalise beyond Delta, and whether brief should be a template skill instead. Decide with Joel before building.

---
▸ 2026-10-03T22:48:20Z [delta-lead]
Design fork: build 'yaks brief <id>' and 'yaks preflight' in the tool, or keep the checklist in a skill? My lean: build 'preflight' first (pure file + git-status queries: verify last PASS, nothing untracked or unstaged under .yaks/, duplicate status dirs, yak move and code in one commit, id in the message), since trials 1-3 showed agents and coordinators miss exactly those checks and doctor already catches duplicate status dirs. Defer 'brief' until the skills restructure (yaks-5c9f) settles what a brief contains, because the template is still changing. Alternative: skill-only. Answer: preflight now / brief later / skill-only.

---
▸ 2026-10-03T23:19:12Z [Joel Webber]
Agreed on both. The preflight gate will be a big help.

---
▸ 2026-10-05T22:44:37Z [delta-lead]
Design fork before I brief a worker (coordinator delta-lead). You agreed on 2026-10-03 to build `yaks brief <id>`; what is open is WHERE the template lives.

Today the fill-in worker brief is prose in `.agents/skills/yaks-coordinating-delta/SKILL.md`. Building the command puts a second copy in the binary, so one of them has to be the source of truth.

## Options
- **A. The binary owns the yak-derived core, the skill owns orchestration.** `yaks brief <id> --as <name>` prints what is derivable from the yak and the farm:
  - the yak id and title, "already `shaving`", the `YAKS_ACTOR=<name>` prefix rule
  - how to get a binary, scope pointers (read `yaks show`), the evidence contract (the yak's `verify:` command, required attachments)
  - forbidden moves and how to finish, chosen by farm mode (team: stage `yaks path <id>` plus artifacts; private: no yak ids in commits)
  - the `Decision:` note rule, the multi-line markdown rule, and the final-message format
  - `--yaks-dir <path>` adds the `YAKS_DIR=` line for private/pointer farms
  The Delta skill's template shrinks to "run `yaks brief`, then add the spawn title and anything environment-specific", so the two cannot drift.
- **B. The skill stays the only template**; `yaks brief` prints it from a copy embedded at build time (same text, generated). Single text, but it ties the binary to the optional coordination skills.
- **C. Do not build it**; keep the template in the skill and rely on the cold-read gate.

## Lean
A. It follows the design test in `yaks-coordinating` section 2: what is true of working a given yak is a query over the farm and belongs in the tool; spawning, titles and landing are orchestration and stay in the skill. It also fixes the failure the observation log keeps showing (O5, O6, O9): a coordinator forgets a clause when it fills a template by hand.

Cost: a user-visible command and a stable-ish output format, and the Delta skill must be edited to point at it. Say A, B, C or your own variant.
