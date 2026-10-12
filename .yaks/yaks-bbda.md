---
id: yaks-bbda
title: 'Flat farm, phase 4: remove the migration and the dual-layout read once every farm is migrated'
status: hairy
type: task
priority: 2
created: '2026-10-12T20:00:38Z'
updated: '2026-10-12T20:00:38Z'
parent: yaks-8b7a
depends_on:
- yaks-3a56
labels:
- cli
---

Phase 4 of yaks-8b7a (after phase 3's release and after Joel's farms are migrated: ~/src/yaks, ~/src/j15r, and any private/pointer farms). Remove the transition: `yaks migrate`, the dual-layout read in src/store.rs, the legacy deleted-plus-added classification in src/changes.rs, the both-layouts doctor/preflight check, and every transitional line in docs and skills. Keep: a schema-3 farm refuses to open with a message naming the last release that can migrate it; `yaks sheds` shows a schema-3 shed's farm as "old layout (pre-0.0.16 clone)", never as an empty farm. Release 0.0.17.
