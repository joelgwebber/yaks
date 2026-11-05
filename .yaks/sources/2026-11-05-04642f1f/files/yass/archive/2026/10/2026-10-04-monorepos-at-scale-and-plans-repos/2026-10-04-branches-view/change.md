---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-pieces-own-their-files
---
# Branches view

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass status shows by default who is working on which change or piece on unmerged local and pushed branches, and how far they are, as of the last fetch, without leaving git. --since sets the window and --no-branches turns it off.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC18–AC22 in [plan.md](../plan.md)
- [x] The README's `yass status` section documents the branch view, `--since` and `--no-branches`
- [x] `docs/monorepo.md` recommends `git commit-graph write --changed-paths` (or `fetch.writeCommitGraph`) and a recent git for large repos, and explains what the branch view reads and how fresh it is

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `branches.go`: recent unmerged branches (`for-each-ref --no-merged`, the window from `--since`), one `git log --source` over them limited to the yass folders' `changes/`, and the `change.md`/`plan.md` at each tip from one `cat-file --batch` with `GIT_NO_LAZY_FETCH` (or no read at all, in a partial clone with git before 2.44); the freshness line from `FETCH_HEAD`
- [x] `yass status` shows each change's and piece's branches under it, and the freshness line; `--since 14d|DATE`, `--no-branches`
- [x] e2e §41 against a remote that's a local folder (610 ok): local and pushed branches with author, progress and Next; code-only, merged and 40-day-old branches left out; `--since`; `--no-branches` running no lookup; no fetch, also in a blobless clone (shown without progress); no remote; no unmerged branches
- [x] `tests/bench.sh` with 100 local and 1,000 remote branches made by `git fast-import`: 849–929ms with 100 (target < 1s, AC16), 1,480–1,579ms with 1,000 more (target < 2s, AC22)
- [x] README and `docs/monorepo.md` (what it reads, freshness, commit-graph, partial clones, timings)
- [x] Jaime reviews the docs

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The local copy of the merged branch (`main` when it's `origin/main`) is left out: its unpushed commits aren't someone's work in progress (claude, 2026-10-09)
- A branch's progress is the boxes in the folder's `change.md` and `plan.md` at its tip; other files there aren't read - two blobs per row in one `cat-file`, and they hold all of a change's boxes in practice (claude, 2026-10-09)
- A Next: inside the template's HTML comment isn't taken for one; the e2e caught `next: … -->` (claude, 2026-10-09)
- The benchmark's `follows:` line had been broken since `archive-names-only`: the Perl edit ate the newline (`follows: 2025-02-02-done-1blocked:`), so every run since resolved 200 unknown names, scanned the archive for pieces and printed 200 warnings. Fixed; the numbers above are with it fixed. The first timing after generating the repo can run high (1,045ms once) while the OS settles the new files; reruns gave 900–935ms. Headroom under 1s is about 70–100ms, most of it git's untracked-file walk (claude, 2026-10-09)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: the branch view in `yass status`, `--since`, `--no-branches`; e2e; the bench with 100 and 1,000 branches; docs. `go test`, `tests/e2e.sh` (610 ok), `tests/examples.sh`, `tests/site.sh`, `tests/bench.sh` pass.
- Next: Jaime reviews the docs; then this piece is done.
### 2026-10-09 (claude)
- Did: Jaime reviewed the docs and merged it (#65). The piece is done.
- Next: none here.
