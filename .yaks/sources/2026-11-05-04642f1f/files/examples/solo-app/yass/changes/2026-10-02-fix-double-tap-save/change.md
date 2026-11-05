---
platforms: [all]
source: gh#41
follows:
blocked:
---
# Fix double-tap save crash

## Goal
Tapping Save twice quickly saves one entry and never crashes.

## Acceptance
- [ ] Given the mood picker, when Save is tapped twice within 300ms, then exactly one entry is saved and the app doesn't crash — verify: unit SaveButtonTests.doubleTap

## Steps
- [x] Reproduce on Android (crash in EntryStore.insert, duplicate key)
- [ ] Disable Save while a save is in flight
- [ ] Unit test

## Decisions
- Disable the button rather than dedupe in the store - fixes the cause, not the symptom (claude)

## Log
### 2026-10-02 (claude)
- Did: reproduced; root cause is two inserts racing on the same generated id.
- Next: disable Save while saving, then the unit test.
