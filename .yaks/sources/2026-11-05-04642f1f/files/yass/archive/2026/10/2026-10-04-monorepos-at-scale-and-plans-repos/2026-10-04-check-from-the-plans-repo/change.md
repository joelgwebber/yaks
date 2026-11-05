---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-repo-qualified-citations, 2026-10-04-yass-trailers
---
# Check from the plans repo

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A plans repo maps repo names to code checkouts, so yass status there, locally or in CI, checks every citation.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC53, AC54 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `repos:` in a plans repo's `yass.yaml`, resolved like `path:` (`${VAR:-default}` included); `checkCode` checks each citation against its repo's checkout and main branch (`citationIn`), an unqualified one against the only repo listed
- [x] Missing checkouts, unset variables and unplaceable unqualified citations are notes, so `--strict` passes
- [x] e2e §47 (700 ok); `docs/separate-plans.md` "Checking from the plans repo"; README settings

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A code checkout's main branch is its `origin/HEAD`, else `main` or `master`, read in that checkout; the plans repo's own `branch:` doesn't apply to it (claude, 2026-10-10)
- R17 also mentions trailers; checking a code repo's trailers stays with that repo (`yass hook --range` there), since the plans repo has no commits of the code's to read - AC53 and AC54, the piece's criteria, are about citations (claude, 2026-10-10)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-10 (claude)
- Did: `repos:` and checking a plans repo's citations against each code checkout. `go test`, `tests/e2e.sh` (700 ok), `tests/examples.sh`, `tests/site.sh`, `tests/bench.sh` (status 811ms, all targets) pass. The piece is done.
- Next: none here; piece 11 (`agent-guidance-for-separate-plans`) can start.
