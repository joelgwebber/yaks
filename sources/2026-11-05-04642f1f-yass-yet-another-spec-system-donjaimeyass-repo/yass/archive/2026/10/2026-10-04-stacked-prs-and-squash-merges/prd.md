# Stacked PRs and squash merges: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->
People land YASS work in many ways: stacked pull requests squash-merged one at a time, branches merged with merge commits, or local merges with no code host at all. YASS's rules hold in all of them because they're about commits: intent edits, archive moves and `queue.md` reorders each get their own commit, and progress rides in the same commit as its code. How a branch is split, stacked and merged is the team's workflow, which their harness learns from them; YASS doesn't need to teach it.

Landing large changes in this repo with squash-merged stacks (2026-10-04 and 2026-10-05) showed four places where YASS itself falls short, whatever the workflow:
- **A squash folds a PR's commits into one,** so a commit that has to stay apart from code (an archive move, a plan revision, a queue reorder) needs a PR apart from code. Nothing in YASS says so, and "in its own commit" reads stricter than the hook's actual rule (apart from code): an agent split a change's last progress from its archive move, and a PRD revision from its plan, into separate PRs that could have shared one.
- **`yass-work` asks for latitude to commit, branch and rebase, but not to push or open PRs,** so an agent following it published two PRs before the human had reviewed them locally.
- **Closing commits multiply PRs.** With pieces owning their files, a parent's criteria are marked in a separate closing commit, so every finished piece costs an extra plan-only PR.
- **`yass-log` reads commits,** and after a squash merge the PR is what a decision maps to.

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->
Anyone landing YASS work, through pull requests (squash-merged or not) or local merges.
- Someone who squash-merges learns from YASS's own docs and `yass-work` that a commit that must stand alone needs its own PR, before it bites.
- An agent following `yass-work` never pushes or opens a PR without the latitude to.
- A large change with N pieces lands with no closing commit per piece: its parent criteria count as done once the pieces delivering them are, and the archive is the one closing commit.
- `yass-log` names the PR behind a squash-merged decision.

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->
- **R2** [M1] `docs/monorepo.md` says what squash merges mean for YASS's rules: a squash folds a PR into one commit, so a plan revision, an archive move or a `queue.md` reorder needs a PR apart from code, though intent, progress and archive commits can share one; how a team splits and stacks its branches is up to it.
- **R13** [M1] `yass-work` and the `AGENTS.md` section say the same where they ask for intent and archives in their own commits: "their own commit" means apart from code, and under squash merges that means a PR apart from code.
- **R14** [M1] `yass-work` treats pushing a branch and opening or updating a PR like committing, branching and rebasing: only with that latitude, otherwise it shows the human the commands.
- **R9** [M1] `yass-log` attributes a squash-merged commit to its PR, from the `(#N)` in its subject, when it explains a decision.
- **R5** [M2] A parent criterion counts as done in `yass status` and `yass archive` once every piece delivering it (by its `Delivers AC…` box) is done, so no closing commit is needed per piece.
- **R6** [M2] `yass archive` marks a large change's delivered parent criteria `[x]` as part of the archive move, so the archive is the one closing commit.
- **R10** [M2] An e2e scenario lands a three-piece change one squash-merged branch at a time: `yass status` is right after each landing, no piece needs a closing commit, the archive closes it, and main's history passes the hook.

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->
- Workflow guidance: how to branch, stack, sync, choose a merge style or land a stack, or which tools to use (`gh stack`, Graphite and the like teach their own). The playbooks stay compatible with however people work, and their harness learns the rest from them.
- Checking PRs or branches as a whole. The hook stays per commit: a commit that mixes intent and code is the line YASS draws, and a branch with separate intent and code commits can always be split into stacked branches by whoever needs it.
- Settings for any of this in `yass.yaml`.
- A stacking tool, or talking to a code host's API.
- Limits on PR size or stack depth.
- Changing the rule that progress rides in the same commit as the code it describes.
- The `yass:` trailer and SHA citations under squash merges: `2026-10-04-monorepos-at-scale-and-plans-repos` (M3).

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | squash-merge YASS work knowing which commits need their own PR, trust an agent not to publish without the latitude to, and trace a squash-merged decision to its PR |
| M2 | land a large change with no closing commit per piece, the archive closing it, as an e2e test shows |

## Open questions
None.
