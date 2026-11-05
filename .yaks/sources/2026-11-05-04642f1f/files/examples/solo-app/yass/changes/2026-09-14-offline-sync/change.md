---
platforms: [ios, android, web]
source: feedback (9 notes in Sept mention the subway)
follows: 2026-08-03-mood-logging
blocked:
---
# Offline sync

## Goal
Logging works with no network, nothing logged offline is ever lost, and users can see what hasn't synced yet. Details in prd.md and plan.md.

## Decisions
- Offline before export - 6 of 9 feedback notes are about the subway; nobody asked for CSV yet (sam)
- Local-first storage with a sync queue - see design.md (claude, approved by sam)
- Sync is last-write-wins per entry - entries are append-only, so conflicts are rare (sam)

## Log
### 2026-09-15 (claude)
- Did: PRD and plan with Sam; split into two pieces.
- Next: offline-queue first; sync-badge needs it.
