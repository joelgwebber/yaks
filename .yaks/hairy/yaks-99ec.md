---
id: yaks-99ec
title: 'Cross-machine Delta threads: worktree discovery when ''local'' is a managed bare repo (or a different path per machine)'
type: task
priority: 2
created: '2026-10-04T16:47:57Z'
updated: '2026-10-04T20:44:44Z'
parent: yaks-b5a0
labels:
- cli
- delta
- agent
---

Source: yaks-df61 O50. A Delta thread shared to another machine gets managed bare clones with 'local' = a Delta-managed bare repo (origin/main only, plus refs/delta pins), not the user's checkout. Findings and open questions are in O50. Questions: (1) how should yaks find sibling worktrees/checkouts when local is bare, absent, or at a different path per machine (candidates: refs/delta pins, git worktree list, the .delta/worktrees convention, YAKS_DIR)? (2) what do preflight/path/discover say there? (3) what do yaks-coordinating-delta and land.sh assume about 'local' (a real checkout that accepts pushes)? (4) what carries farm state and archive branches between machines (only origin crosses; never push origin without the human)? Relates to yaks-38dc (lanes view), yaks-70e5 (yaks diff), yaks-814a (init --mode). Principle: every facility exists as a CLI command, and every tool is worktree-aware in both senses (git worktrees, Delta clones). Next step: a short design discussion with Joel before coding; possibly a Delta-side feature request.

---
▸ 2026-10-04T19:31:09Z [delta-lead]
Design fork (coordinator delta-lead). Evidence: yaks-df61 O50-O53, probe yaks-f49c. On a shared-thread machine the thread's clones have a bare managed 'local' with NO refs/heads; a worker's 'local' is that bare repo even when the coordinator's was repointed by hand; 'git push local <branch>:main' there is accepted and lands where the human never looks. (1) yaks-level discovery: add a CLI command (working name 'yaks worktrees') that lists sibling checkouts of this repo from git alone: 'git worktree list' for plain worktrees; for Delta, the clone's objects/info/alternates -> the repo holding refs/delta/<dir>/<repo>/<sha> pins, resolved to <root>/<dir>/<repo> when the directory exists (both layouts, O53); shows path, HEAD, dirty, and the shaving yaks of that checkout; the TUI lanes view (yaks-38dc) and 'yaks diff' (yaks-70e5) then consume it. My lean: YES, git-only, never read Delta's database. (2) The skills stop assuming 'local' is a checkout: before pushing the human's main, the coordinator checks that 'local' is non-bare and has refs/heads/main (e.g. 'git ls-remote --heads local main' non-empty); if not, it never pushes main: it pushes 'pr/<name>' to local and tells the human the 'git fetch <path> pr/<name>' command, or asks them to repoint local. My lean: YES, and I will encode the check in land.sh/a 'yaks preflight' item rather than prose. (3) Delta-side: a request to bind a shared thread to the receiving machine's checkout is a separate feature request for Delta, not blocking this. My lean: you decide whether and where to file it; we do not wait. Questions for you: (a) answer (1)/(2) yes/no or an alternative; (b) is repointing 'local' by hand (what you did in the yaks worktrees) the workflow you want us to support, or only to survive?

---
▸ 2026-10-04T20:43:00Z [Joel Webber]
1. If we can get away with just git worktrees, that's awesome.

---
▸ 2026-10-04T20:43:45Z [Joel Webber]
2. That makes sense. We have to be resilient to the delta workflow and environment as well.

---
▸ 2026-10-04T20:44:44Z [Joel Webber]
3. Yeah, it would be nice to get better connections throughout delta, but I think that's a ways off.
