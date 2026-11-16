---
platforms: [all]
source: 
follows: 
blocked:
---
# yass paths and CI recipes

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
CI can skip plan-only changes with one gate job built on yass paths, with tested recipes for GitHub Actions (including required checks and merge queues), Bazel and a generic CI, and this repo's own CI uses the GitHub one.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC1–AC3, AC57, AC58 in [plan.md](../plan.md)
- [x] `docs/monorepo.md` exists and the README's Monorepos section links to it

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass paths`: globs for each yass folder inside the repo and each `yass.yaml`; `**` for a plans repo (AC1)
- [x] `--only <range>`: 0 plans only, 1 anything else or nothing, 2 unreadable range (AC2)
- [x] e2e §13
- [x] `docs/monorepo.md`: why a job and not a path filter, GitHub gate job and `ci-ok`, a no-install diff check, merge queues, Bazel, other CI (AC4, AC5)
- [x] This repo's `ci.yml`: the gate job (`go run ./cmd/yass`), `test`/`build` behind it, `ci-ok` aggregating, `merge_group` added (AC3)
- [x] README: `yass paths` in the CLI list, link to docs/monorepo.md
- [x] AC3 on GitHub: a plan-only push skips `test` and `build` and `ci-ok` passes; a code push runs them
- [-] AC4: the Bazel recipe run against a real Bazel repo (moved to `2026-10-04-validate-the-ci-recipes-in-the-field`)
- [-] AC5: a plan-only pull request through a real merge queue (moved to `2026-10-04-validate-the-ci-recipes-in-the-field`)
- [x] Mark the Bazel and merge-queue sections of `docs/monorepo.md` as untested, with where to report results (AC57, AC58)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- `yass paths` lists each whole yass folder (`yass/**`), not just the parts the hook classifies (changes/, archive/, queue.md, README.md) - a yass folder is YASS's by definition, and one glob per folder is what CI filters and `.bazelignore` want (claude)
- `--only` exits 1 for a range with no changed files - when in doubt, a gate should run the checks, not skip them (claude)
- Recipes compare `BASE...HEAD` (three dots) - a branch is compared with where it left its base, so commits that landed on the base meanwhile don't count as its changes (claude)
- This repo's CI builds the gate's yass with `go run ./cmd/yass` - no release has `yass paths` yet, and it dogfoods the branch's own version (claude)
- The GitHub facts in the docs come from GitHub's documentation, checked on 2026-10-04: a workflow skipped by a path filter leaves required checks Pending; a job skipped by `if:` reports Success; `merge_group` takes `branches` filters, no `paths` (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: `yass paths` and `--only`; e2e §13 (10 checks); `docs/monorepo.md` with the GitHub gate-job recipe (plus `ci-ok`, a no-install variant, merge queues), Bazel and generic recipes; this repo's `ci.yml` gated the same way; README updated. Checks: go test, go vet, gofmt, e2e 263/263, examples. The gate step simulated locally: plan-only commit c7a9f90 → skip, code commit e6bdcbc → run, new branch → run. Workflow YAML parses.
- Not yet verified (needs GitHub or tools not here): AC3 on a real push, AC4's Bazel recipe (no Bazel on this machine), AC5 through a real merge queue. The `merge_group.base_sha` field name comes from the webhook payload and isn't confirmed on GitHub's events page.
- Next: when this branch is pushed, check one plan-only push and one code push in Actions (record the run links here), and consider requiring `ci-ok` on main. Try the Bazel recipe on a repo that uses Bazel, and a merge queue on a scratch repo, or record why not. Then mark AC3–AC5 and the Delivers box.
### 2026-10-04 (claude)
- Did: AC3 verified on GitHub after Jaime had me push main. Plan-only push d5ee308: run https://github.com/donjaime/yass/actions/runs/37235914174, `changes` passed, `test` and `build` skipped, `ci-ok` passed (26s). Code pushes ran everything: https://github.com/donjaime/yass/actions/runs/37235844602 (which caught a macOS-only bug, fixed in `2026-10-04-paths-that-don-t-exist-yet-compare-throu`) and https://github.com/donjaime/yass/actions/runs/37236153589 (all passed).
- Did: AC4 and AC5 moved to `2026-10-04-validate-the-ci-recipes-in-the-field` (Jaime). In their place, AC57 and AC58: the docs now say the Bazel and merge-queue recipes are untested and where to report (8ad7aac); the GitHub recipe is the one these runs used, and the generic one is what e2e §13 checks.
- Next: done. Its parent criteria (AC1–AC3, AC57, AC58) are ready to mark in a closing commit. `ci-ok` could be made the required check on main (Jaime's call, in GitHub settings).

