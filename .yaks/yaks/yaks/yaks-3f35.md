---
id: yaks-3f35
title: 'Feedback from the schism import: session notes, doks and imports'
status: hairy
type: task
priority: 2
created: '2026-11-06T21:36:14Z'
updated: '2026-11-06T21:36:14Z'
parent: yaks-1c89
tags:
- doks
- skills
---

Joel relayed ten suggestions from the agent (`delta-doks`) that moved schism's `devlog/`, `design/` and `screenshots/` into its farm (schism-5b88): 18 backdated session notes, 15 doks and 8 `screenshot` doks, with six review passes. It was the first real use of the upkeep path (`yaks session`, the `yaks-upkeep` skill) built on the unlanded `upkeep` work (505d2058..d78b25a7).

This yak groups the follow-ups. Each child names the item it came from.

| # | Feedback | Yak |
|---|---|---|
| 1 | No way to backdate a session note or a promotion | child: backdating |
| 2 | `reviewed:` is the write time; path is local, `reviewed` is UTC | child: timestamps |
| 3 | `thread:` recorded under another `--harness` | child: thread bug |
| 4 | No attachments for doks and sessions | child: images |
| 5 | No guidance for importing repo docs as doks, or repointing yaks | child: import guide |
| 6 | No convention for dated doks; no created date | child: dok conventions |
| 7 | `doks/topics/` vs subject folders unclear | child: dok conventions |
| 8 | Generated `## Saved` misses imported work, may catch unrelated writes | child: Saved scope |
| 9 | Stale checkout says migrate; Delta clones key farms `repository-…` | top level (not upkeep) |
| 10 | Offer the recurring upkeep yaks after a bulk import | child: import guide |

Also: Joel filed yaks-85a6 (sessions not visible in the TUI), and the 62 `[session](/sessions/...)` citations the review wrote into schism's doks read as a bare `(session)`; a child covers that.

The code children change `src/sessions.rs` and `yaks session` in `src/main.rs`, which exist only on the `upkeep` work: land that first, or build on it.
