---
platforms: [ios, android, web]
source:
follows:
blocked:
---
# Log a mood

## Goal
Opening the app lands on the mood picker; one tap saves an entry with an optional note. Delivers AC1.

## Acceptance
- [x] Delivers AC1 (plan.md)

## Steps
- [x] Picker as the launch screen
- [x] Save with an optimistic list update
- [x] Maestro flow log_mood on iOS and Android, Playwright on web

## Decisions
- Launch straight into the picker - saves ~4s versus a home screen (claude, approved by sam)

## Log
### 2026-08-11 (claude)
- Did: picker and save on all three platforms; log_mood passes everywhere (p50 2.8s).
- Next: done.
