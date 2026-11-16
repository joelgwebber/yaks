# Offline sync: PRD

## Why
Moodlog's most common feedback in September: "it lost what I logged on the subway". Entries go straight to the API today and fail without a network. A journal that loses entries isn't trusted.

## Users and outcomes
Commuters and anyone with patchy coverage, on iOS, Android and the web.
- Zero lost entries (support tickets tagged "lost entry", currently ~4 a week).
- No change to p50 time-to-log (< 5s).

## Requirements
- **R1** [M1] Entries logged offline are never lost, and sync in order when the device reconnects.
- **R2** [M1] Queued entries survive the app being killed.
- **R3** [M2] [ios, android] The user can see which entries haven't synced yet.

## Non-goals
- Multi-device conflict resolution UI. Last write wins until we see real conflicts.
- Background sync while the app is closed (OS limits; revisit later).

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | log on the subway and never lose an entry |
| M2 | see what hasn't synced yet (mobile) |

## Open questions
- Does the web need the badge too? (Sam to decide; see the sync-badge piece.)
