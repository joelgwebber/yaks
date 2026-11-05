---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-yass-paths-and-ci-recipes
---
# Team-scale docs

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A platform owner can roll YASS out across many teams: CODEOWNERS per yass folder, LFS for assets, history filters, and yass status that behaves in sparse checkouts.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC6–AC9, AC15 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Sparse checkouts: a `blocked:` or `follows:` naming a change git has but this checkout doesn't (`notCheckedOut`, from `git ls-files` only when a name doesn't resolve) gets a note naming its yass folder, not a warning
- [x] e2e §39: a cone-mode sparse checkout without `services/search/` (status, `--strict`, `blocked:`, `follows:`, a real typo still warning), and a binary asset archived with its change (585 ok)
- [x] `docs/monorepo.md` "Many teams in one repo": CODEOWNERS, sparse checkouts, LFS for assets, history without the plans (each command run in this repo, in zsh and bash); README points to it
- [x] Jaime reviews the docs (AC6, AC9)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A cone-mode sparse checkout already left other teams' yass folders out of `yass status` (AC7 held before any change); only names in `blocked:` and `follows:` needed a note (claude, 2026-10-09)
- The history filters pipe `yass paths` through `xargs`, not `$(…)` - zsh doesn't split an unquoted variable, so the `$(…)` form silently filtered nothing there; checked in zsh and bash (claude, 2026-10-09)
- `git blame` gets an explanation, not a command: a plans-only commit never changes a line of code, so it never appears in a code file's blame (claude, 2026-10-09)
- The box on `git commit-graph` and the branch view's freshness waits: the branch view is piece 6 and doesn't exist yet; proposed moving it there (claude, 2026-10-09)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: sparse checkouts say where an unseen change is; e2e for that and for assets; the many-teams docs.
- Next: Jaime reviews the docs (AC6, AC9). The commit-graph and branch-view box should move to `branches-view` (a plan revision).
### 2026-10-09 (claude)
- Did: Jaime reviewed the docs and merged them (#62); the commit-graph box moved to `branches-view` in the revision. The piece is done.
- Next: none here.
