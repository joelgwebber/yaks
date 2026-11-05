---
platforms: [all]
source: plan.md
follows: 
blocked:
created: 2026-10-05T20:06:03Z
---
# Delivered criteria count

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A parent criterion counts as done in yass status and yass archive once every piece delivering it is done, yass archive marks those criteria [x], and an e2e scenario lands a three-piece change one squash-merged branch at a time.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC7–AC13 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Parse `Delivers` boxes (lists, ranges, trailing words); go test
- [x] Count delivered criteria in `status` progress and `next:`
- [x] `yass archive`: don't refuse for them; mark them `[x]` in the archived `plan.md`
- [x] e2e: each case, and the three-piece squash scenario

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A criterion counts as delivered once every piece that names it in a `Delivers` box is done, whatever words surround the reference: "and AC19's preview card" in one piece and "AC19's tags" in another means AC19 is met when both are - that's how pieces in this repo already split a criterion (claude, 2026-10-09)
- Delivered criteria count in `yass status` without the file changing (`countedBoxes`); only `yass archive` writes the tick, as part of the move - the archive is the one closing commit (R6), and the hook already allows the archive move to tick boxes (claude, 2026-10-09)
- `markDelivered` skips HTML comments, so a plan's commented-out example `AC1` stays as it was - the same trap caught me twice when ticking criteria by script during archive-that-scales and yass-update (claude, 2026-10-09)
- README and `yass-plan` say so, so nobody plans a closing commit for the parent (claude, 2026-10-09)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: built and tested delivered criteria: `deliveredIDs`, `delivered`, `countedBoxes` in status, and `markDelivered` in `yass archive`. `go test` (`TestDeliveredIDs`, `TestMarkDelivered`), `tests/e2e.sh` section 38 (576 ok, with three pieces landed one squash-merged branch at a time, the archive closing them, and main's history passing the hook), `tests/examples.sh`, `tests/site.sh`. The piece is done, and so is the change.
- Next: archive the change, in its own pull request.
