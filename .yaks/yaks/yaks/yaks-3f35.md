---
id: yaks-3f35
title: 'Feedback from the schism import: session notes, doks and imports'
status: shaving
type: task
priority: 2
created: '2026-11-06T21:36:14Z'
updated: '2026-11-07T15:43:43Z'
parent: yaks-1c89
tags:
- doks
- skills
---

Joel relayed ten suggestions from the agent (`delta-doks`) that moved schism's `devlog/`, `design/` and `screenshots/` into its farm (schism-5b88): 18 backdated session notes, 15 doks and 8 `screenshot` doks, with six review passes. It was the first real use of the upkeep path (`yaks session`, the `yaks-upkeep` skill) built on the unlanded `upkeep` work (505d2058..d78b25a7).

This yak groups the follow-ups. Each child names the item it came from.

| # | Feedback | Yak |
|---|---|---|
| 1 | No way to backdate a session note or a promotion | yaks-0fa7 |
| 2 | `reviewed:` is the write time; path is local, `reviewed` is UTC | yaks-3b52 |
| 3 | `thread:` recorded under another `--harness` | yaks-3fc6 |
| 4 | No attachments for doks and sessions | yaks-db66 |
| 5 | No guidance for importing repo docs as doks, or repointing yaks | yaks-a096 |
| 6 | No convention for dated doks; no created date | yaks-c316 |
| 7 | `doks/topics/` vs subject folders unclear | yaks-c316 |
| 8 | Generated `## Saved` misses imported work, may catch unrelated writes | yaks-d11e |
| 9 | Stale checkout says migrate; Delta clones key farms `repository-…` | yaks-8a92, yaks-9a13 (top level: not upkeep) |
| 10 | Offer the recurring upkeep yaks after a bulk import | yaks-a096 |

Also: Joel filed yaks-85a6 (sessions not visible in the TUI), and the 62 `[session](/sessions/...)` citations the review wrote into schism's doks read as a bare `(session)`: yaks-5f70.

The code children change `src/sessions.rs` and `yaks session` in `src/main.rs`, which exist only on the `upkeep` work: land that first, or build on it.

---
▸ 2026-11-06T22:50:58Z [sched-1]
kind: move
from: hairy
to: shaving
---

moved: hairy -> shaving

---
▸ 2026-11-07T15:43:43Z [herd-upkeep-2]

For whoever is on this family (herd-upkeep-2, 2026-11-07): `main` moved to 38fc4358 (pushed to `origin`), so rebase or start from there. Nothing in it touches `src/sessions.rs` or the `yaks session` commands, but several things bear on these children:

- yaks-d11e: `yaks commit --as <actor>` (yaks-525b) now credits hand edits, so `session touched` can see an agent's editor work; the `yaks-upkeep` Saved bullet already says to use it.
- yaks-0fa7, yaks-a096: an import can now mark sources it saves on purpose (`yaks source mark --uncited/--superseded-by`, yaks-80bd) and write schemas (`yaks dok define`).
- yaks-5f70, yaks-85a6: `yaks refs <id> --incoming` lists doks and session notes that link a yak (yaks-2e00); a session link from a dok is found the same way.
- yaks-c316: `dok query --where` has `~`, `<`, `>=`, `!` and empty (yaks-39f9), so a `created:` field would be queryable by date.
- `yaks-upkeep` gained "Reflecting on sessions" (yaks-7168); edit it on top of that.
- yaks-323b (`session list --since`) waits on this family because it changes `src/sessions.rs`; take it with these if convenient.
