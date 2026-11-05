---
platforms: [all]
source: 
follows: 
blocked:
---
# Speed at scale

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass status and the pre-commit hook meet the PRD's speed targets on generated large repos, held there by a benchmark script.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC16, AC17 in [plan.md](../plan.md)
- [x] `tests/bench.sh` generates its repos in a temporary folder and cleans up after itself

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `tests/bench.sh`: a repo with 200 yass folders and 2,000 changes, timed with `yass status` (AC16, without the branch view)
- [x] A repo of 100,000 files with a staged one-file change, timed with `yass hook` (AC17)
- [x] Median of 5 runs after an untimed warm-up; exits 1 on a missed target; `BENCH_KEEP=1` keeps the repos
- [x] CONTRIBUTING lists it
- [x] AC16 again once the branch view exists: 100 recently active branches in the status repo (piece `2026-10-04-branches-view` extends this script)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The hook is timed as `yass hook` on a staged change, not as a whole `git commit` - AC17 is about what the hook adds, and timing it alone keeps git's own commit cost out of the number (claude)
- Timings are the median of 5 runs after one untimed warm-up - the first run over freshly generated files measured 1,176ms against 380–410ms afterwards, which was the OS still caching and indexing new files, not yass (claude)
- Not run in CI - shared runners' timings vary too much for pass/fail targets; it's a local check, listed in CONTRIBUTING (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: `tests/bench.sh`. On an Apple M5 Mac, macOS 27.0.1: `yass status` with 200 yass folders and 2,000 changes, 384ms and 410ms on two runs (target < 1s); `yass hook` on a 100,000-file repo, 100ms and 102ms (target < 200ms). Generated repos are removed afterwards (checked). AC17 met.
- AC16 isn't finished: its target includes the branch view with 100 active branches, which doesn't exist yet. About 600ms of the budget is left for it. `yass status` spends about two thirds of its time in system calls (reading 2,000 files), so that's where to look if the branch view runs short.
- Next: the branch-view piece adds 100 recently active branches to the status repo here, and reruns; then mark AC16 and the Delivers box.
### 2026-10-09 (claude)
- Did: AC16 with the branch view: 100 recently active branches in the status repo, 849–929ms over three runs (target < 1s); AC17's hook 118–119ms (target < 200ms). The bench's `follows:` fix (see `branches-view`) means earlier status numbers here were measured with 200 unknown names, so ran slow. The piece is done.
- Next: none here.
