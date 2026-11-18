---
platforms: [web]
source: gh#29
follows: 2026-08-03-mood-logging
blocked:
archived: 2026-09-08T16:05:00Z
---
# Fix History sort on the web

## Goal
History on the web shows newest first, like mobile.

## Acceptance
- [x] Given entries from three days, when opening History on the web, then they're newest first — verify: unit HistoryTests.web

## Steps
- [x] Sort by `createdAt` descending in the web query

## Decisions
- Sort in the query, not the component - the core already pages by date (claude)

## Log
### 2026-09-08 (claude)
- Did: fixed and tested.
- Next: done.
