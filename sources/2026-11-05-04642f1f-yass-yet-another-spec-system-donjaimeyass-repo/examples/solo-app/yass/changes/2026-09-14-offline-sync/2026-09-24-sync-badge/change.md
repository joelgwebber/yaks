---
platforms: [ios, android]
source:
follows:
blocked: does the web get the badge too? (open question in prd.md, for Sam)
---
# Sync badge

## Goal
History marks entries still waiting in the sync queue with a small "not synced" badge. Delivers AC3.

## Acceptance
- [ ] Delivers AC3 (plan.md)
- [ ] The badge disappears within 1s of the entry syncing — verify: e2e flows/sync_badge

## Steps
- [ ] Queue exposes `pending(entryId)`
- [ ] Badge component
- [ ] Wire into History

## Decisions

## Log
### 2026-09-24 (claude)
- Did: nothing yet; read the PRD.
- Next: needs Sam's answer on the web, then start with `pending(entryId)`.
