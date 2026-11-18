---
platforms: [all]
source: design discussion with Jaime, 2026-10-04
follows: 
blocked:
archived: 2026-10-09T14:22:20Z
---
# Stacked PRs and squash merges

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
YASS stays compatible with however people land work, and is clear about what squash merges mean for its own rules: a commit that has to stand alone (a plan revision, an archive move, a queue reorder) needs its own PR. `yass-work` asks for latitude before pushing or opening a PR, as it does before committing. A large change needs no closing commit per piece: parent criteria count as done once the pieces delivering them are, and the archive marks them. `yass-log` traces a squash-merged decision to its PR.

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)" for calls that span the whole change. -->
- Its own change, not part of `2026-10-04-monorepos-at-scale-and-plans-repos` - it cuts across inline and separate plans, and is mostly rules, skill guidance and the hook (Jaime)
- The basic rule: YASS-only PRs (PRDs, plans, archives) skip CI; code and its YASS progress land together in PRs that run CI; the large-monorepo docs say so (Jaime)
- Under squash merges, the unit YASS rules apply to is the PR, not the commit - a squash keeps a PR's content and drops its commit boundaries (Jaime, claude) - superseded below
- `merges: squash | commits` in `yass.yaml`, defaulting to `squash` - most orgs squash, and the per-PR rules are the stricter reading; repos that keep merge commits opt out to the per-commit rules (Jaime) - superseded below
- This change takes over counting delivered parent criteria as done (R5, R6) from `2026-10-04-monorepos-at-scale-and-plans-repos` piece 5, which keeps only "pieces own their files" (the rule and the hook's heads-up); its AC12 is dropped there in favour of R5 here (Jaime)
- PRD approved (Jaime, 2026-10-04)
- Drastically simplified (2026-10-05): YASS's rules stay per commit, and the playbooks stay workflow-agnostic. How to branch, stack, sync and merge is the team's workflow, which their harness learns from them (gh-stack ships its own agent skill). Kept: what squash merges mean for YASS's own rules (R2, R13), latitude before pushing or opening PRs (R14), `yass-log` naming a squash commit's PR (R9, now M1), delivered criteria with no closing commit per piece (R5, R6), and an e2e test (R10, now about those). Dropped: R1 (`docs/monorepo.md` already states the basic rule), R3 (checking a PR's range as one squash; per-commit is as strong a check as YASS needs), R4 (the branch look-back, which assumed a workflow and would warn wrongly under merge commits), R7 (branching from main), R8 (PR shapes), R11 (dogfooding), the `merges:` key, and the never-committed R12, R15, R16 (a landing routine, syncing from a worktree, detecting the landing workflow). This also supersedes the PR-as-the-unit and `merges:` decisions above (Jaime)
- Planned in two pieces: the squash notes (M1, text) and delivered criteria counting with its e2e (M2), independent of each other; no design.md, since nothing here is hard to undo (claude)
- "Their own commit" means apart from code, and under squash merges a PR apart from code; intent, progress and archive commits can share a PR. The squash notes say so in `docs/monorepo.md`, `yass-work` and the `AGENTS.md` section (R2, R13, AC1–AC3), so agents don't split a PRD from its plan, or a change's last progress from its archive, into separate PRs as happened here (Jaime)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
