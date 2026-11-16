---
platforms: [all]
source: 
follows: 
blocked:
---
# Dependencies through blocked:

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
blocked: can name the changes a change waits on; yass status shows what it's waiting on until they're done, and yass archive respects it.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [x] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC9, AC10, AC11, AC12, AC13, AC14 and AC15 of the plan.

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Resolve `blocked:` items to changes across yass folders; all resolve means dependencies
- [x] `waiting on:` in `yass status`, and the clear-it hint in `yass status <change>`
- [x] `yass archive` refuses only on a reason or an unmet dependency
- [x] Warn about unresolved dated names and cycles
- [x] e2e checks
- [x] README, `yass-work`, `yass-plan`, `yass-status`, monorepo example

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Names resolve by folder name; a path only breaks a tie between changes that share a name - a path written while a change was active (`yass/changes/…`) has to keep working after it's archived; the e2e caught this (claude)
- A dependency item is a single word - so free text that happens to contain a path ("waiting on legal and services/…/2026-…") stays a reason instead of clearing itself (claude)
- A dated-looking name that isn't a change warns and leaves the value a reason - a typo should keep the change blocked, not silently unblock it (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-03 (claude)
- Did: `blocked:` naming changes is a dependency (`waiting on:` in status, clear-it note in the detail view, archive refuses only while waiting); warnings for unknown dated names and cycles; e2e section 12 (219 passing), examples pass; README "Waiting on another change", `yass-work`, `yass-plan`, `yass-status`, monorepo example.
- Next: review, then archive order-and-dependencies.
