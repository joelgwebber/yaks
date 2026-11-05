---
platforms: [ios, android, web]
source:
follows:
blocked:
archived: 2026-09-12T17:20:00Z
---
# Mood logging

## Goal
The first version of Moodlog: log a mood in under five seconds and look back over the last 30 days, on iOS, Android and the web.

## Decisions
- One TypeScript core shared by React Native and the web app - most logic is the same, the UI isn't (sam)
- Launch straight into the mood picker - saves ~4s versus a home screen (claude, approved by sam)
- Shipped as v0.1.0 on 2026-09-12; p50 time-to-log 2.8s against a 5s target (sam)

## Log
### 2026-09-12 (sam)
- Did: shipped v0.1.0 on all three platforms.
- Next: nothing; offline is a separate change.
