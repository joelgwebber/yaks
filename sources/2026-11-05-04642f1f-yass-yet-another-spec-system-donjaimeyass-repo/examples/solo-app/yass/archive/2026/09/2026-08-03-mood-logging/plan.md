# Mood logging: plan

## Approach
Shared TypeScript core (entries, storage interface) used by a React Native app and a web app, backed by a small HTTP API.

## Acceptance
### M1
- [x] AC1 (R1) Given a fresh launch, when the user taps a mood, then it is saved within 5s — verify: e2e flows/log_mood
- [x] AC2 (R2) Given 40 days of entries, when opening History, then the last 30 show newest first — verify: unit HistoryTests

## Pieces
1. `log-a-mood` (AC1), then 2. `history` (AC2).

## Validation
`tools/verify all e2e` on all three platforms.
