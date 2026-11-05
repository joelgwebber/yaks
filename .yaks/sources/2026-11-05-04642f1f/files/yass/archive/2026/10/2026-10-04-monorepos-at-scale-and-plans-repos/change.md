---
platforms: [all]
source: 
follows: 
blocked:
archived: 2026-10-11T20:57:58Z
---
# Large monorepos first, separate plans repo done right

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
YASS works at a large company with one big monorepo, many teams and expensive CI. Plan-only commits cost no CI, each team owns its own plans, and the tools stay fast at that scale, so keeping plans next to the code is the clear recommendation. For projects that need private plans, a separate plans repo is a supported fallback. It works correctly in worktrees, with several code repos sharing it, and through archiving. Its link between code and plans is a `yass:` trailer that tooling adds automatically and checks, warning by default, and only in that setup.

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)" for calls that span the whole change. -->
- Monorepo with plans inline is the recommended path, including for large orgs; a separate plans repo is the fallback for privacy - CI cost and size are solvable inside the monorepo (path filters, affected-target builds, LFS), and inline keeps progress in the same commit as code (Jaime)
- No orphan branches, submodules or subtrees - they bring separate-repo semantics or submodule pain without solving privacy (Jaime)
- Trailers on code commits are checked only when plans are in a separate repo - they replace the same-commit guarantee that repo loses, and the monorepo setup stays as simple as it is (Jaime)
- Link code to plans with a trailer in the code commit, not only SHA citations in the plans - trailers can't be rewritten and survive squash merges, a lesson from Gerrit/Zuul `Depends-On:` (Jaime, claude)
- Assets (mocks, images) live in the change folder, with no tooling change - checked on 2026-10-04: archiving moves them as renames, status ignores them, and the archive's append-only rule covers them (claude)
- PRD approved, open questions to be settled during planning (Jaime, 2026-10-04)
- Plan written: 56 criteria in eleven pieces, worktree fix first (claude)
- Open questions settled: `code:` citations stay as optional detail; Bazel is the affected-targets recipe; the speed targets stay as written (Jaime, 2026-10-04)
- Trailers: one lower-case `yass:` trailer with a small DSL (`yass: <change>[/<piece>] [ac:ac1,ac2]`, `yass: none`), read case-insensitively. A warning by default, with `trailers: require` in the code repo's `yass.yaml` (which separate plans need anyway) to enforce it. Added automatically by a prepare-commit-msg hook from `yass use`, the branch name, or earlier commits - so trailers are never a burden to agents or people, and teams choose how strict to be (Jaime)
- Pieces own their files, instead of a merge driver; R7 reworded to match - the conflicts come from pieces on parallel branches sharing the parent's plan.md and Log, so the plan's shape is the fix. A driver would need setting up in every clone, wouldn't run in hosted merges, and could silently lose content. This lowers R7's bar: parallel archive commits can still conflict on adjacent queue.md lines (rare, trivial to fix), and queue reorders conflict on purpose because they're intent (Jaime)
- Progress stays in files rather than a tracker (Jira, Linear); a tracker's real advantage, a live view of who's working on what, comes from a git-native `yass status --branches` instead, added as R19 in M1. A one-way projection into a tracker may be a later change (Jaime)
- Remaining design calls settled: separate = `path:` resolves outside the repo, plus a gitignored `yass/` symlink as a convenience; optional `web@sha` with names inferred; `repos:` with `${VAR:-default}` paths, offline; `yass paths` as proposed; the branch view on by default, with `--since` and `--no-branches` (Jaime, 2026-10-04)
- Branch view stays on by default everywhere, after reviewing its cost (a local walk of unmerged branches limited to YASS paths, plus reading plan files at branch tips) - safeguards: no on-demand downloads in partial clones, commit-graph recommended in the docs, freshness shown (Jaime, 2026-10-04)
- AC4 and AC5 dropped (`[-]`) here, which lowers this change's bar: neither a Bazel repo nor a GitHub merge queue (it needs an organization-owned repo) is available yet. Testing both moves to `2026-10-04-validate-the-ci-recipes-in-the-field`, along with inviting reports for other CI systems. In their place, AC57 and AC58 hold the docs to saying plainly what's tested and what isn't; R2 and R3 are reworded to match (Jaime, 2026-10-04)

- AC12 ("ready to mark" in `yass status`) dropped here: `2026-10-04-stacked-prs-and-squash-merges` takes over counting delivered parent criteria (its R5, R6), so piece 5 keeps only the rule that pieces own their files and the hook's heads-up. design.md §5 notes it (Jaime, 2026-10-04)
- Closing commits are gone: since `stacked-prs-and-squash-merges` (#59), `yass status` counts the criteria pieces deliver and `yass archive` ticks them. AC13 (a closing commit marks the parent done) is dropped, which lowers the bar: #59's e2e already proves the behaviour without one. AC14, piece 5's Goal and README box, the Pieces note and design §'Pieces own their files' now say no closing commit is needed. The commit-graph and branch-view box moves from piece 4 to piece 6, where the branch view is built (Jaime, 2026-10-09)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: closing commit for the two finished pieces: marked AC1–AC3, AC57, AC58 (`2026-10-04-yass-paths-and-ci-recipes`) and AC23–AC29 (`2026-10-04-worktrees-find-the-plans`) in plan.md, with no code.
- Next: piece 4 (`2026-10-04-team-scale-docs`) and piece 5 (`2026-10-04-pieces-own-their-files`) are unblocked; piece 3 waits on the branch view (piece 6).
### 2026-10-09 (claude)
- Did: piece 4 (`team-scale-docs`) done (#62). Revised the plan for #59: no closing commits (AC13 dropped), the commit-graph box moved to piece 6.
- Next: piece 5 (`2026-10-04-pieces-own-their-files`); then piece 6 (`branches-view`), which also finishes piece 3's last benchmark.
### 2026-10-09 (claude)
- Did: pieces 3 (`speed-at-scale`) and 5 (`pieces-own-their-files`) done; piece 6 (`branches-view`) built, waiting on Jaime's review of its docs.
- Next: piece 7 (`separate-plans-mode`), the start of the separate-plans pieces (8–11).
