# Simplify before 0.5.0: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->
YASS calls itself a thin, file-based layer with a small CLI. In the six days from 0.3.0 to 2026-10-11, its non-test Go went from 2,893 lines to 6,097, its playbooks from 3,751 words to 5,085, its docs from 4,684 words to 7,670, and `yass.yaml` grew to 8 settings. Most of that is earning its keep (archives by month, `yass decisions`, delivered criteria, the branch view), but the half of [`monorepos-at-scale-and-plans-repos`](../2026-10-04-monorepos-at-scale-and-plans-repos/) for plans kept outside the code repo has outgrown its purpose:

- **Its PRD called separate plans the privacy fallback,** yet it took 5 of 11 pieces and most of the new concepts: inline, separate and mixed repos, repo naming, a map of code checkouts, three trailer modes, per-worktree state.
- **There are two ways to link code to plans:** `code:` citations (now with `repo@sha`, `repos:`, merged checks and an archive refusal) and `yass:` trailers (with `ac:`, `yass use`, three ways to infer, two hooks, three modes, range checks). Each has its own settings, checks and edge cases.
- **The rules hang on a repo-level mode** (trailers apply only when every yass folder is separate), so a repo with both inline and separate plans gets no trailers even for a change kept outside it. The AC56 agent run hit exactly this.
- **Bugs now come from interactions, not features:** hooks in linked worktrees reading the wrong repo, new hooks blocking commits under an older `yass`, a per-folder git lookup costing 2.3s, a benchmark silently broken for six pull requests.

Everything after 0.4.0 is unreleased, so it can still be simplified without breaking anyone. After 0.5.0 ships, it can't.

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->
- **People keeping plans outside their code repo** (the privacy case): one way to link code, one rule for when commits name their change.
- **Everyone else:** a smaller YASS to learn, read and trust, and fewer interactions to break.

Targets, against main on 2026-10-11:
- `yass.yaml` settings drop from 8 to 5 (`repo`, `repos` and `trailers` go).
- Non-test Go (6,097 lines), playbook words (5,085) and docs words (7,670) are each smaller.
- A code commit for a change kept outside its repo gets its trailer, whatever else that repo holds.
- `tests/e2e.sh`, `go test` and `tests/bench.sh` pass, with every bench target met.

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->
- **R1** [M1] A `code: <sha>` citation is an optional note on a box, as in 0.4.0: `yass status` checks that the commit exists in the repo it runs in and, on a done box, that it's merged there. Nothing more.
- **R2** [M1] `repo@sha` isn't a citation form: YASS doesn't read it.
- **R3** [M1] The `repo:` and `repos:` settings are gone, and with them naming code repos and checking citations from a plans repo. A `yass.yaml` that still has them gets the usual unknown-setting warning.
- **R4** [M1] `yass archive` doesn't refuse a change over its citations; it warns about a done box citing a commit that's missing here or not merged, and archives it.
- **R5** [M2] The `prepare-commit-msg` hook adds `yass: <change>` whenever the change it works out (from `yass use`, the branch name, or an earlier commit) lives in a yass folder outside this repo, whether or not the repo also has inline plans.
- **R6** [M2] The `commit-msg` hook reports only a malformed trailer, or one naming no change. It never asks for a missing trailer and never refuses a commit.
- **R7** [M2] `yass hook --range` checks the trailers commits carry. `--require-trailers` also reports commits without one, and `--strict` makes any report fail.
- **R8** [M2] The `trailers:` setting and the repo-wide "every folder is separate" mode are gone. `yass root -v` and `yass status`'s `plans: separate` line stay.
- **R9** [M2] `yass use`, `ac:`, and `yass status <change>` listing a change's code and criteria from trailers work as they do now.
- **R10** [M3] `docs/separate-plans.md`, the README, `yass-work` and `docs/skills.md` describe one link and one rule, and nothing that's been removed.
- **R11** [M3] The change's Log records the sizes before and after: non-test Go, tests, playbook words, docs words, settings.
- **R12** [M3] `CHANGELOG.md` has a 0.5.0 section describing what ships, not what was built and removed in between.

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->
- New features, until 0.5.0 ships.
- Changing what 0.4.0 shipped: citations as they were, archives by month, `keep` and `yass evict`, `yass decisions`, `yass update`.
- Revisiting what's earning its keep: the branch view, delivered criteria, pieces owning their files, sparse-checkout notes, `--brief`.
- Shared plans for several code repos, in any form.
- Changing YASS's process rules (how intent, progress and archives are committed); that's a conversation of its own.
- Windows and Linux validation of `yass update` (its own change).

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | link code to plans one way in their own repo, with no repo naming or checkout maps to learn |
| M2 | keep plans outside the repo and have every commit for such a change name it, with one rule and no modes |
| M3 | read docs and playbooks that match, and install 0.5.0 |

## Open questions
- None.
