---
platforms: [all]
source: yass/changes/2026-10-04-monorepos-at-scale-and-plans-repos (AC4, AC5)
follows: 
blocked:
---
# Validate the CI recipes in the field

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
The CI recipes in `docs/monorepo.md` that couldn't be tested when they were written have been run for real: the Bazel recipe on a Bazel repo, and the GitHub gate job through a real merge queue. The docs say which recipes are tested and where, and invite reports for other CI systems and build tools. Split off from `2026-10-04-monorepos-at-scale-and-plans-repos` (its AC4 and AC5), so that change can finish without waiting on a test setup.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [ ] Given a small Bazel repo with a yass folder set up as the docs say (`.bazelignore`, no `glob(["**"])` pulling plans into a target), when an affected-targets tool (bazel-diff or target-determinator) compares a plan-only commit, then it finds no affected targets, and for a code commit it finds the code's target — verify: a script or fixture that can be rerun (Bazelisk installed), with its output in the Log
- [ ] Given a GitHub repo owned by an organization, with a merge queue on its main branch and the documented gate job, when a plan-only pull request goes through the queue, then `ci-ok` passes without the test jobs running, and a code pull request runs them — verify: manual: record both runs' links in the Log, and confirm the `merge_group.base_sha` field the recipe uses
- [ ] Given those runs, then the Bazel and merge-queue sections of `docs/monorepo.md` no longer say they're untested, and any fixes they needed are in — verify: manual: review
- [ ] Given `docs/monorepo.md`, then it says where to report results for other CI systems and build tools (Buck, Pants, Nx, Buildkite, GitLab CI), and the recipes people report as working are listed with who tested them — verify: manual: review

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [ ] 

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
