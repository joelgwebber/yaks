---
id: yaks-5c9f
title: Restructure coordinator/worker skills by farm mode (public / private / out-of-tree)
type: task
priority: 2
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T23:04:20Z'
parent: yaks-b5a0
depends_on:
- yaks-80d5
labels:
- skills
needs: human
---

Replace today's mixed public/private, harness-agnostic prose with a small shared core (claim, lanes, evidence, ask/answer) plus one coordinator + worker pair per environment (git worktrees; Delta), each followable without reading the others. Fold in pending guidance from yaks-0a87 and yaks-75cc. Embed and test-guard them like the yaks skill.

---
▸ 2026-10-03T20:27:55Z [delta-agent]
Retitled per b5a0 direction: split by farm mode, not environment; Delta is a separate layered skill (yaks-2f9b).

![restructure-plan](artifacts/yaks-5c9f/restructure-plan.md)

---
▸ 2026-10-03T23:02:44Z [delta-lead]
Restructure plan: target skill set (core + team + private + worktrees + delta), section-by-section map of today's 386-line yaks-coordinating, contradictions it closes, guards, proof plan (trial 4), order of work, and two forks.

---
▸ 2026-10-03T23:02:44Z [delta-lead]
Two forks, both with a lean (details in restructure-plan.md section 9). (1) Layering: A = core + two mode skills (team, private) + two environment skills (worktrees, delta), a reader loads core + one mode + one environment; B = two standalone mode skills with the git-worktree mechanics folded in, Delta layered on top (the original phrasing). Lean A: B makes every Delta reader load worktree material that does not apply and duplicates the shared core. (2) Shipping: embed any of these in the binary, or keep them repo-internal? Lean: internal until trial 4 shows they work for someone who is not us. I am drafting core + team + private now under the artifact dir (fork-independent), nothing into .agents/skills until you answer.

![draft-yaks-coordinating-core](artifacts/yaks-5c9f/draft-yaks-coordinating-core.md)

![draft-yaks-coordinating-team](artifacts/yaks-5c9f/draft-yaks-coordinating-team.md)

![draft-yaks-coordinating-private](artifacts/yaks-5c9f/draft-yaks-coordinating-private.md)

---
▸ 2026-10-03T23:04:20Z [delta-lead]
Progress: plan attached; drafts of core (111 lines), team (64) and private (57) attached as artifacts, kept out of .agents/skills until the forks are answered. Cold-read check running (a fresh scout with only the three drafts answers 8 scenarios). Next: worktrees and delta environment skills, then yaks-working fold-in (0a87, 75cc), guards, trial 4.
