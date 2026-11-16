# Offline sync: plan

## Approach
Write every entry to local storage first and append it to a sync queue in the shared TypeScript core; a worker drains the queue in order with backoff. Mobile uses SQLite (WAL mode), web uses IndexedDB. See design.md for why.

## Acceptance
### M1
- [x] AC1 (R1) Given no network, when the user logs 3 entries and reconnects, then all 3 reach the server in order — verify: e2e flows/offline_sync (ios, android), Playwright offline.spec.ts (web)
- [ ] AC2 (R2) Given queued entries, when the app is killed and reopened, then the queue is intact — verify: e2e flows/offline_kill
### M2
- [ ] AC3 (R3) [ios, android] Given unsynced entries, when viewing History, then each shows a "not synced" badge — verify: e2e flows/sync_badge

## Pieces
1. `offline-queue` (AC1, AC2): storage, queue and worker. Everything else depends on it.
2. `sync-badge` (AC3): UI only, once the queue exposes "pending".

## Validation
`tools/verify all e2e` on all three platforms before archiving, plus one manual subway-style run (airplane mode, kill the app, reconnect).
