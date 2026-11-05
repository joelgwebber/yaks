# Simplify before 0.5.0: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->
Mostly deletion, in the order the code depends on itself. No design.md: the calls are in the PRD and `change.md`, and removing unreleased surface is easy to undo from git if it's ever wanted back.

**M1, one link in one repo.** In [`text.go`](../../../internal/yass/text.go), `codeRefs` goes back to plain shas (a `repo@sha` token isn't read). In [`repo.go`](../../../internal/yass/repo.go), `checkCode` checks citations against this repo only, as in 0.4.0; `repoName`, `repoFromURL`, `reposOf`, `codeRepos`, `citationIn` and the "left to another repo" notes go. `archiveCitations` becomes warnings that `yass archive` prints as it archives. [`config.go`](../../../internal/yass/config.go) loses `repo` and `repos`. `yass decisions --cites` matches plain shas, as before #67.

**M2, one rule for trailers.** In [`trailers.go`](../../../internal/yass/trailers.go), `trailerMode` and `plansSeparate` go, and with them `trailers:`. The prepare hook adds a trailer when the change it works out lives in a yass folder `separate()` says is outside this repo. The commit-msg hook reports only malformed trailers or ones naming no change, and always exits 0. `yass hook --range` checks the trailers commits carry, adds `--require-trailers` for commits without one, and fails with `--strict`. `yass status <change>` lists code from trailers for a change in a separate folder. `yass use` says when a change's plans are in this repo, so its commits won't carry a trailer.

**M3, say only what ships.** README, [`docs/separate-plans.md`](../../../docs/separate-plans.md), `docs/monorepo.md`, `docs/skills.md`, `yass-work` and the help text drop what's gone and describe the one link and the one rule. The Log records sizes before and after. `CHANGELOG.md` gets a 0.5.0 section.

The e2e sections for multi-repo citations and `repos:` are removed; the trailer section is rewritten for the one rule, including a repo with both kinds of plans.

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->
### M1
- [x] AC1 (R1) Given a done box citing a commit that isn't in this repo, or isn't merged, when you run `yass status`, then it warns as 0.4.0 did; a merged citation draws nothing — verify: e2e (the existing citation checks)
- [x] AC2 (R2) Given a box ending `— code: web@a1b2c3d`, then YASS reads no citation from it: no check, no warning, and `yass decisions --cites a1b2c3d` doesn't list it — verify: go test (`codeRefs`); e2e
- [x] AC3 (R3) Given a `yass.yaml` with `repo:` or `repos:`, when you run `yass status`, then it warns that the setting is unknown, and nothing else uses it — verify: e2e
- [x] AC4 (R4) Given a finished change whose done box cites a missing or unmerged commit, when you run `yass archive`, then it archives it and warns, naming the box and the commit — verify: e2e

### M2
- [x] AC5 (R5) Given a repo with an inline `yass/` and a separate plans folder, and `yass use` naming a change in the separate one, when you commit, then the commit gets `yass: <change>` — verify: e2e
- [x] AC6 (R5) Given `yass use` naming a change in the inline `yass/`, when you commit, then no trailer is added, and `yass use` said so when it was set — verify: e2e
- [x] AC7 (R5) Given a branch named after a change in a separate folder, or an earlier commit's trailer on the branch, then the trailer comes from them, as now — verify: e2e
- [x] AC8 (R6) Given a commit without a trailer, then the commit-msg hook says nothing and the commit goes through, whatever the repo holds — verify: e2e
- [x] AC9 (R6) Given a malformed trailer, or one naming no change, then the commit goes through with a heads-up naming it and the closest change names — verify: e2e
- [x] AC10 (R7) Given `yass hook --range` over commits with a bad trailer and commits without one, then it reports the bad one; with `--require-trailers` it also reports the ones without; with `--strict` either exits 1 — verify: e2e
- [x] AC11 (R7) Given `--range` where the plans aren't checked out, then it checks only the trailers' form and says so, as now — verify: e2e
- [x] AC12 (R8) Given a `yass.yaml` with `trailers:`, then it warns that the setting is unknown — verify: e2e
- [x] AC13 (R9) Given commits naming a change in a separate folder with `ac:`, then `yass status <change>` lists them and each criterion's, as now — verify: e2e (the existing section)

### M3
- [x] AC14 (R10) Given the README, `docs/`, the playbooks and `yass --help`, then none mentions `repo@sha`, `repo:`, `repos:`, `trailers:` or refusing commits, and `docs/separate-plans.md` describes one link and one rule — verify: `grep` for each; manual: review
- [x] AC15 (R11) Given the change's Log, then it records non-test Go lines, test lines, playbook words, docs words and settings at 2026-10-11 main and after; non-test Go, playbook words and docs words are each smaller, and settings are 5 — verify: manual: the numbers, from the commands in the Log
- [x] AC16 (R12) Given `CHANGELOG.md`, then a 0.5.0 section describes what ships since 0.4.0 and says nothing of what was removed before release — verify: manual: review
- [x] AC17 (all) Given the change done, then `go test`, `tests/e2e.sh`, `tests/examples.sh`, `tests/site.sh` and `tests/bench.sh` pass, every bench target met — verify: run them; record the bench numbers in the Log

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->
None: three pull requests, one per milestone, in order (M2 rewrites code M1 shrinks first; M3 describes both). Each is small and mostly removes code.

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->
- Every suite passes after each pull request; `tests/bench.sh` before M3 merges.
- The sizes before and after (AC15).
- A read of the docs and playbooks against the CLI's help, for anything removed that's still described.
