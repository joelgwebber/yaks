---
id: yak-bf54.2
title: 'Sync check mode: identify yaks that likely need syncing'
type: feature
priority: 2
created: '2026-04-24T03:01:50Z'
updated: '2026-04-24T03:01:50Z'
depends_on:
- yak-bf54.1
---

A cheap 'which of my yaks with sources might have drifted?' query, so the user can pick what to /yaks:sync. Depends on bf54.1 — without reliable upstream timestamps, the only honest answer is 'all of them,' which isn't useful. When that investigation lands, revisit scope (CLI flag on list? new /yaks:sync-check command?).
