---
id: yaks-ca28
title: 'Flat farm, phase 2: sheds read both layouts (base_farm, mixed-layout comparisons)'
type: task
priority: 1
created: '2026-10-12T19:53:57Z'
updated: '2026-10-12T19:53:57Z'
parent: yaks-8b7a
depends_on:
- yaks-95be
labels:
- cli
- delta
---

Phase 2 of yaks-8b7a (plan note: D4's permanent dual read; runs after phase 1, yaks-95be). Sonnet worker.
Scope: src/sheds.rs `base_farm` (it parses `git ls-tree` paths per status dir today) so a base commit in EITHER layout materializes correctly; every cross-shed comparison between a migrated and an unmigrated checkout (`sheds`, `changes`, `show --sheds`, `inbox --sheds`, `landed`): a yak whose only difference is the layout is NOT a change; scripts/discover-fixture.sh; tests.
Out of scope: src/store.rs (phase 1 already reads both layouts from disk), migrate, git reporting, skills.
Evidence (fail first): a shed whose fork point is a v3 commit and whose working tree is v4 (it migrated) shows only its real changes; a v4 viewer and a v3 shed (not migrated) compare correctly both ways; a migration alone reads as "no changes of its own".
