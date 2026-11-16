---
platforms: [all]
source: plan.md
follows: 
blocked:
created: 2026-10-05T20:06:03Z
---
# Squash notes

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
docs/monorepo.md and yass-work say what squash merges mean for YASS's rules (a commit that must stand alone needs its own PR), yass-work asks for latitude before pushing or opening a PR, and yass-log names a squash-merged commit's PR. No workflow guidance.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC1–AC6 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `docs/monorepo.md`: a short section on squash merges
- [x] `yass-work`: "apart from code" and the squash-merge sentence; push and PRs need latitude
- [x] `templates/agents.md`: "apart from code" on the intent and archive lines
- [x] `yass-log`: name a squash-merged commit's PR
- [x] Review the diff: no workflow guidance

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The latitude line in the `AGENTS.md` section gets push and pull requests too, not only `yass-work` - the section repeats `yass-work`'s latitude rule, and they should match (claude)
- `yass-log` also says to follow an archived change's `change.md` across its move (`git log --follow`): trying AC5, a plain `git log -- <folder>` on an archived change showed only the archive commit (claude)
- This repo's own `AGENTS.md` and installed playbooks pick the new text up at the next release's `yass upgrade`; upgrading them with a development build would stamp a pseudo-version (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `docs/monorepo.md` gets a "Squash merges" section (apart from code means a pull request apart from code; intent, progress and archive can share one; how to split and stack is the team's call; the hook checks per commit). `yass-work`'s git paragraph asks for latitude to push and open pull requests and says what "separately" means under squash merges; its Finish step says the same for the archive. The `AGENTS.md` template says it on the intent, latitude and archive lines. `yass-log` names the `(#N)` pull request of a squash-merged commit, and follows an archived change's file across its move. e2e §30 (4 checks: the section's text after init, and after `yass upgrade` of an older one). All 390 e2e checks, examples, `tests/site.sh` and `go test` pass. AC5 tried by hand: following the steps for the hooks change names #15 (the decision) and #16 (the code). No workflow guidance added: no stacking tools, merge styles or branching advice in the kit diff.
- Next: Jaime reviews the text (AC1, AC2, AC4, AC6); then piece 2, `delivered-criteria-count`, stacked on this branch.
### 2026-10-09 (claude)
- Did: Jaime reviewed the text and merged it (#31); AC1, AC2, AC4 and AC6 done. The piece is done.
- Next: none here.
