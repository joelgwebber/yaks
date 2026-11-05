---
platforms: [all]
source: CI fixes on 2026-10-04: `yass new` cut "…compare through symlinks" to `…-compare-throu`, and a Log then quoted the wrong name
follows: 
blocked:
archived: 2026-10-04T21:32:30Z
---
# Change names end on a whole word

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A long title gives a change folder name that ends on a whole word (cut at the last hyphen within 40 characters), so names stay readable and are easy to quote correctly in Logs and commit messages.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a title whose slug is longer than 40 characters, when you run `yass new`, then the folder name ends at the last whole word that fits, with no trailing hyphen — verify: go test TestSlug; e2e
- [x] Given a single word longer than 40 characters, then the slug is cut at 40, as before — verify: go test TestSlug
- [x] Given changes created with the old cut (`…-compare-throu`), then `yass status --archived` still lists them as before — verify: e2e

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Cut `slug` at the last hyphen within the limit
- [x] Go test and e2e checks

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A hyphen right after the limit counts as a whole-word cut - a 40-character slug followed by a hyphen keeps all 40 (claude)
- A single word longer than the limit is still cut at the limit - there's no whole word to keep (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: `slug` cuts at the last hyphen within 40 characters. Go tests for the two titles that were cut mid-word today, the exact-40 boundary and a long single word; e2e §23 (a new change, and an archived one named the old way still listing by name). Checks: go test, go vet, gofmt, e2e 281/281, examples.
- Next: archive.

