---
platforms: [ios, android, web]
source:
follows:
blocked:
---
# Offline queue

## Goal
Entries are written locally first and queued; a worker replays the queue in order when the network returns, and the queue survives the app being killed. Delivers AC1 and AC2.

## Acceptance
- [x] Delivers AC1 (plan.md)
- [ ] Delivers AC2 (plan.md)
- [ ] Given a 500-entry queue, when it drains, then the UI stays responsive (no frame > 50ms) — verify: perf flows/drain_500

## Steps
- [x] Queue table + IndexedDB store
- [x] Worker drains in order with backoff
- [x] WAL mode on Android
- [ ] Web: drain on the `online` event
- [ ] Drain in batches of 50

## Decisions
- Replay strictly in order - parallel replay reordered entries on Android (claude)
- Backoff caps at 60s - longer felt broken in testing (sam)

## Log
### 2026-09-30 (claude)
- Did: queue + worker; offline_sync passes on iOS and Android.
- Learned: Android kills the worker before the DB transaction commits; needs WAL mode.
- Next: enable WAL on Android, rerun flows/offline_kill.

### 2026-10-02 (codex)
- Did: WAL on Android; offline_kill passes on Android and iOS. Playwright offline.spec.ts added; AC1 passes on web.
- Learned: the web has no background worker; draining must hook the `online` event.
- Next: drain on `online` for the web, then batch the drain.
