# Large monorepos first, separate plans repo done right: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
YASS has to work at a large company with one big monorepo, many teams, and expensive, complicated CI. That's where its main promise matters most: plans sit next to the code, and progress lands in the same commit. Teams there worry that planning files, which change often and can carry mocks and images, will add CI runs, slow the merge queue, bloat clones and clutter history. Most of that can be solved inside the monorepo, but YASS doesn't show how yet, so the easy wrong answer is to move the plans out.

Some projects really do need plans in a separate repo, usually for privacy (an open source project with private plans). That setup exists today ([README: Keeping plans out of the repo](../../../README.md#keeping-plans-out-of-the-repo)), but trying it on 2026-10-04 found it fragile:
- In a linked git worktree, which is where agents usually run, a relative `path:` resolves from the worktree and finds nothing. The warning then suggests `yass init`, which would create a stray, empty plans folder.
- When two code repos share one plans repo, each repo warns about the other's citations as missing commits, and a repo-qualified citation (`code: web@e1553ee`) isn't checked at all.
- `yass archive` archives a change whose `[x]` boxes cite unmerged code.
- `yass status` run from the plans repo flags every citation, so the plans repo can't check itself in CI.
- Nothing warns about uncommitted plans or a plans repo that's behind its remote.
- Plans cite code by SHA, and a squash merge invalidates those SHAs, so they have to be fixed by hand.

## Users and outcomes
- **Teams in a large monorepo** (the primary path), and the platform or CI owners who decide whether YASS is allowed in.
  - With the documented setup, a commit that touches only yass folders runs no build or test jobs, and doesn't block merging with a check stuck waiting.
  - `yass status` takes under 1s on a repo with 200 yass folders and 2,000 changes. The pre-commit hook adds under 200ms on a repo of 100,000 files.
  - A platform owner can roll YASS out from one page of docs, without writing their own glue.
- **Projects that keep plans in a separate repo for privacy** (the fallback), and the agents working in them.
  - Every merged code commit in a separate-repo setup names its change. Tooling adds the name automatically, and checks it: a warning by default, enforced where a team opts in.
  - `yass status` gives no false warnings when two code repos share one plans repo.
  - An agent in any linked worktree resolves the same plans folder as the main checkout.

## Requirements
- **R1** [M1] A CI owner can tell which paths are YASS's from `yass paths`, and use that list in pipeline filters instead of hard-coding folder layouts.
- **R2** [M1] The docs give recipes for skipping build and test jobs on plan-only commits: GitHub Actions (including required checks that would otherwise wait forever when a path filter skips them) and one generic CI, tested; and Bazel or another affected-targets build, documented here and tested in `2026-10-04-validate-the-ci-recipes-in-the-field`.
- **R3** [M1] The docs explain how plan-only PRs go through a merge queue without full test runs. Testing that on a real merge queue is part of `2026-10-04-validate-the-ci-recipes-in-the-field`.
- **R4** [M1] The docs show CODEOWNERS for each team's yass folder, so plan reviews go to the owning team.
- **R5** [M1] `yass status` in a sparse checkout reports on the yass folders that are checked out. A `blocked:` that names a change outside the checkout says it can't see that change, instead of calling it unknown.
- **R6** [M1] The docs give Git LFS rules for binary assets under yass folders, and say that assets live in the change folder and move with it when it's archived.
- **R7** [M1] When agents on parallel branches each work on a different piece of the same change, those branches rebase onto each other without conflicts in YASS files. Each piece's progress lives only in its own folder, and `yass status` shows when a parent's criteria have been delivered by its pieces.
- **R8** [M1] The docs give commands for leaving YASS paths out of `git log`, `blame` and diff views.
- **R9** [M1] `yass status` and the pre-commit hook meet the speed targets above.
- **R19** [M1] `yass status` shows, by default, work on changes that hasn't merged yet: which local and pushed branches with recent commits (the last 30 days, or what `--since` says) touch each change or piece, their progress there, and their latest **Next:**, as of the last fetch, without leaving git. `--no-branches` leaves it out.
- **R10** [M2] `yass status` and `yass root` say whether the plans are inline (in the same git repo as the code) or separate.
- **R11** [M2] In a linked worktree, a relative `path:` resolves to the same folder as in the main checkout. When the plans folder is missing, YASS never suggests `yass init` from a linked worktree.
- **R22** [M2] When `path:` points outside the repo, YASS keeps a gitignored `yass/` symlink to the plans folder in each checkout and worktree, created and repaired by `yass` commands. It's a convenience: nothing depends on it.
- **R23** [M2] Paths in `yass.yaml` (`path:` and `repos:`) accept `${VAR:-default}`, so side-by-side clones work with no setup and anyone can override them.
- **R12** [M2] Citations can name their repo (`code: web@a1b2c3d`). Each code repo checks its own citations and says which ones it didn't check, without warning about them. An unqualified citation keeps today's behavior.
- **R13** [M2] `yass archive` refuses (unless forced) a change whose done boxes cite commits that are missing or unmerged, and lists any citations it couldn't check from where it runs.
- **R14** [M2] With separate plans, `yass status` warns when the plans repo has uncommitted changes or is behind its upstream.
- **R15** [M3] With separate plans, every code commit carries a `yass:` trailer naming its change, or `yass: none`. A commit-msg check warns about a missing or invalid one, or rejects it when `yass.yaml` sets `trailers: require`. `yass hook --range` does the same in CI, squash merges included. With inline plans, nothing asks for trailers.
- **R20** [M3] The hooks add the `yass:` trailer automatically, working out the change from `yass use <change>` (per worktree), then the branch name, then earlier commits on the branch, so neither agents nor people type it.
- **R21** [M3] A `yass:` trailer can name the criteria a commit works toward (`ac:ac1,ac2`), and `yass status <change>` lists each criterion's merged commits.
- **R16** [M3] `yass status` lists each change's code commits from its trailers on the merged branch, so finding a change's code doesn't depend on SHAs that a squash merge rewrites.
- **R17** [M3] A plans repo can map repo names to code checkouts (`repos:` in its `yass.yaml`), so `yass status` run there, locally or in CI, checks every citation and trailer. A repo whose checkout isn't there is reported as unchecked, not as an error.
- **R18** [M3] The `yass-work` skill's separate-plans guidance covers the whole routine. At session start, pull the plans and check their state. Commit code first, then plans. Cross-link the code PR and the change. Treat git latitude separately for each repo. Never run `yass init` to fix a missing plans folder.

## Non-goals
- Orphan branches, git submodules, subtrees, or manifest tools (`repo`, `west`) as a home for plans.
- Version control other than git (Piper, Perforce, Sapling).
- Trailers, or any new rule, for inline plans. The monorepo setup stays as simple as it is.
- CI plugins or integrations to install. YASS ships recipes and `yass paths`, not per-CI products.
- Moving plans out of the monorepo to save CI cost. That's solved inside the repo.
- Automatically rewriting SHA citations after a squash merge.
- A git merge driver for YASS files. Pieces owning their own files avoids the conflicts instead.
- Integrating with trackers (Jira, Linear), or keeping progress in one. Files stay the source of truth. A one-way projection from main into a tracker may be a later change.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | roll YASS out across a large monorepo with many teams, with plan-only commits costing no CI, owned per team, fast at scale, and parallel work visible and conflict-free |
| M2 | keep plans in a separate repo, with worktrees, shared plans repos and archiving all working correctly |
| M3 | rely on automatic, checked trailers to link code to plans in a separate-repo setup, and check everything from the plans repo, including in CI |

## Open questions
None. The four raised while shaping were settled on 2026-10-04 (see design.md and `change.md` Decisions).
