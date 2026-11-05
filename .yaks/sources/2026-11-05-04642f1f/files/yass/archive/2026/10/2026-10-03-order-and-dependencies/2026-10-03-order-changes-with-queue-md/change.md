---
platforms: [all]
source: 
follows: 
blocked:
---
# Order changes with queue.md

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A yass folder's optional queue.md ranks its changes, yass status follows it, yass archive keeps it tidy, and the hook treats reordering as intent.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [x] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC1, AC2, AC3, AC4, AC5, AC6, AC7 and AC8 of the plan.

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Parse `queue.md` per yass folder (list items only, notes ignored), with a unit test
- [x] Sort each folder's changes in `yass status` by queue position, unlisted after by date
- [x] Warn about unknown, archived, piece and duplicate entries
- [x] `yass archive` drops the change's line and stages it
- [x] Hook: modifying `queue.md` with code is intent
- [x] e2e checks
- [x] README, yass README template, `yass-status`, solo-app example `queue.md`

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- An entry can also be in backticks or a markdown link (`[name](changes/name/)`) - links make the queue clickable on GitHub at no cost to the parser (claude)
- `yass archive` removes the line without renumbering the rest - other lines stay byte-for-byte, and markdown renders ordered lists in sequence anyway (claude)
- No marker in `yass status` for where the ranked changes end - the order is the signal; the file says the rest (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-03 (claude)
- Did: `queue.md` parsing (unit test), ordering in `yass status`, warnings, `yass archive` cleanup, hook rule; e2e section 11 (203 passing), examples pass; README "The queue", yass README template and copies, `yass-status`, solo-app example `queue.md`.
- Next: review; then the dependencies piece.
