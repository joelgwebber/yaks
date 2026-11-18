---
platforms: [all]
source: 
follows: 
blocked: 2026-10-07-yass-decisions, 2026-10-07-keep-and-evict
created: 2026-10-07T17:34:55Z
---
# Decisions from history

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass decisions reads evicted months from git, only when a query reaches them, and says what it couldn't read in a shallow clone.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC43–AC47, AC49, AC50, AC29 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `history.go`: evicted months read from git by the commit and path in their `.evicted` file: one `ls-tree` per month, one `cat-file --batch` for every file; pieces and other `.md` files included; months whose commit is missing are reported
- [x] `yass decisions` reads them only when a query reaches them: `--since` up to the month's end, `--change` following the chain month by month, `--cites`, or a plain query the tree doesn't fill
- [x] `--cites <sha>`: boxes citing a commit by any prefix of its hash, in active, archived and evicted changes; text and `--json`
- [x] A missing month prints a note after the results (on stderr with `--json`), with `git fetch --unshallow`; exit 0
- [x] `yass-log` uses `--cites`; nothing in the kit reads `archive/` directly (AC29); `docs/skills.md`, `docs/monorepo.md`, `README.md`
- [x] go test (`TestReadEvictedGitProcesses`: 1,000 evicted changes, at most 4 git processes); e2e section 34 (503 ok), including shallow clones

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A query with neither `--since` nor `--change` reads evicted months only when the tree's matches don't fill `--limit` - evicted months are older than everything left in the tree, so a full answer from the tree doesn't need them; an old dated entry in a long-lived change could in principle sort below an evicted one, which this accepts (claude, 2026-10-08)
- `--change` reads only the evicted months its chain reaches, one round at a time, rather than every evicted month - an evicted change can only follow changes archived before it, so the chain never leads back from evicted months into the tree (claude, 2026-10-08)
- `--cites` reads every evicted month - a commit can be cited anywhere, and "no box cites it" must be true when it says so (claude, 2026-10-08)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-08 (claude)
- Did: built and tested reading evicted months from git and `--cites`, and switched `yass-log` to it. `go test`, `tests/e2e.sh` (503 ok), `tests/examples.sh` pass. The piece is done, and so is the change.
- Next: `yass archive 2026-10-07-archive-that-scales`, in its own pull request.
### 2026-10-08 (claude)
- Did: fixed CI on Linux: the note naming unreadable months listed them in Go's random map order (2025/02 before 2025/01); months are now read and named oldest first.
