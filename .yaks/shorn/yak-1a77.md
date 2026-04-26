---
id: yak-1a77
title: 'Improve onboarding: init injects mandate, update --note, skill guidance'
type: task
priority: 1
created: '2026-04-06T14:49:05Z'
updated: '2026-04-06T14:52:31Z'
commit: 5c7468e
---

Three connected changes to put the yak workflow on rails:
1. init appends a mandate block to CLAUDE.md (or AGENTS.md if found, with --agents flag to force). Creates file if missing.
2. update gains --note flag that appends timestamped text to the markdown body rather than replacing it.
3. SKILL.md gains guidance encouraging progress notes on shave and shorn summaries.

---
▸ 2026-04-06T14:51:43Z
Implementing: init mandate injection, update --note, and SKILL.md guidance.

---
▸ 2026-04-06T14:52:31Z
All three pieces implemented and tested. init injects mandate, --note appends timestamped entries, SKILL.md updated with progress-note guidance. Plugin bumped to 0.1.9.
