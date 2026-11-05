---
platforms: [ios, android, web]
source:
follows:
blocked:
---
# History

## Goal
A History screen shows the last 30 days of entries, newest first. Delivers AC2.

## Acceptance
- [x] Delivers AC2 (plan.md)
- [-] Infinite scroll past 30 days - dropped: not in the PRD; see Decisions

## Steps
- [x] Query + list
- [x] HistoryTests

## Decisions
- Dropped infinite scroll - out of scope for M1; nobody asked (sam)

## Log
### 2026-09-10 (claude)
- Did: History on all platforms; HistoryTests pass.
- Next: done.
